use super::domains::dom::{
    BackendNodeId, DescribeNode, DescribeNodeResponse, FullNode, GetDocument,
    GetDocumentResponseFull, MinimalNode, QuerySelector, QuerySelectorAll,
    QuerySelectorAllResponse, QuerySelectorResponse,
};
use super::domains::target::TargetId;

use super::frame_inner::FrameInner;
use super::target_manager::TargetManager;
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::sync::{Arc, Weak};

#[derive(Debug)]
enum SelectorStep {
    ShadowRootDeep,   // >>>
    ShadowRootDirect, // >>>>
    CssSelector(String),
}

#[derive(Debug)]
struct SelectorPath {
    steps: Vec<SelectorStep>,
}

enum ExecuteSelectorResult {
    Single((BackendNodeId, Arc<FrameInner>)),
    Multiple(Vec<(BackendNodeId, Arc<FrameInner>)>),
}

#[derive(Debug, Clone)]
pub struct QueryBuilder<'a> {
    query: &'a str,
    frame_inner: Weak<FrameInner>,
    backend_node_id: Option<BackendNodeId>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(
        query: &'a str,
        frame_inner: Weak<FrameInner>,
        backend_node_id: Option<BackendNodeId>,
    ) -> Self {
        Self {
            query,
            frame_inner,
            backend_node_id,
        }
    }

    pub async fn parse(&self) -> Result<Option<(BackendNodeId, Arc<FrameInner>)>> {
        if self.query.starts_with("text(") {
            let text = match self.parse_text_finder() {
                Some(text) => text,
                None => return Ok(None),
            };
            let frame_inner = match self.frame_inner() {
                Some(frame_inner) => frame_inner,
                None => return Ok(None),
            };

            let nodes = match self.find_by_text_everywhere(text, frame_inner, false).await {
                Some(nodes) => nodes,
                None => return Ok(None),
            };

            if nodes.len() == 0 {
                return Ok(None);
            }

            let (backend_node_id, frame_inner) = nodes[0].clone();

            return Ok(Some((backend_node_id, frame_inner)));
        }

        let alternatives: Vec<&str> = self.query.split(',').map(|s| s.trim()).collect();
        for selector in alternatives {
            if let Some(path) = self.parse_selector_path(selector)? {
                let frame_inner = match self.frame_inner() {
                    Some(frame_inner) => frame_inner,
                    None => continue,
                };

                let start_backend_node_id = if let Some(backend_node_id) = &self.backend_node_id {
                    *backend_node_id
                } else {
                    let node = match frame_inner.node(0).await {
                        Ok(node) => node,
                        Err(_) => return Ok(None),
                    };
                    node.backend_node_id
                };

                if let Some(result) = self
                    .execute_selector_path(&path, start_backend_node_id, frame_inner, false)
                    .await?
                {
                    match result {
                        ExecuteSelectorResult::Single(tuple) => return Ok(Some(tuple)),
                        ExecuteSelectorResult::Multiple(mut vec) => return Ok(vec.pop()),
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn parse_all(&self) -> Result<Option<Vec<(BackendNodeId, Arc<FrameInner>)>>> {
        if self.query.starts_with("text(") {
            let text = match self.parse_text_finder() {
                Some(text) => text,
                None => return Ok(None),
            };
            let frame_inner = match self.frame_inner() {
                Some(frame_inner) => frame_inner,
                None => return Ok(None),
            };
            let nodes = match self.find_by_text_everywhere(text, frame_inner, true).await {
                Some(nodes) => nodes,
                None => return Ok(None),
            };

            if nodes.len() == 0 {
                return Ok(None);
            }

            return Ok(Some(nodes));
        }

        let alternatives: Vec<&str> = self.query.split(',').map(|s| s.trim()).collect();
        for selector in alternatives {
            if let Some(path) = self.parse_selector_path(selector)? {
                let frame_inner = match self.frame_inner() {
                    Some(frame_inner) => frame_inner,
                    None => continue,
                };

                let start_backend_node_id = if let Some(backend_node_id) = &self.backend_node_id {
                    *backend_node_id
                } else {
                    let node = match frame_inner.node(0).await {
                        Ok(node) => node,
                        Err(_) => return Ok(None),
                    };
                    node.backend_node_id
                };

                if let Some(result) = self
                    .execute_selector_path(&path, start_backend_node_id, frame_inner, true)
                    .await?
                {
                    match result {
                        ExecuteSelectorResult::Single(tuple) => return Ok(Some(vec![tuple])),
                        ExecuteSelectorResult::Multiple(vec) => return Ok(Some(vec)),
                    }
                }
            }
        }
        Ok(None)
    }

    fn parse_text_finder(&self) -> Option<&str> {
        let re = match Regex::new(r"text\((.*?)\)") {
            Ok(re) => re,
            Err(_) => return None,
        };
        let caps = match re.captures(self.query) {
            Some(caps) => caps,
            None => return None,
        };
        let text = match caps.get(1) {
            Some(text) => text.as_str(),
            None => return None,
        };
        Some(text)
    }

    async fn find_by_text_everywhere(
        &self,
        text: &str,
        frame_inner: Arc<FrameInner>,
        all: bool,
    ) -> Option<Vec<(BackendNodeId, Arc<FrameInner>)>> {
        //I guess default capacities of 4 is enough for this case.
        let mut processed_frames: HashSet<Arc<String>> = HashSet::new();
        //Search performes only in target context, not in frame as node, so we need to get unique targets.
        //Several frames can have the same target.
        let mut processed_targets: HashSet<Arc<TargetId>> = HashSet::new();
        let mut frames_to_process: Vec<Arc<FrameInner>> = Vec::new();
        let mut nodes: Vec<(BackendNodeId, Arc<FrameInner>)> = Vec::new();

        frames_to_process.push(frame_inner);

        while let Some(current_frame) = frames_to_process.pop() {
            let target = current_frame.target().await;
            let target_id = target.target_id();
            let frame_id = current_frame.frame_id();
            if processed_targets.contains(&target_id) {
                continue;
            }
            if processed_frames.contains(&frame_id) {
                continue;
            }

            let get_document_params = GetDocument::default();
            let response = match current_frame
                .send("DOM.getDocument", &get_document_params)
                .await
            {
                Ok(response) => response,
                Err(_) => continue,
            };
            let document = match response.result_as::<GetDocumentResponseFull>() {
                Ok(response) => response.root,
                Err(_) => continue,
            };

            if all {
                if let Some(result) = self.find_by_text_all(&document, text, current_frame.clone())
                {
                    nodes.extend(result);
                }
            } else {
                if let Some(result) = self.find_by_text(&document, text, current_frame.clone()) {
                    nodes.push(result);
                    return Some(nodes);
                }
            }
            processed_targets.insert(target_id);
            processed_frames.insert(frame_id);
            let frame_inners = current_frame.child_frames().await;
            match frame_inners {
                Some(frame_inners) => {
                    for frame_inner in frame_inners {
                        frames_to_process.push(frame_inner);
                    }
                }
                None => {}
            }
        }
        Some(nodes)
    }

    fn find_by_text(
        &self,
        node: &FullNode,
        text: &str,
        frame_inner: Arc<FrameInner>,
    ) -> Option<(BackendNodeId, Arc<FrameInner>)> {
        if let Some(children) = &node.children {
            for child in children.iter() {
                if child.node_value == text || child.node_value.contains(text) {
                    if node.local_name != "script" && node.local_name != "style" {
                        return Some((node.backend_node_id, frame_inner.clone()));
                    }
                }
                if let Some(result) = self.find_by_text(child, text, frame_inner.clone()) {
                    return Some(result);
                }
            }
        }
        if let Some(pseudo_elements) = &node.pseudo_elements {
            for pseudo_element in pseudo_elements.iter() {
                if let Some(result) = self.find_by_text(pseudo_element, text, frame_inner.clone()) {
                    return Some(result);
                }
            }
        }
        if let Some(shadow_roots) = &node.shadow_roots {
            for shadow_root in shadow_roots.iter() {
                if let Some(result) = self.find_by_text(shadow_root, text, frame_inner.clone()) {
                    return Some(result);
                }
            }
        }
        None
    }

    fn find_by_text_all(
        &self,
        node: &FullNode,
        text: &str,
        frame_inner: Arc<FrameInner>,
    ) -> Option<Vec<(BackendNodeId, Arc<FrameInner>)>> {
        let mut nodes: Vec<(BackendNodeId, Arc<FrameInner>)> = Vec::new();
        if let Some(children) = &node.children {
            for child in children.iter() {
                if child.node_value == text || child.node_value.contains(text) {
                    if node.local_name != "script" && node.local_name != "style" {
                        nodes.push((node.backend_node_id, frame_inner.clone()));
                    }
                }
                if let Some(result) = self.find_by_text_all(child, text, frame_inner.clone()) {
                    nodes.extend(result);
                }
            }
        }
        if let Some(pseudo_elements) = &node.pseudo_elements {
            for pseudo_element in pseudo_elements.iter() {
                if let Some(result) =
                    self.find_by_text_all(pseudo_element, text, frame_inner.clone())
                {
                    nodes.extend(result);
                }
            }
        }
        if let Some(shadow_roots) = &node.shadow_roots {
            for shadow_root in shadow_roots.iter() {
                if let Some(result) = self.find_by_text_all(shadow_root, text, frame_inner.clone())
                {
                    nodes.extend(result);
                }
            }
        }
        Some(nodes)
    }

    fn frame_inner(&self) -> Option<Arc<FrameInner>> {
        match self.frame_inner.upgrade() {
            Some(frame_inner) => Some(frame_inner),
            None => None,
        }
    }

    async fn target_manager(&self) -> Option<Arc<TargetManager>> {
        let frame_inner = self.frame_inner()?;
        let target_manager = frame_inner.target_manager().await;
        Some(target_manager)
    }

    async fn frame_inner_by_id(&self, frame_id: &String) -> Option<Arc<FrameInner>> {
        let target_manager = self.target_manager().await?;
        let frame_inner = target_manager.get_frame_inner(frame_id).await?;
        Some(frame_inner)
    }

    async fn query_selector(
        &self,
        selector: &str,
        backend_node_id: BackendNodeId,
        frame_inner: Arc<FrameInner>,
    ) -> Option<(BackendNodeId, Arc<FrameInner>)> {
        let node_id = match frame_inner.bound_node(&backend_node_id).await {
            Ok(node_id) => node_id,
            Err(_) => return None,
        };
        let response = match frame_inner
            .send(
                "DOM.querySelector",
                &QuerySelector::default(selector, &node_id),
            )
            .await
        {
            Ok(response) => response,
            Err(_) => return None,
        };
        let node_id = match response.result_as::<QuerySelectorResponse>() {
            Ok(response) => response.node_id,
            Err(_) => return None,
        };
        let response = match frame_inner
            .send(
                "DOM.describeNode",
                &DescribeNode::new().node_id(&node_id).depth(0).build(),
            )
            .await
        {
            Ok(response) => response,
            Err(_) => return None,
        };
        let node = match response.result_as::<DescribeNodeResponse>() {
            Ok(response) => response.node,
            Err(_) => return None,
        };
        if let Some(frame_id) = node.frame_id {
            let frame_inner = match self.frame_inner_by_id(&frame_id).await {
                Some(frame_inner) => frame_inner,
                None => return None,
            };
            let node = match frame_inner.node(0).await {
                Ok(node) => node,
                Err(_) => return None,
            };
            return Some((node.backend_node_id, frame_inner));
        }
        Some((node.backend_node_id, frame_inner))
    }

    async fn query_selector_all(
        &self,
        selector: &str,
        backend_node_id: BackendNodeId,
        frame_inner: Arc<FrameInner>,
    ) -> Option<Vec<(BackendNodeId, Arc<FrameInner>)>> {
        let node_id = match frame_inner.bound_node(&backend_node_id).await {
            Ok(node_id) => node_id,
            Err(_) => return None,
        };

        let response = match frame_inner
            .send(
                "DOM.querySelectorAll",
                &QuerySelectorAll::default(selector, &node_id),
            )
            .await
        {
            Ok(response) => response,
            Err(_) => return None,
        };

        let node_ids = match response.result_as::<QuerySelectorAllResponse>() {
            Ok(response) => response.node_ids,
            Err(_) => return None,
        };

        let mut backend_node_ids: Vec<(BackendNodeId, Arc<FrameInner>)> = Vec::new();
        for node_id in node_ids {
            let response = match frame_inner
                .send(
                    "DOM.describeNode",
                    &DescribeNode::new().node_id(&node_id).depth(0).build(),
                )
                .await
            {
                Ok(response) => response,
                Err(_) => continue,
            };
            let node = match response.result_as::<DescribeNodeResponse>() {
                Ok(response) => response.node,
                Err(_) => continue,
            };
            if let Some(frame_id) = node.frame_id {
                let frame_inner = match self.frame_inner_by_id(&frame_id).await {
                    Some(frame_inner) => frame_inner,
                    None => continue,
                };
                let node = match frame_inner.node(0).await {
                    Ok(node) => node,
                    Err(_) => continue,
                };
                backend_node_ids.push((node.backend_node_id, frame_inner));
            } else {
                backend_node_ids.push((node.backend_node_id, frame_inner.clone()));
            }
        }
        Some(backend_node_ids)
    }

    //Find child shadow root of node
    fn into_shadow_root(&self, node: &MinimalNode) -> Option<BackendNodeId> {
        if let Some(shadow_roots) = &node.shadow_roots {
            for shadow_root in shadow_roots.iter() {
                return Some(shadow_root.backend_node_id);
            }
        }
        None
    }

    //Find first shadow root of node
    fn find_shadow_root(&self, node: &MinimalNode) -> Option<BackendNodeId> {
        if let Some(shadow_root) = self.into_shadow_root(node) {
            return Some(shadow_root);
        }
        if let Some(children) = &node.children {
            for child in children.iter() {
                if let Some(result) = self.find_shadow_root(child) {
                    return Some(result);
                }
            }
        }
        if let Some(pseudo_elements) = &node.pseudo_elements {
            for pseudo_element in pseudo_elements.iter() {
                if let Some(result) = self.find_shadow_root(pseudo_element) {
                    return Some(result);
                }
            }
        }
        None
    }

    fn parse_selector_path(&self, selector: &str) -> Result<Option<SelectorPath>> {
        let mut steps = Vec::new();
        let mut current = String::new();
        let mut chars = selector.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                '>' => {
                    let mut count = 0;
                    while chars.peek() == Some(&'>') {
                        chars.next();
                        count += 1;
                    }
                    if !current.trim().is_empty() {
                        steps.push(SelectorStep::CssSelector(current.trim().to_string()));
                        current.clear();
                    }
                    match count {
                        3 => steps.push(SelectorStep::ShadowRootDeep),
                        4 => steps.push(SelectorStep::ShadowRootDirect),
                        1 => {}
                        _ => {}
                    }
                    continue;
                }
                _ => {
                    current.push(c);
                    chars.next();
                }
            }
        }

        if !current.trim().is_empty() {
            steps.push(SelectorStep::CssSelector(current.trim().to_string()));
        }

        Ok(Some(SelectorPath { steps }))
    }

    async fn execute_selector_path(
        &self,
        path: &SelectorPath,
        start_backend_node_id: BackendNodeId,
        start_frame: Arc<FrameInner>,
        all: bool,
    ) -> Result<Option<ExecuteSelectorResult>> {
        let mut current_backend_node_id = start_backend_node_id;
        let mut current_frame = start_frame;

        let last_idx = path.steps.len().saturating_sub(1);
        for (i, step) in path.steps.iter().enumerate() {
            match step {
                SelectorStep::CssSelector(selector) => {
                    if all && i == last_idx {
                        if let Some(nodes) = self
                            .query_selector_all(
                                selector,
                                current_backend_node_id,
                                current_frame.clone(),
                            )
                            .await
                        {
                            return Ok(Some(ExecuteSelectorResult::Multiple(nodes)));
                        } else {
                            return Ok(None);
                        }
                    } else {
                        (current_backend_node_id, current_frame) = match self
                            .query_selector(
                                selector,
                                current_backend_node_id,
                                current_frame.clone(),
                            )
                            .await
                        {
                            Some((backend_node_id, frame_inner)) => {
                                (backend_node_id, frame_inner.clone())
                            }
                            None => return Ok(None),
                        };
                    }
                }
                SelectorStep::ShadowRootDeep => {
                    let node = match current_frame
                        .send(
                            "DOM.describeNode",
                            &DescribeNode::default(&current_backend_node_id),
                        )
                        .await
                    {
                        Ok(response) => response,
                        Err(_) => return Ok(None),
                    };
                    let node = match node.result_as::<DescribeNodeResponse>() {
                        Ok(response) => response.node,
                        Err(_) => return Ok(None),
                    };
                    current_backend_node_id = match self.find_shadow_root(&node) {
                        Some(backend_node_id) => backend_node_id,
                        None => return Ok(None),
                    };
                }
                SelectorStep::ShadowRootDirect => {
                    let node = match current_frame
                        .send(
                            "DOM.describeNode",
                            &DescribeNode::new()
                                .backend_node_id(&current_backend_node_id)
                                .depth(1)
                                .build(),
                        )
                        .await
                    {
                        Ok(response) => response,
                        Err(_) => return Ok(None),
                    };
                    let node = match node.result_as::<DescribeNodeResponse>() {
                        Ok(response) => response.node,
                        Err(_) => return Ok(None),
                    };
                    current_backend_node_id = match self.into_shadow_root(&node) {
                        Some(backend_node_id) => backend_node_id,
                        None => return Ok(None),
                    };
                }
            }
        }
        Ok(Some(ExecuteSelectorResult::Single((
            current_backend_node_id,
            current_frame,
        ))))
    }
}
