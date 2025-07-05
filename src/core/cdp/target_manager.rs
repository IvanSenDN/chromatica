use anyhow::{Result, anyhow};
use dashmap::{DashMap, DashSet};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;

use super::browser_context::BrowserContext;
use super::connection::{Connection, EventParams};
use super::domains::browser::BrowserContextID;
use super::domains::dom::*;
use super::domains::page::*;
use super::domains::target::*;
use super::frame_inner::FrameInner;
use super::page::Page;
use super::target::Target;

//Here's so much Arc for TargetId and FrameId, but i gained 30-40% of performance.
//Previously, i cloned them so much espessially for quering elements (but cloning strings costs so much), so it was good idea, i guess.
//But rn i need much more practical testing of lifetimes.
#[derive(Debug, Clone)]
pub struct TargetManager {
    connection: Weak<Connection>,
    browser_contexts: DashMap<BrowserContextID, Arc<BrowserContext>>,
    targets: DashMap<Arc<TargetId>, Arc<Target>>,
    frame_inners: DashMap<Arc<FrameId>, Arc<FrameInner>>,
    pending_targets: Arc<RwLock<HashMap<Arc<TargetId>, oneshot::Sender<Weak<Target>>>>>,
    // First TargetId is Parent to await for, Second is Child-iframe, which waits for parent to be created; We pass Target to be able to take it as mut and init it;
    pending_iframes: DashMap<(Arc<TargetId>, Arc<TargetId>), Target>,
    iframe_channel: Arc<RwLock<Option<mpsc::Sender<Arc<TargetId>>>>>,
    target_event_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
    iframe_channel_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl TargetManager {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self {
            connection: Arc::downgrade(&connection),
            browser_contexts: DashMap::with_capacity(1024),
            targets: DashMap::with_capacity(1024),
            frame_inners: DashMap::with_capacity(1024),
            pending_targets: Arc::new(RwLock::new(HashMap::with_capacity(1024))),
            pending_iframes: DashMap::with_capacity(1024),
            iframe_channel: Arc::new(RwLock::new(None)),
            target_event_handler: Arc::new(RwLock::new(None)),
            iframe_channel_handler: Arc::new(RwLock::new(None)),
        }
    }

    fn connection(&self) -> Option<Arc<Connection>> {
        self.connection.upgrade()
    }

    pub async fn get_target(&self, target_id: &TargetId) -> Option<Arc<Target>> {
        self.targets
            .get(target_id)
            .map(|entry| entry.value().clone())
    }

    pub async fn get_targets(&self) -> Vec<Arc<Target>> {
        let mut targets = Vec::with_capacity(self.targets.len());
        self.targets.iter().for_each(|entry| {
            targets.push(entry.value().clone());
        });
        targets
    }

    async fn add_target(&self, mut target: Target, parent_target: Option<Weak<Target>>) {
        let _ = target.init(parent_target).await;
        let arc_target = Arc::new(target);
        let target_id = arc_target.target_id().clone();

        self.targets.insert(target_id.clone(), arc_target.clone());
    }

    pub async fn get_frame_inner(&self, frame_id: &FrameId) -> Option<Arc<FrameInner>> {
        let frame_inner = self
            .frame_inners
            .get(frame_id)
            .map(|entry| entry.value().clone());
        frame_inner
    }

    pub async fn frames(&self) -> Vec<Arc<FrameInner>> {
        let mut frames = Vec::with_capacity(self.frame_inners.len());
        self.frame_inners.iter().for_each(|entry| {
            frames.push(entry.value().clone());
        });
        frames
    }

    pub async fn add_frame_inner(&self, frame_inner: Arc<FrameInner>) {
        let frame_id = frame_inner.frame_id();
        self.frame_inners
            .insert(frame_id.clone(), frame_inner.clone());
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        let methods = DashSet::with_capacity(5);
        methods.insert("Target.targetCreated".to_string());
        methods.insert("Target.targetDestroyed".to_string());
        methods.insert("Target.targetCrashed".to_string());
        methods.insert("Page.frameAttached".to_string());
        methods.insert("Page.frameDetached".to_string());

        let session_ids = DashSet::with_capacity(1024);

        let (_, mut rx) = conn.subscribe(methods, session_ids).await;

        conn.send(
            "Target.setDiscoverTargets",
            &SetDiscoverTargets::default(),
            None,
        )
        .await?;

        let downgraded_manager = Arc::downgrade(&self);
        let downgraded_manager_clone = downgraded_manager.clone();

        let event_handler = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // You can provide extra async close to JavaScript EventEmitter, but, to be honest, you won't win so much.

                // let downgraded_manager = downgraded_manager.clone();
                // tokio::spawn(async move {
                let manager = match downgraded_manager.upgrade() {
                    Some(manager) => manager,
                    // None => return,
                    None => break,
                };
                match &event.params {
                    EventParams::TargetCreated(created) => {
                        match manager.on_target_created(created).await {
                            Ok(_) => {}
                            // Err(e) => eprintln!("Error on_target_created: {}", e),
                            Err(_) => (),
                        }
                    }
                    EventParams::TargetDestroyed(destroyed) => {
                        match manager.on_target_destroyed(destroyed).await {
                            Ok(_) => {}
                            // Err(e) => eprintln!("Error on_target_destroyed: {}", e),
                            Err(_) => (),
                        }
                    }
                    EventParams::TargetCrashed(crashed) => {
                        match manager.on_target_crashed(crashed).await {
                            Ok(_) => {}
                            // Err(e) => eprintln!("Error on_target_crashed: {}", e),
                            Err(_) => (),
                        }
                    }
                    EventParams::FrameAttached(attached) => {
                        let session_id = event.session_id.as_ref().unwrap();
                        match manager.on_frame_attached(attached, &session_id).await {
                            Ok(_) => {}
                            // Err(e) => eprintln!("Error on_frame_attached: {}", e),
                            Err(_) => (),
                        }
                    }
                    EventParams::FrameDetached(detached) => {
                        match manager.on_frame_detached(detached).await {
                            Ok(_) => {}
                            // Err(e) => eprintln!("Error on_frame_detached: {}", e),
                            Err(_) => (),
                        }
                    }
                    _ => {
                        println!("Unknown event: {:?}", event);
                    }
                }
                tokio::task::yield_now().await;
                // });
            }
        });

        let mut target_event_handler = self.target_event_handler.write().await;
        *target_event_handler = Some(event_handler);

        // We send parent target id to iframe channel to be handled later
        let (iframe_tx, mut iframe_rx) = mpsc::channel::<Arc<TargetId>>(1024);

        *self.iframe_channel.write().await = Some(iframe_tx);

        // let pending_iframes = self.pending_iframes.clone();
        // let targets = self.targets.clone();

        let channel_handler = tokio::spawn(async move {
            while let Some(parent_id) = iframe_rx.recv().await {
                let manager = match downgraded_manager_clone.upgrade() {
                    Some(manager) => manager,
                    None => break,
                };

                let mut keys_to_remove = Vec::with_capacity(manager.pending_iframes.len());
                manager
                    .pending_iframes
                    .iter()
                    .filter(|entry| entry.key().0 == parent_id)
                    .for_each(|entry| {
                        keys_to_remove.push(entry.key().clone());
                    });
                keys_to_remove.shrink_to_fit();

                for key in keys_to_remove {
                    if let Some((_, mut target)) = manager.pending_iframes.remove(&key) {
                        let parent_target = manager.targets.get(&key.0).unwrap().clone();
                        let _ = target.init(Some(Arc::downgrade(&parent_target))).await;
                        let arc_target = Arc::new(target);

                        let frame_inner = manager.get_frame_inner(&key.1).await;
                        match frame_inner {
                            Some(frame_inner) => {
                                frame_inner
                                    .init_as_target(Arc::downgrade(&arc_target))
                                    .await;
                            }
                            None => {
                                let frame_inner = FrameInner::new(
                                    Arc::downgrade(&arc_target),
                                    key.1.clone(),
                                    Some(key.0.clone()),
                                    None,
                                );
                                //Actually, if we are in this block, it means that parent frame exists, so we can unwrap.
                                let parent_frame_inner =
                                    manager.get_frame_inner(&key.0).await.unwrap();
                                parent_frame_inner.add_child_frame(key.1.clone()).await;

                                manager.add_frame_inner(Arc::new(frame_inner)).await;
                            }
                        }

                        let _ = manager.targets.insert(key.1.clone(), arc_target);
                    }
                }
            }
        });

        let mut iframe_channel_handler = self.iframe_channel_handler.write().await;
        *iframe_channel_handler = Some(channel_handler);

        Ok(())
    }

    pub async fn shutdown(&self) {
        let target_event_handler = self.target_event_handler.read().await;
        if let Some(handler) = target_event_handler.as_ref() {
            handler.abort();
        }
        let iframe_channel_handler = self.iframe_channel_handler.read().await;
        if let Some(handler) = iframe_channel_handler.as_ref() {
            handler.abort();
        }
    }

    pub async fn on_frame_attached(
        &self,
        params: &FrameAttached,
        session_id: &SessionId,
    ) -> Result<()> {
        let frame_id = Arc::new(params.frame_id.clone());
        let parent_frame_id = Arc::new(params.parent_frame_id.clone());
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let backend_node_id = conn
            .send(
                "DOM.getFrameOwner",
                &GetFrameOwner::default(&frame_id),
                Some(session_id),
            )
            .await?
            .result_as::<GetFrameOwnerResponse>()?
            .backend_node_id;

        let target = self.get_target(&frame_id).await;

        match target {
            Some(target) => {
                let frame_inner = FrameInner::new(
                    Arc::downgrade(&target),
                    frame_id.clone(),
                    Some(parent_frame_id.clone()),
                    Some(backend_node_id),
                );
                self.add_frame_inner(Arc::new(frame_inner)).await;
            }
            None => {
                let parent_target = self.get_target(&parent_frame_id).await;
                match parent_target {
                    Some(parent_target) => {
                        let frame_inner = FrameInner::new(
                            Arc::downgrade(&parent_target),
                            frame_id.clone(),
                            Some(parent_frame_id.clone()),
                            Some(backend_node_id),
                        );
                        self.add_frame_inner(Arc::new(frame_inner)).await;
                    }
                    //It's impossible for parent target to not exist, cuz  this event triggers in parent target only.
                    None => {
                        //But actually should print if it's actually happens, cuz of UB of CDP.
                        //I can imagine situation as CDP can create child target before parent.
                        println!("Parent target not found for frame: {:?}", frame_id);
                    }
                }
            }
        }

        let parent_frame_inner = self.get_frame_inner(&parent_frame_id).await;
        match parent_frame_inner {
            Some(parent_frame_inner) => {
                parent_frame_inner.add_child_frame(frame_id.clone()).await;
            }
            None => {}
        }

        Ok(())
    }

    pub async fn on_target_created(&self, params: &TargetCreated) -> Result<()> {
        let target_info = params.target_info.clone();
        let target_type = target_info.target_type;
        match target_type.as_str() {
            "iframe" | "page" | "webview" | "tab" => {}
            _ => return Ok(()),
        };

        let target_id = Arc::new(target_info.target_id);

        let browser_context_id = target_info
            .browser_context_id
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| "default".to_string())
            .to_string();

        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let session_id = match conn
            .send(
                "Target.attachToTarget",
                &AttachToTarget::default(&target_id),
                None,
            )
            .await
        {
            Ok(response) => response.result_as::<AttachToTargetResponse>()?.session_id,
            Err(e) => {
                println!("Error on attach to target: {:?}", e);
                return Err(e);
            }
        };

        match conn
            .send("Page.enable", &PageEnable::default(), Some(&session_id))
            .await
        {
            Ok(response) => response,
            Err(e) => {
                println!("Error on enable page: {:?}", e);
                return Err(e);
            }
        };

        let frame_tree = match conn
            .send(
                "Page.getFrameTree",
                &GetFrameTree::default(),
                Some(&session_id),
            )
            .await
        {
            Ok(response) => response.result_as::<GetFrameTreeResponse>()?.frame_tree,
            Err(e) => {
                println!("Error on get frame tree: {:?}", e);
                return Err(e);
            }
        };

        // let parent_id = frame_tree.frame.parent_id;
        let parent_id = match frame_tree.frame.parent_id {
            Some(parent_id) => Some(Arc::new(parent_id)),
            None => None,
        };

        let target = Target::new(
            Arc::downgrade(&conn),
            target_id.clone(),
            parent_id.clone(),
            target_type.clone(),
            browser_context_id,
            Arc::new(session_id),
        );

        // If CDP with it's undefined behavior creates sometime child target before parent, we need to await for parent target to be created
        // So, we send iframe target id to iframe channel to be handled later
        if target_type == "iframe" {
            if let Some(parent) = self.get_target(&parent_id.as_ref().unwrap()).await {
                self.add_target(target, Some(Arc::downgrade(&parent))).await;
                //iframe might be parent for another iframe as well
                let _ = self.send_to_iframe_channel(target_id.clone()).await;

                let target = self.get_target(&target_id).await.unwrap();
                let target_id = target.target_id();
                let frame_inner = self.get_frame_inner(&target_id).await;
                match frame_inner {
                    Some(frame_inner) => {
                        frame_inner.init_as_target(Arc::downgrade(&target)).await;
                    }
                    None => {
                        let frame = FrameInner::new(
                            Arc::downgrade(&target),
                            target_id.clone(),
                            parent_id.clone(),
                            None,
                        );
                        self.add_frame_inner(Arc::new(frame)).await;
                    }
                }

                let parent_frame_inner = self.get_frame_inner(&parent_id.as_ref().unwrap()).await;
                match parent_frame_inner {
                    Some(parent_frame_inner) => {
                        parent_frame_inner.add_child_frame(target_id).await;
                    }
                    None => {}
                }
            } else {
                self.pending_iframes.insert(
                    (parent_id.as_ref().unwrap().clone(), target_id.clone()),
                    target,
                );
            }
        } else {
            self.add_target(target, None).await;
            let _ = self.send_to_iframe_channel(target_id.clone()).await;

            let target = self.get_target(&target_id).await.unwrap();
            let target_id = target.target_id();
            let frame = FrameInner::new(Arc::downgrade(&target), target_id.clone(), None, None);
            self.add_frame_inner(Arc::new(frame)).await;

            let mut pending_targets = self.pending_targets.write().await;
            if let Some(sender) = pending_targets.remove(&target_id) {
                let _ = sender.send(Arc::downgrade(&target));
            }
        }

        Ok(())
    }

    async fn send_to_iframe_channel(&self, target_id: Arc<TargetId>) -> Result<()> {
        if let Some(tx) = self.iframe_channel.read().await.as_ref() {
            let _ = tx.send(target_id).await;
        }
        Ok(())
    }

    pub async fn on_frame_detached(&self, params: &FrameDetached) -> Result<()> {
        let frame_id = Arc::new(params.frame_id.clone());
        let target = self.get_target(&frame_id).await;
        match target {
            Some(_) => {}
            None => {
                //This event triggers if frame was init as target itself, so if target exists we should remove it, but if not, we remove frame_inner.
                let frame_inner = self.get_frame_inner(&frame_id).await;
                match frame_inner {
                    Some(frame_inner) => {
                        let parent_frame_inner = frame_inner.parent_frame().await;
                        match parent_frame_inner {
                            Some(parent_frame_inner) => {
                                parent_frame_inner
                                    .remove_child_frame(frame_id.clone())
                                    .await;
                            }
                            None => {}
                        }
                    }
                    None => {}
                }
                self.frame_inners.remove(&frame_id);
            }
        }
        Ok(())
    }

    pub async fn on_target_destroyed(&self, params: &TargetDestroyed) -> Result<()> {
        let target_id = Arc::new(params.target_id.clone());
        if let Some((_, target)) = self.targets.remove(&target_id) {
            let frame_inner = self.get_frame_inner(&target_id).await;
            match frame_inner {
                Some(frame_inner) => {
                    let parent_frame_inner = frame_inner.parent_frame().await;
                    match parent_frame_inner {
                        Some(parent_frame_inner) => {
                            parent_frame_inner
                                .remove_child_frame(target_id.clone())
                                .await;
                        }
                        None => {}
                    }
                }
                None => {}
            }
            self.frame_inners.remove(&target_id);
            if let Ok(target) = Arc::try_unwrap(target) {
                let _ = target.shutdown().await;
            }
        }
        Ok(())
    }

    pub async fn on_target_crashed(&self, params: &TargetCrashed) -> Result<()> {
        let target_id = Arc::new(params.target_id.clone());
        if let Some((_, target)) = self.targets.remove(&target_id) {
            let frame_inner = self.get_frame_inner(&target_id).await;
            match frame_inner {
                Some(frame_inner) => {
                    let parent_frame_inner = frame_inner.parent_frame().await;
                    match parent_frame_inner {
                        Some(parent_frame_inner) => {
                            parent_frame_inner
                                .remove_child_frame(target_id.clone())
                                .await;
                        }
                        None => {}
                    }
                }
                None => {}
            }
            self.frame_inners.remove(&target_id);
            if let Ok(target) = Arc::try_unwrap(target) {
                let _ = target.shutdown().await;
            }
        }
        Ok(())
    }

    async fn create_target(
        &self,
        browser_context_id: Option<&BrowserContextID>,
    ) -> Result<(Weak<Target>, Arc<TargetId>)> {
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let (sender, receiver) = oneshot::channel();

        //This lock guarantees that we will put target_id into pending_targets
        //before we receive on_target_created event that will remove it from pending_targets.
        //Dashmap doesn't lock so it has low chance that on_target_created will be called before we put target_id into pending_targets.
        //We drop lock after we put target_id into pending_targets, so on_target_created can remove it from pending_targets only it were placed there.
        let mut pending_targets = self.pending_targets.write().await;

        let response = match conn
            .send(
                "Target.createTarget",
                &CreateTarget::default(browser_context_id),
                None,
            )
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                println!("Error on create target: {:?}", e);
                return Err(e);
            }
        };
        let target_id = match response.result_as::<CreateTargetResponse>() {
            Ok(resp) => Arc::new(resp.target_id),
            Err(e) => {
                println!("Error on create target: {:?}", e);
                return Err(e);
            }
        };

        pending_targets.insert(target_id.clone(), sender);
        drop(pending_targets);

        let target = match receiver.await {
            Ok(target) => target,
            Err(e) => {
                println!("Error on create target: {:?}", e);
                return Err(anyhow!("Failed to receive target: {:?}", e));
            }
        };

        Ok((target, target_id))
    }

    pub async fn create_page(&self, browser_context_id: Option<&BrowserContextID>) -> Result<Page> {
        let (_, target_id) = match self.create_target(browser_context_id).await {
            Ok((target, target_id)) => (target, target_id),
            Err(e) => {
                println!("Error on create_page: {:?}", e);
                return Err(e);
            }
        };
        let frame_inner = match self.get_frame_inner(&target_id).await {
            Some(frame_inner) => frame_inner,
            None => {
                println!("Frame inner not found: {:?}", target_id);
                return Err(anyhow!("Frame inner not found: {:?}", target_id));
            }
        };
        let page = Page::new(frame_inner);
        Ok(page)
    }

    pub async fn create_browser_context(
        &self,
        proxy: Option<&str>,
        proxy_bypass_list: Option<&str>,
    ) -> Result<Arc<BrowserContext>> {
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let mut context_builder = CreateBrowserContext::new();

        if let Some(proxy) = proxy {
            context_builder = context_builder.proxy_server(proxy);
        }
        if let Some(proxy_bypass_list) = proxy_bypass_list {
            context_builder = context_builder.proxy_bypass_list(proxy_bypass_list);
        }

        let response = match conn
            .send(
                "Target.createBrowserContext",
                &context_builder.build(),
                None,
            )
            .await
        {
            Ok(resp) => resp,
            Err(e) => return Err(e),
        };

        let browser_context_id: BrowserContextID =
            match response.result_as::<CreateBrowserContextResponse>() {
                Ok(resp) => resp.browser_context_id,
                Err(e) => return Err(e),
            };

        let browser_context = Arc::new(BrowserContext::new(
            browser_context_id.clone(),
            Arc::downgrade(&conn),
        ));

        self.browser_contexts
            .insert(browser_context_id.clone(), browser_context.clone());
        Ok(browser_context)
    }

    pub async fn close_browser_context(&self, browser_context_id: &BrowserContextID) -> Result<()> {
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let _ = match conn
            .send(
                "Target.disposeBrowserContext",
                &DisposeBrowserContext::default(browser_context_id),
                None,
            )
            .await
        {
            Ok(resp) => resp,
            Err(e) => return Err(e),
        };

        self.browser_contexts.remove(browser_context_id);

        Ok(())
    }
}
