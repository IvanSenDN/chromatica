use super::connection::{Event, EventParams, Response};
use super::domains::dom::{
    BackendNodeId, DescribeNode, DescribeNodeResponse, Focus, GetAttributes, GetAttributesResponse,
    GetBoxModel, GetBoxModelResponse, GetDocument, GetDocumentResponse, MinimalNode, NodeId,
    PushNodesByBackendIdsToFrontend, PushNodesByBackendIdsToFrontendResponse,
    ScrollIntoViewIfNeeded,
};
use super::domains::dom_storage::{
    DomStorageDisable, DomStorageEnable, GetDOMStorageItems, GetDOMStorageItemsResponse, Item,
    RemoveDOMStorageItem, SerializedStorageKey, StorageId,
};
use super::domains::input::{
    DispatchKeyEvent, DispatchMouseEvent, KeyEventType, MouseButton, MouseEventType,
};
use super::domains::page::{
    AddScriptToEvaluateOnNewDocument, AddScriptToEvaluateOnNewDocumentResponse, CaptureScreenshot,
    CaptureScreenshotResponse, FrameId, Navigate, PrintToPDF, PrintToPDFResponse, Reload,
    RemoveScriptToEvaluateOnNewDocument, ScriptIdentifier, Viewport,
};
use super::domains::target::{ActivateTarget, CloseTarget};
use super::element::Element;
use super::emulation_manager::{EmulationManager, UserAgentOverride};
use super::file_chooser::FileChooser;
use super::http_response::HttpResponse;
use super::js_dialogs::JsDialog;
use super::js_manager::JsManager;
use super::network_manager::{NetworkManager, RequestStream, ResponseStream};
use super::query_builder::QueryBuilder;
use super::target::Target;
use super::target_manager::TargetManager;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use serde::Serialize;
use std::collections::HashMap;

use std::path::PathBuf;
use std::sync::{Arc, Weak};
use tokio::join;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct FrameInner {
    target: Arc<RwLock<Weak<Target>>>,
    frame_id: Arc<FrameId>,
    parent_frame_id: Option<Arc<FrameId>>,
    child_frame_ids: DashSet<Arc<FrameId>>,
    backend_node_id: Arc<RwLock<Option<BackendNodeId>>>,
    default_timeout: Arc<RwLock<Duration>>,
}

impl FrameInner {
    pub fn new(
        target: Weak<Target>,
        frame_id: Arc<FrameId>,
        parent_frame_id: Option<Arc<FrameId>>,
        backend_node_id: Option<BackendNodeId>,
    ) -> Self {
        Self {
            target: Arc::new(RwLock::new(target)),
            frame_id,
            parent_frame_id,
            child_frame_ids: DashSet::with_capacity(4),
            backend_node_id: Arc::new(RwLock::new(backend_node_id)),
            default_timeout: Arc::new(RwLock::new(Duration::from_secs(30))),
            // dom_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn init_as_target(&self, target: Weak<Target>) {
        let mut current_target = self.target.write().await;
        *current_target = target.clone();
        let mut backend_node_id = self.backend_node_id.write().await;
        *backend_node_id = None;
    }

    pub async fn target(&self) -> Arc<Target> {
        let target = self.target.read().await;
        match target.upgrade() {
            Some(target) => target,
            None => panic!("Target is dropped"),
        }
    }

    pub async fn target_manager(&self) -> Arc<TargetManager> {
        let target = self.target().await;
        target.target_manager()
    }

    pub async fn parent_frame(&self) -> Option<Arc<FrameInner>> {
        let target_manager = self.target_manager().await;
        if let Some(parent_frame_id) = &self.parent_frame_id {
            let frame_inner = target_manager.get_frame_inner(&parent_frame_id).await;
            if let Some(frame_inner) = frame_inner {
                Some(frame_inner)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn child_frames(&self) -> Option<Vec<Arc<FrameInner>>> {
        let target_manager = self.target_manager().await;
        let capacity = self.child_frame_ids.len();
        let mut futures = Vec::with_capacity(capacity);

        for frame_id in self.child_frame_ids.iter() {
            let target_manager = target_manager.clone();
            let future = async move {
                let frame_inner = target_manager.get_frame_inner(&frame_id).await;
                if let Some(frame_inner) = frame_inner {
                    Some(frame_inner)
                } else {
                    None
                }
            };
            futures.push(future);
        }

        let frames = join_all(futures).await.into_iter().flatten().collect();
        Some(frames)
    }

    pub async fn add_child_frame(&self, frame_id: Arc<FrameId>) {
        self.child_frame_ids.insert(frame_id.clone());
    }

    pub async fn remove_child_frame(&self, frame_id: Arc<FrameId>) {
        self.child_frame_ids.remove(&frame_id);
    }

    async fn dom_lock(&self) -> Arc<Mutex<()>> {
        let target = self.target().await;
        target.dom_lock()
    }

    pub async fn bring_to_front(&self) -> Result<()> {
        let target = self.target().await;
        let connection = match target.connection() {
            Some(connection) => connection,
            None => return Err(anyhow!("Target is not connected")),
        };
        let target_id = self.frame_id();
        let params = ActivateTarget::default(&target_id);
        match connection
            .send("Target.activateTarget", &params, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Failed to bring to front: {}", e)),
        }
    }

    pub async fn send<P: Serialize>(&self, method: &str, params: &P) -> Result<Response> {
        let target = self.target().await;
        target.send(method, params).await
    }

    pub async fn subscribe(
        &self,
        methods: DashSet<String>,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Event>>> {
        let target = self.target().await;
        target.subscribe(methods).await
    }

    async fn network_manager(&self) -> Arc<NetworkManager> {
        let target = self.target().await;
        target.network_manager().unwrap()
    }

    pub async fn set_credentials(&self, username: &str, password: &str) -> Result<()> {
        let network_manager = self.network_manager().await;
        network_manager.set_credentials(username, password).await
    }

    pub async fn subscribe_to_requests(&self) -> Result<(RequestStream, ResponseStream)> {
        let network_manager = self.network_manager().await;
        network_manager.set_request_interception(true);
        let requests = network_manager.clone().subscribe_to_requests();
        let responses = network_manager.clone().subscribe_to_responses();
        Ok((requests, responses))
    }

    pub async fn set_extra_headers(&self, headers: HashMap<&str, &str>) -> Result<()> {
        let network_manager = self.network_manager().await;
        network_manager.set_extra_headers(headers).await
    }

    pub async fn clear_extra_headers(&self) -> Result<()> {
        let network_manager = self.network_manager().await;
        network_manager.clear_extra_headers().await
    }

    async fn emulation_manager(&self) -> Arc<EmulationManager> {
        let target = self.target().await;
        target.emulation_manager().unwrap()
    }

    async fn js_manager(&self) -> Arc<JsManager> {
        let target = self.target().await;
        target.js_manager().unwrap()
    }

    pub async fn subscribe_to_js_dialogs(&self) -> Result<broadcast::Receiver<JsDialog>> {
        let js_manager = self.js_manager().await;
        let dialogs = js_manager.subscribe_to_js_dialogs();
        Ok(dialogs)
    }

    pub fn frame_id(&self) -> Arc<FrameId> {
        self.frame_id.clone()
    }

    pub async fn node(&self, depth: i32) -> Result<MinimalNode> {
        let backend_node_id = self.backend_node_id.read().await;
        match backend_node_id.as_ref() {
            Some(backend_node_id) => {
                let node = self
                    .send(
                        "DOM.describeNode",
                        &DescribeNode::new()
                            .backend_node_id(backend_node_id)
                            .depth(depth)
                            .build(),
                    )
                    .await?;
                let node = node.result_as::<DescribeNodeResponse>()?.node;
                Ok(node)
            }
            None => {
                let node = self
                    .send("DOM.getDocument", &GetDocument::new().depth(depth).build())
                    .await?;
                let node = node.result_as::<GetDocumentResponse>()?.root;
                Ok(node)
            }
        }
    }

    pub async fn bound_node(&self, backend_node_id: &BackendNodeId) -> Result<NodeId> {
        let response = self
            .send(
                "DOM.pushNodesByBackendIdsToFrontend",
                &PushNodesByBackendIdsToFrontend::default(vec![backend_node_id.clone()]),
            )
            .await?;
        let node_id = response
            .result_as::<PushNodesByBackendIdsToFrontendResponse>()?
            .node_ids[0]
            .clone();
        Ok(node_id)
    }

    pub async fn backend_node_id(&self) -> BackendNodeId {
        let backend_node_id = self.backend_node_id.read().await;
        match backend_node_id.as_ref() {
            Some(backend_node_id) => *backend_node_id,
            None => panic!("Backend node id is not set"),
        }
    }

    pub async fn default_timeout(&self) -> Duration {
        *self.default_timeout.read().await
    }

    pub async fn set_default_timeout(&self, timeout: Duration) {
        *self.default_timeout.write().await = timeout;
    }

    pub async fn wait_for_js_dialog<F>(
        &self,
        predicate: F,
        timeout: Option<Duration>,
    ) -> Result<JsDialog>
    where
        F: Fn(&JsDialog) -> bool + Send + 'static,
    {
        let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();
        let js_manager = self.js_manager().await;
        let mut dialogs = js_manager.subscribe_to_js_dialogs();

        let mut handle = tokio::spawn(async move {
            while let Ok(dialog) = dialogs.recv().await {
                if predicate(&dialog) {
                    return Ok(dialog);
                }
                tokio::task::yield_now().await;
            }
            Err(anyhow!("No matching dialog found"))
        });

        if timeout.is_zero() {
            let dialog = handle.await?;
            dialog
        } else {
            match tokio::time::timeout(timeout, &mut handle).await {
                Ok(result) => {
                    let dialog = result?;
                    dialog
                }
                Err(_) => {
                    handle.abort();
                    Err(anyhow!("Waiting for dialog timed out"))
                }
            }
        }
    }
    pub async fn wait_for_response<F>(
        &self,
        predicate: F,
        timeout: Option<Duration>,
    ) -> Result<HttpResponse>
    where
        F: Fn(&HttpResponse) -> bool + Send + 'static,
    {
        let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();
        let network_manager = self.network_manager().await;
        let mut responses = network_manager.subscribe_to_responses();

        let mut handle = tokio::spawn(async move {
            while let Some(response) = responses.next().await {
                if predicate(&response) {
                    return Ok(response);
                }
                tokio::task::yield_now().await;
            }
            Err(anyhow!("No matching response found"))
        });

        if timeout.is_zero() {
            let response = handle.await?;
            Ok(response?)
        } else {
            match tokio::time::timeout(timeout, &mut handle).await {
                Ok(response) => {
                    let response = response?;
                    Ok(response?)
                }
                Err(_) => {
                    handle.abort();
                    Err(anyhow!("Waiting for response timed out"))
                }
            }
        }
    }
    pub async fn wait_for_navigation(
        &self,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let wait_until = wait_until.unwrap_or("load");
        let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();

        let lifecycle_event_map: DashMap<&str, &str> = [
            ("init", "init"),
            ("load", "load"),
            ("domcontentloaded", "DOMContentLoaded"),
            ("networkidle2", "networkAlmostIdle"),
            ("networkidle0", "networkIdle"),
        ]
        .iter()
        .cloned()
        .collect();

        let expected_event = *lifecycle_event_map
            .get(wait_until)
            .ok_or_else(|| anyhow!("Unknown waitUntil option: {}", wait_until))?;

        let lifecycle_event_order = [
            "init",
            "load",
            "DOMContentLoaded",
            "networkAlmostIdle",
            "networkIdle",
        ];

        let target_index = lifecycle_event_order
            .iter()
            .position(|&e| e == expected_event)
            .ok_or_else(|| anyhow!("Unexpected internal event: {}", expected_event))?;

        let expected_events: DashSet<&str> = lifecycle_event_order[target_index..]
            .iter()
            .cloned()
            .collect();

        let methods = DashSet::with_capacity(2);
        methods.insert("Page.lifecycleEvent".to_string());
        methods.insert("Network.loadingFailed".to_string());

        let mut events = self.subscribe(methods).await?;
        let frame_id = self.frame_id();
        let (tx, rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let lifecycle_event = match &event.params {
                    EventParams::LifecycleEvent(lifecycle_event) => lifecycle_event,
                    EventParams::LoadingFailed(loading_failed)
                        if loading_failed.recource_type == "Document" =>
                    {
                        let _ = tx.send(Err(anyhow!("Loading failed: {:?}", loading_failed)));
                        return;
                    }
                    _ => continue,
                };
                if lifecycle_event.frame_id == frame_id.as_str()
                    && expected_events.contains(lifecycle_event.name.as_str())
                {
                    let event_index = lifecycle_event_order
                        .iter()
                        .position(|&e| e == lifecycle_event.name.as_str())
                        .unwrap();
                    if event_index >= target_index {
                        let _ = tx.send(Ok(()));
                        return;
                    }
                }
                tokio::task::yield_now().await;
            }
            let _ = tx.send(Err(anyhow!("Channel closed")));
        });

        if timeout.is_zero() {
            rx.await?
        } else {
            match tokio::time::timeout(timeout, rx).await {
                Ok(result) => {
                    handle.await?;
                    result?
                }
                Err(_) => {
                    handle.abort();
                    Err(anyhow!("Navigation timed out"))
                }
            }
        }
    }

    pub async fn navigate(
        self: &Arc<Self>,
        url: &str,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let url = url.to_string();

        let navigate_params = Navigate::default(&url, &self.frame_id);
        let navigate = self.send("Page.navigate", &navigate_params);

        let wait_for_navigation = self.wait_for_navigation(wait_until, timeout);

        let (navigate, wait_for_navigation) = join!(navigate, wait_for_navigation);
        match navigate {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Navigation failed: {}", e)),
        };
        match wait_for_navigation {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
        Ok(())
    }

    pub async fn reload(
        self: &Arc<Self>,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let reload_params = Reload::default();
        let reload = self.send("Page.reload", &reload_params);
        let wait_for_navigation = self.wait_for_navigation(wait_until, timeout);

        let (reload, wait_for_navigation) = join!(reload, wait_for_navigation);
        match reload {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Reload failed: {}", e)),
        };
        match wait_for_navigation {
            Ok(_) => (),
            Err(e) => return Err(e),
        };
        Ok(())
    }
    pub async fn close(&self) -> Result<()> {
        let target_id = self.frame_id();
        match self
            .send("Target.closeTarget", &CloseTarget::default(&target_id))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn screenshot(
        self: &Arc<Self>,
        save_path: Option<&str>,
        backend_node_id: Option<&BackendNodeId>,
        format: Option<&str>,
        quality: Option<u64>,
        full_page: Option<bool>,
    ) -> Result<String> {
        let mut params = CaptureScreenshot::new();
        if let Some(format) = format {
            params.format = Some(format);
        }
        if let Some(quality) = quality {
            params.quality = Some(quality);
        }
        if let Some(backend_node_id) = backend_node_id {
            let response = self
                .send("DOM.getBoxModel", &GetBoxModel::default(backend_node_id))
                .await?;
            let content = response.result_as::<GetBoxModelResponse>()?.model.content;
            let xs = [content[0], content[2], content[4], content[6]];
            let ys = [content[1], content[3], content[5], content[7]];
            let min_x = xs.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_x = xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let min_y = ys.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_y = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let width = max_x - min_x;
            let height = max_y - min_y;
            params.clip = Some(Viewport::default(min_x, min_y, width, height, 1.0));
        }

        if let Some(full_page) = full_page {
            params.capture_beyond_viewport = Some(full_page);
        }

        let response = self.send("Page.captureScreenshot", &params.build()).await?;
        let data = response.result_as::<CaptureScreenshotResponse>()?.data;
        if let Some(path) = save_path {
            let data = BASE64.decode(&data)?;
            tokio::fs::write(path, data).await?;
            Ok(path.to_string())
        } else {
            Ok(data)
        }
    }

    pub async fn print_to_pdf<'a>(
        self: &Arc<Self>,
        save_path: Option<&str>,
        options: Option<PrintToPDF<'a>>,
    ) -> Result<String> {
        let params = options.unwrap_or(PrintToPDF::default());
        let response = self.send("Page.printToPDF", &params).await?;
        let data = response.result_as::<PrintToPDFResponse>()?.data;
        if let Some(path) = save_path {
            let data = BASE64.decode(&data)?;
            tokio::fs::write(path, data).await?;
            Ok(path.to_string())
        } else {
            Ok(data)
        }
    }

    pub async fn set_user_agent(&self, user_agent: UserAgentOverride) -> Result<()> {
        let emulation_manager = self.emulation_manager().await;
        let _ = emulation_manager.set_user_agent(user_agent).await;
        Ok(())
    }

    pub async fn user_agent(&self) -> Result<String> {
        let emulation_manager = self.emulation_manager().await;
        let user_agent = emulation_manager.user_agent().await?;
        Ok(user_agent)
    }

    pub async fn query_selector(
        self: &Arc<Self>,
        query: &str,
        backend_node_id: Option<BackendNodeId>,
    ) -> Result<Element> {
        let _ = self.dom_lock().await.lock().await;
        // let _ = self
        //     .send(
        //         "DOM.getDocument",
        //         &GetDocument::new().depth(0).pierce(true).build(),
        //     )
        //     .await?;
        let query_builder = QueryBuilder::new(query, Arc::downgrade(self), backend_node_id);
        let result = query_builder.parse().await?;
        let (backend_node_id, frame_inner) = match result {
            Some((backend_node_id, frame_inner)) => (backend_node_id, frame_inner),
            None => return Err(anyhow!("No element found")),
        };
        Ok(Element::new(Arc::downgrade(&frame_inner), backend_node_id))
    }

    pub async fn query_selector_all(
        self: &Arc<Self>,
        query: &str,
        backend_node_id: Option<BackendNodeId>,
    ) -> Result<Vec<Element>> {
        let _ = self.dom_lock().await.lock().await;
        let _ = self
            .send(
                "DOM.getDocument",
                &GetDocument::new().depth(0).pierce(true).build(),
            )
            .await?;
        let query_builder = QueryBuilder::new(query, Arc::downgrade(self), backend_node_id);
        let result = query_builder.parse_all().await?;
        let vec = match result {
            Some(vec) => vec,
            None => return Err(anyhow!("No elements found")),
        };
        let elements = vec
            .into_iter()
            .map(|(backend_node_id, frame_inner)| {
                Element::new(Arc::downgrade(&frame_inner), backend_node_id)
            })
            .collect();
        Ok(elements)
    }

    // pub async fn wait_for_selector(
    //     self: &Arc<Self>,
    //     query: &str,
    //     timeout: Option<Duration>,
    //     delay: Option<Duration>,
    //     backend_node_id: Option<BackendNodeId>,
    // ) -> Result<Option<Element>> {
    //     let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();
    //     let self_clone = self.clone();
    //     let query = query.to_string();

    //     let mut handle: JoinHandle<Result<Option<Element>>> = tokio::spawn(async move {
    //         loop {
    //             let lock = self_clone.dom_lock.lock().await;
    //             let query_builder =
    //                 QueryBuilder::new(&query, Arc::downgrade(&self_clone), backend_node_id);
    //             let result = query_builder.parse().await?;
    //             let (backend_node_id, frame_inner) = match result {
    //                 Some((backend_node_id, frame_inner)) => (backend_node_id, frame_inner),
    //                 None => {
    //                     drop(lock);
    //                     tokio::task::yield_now().await;
    //                     tokio::time::sleep(delay.unwrap_or(Duration::from_millis(200))).await;
    //                     continue;
    //                 }
    //             };
    //             drop(lock);
    //             let element = Element::new(Arc::downgrade(&frame_inner), backend_node_id);
    //             return Ok(Some(element));
    //         }
    //     });

    //     if timeout.is_zero() {
    //         handle.await?
    //     } else {
    //         match tokio::time::timeout(timeout, &mut handle).await {
    //             Ok(result) => {
    //                 let result = result??;
    //                 Ok(result)
    //             }
    //             Err(e) => {
    //                 println!("Wait for selector timed out: {:?}", e);
    //                 handle.abort();
    //                 Err(anyhow!("Wait for selector timed out"))
    //             }
    //         }
    //     }
    // }

    //Lazy version
    pub async fn wait_for_selector(
        self: &Arc<Self>,
        query: &str,
        timeout: Option<Duration>,
        backend_node_id: Option<BackendNodeId>,
    ) -> Result<Element> {
        let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();
        let query_str = query.to_string();

        let self_clone = self.clone();

        let mut handle: JoinHandle<Result<Element>> = tokio::spawn(async move {
            let mut dom_events = self_clone.js_manager().await.subscribe_to_dom_events();
            loop {
                match dom_events.recv().await {
                    Ok(()) | Err(broadcast::error::RecvError::Lagged(_)) => {
                        match self_clone.query_selector(&query_str, backend_node_id).await {
                            Ok(element) => {
                                return Ok(element);
                            }
                            Err(_) => {
                                tokio::task::yield_now().await;
                                tokio::time::sleep(Duration::from_millis(10)).await;
                                continue;
                            }
                        }
                        // println!("element: {:?}", element);
                        // if let Some(element) = element {
                        // return Ok(element);
                        // } else {
                        //     tokio::task::yield_now().await;
                        //     tokio::time::sleep(Duration::from_millis(10)).await;
                        //     continue;
                        // }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(anyhow!("Channel closed"));
                    }
                }
            }
        });

        let element = match self.query_selector(query, backend_node_id).await {
            Ok(element) => {
                handle.abort();
                return Ok(element);
            }
            Err(_) => (),
        };

        if timeout.is_zero() {
            handle.await?
        } else {
            match tokio::time::timeout(timeout, &mut handle).await {
                Ok(result) => {
                    let result = result??;
                    Ok(result)
                }
                Err(_) => {
                    // println!("Wait for selector timed out: {:?}", e);
                    handle.abort();
                    Err(anyhow!("Wait for selector timed out"))
                }
            }
        }
    }

    async fn wait_for_file_chooser(
        self: &Arc<Self>,
        timeout: Option<Duration>,
    ) -> Result<FileChooser> {
        let timeout = timeout.or(Some(self.default_timeout().await)).unwrap();
        let js_manager = self.js_manager().await;
        let _ = js_manager.set_intercept_file_chooser(true).await;
        let mut file_chooser = js_manager.subscribe_to_file_chooser();

        let mut handle = tokio::spawn(async move {
            while let Ok(file_chooser) = file_chooser.recv().await {
                return Ok(file_chooser);
            }
            Err(anyhow!("No matching dialog found"))
        });

        if timeout.is_zero() {
            let file_chooser = handle.await?;
            let _ = js_manager.set_intercept_file_chooser(false).await;
            Ok(file_chooser?)
        } else {
            match tokio::time::timeout(timeout, &mut handle).await {
                Ok(result) => {
                    let file_chooser = result?;
                    let _ = js_manager.set_intercept_file_chooser(false).await;
                    Ok(file_chooser?)
                }
                Err(_) => {
                    handle.abort();
                    let _ = js_manager.set_intercept_file_chooser(false).await;
                    Err(anyhow!("Waiting for file chooser timed out"))
                }
            }
        }
    }

    pub async fn click(self: &Arc<Self>, backend_node_id: &BackendNodeId) -> Result<()> {
        let self_clone = self.clone();
        //Actually we can ignore this error, not all elements are able to scroll into view especially if they are in iframes.
        match self_clone
            .send(
                "DOM.scrollIntoViewIfNeeded",
                &ScrollIntoViewIfNeeded::default(backend_node_id),
            )
            .await
        {
            Ok(_) => (),
            Err(_) => (),
        };

        let response = match self_clone
            .send("DOM.getBoxModel", &GetBoxModel::default(backend_node_id))
            .await
        {
            Ok(response) => response,
            //Critical error, if we can't get box model, then we can't click on the element.
            Err(e) => return Err(anyhow!("Get box model failed: {}", e)),
        };

        let content = response.result_as::<GetBoxModelResponse>()?.model.content;

        let x = content[0] + (content[2] - content[0]) / 2.0;
        let y = content[1] + (content[5] - content[1]) / 2.0;

        match self_clone
            .send(
                "Input.dispatchMouseEvent",
                &DispatchMouseEvent::default(MouseEventType::MouseMoved, x, y),
            )
            .await
        {
            Ok(_) => (),
            //That's not really critical error, we can ignore it, but we should log it.
            Err(_) => (),
        };

        let press_mouse = DispatchMouseEvent::new(MouseEventType::MousePressed, x, y)
            .button(MouseButton::Left)
            .click_count(1)
            .build();

        match self_clone
            .send("Input.dispatchMouseEvent", &press_mouse)
            .await
        {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Mouse pressed failed: {}", e)),
        };

        match self_clone
            .send("DOM.focus", &Focus::default(backend_node_id))
            .await
        {
            Ok(_) => (),
            //So much elements are clickable, but not focusable, so we can ignore this error.
            Err(_) => (),
        };

        let release_mouse = DispatchMouseEvent::new(MouseEventType::MouseReleased, x, y)
            .button(MouseButton::Left)
            .click_count(1)
            .build();

        match self_clone
            .send("Input.dispatchMouseEvent", &release_mouse)
            .await
        {
            Ok(_) => (),
            Err(e) => return Err(anyhow!("Mouse released failed: {}", e)),
        };

        Ok(())
    }

    pub async fn get_attributes(
        &self,
        backend_node_id: &BackendNodeId,
    ) -> Result<HashMap<String, String>> {
        // let describe_node = DescribeNode::new()
        //     .backend_node_id(backend_node_id)
        //     .depth(1)
        //     .build();
        // let response = match self.send("DOM.describeNode", &describe_node).await {
        //     Ok(response) => response,
        //     Err(e) => return Err(anyhow!("Get attributes failed: {}", e)),
        // };
        // let node = response.result_as::<DescribeNodeResponse>()?.node;
        // let attributes = node.attributes.unwrap_or(vec![]);
        let dom_lock = self.dom_lock().await;
        let lock = dom_lock.lock().await;
        let node_id = self.bound_node(backend_node_id).await?;
        let response = self
            .send("DOM.getAttributes", &GetAttributes::default(&node_id))
            .await?;
        drop(lock);
        let attributes = response.result_as::<GetAttributesResponse>()?.attributes;
        let mut result = HashMap::new();
        for chunk in attributes.chunks(2) {
            if chunk.len() == 2 {
                result.insert(chunk[0].clone(), chunk[1].clone());
            }
        }
        Ok(result)
    }

    pub async fn type_text(
        self: &Arc<Self>,
        backend_node_id: &BackendNodeId,
        text: &str,
        delay: Option<u64>,
    ) -> Result<()> {
        let self_clone = self.clone();
        self_clone.click(backend_node_id).await?;

        for c in text.chars() {
            match self_clone
                .send(
                    "Input.dispatchKeyEvent",
                    &DispatchKeyEvent::default(KeyEventType::KeyDown, &c.to_string()),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Type text failed: {}", e)),
            }

            match self_clone
                .send(
                    "Input.dispatchKeyEvent",
                    &DispatchKeyEvent::default(KeyEventType::KeyUp, &c.to_string()),
                )
                .await
            {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Type text failed: {}", e)),
            }

            if let Some(delay) = delay {
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
        Ok(())
    }

    pub async fn upload_file(
        self: &Arc<Self>,
        backend_node_id: &BackendNodeId,
        file_paths: Vec<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        let self_clone = self.clone();

        let absolute_paths: Vec<String> = file_paths
            .iter()
            .map(|path| {
                let path_buf = PathBuf::from(path);
                if path_buf.is_absolute() {
                    path.to_string()
                } else {
                    std::env::current_dir()
                        .unwrap_or_default()
                        .join(path_buf)
                        .to_string_lossy()
                        .into_owned()
                }
            })
            .collect();

        let self_clone_for_chooser = self_clone.clone();
        let (file_chooser, _) = tokio::join!(
            self_clone_for_chooser.wait_for_file_chooser(timeout),
            self_clone.click(backend_node_id)
        );

        match file_chooser {
            Ok(file_chooser) => {
                let paths: Vec<&str> = absolute_paths.iter().map(|s| s.as_str()).collect();
                file_chooser.upload_file(paths).await?;
            }
            Err(e) => return Err(anyhow!("Upload file failed: {}", e)),
        };
        Ok(())
    }

    pub async fn add_evaluate_on_new_document(
        self: &Arc<Self>,
        script: &str,
    ) -> Result<ScriptIdentifier> {
        let self_clone = self.clone();

        let response = self_clone
            .send(
                "Page.addScriptToEvaluateOnNewDocument",
                &AddScriptToEvaluateOnNewDocument::default(script),
            )
            .await?;
        let script_identifier = response
            .result_as::<AddScriptToEvaluateOnNewDocumentResponse>()?
            .identifier;
        Ok(script_identifier)
    }

    pub async fn remove_evaluate_on_new_document(
        self: &Arc<Self>,
        script_identifier: &ScriptIdentifier,
    ) -> Result<()> {
        let self_clone = self.clone();
        self_clone
            .send(
                "Page.removeScriptToEvaluateOnNewDocument",
                &RemoveScriptToEvaluateOnNewDocument::default(script_identifier),
            )
            .await?;
        Ok(())
    }

    pub async fn dom_storage_enable(self: &Arc<Self>) -> Result<()> {
        let self_clone = self.clone();
        self_clone
            .send("DOMStorage.enable", &DomStorageEnable::default())
            .await?;
        Ok(())
    }

    pub async fn dom_storage_disable(self: &Arc<Self>) -> Result<()> {
        let self_clone = self.clone();
        self_clone
            .send("DOMStorage.disable", &DomStorageDisable::default())
            .await?;
        Ok(())
    }
}
