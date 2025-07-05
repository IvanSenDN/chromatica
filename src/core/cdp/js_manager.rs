use super::connection::{Connection, EventParams, EventSubscriber};
use super::domains::page::*;
use super::domains::target::*;
use super::file_chooser::FileChooser;
use super::js_dialogs::JsDialog;
use anyhow::{Result, anyhow};
use dashmap::DashSet;
use serde::Serialize;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

#[derive(Debug, Clone)]
pub struct JsManager {
    connection: Weak<Connection>,
    session_ids: DashSet<Arc<SessionId>>,
    js_dialog_sender: broadcast::Sender<JsDialog>,
    file_chooser_sender: broadcast::Sender<FileChooser>,
    dom_sender: broadcast::Sender<()>,
    event_subscriber: Arc<RwLock<Option<Weak<EventSubscriber>>>>,
    event_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
    intercept_file_chooser: Arc<AtomicBool>,
}

impl JsManager {
    pub fn new(connection: Weak<Connection>) -> Arc<Self> {
        let session_ids = DashSet::with_capacity(4);
        let (js_dialog_sender, _) = broadcast::channel(1024);
        let (file_chooser_sender, _) = broadcast::channel(1024);
        let (dom_sender, _) = broadcast::channel(1024);
        let intercept_file_chooser = Arc::new(AtomicBool::new(false));

        Arc::new(Self {
            connection,
            session_ids,
            js_dialog_sender,
            file_chooser_sender,
            dom_sender,
            event_handler: Arc::new(RwLock::new(None)),
            event_subscriber: Arc::new(RwLock::new(None)),
            intercept_file_chooser,
        })
    }

    pub fn connection(&self) -> Result<Arc<Connection>> {
        match self.connection.upgrade() {
            Some(connection) => Ok(connection),
            None => Err(anyhow!("Connection is not available")),
        }
    }

    async fn send<P: Serialize>(
        &self,
        method: &str,
        params: &P,
        session_id: &SessionId,
    ) -> Result<()> {
        let conn = self.connection()?;
        conn.send(method, params, Some(session_id)).await?;
        Ok(())
    }

    pub fn subscribe_to_js_dialogs(&self) -> broadcast::Receiver<JsDialog> {
        self.js_dialog_sender.subscribe()
    }

    pub fn subscribe_to_file_chooser(&self) -> broadcast::Receiver<FileChooser> {
        self.file_chooser_sender.subscribe()
    }

    pub fn subscribe_to_dom_events(&self) -> broadcast::Receiver<()> {
        let receiver = self.dom_sender.subscribe();
        receiver
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        let conn = match self.connection() {
            Ok(connection) => connection,
            Err(e) => return Err(e),
        };

        let js_manager_downgraded = Arc::downgrade(&self);

        let methods = DashSet::with_capacity(18);
        methods.insert("Page.javascriptDialogOpening".to_string());
        methods.insert("Page.fileChooserOpened".to_string());
        methods.insert("DOM.attributeModified".to_string());
        methods.insert("DOM.attributeRemoved".to_string());
        methods.insert("DOM.characterDataModified".to_string());
        methods.insert("DOM.childNodeCountUpdated".to_string());
        methods.insert("DOM.childNodeInserted".to_string());
        methods.insert("DOM.childNodeRemoved".to_string());
        methods.insert("DOM.documentUpdated".to_string());
        methods.insert("DOM.setChildNodes".to_string());
        methods.insert("DOM.distributedNodesUpdated".to_string());
        methods.insert("DOM.inlineStyleInvalidated".to_string());
        methods.insert("DOM.pseudoElementAdded".to_string());
        methods.insert("DOM.pseudoElementRemoved".to_string());
        methods.insert("DOM.scrollableFlagUpdated".to_string());
        methods.insert("DOM.shadowRootPushed".to_string());
        methods.insert("DOM.shadowRootPopped".to_string());
        methods.insert("DOM.topLayerElementsUpdated".to_string());

        let session_ids = DashSet::with_capacity(4);

        let (downgraded_event_subscriber, mut rx) = conn.subscribe(methods, session_ids).await;

        let js_event_handler = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // You can provide extra async close to JavaScript EventEmitter, but, to be honest, you won't win so much.

                // let js_manager_downgraded = js_manager_downgraded.clone();
                // tokio::spawn(async move {
                let js_manager = match js_manager_downgraded.upgrade() {
                    Some(js_manager) => js_manager,
                    // None => return,
                    None => break,
                };
                match &event.params {
                    EventParams::JavascriptDialogOpening(dialog) => {
                        let session_id = event.session_id.as_ref().unwrap();
                        let _ = js_manager.on_js_dialog_opening(session_id, dialog).await;
                    }
                    EventParams::FileChooserOpened(file_chooser) => {
                        let session_id = event.session_id.as_ref().unwrap();
                        let _ = js_manager
                            .on_file_chooser_opened(session_id, file_chooser)
                            .await;
                    }
                    _ => {
                        let _ = js_manager.on_dom_event().await;
                    }
                }
                tokio::task::yield_now().await;
                // });
            }
        });
        let mut event_handler = self.event_handler.write().await;
        *event_handler = Some(js_event_handler);
        let mut event_subscriber = self.event_subscriber.write().await;
        *event_subscriber = Some(downgraded_event_subscriber);

        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut event_handler = self.event_handler.write().await;
        if let Some(handler) = event_handler.as_mut() {
            handler.abort();
        }
    }

    pub async fn add_session(&self, session_id: Arc<SessionId>) -> Result<()> {
        self.session_ids.insert(session_id.clone());
        let mut event_subscriber = self.event_subscriber.write().await;
        if let Some(event_subscriber) = event_subscriber.as_mut() {
            if let Some(event_subscriber) = event_subscriber.upgrade() {
                event_subscriber.add_session(session_id.clone()).await;
            }
        }

        let intercept_file_chooser = self.intercept_file_chooser.load(Ordering::SeqCst);
        if intercept_file_chooser {
            for session_id in self.session_ids.iter() {
                self.send(
                    "Page.setInterceptFileChooserDialog",
                    &SetInterceptFileChooserDialog::default(intercept_file_chooser),
                    &session_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn remove_session(&self, session_id: &Arc<SessionId>) {
        self.session_ids.remove(session_id);
        let mut event_subscriber = self.event_subscriber.write().await;
        if let Some(event_subscriber) = event_subscriber.as_mut() {
            if let Some(event_subscriber) = event_subscriber.upgrade() {
                event_subscriber.remove_session(session_id).await;
            }
        }
    }

    pub async fn on_js_dialog_opening(
        &self,
        session_id: &SessionId,
        event: &JavascriptDialogOpening,
    ) -> Result<()> {
        let conn = match self.connection() {
            Ok(connection) => connection,
            Err(e) => return Err(e),
        };

        let downgrade_connection = Arc::downgrade(&conn);

        let session_id = match self.session_ids.get(session_id) {
            Some(session_id) => Arc::downgrade(&session_id),
            None => return Err(anyhow!("Session id is not available")),
        };

        let dialog = JsDialog::new(downgrade_connection, session_id, &event);
        let _ = self.js_dialog_sender.send(dialog);

        Ok(())
    }

    pub async fn set_intercept_file_chooser(&self, enabled: bool) -> Result<()> {
        self.intercept_file_chooser.store(enabled, Ordering::SeqCst);
        for session_id in self.session_ids.iter() {
            self.send(
                "Page.setInterceptFileChooserDialog",
                &SetInterceptFileChooserDialog::default(enabled),
                &session_id,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn on_file_chooser_opened(
        &self,
        session_id: &SessionId,
        event: &FileChooserOpened,
    ) -> Result<()> {
        let conn = match self.connection() {
            Ok(connection) => connection,
            Err(e) => return Err(e),
        };

        let downgrade_connection = Arc::downgrade(&conn);

        let session_id = match self.session_ids.get(session_id) {
            Some(session_id) => Arc::downgrade(&session_id),
            None => return Err(anyhow!("Session id is not available")),
        };

        let file_chooser = FileChooser::new(downgrade_connection, session_id, &event);
        let _ = self.file_chooser_sender.send(file_chooser);
        Ok(())
    }

    //We don't need any info about event, we just need subscribe to it
    pub async fn on_dom_event(&self) -> Result<()> {
        let _ = self.dom_sender.send(());
        Ok(())
    }
}
