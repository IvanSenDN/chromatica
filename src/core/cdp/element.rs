use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tokio::time::Duration;

use super::domains::dom::BackendNodeId;
use super::frame_inner::FrameInner;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct Element {
    frame_inner: Weak<FrameInner>,
    backend_node_id: BackendNodeId,
}

impl Element {
    pub fn new(frame_inner: Weak<FrameInner>, backend_node_id: BackendNodeId) -> Self {
        Self {
            frame_inner,
            backend_node_id,
        }
    }

    fn frame_inner(&self) -> Option<Arc<FrameInner>> {
        match self.frame_inner.upgrade() {
            Some(frame_inner) => Some(frame_inner),
            None => None,
        }
    }

    pub async fn query_selector(&self, query: &str) -> Result<Element> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .query_selector(query, Some(self.backend_node_id))
            .await
    }

    pub async fn query_selector_all(&self, query: &str) -> Result<Vec<Element>> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .query_selector_all(query, Some(self.backend_node_id))
            .await
    }

    // pub async fn wait_for_selector(
    //     &self,
    //     query: &str,
    //     timeout: Option<Duration>,
    //     delay: Option<Duration>,
    // ) -> Result<Option<Element>> {
    //     let frame_inner = match self.frame_inner() {
    //         Some(frame_inner) => frame_inner,
    //         None => return Err(anyhow!("Frame inner is not available")),
    //     };
    //     frame_inner
    //         .wait_for_selector(query, timeout, delay, Some(self.backend_node_id))
    //         .await
    // }

    pub async fn wait_for_selector(
        &self,
        query: &str,
        timeout: Option<Duration>,
    ) -> Result<Element> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .wait_for_selector(query, timeout, Some(self.backend_node_id))
            .await
    }

    pub async fn click(&self) -> Result<()> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner.click(&self.backend_node_id).await
    }

    pub async fn attributes(&self) -> Result<HashMap<String, String>> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner.get_attributes(&self.backend_node_id).await
    }

    pub async fn type_text(&self, text: &str, delay: Option<u64>) -> Result<()> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .type_text(&self.backend_node_id, text, delay)
            .await
    }

    pub async fn upload_file(
        &self,
        file_paths: Vec<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .upload_file(&self.backend_node_id, file_paths, timeout)
            .await
    }

    ///It won't work if the element is not in top frame (page) target.
    pub async fn screenshot(
        &self,
        save_path: Option<&str>,
        format: Option<&str>,
        quality: Option<u64>,
        full_page: Option<bool>,
    ) -> Result<String> {
        let frame_inner = match self.frame_inner() {
            Some(frame_inner) => frame_inner,
            None => return Err(anyhow!("Frame inner is not available")),
        };
        frame_inner
            .screenshot(
                save_path,
                Some(&self.backend_node_id),
                format,
                quality,
                full_page,
            )
            .await
    }
}
