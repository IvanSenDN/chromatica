// use super::domains::dom::{
//     AttributeModified, AttributeRemoved, CharacterDataModified, ChildNodeCountUpdated,
//     ChildNodeInserted, ChildNodeRemoved, DistributedNodesUpdated, DocumentUpdated,
//     InlineStyleInvalidated, PseudoElementAdded, PseudoElementRemoved, ScrollableFlagUpdated,
//     SetChildNodes, ShadowRootPopped, ShadowRootPushed, TopLayerElementsUpdated,
// };
use super::domains::dom_storage::DomStorageItemAdded;
use super::domains::fetch::{AuthRequired, RequestPaused};
use super::domains::network::LoadingFailed;
use super::domains::page::{
    FileChooserOpened, FrameAttached, FrameDetached, JavascriptDialogOpening, LifecycleEvent,
};
use super::domains::target::{SessionId, TargetCrashed, TargetCreated, TargetDestroyed};

use super::target_manager::TargetManager;

use anyhow::{Result, anyhow};
use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
// use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::{RwLock, mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::protocol::frame::Utf8Bytes;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: usize,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<ErrorResponse>,
}

impl Response {
    pub fn result_as<T: for<'de> Deserialize<'de>>(self) -> Result<T> {
        match self.result {
            Some(value) => Ok(serde_json::from_value(value)?),
            None => Err(anyhow!(self.error.unwrap().message)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum EventParams {
    TargetCreated(TargetCreated),
    TargetDestroyed(TargetDestroyed),
    TargetCrashed(TargetCrashed),
    FrameAttached(FrameAttached),
    FrameDetached(FrameDetached),
    LifecycleEvent(LifecycleEvent),
    JavascriptDialogOpening(JavascriptDialogOpening),
    FileChooserOpened(FileChooserOpened),
    RequestPaused(RequestPaused),
    AuthRequired(AuthRequired),
    LoadingFailed(LoadingFailed),
    Value(Value),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub method: String,
    pub params: EventParams,
    pub session_id: Option<SessionId>,
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = Value::deserialize(deserializer)?;
        let params = value
            .as_object_mut()
            .and_then(|obj| obj.remove("params"))
            .ok_or_else(|| serde::de::Error::custom("missing params"))?;
        let session_id = value
            .as_object_mut()
            .and_then(|obj| obj.remove("sessionId"))
            .and_then(|v| serde_json::from_value(v).ok());
        let method = value
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::custom("missing method"))?;

        let params = match method {
            "Target.targetCreated" => EventParams::TargetCreated(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Target.targetDestroyed" => EventParams::TargetDestroyed(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Target.targetCrashed" => EventParams::TargetCrashed(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Page.frameAttached" => EventParams::FrameAttached(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Page.frameDetached" => EventParams::FrameDetached(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Page.lifecycleEvent" => EventParams::LifecycleEvent(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Page.javascriptDialogOpening" => EventParams::JavascriptDialogOpening(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Page.fileChooserOpened" => EventParams::FileChooserOpened(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Fetch.requestPaused" => EventParams::RequestPaused(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Fetch.authRequired" => EventParams::AuthRequired(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "Network.loadingFailed" => EventParams::LoadingFailed(
                serde_json::from_value(params).map_err(serde::de::Error::custom)?,
            ),
            "DOM.attributeModified" => EventParams::Value(params),
            "DOM.attributeRemoved" => EventParams::Value(params),
            "DOM.characterDataModified" => EventParams::Value(params),
            "DOM.childNodeInserted" => EventParams::Value(params),
            "DOM.childNodeRemoved" => EventParams::Value(params),
            "DOM.distributedNodesUpdated" => EventParams::Value(params),
            "DOM.inlineStyleInvalidated" => EventParams::Value(params),
            "DOM.pseudoElementAdded" => EventParams::Value(params),
            "DOM.pseudoElementRemoved" => EventParams::Value(params),
            "DOM.shadowRootPushed" => EventParams::Value(params),
            "DOM.shadowRootPopped" => EventParams::Value(params),
            "DOM.documentUpdated" => EventParams::Value(params),
            "DOM.topLayerElementsUpdated" => EventParams::Value(params),
            "DOM.scrollableFlagUpdated" => EventParams::Value(params),
            "DOM.childNodeCountUpdated" => EventParams::Value(params),
            "DOM.setChildNodes" => EventParams::Value(params),
            _ => EventParams::Value(params),
        };
        Ok(Event {
            method: method.to_string(),
            params,
            session_id,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Request<'a, P: Serialize> {
    pub id: usize,
    pub method: &'a str,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<&'a SessionId>,
    pub params: &'a P,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum IncomingMessage {
    Response(Response),
    Event(Event),
}

#[derive(Debug)]
pub struct EventSubscriber {
    methods: DashSet<String>,
    session_ids: DashSet<Arc<SessionId>>,
    tx: mpsc::UnboundedSender<Arc<Event>>,
}

impl EventSubscriber {
    pub fn new(
        methods: DashSet<String>,
        session_ids: DashSet<Arc<SessionId>>,
        tx: mpsc::UnboundedSender<Arc<Event>>,
    ) -> Self {
        Self {
            methods,
            session_ids,
            tx,
        }
    }

    pub fn methods(&self) -> &DashSet<String> {
        &self.methods
    }

    pub async fn session_ids(&self) -> &DashSet<Arc<SessionId>> {
        &self.session_ids
    }

    pub async fn send(
        &self,
        event: Arc<Event>,
        // method: &str,
        // session_id: &Option<SessionId>,
    ) -> Result<()> {
        if !self.methods.contains(&event.method) {
            return Ok(());
        }
        if let Some(session_id) = &event.session_id {
            if !self.session_ids.is_empty() && !self.session_ids.contains(session_id) {
                return Ok(());
            }
        }
        match self.tx.send(event) {
            Ok(_) => Ok(()),
            Err(e) => {
                //Leads to drop, cuz only Connection owns Arc, subscribe method returns Weak;
                //Current impl won't drop EventSubscriber if tx won't send something or connection is disconnecting, but it's not a problem in runtime;
                return Err(anyhow!("Failed to send event: {}", e));
            }
        }
    }

    pub async fn add_session(&self, session_id: Arc<SessionId>) {
        self.session_ids.insert(session_id);
    }

    pub async fn remove_session(&self, session_id: &Arc<SessionId>) {
        self.session_ids.remove(session_id);
    }
}

impl Drop for EventSubscriber {
    fn drop(&mut self) {}
}

#[derive(Debug)]
pub struct Connection {
    sender: mpsc::UnboundedSender<Utf8Bytes>,
    next_id: AtomicUsize,
    target_manager: Option<Arc<TargetManager>>,
    next_subscriber_id: AtomicUsize,
    event_subscribers: DashMap<usize, Arc<EventSubscriber>>,
    response_waiters: DashMap<usize, oneshot::Sender<Response>>,
    sender_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    receiver_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    dispatcher_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    is_disconnecting: AtomicBool,
    event_dispatcher: mpsc::UnboundedSender<Arc<Event>>,
}

impl Connection {
    pub async fn connect(ws_url: &str) -> Result<Arc<Self>> {
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut ws_sink, mut ws_stream) = ws_stream.split();
        let (sender, mut rx) = mpsc::unbounded_channel::<Utf8Bytes>();
        let next_id = AtomicUsize::new(0);
        let next_subscriber_id = AtomicUsize::new(0);
        let event_subscribers: DashMap<usize, Arc<EventSubscriber>> = DashMap::with_capacity(1024);
        let response_waiters = DashMap::with_capacity(2048);
        let (event_dispatcher, mut event_rx) = mpsc::unbounded_channel::<Arc<Event>>();

        let conn = Arc::new(Self {
            sender,
            next_id,
            target_manager: None,
            next_subscriber_id,
            event_subscribers,
            response_waiters,
            sender_handle: Arc::new(RwLock::new(None)),
            receiver_handle: Arc::new(RwLock::new(None)),
            dispatcher_handle: Arc::new(RwLock::new(None)),
            is_disconnecting: AtomicBool::new(false),
            event_dispatcher,
        });

        let target_manager = TargetManager::new(conn.clone());
        //SAFETY: REALLY didn't want to do unsafe, but i don't really want to use Mutex or RwLock just to set target_manager and no more reason to use it for safety;
        unsafe {
            let conn_ptr = Arc::as_ptr(&conn) as *mut Self;
            (*conn_ptr).target_manager = Some(Arc::new(target_manager));
        }

        let _sender_handle = tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let Err(e) = ws_sink.send(WsMessage::Text(request)).await {
                    eprintln!("Failed to send request: {}", e);
                }
                // tokio::task::yield_now().await;
            }
        });

        let conn_clone = conn.clone();
        let _dispatcher_handle = tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                // You can provide extra async close to JavaScript EventEmitter, but, to be honest, you won't win so much.

                // let conn_clone = conn_clone.clone();
                // tokio::spawn(async move {
                // let method = event.method.clone();
                // let session_id = event.session_id.clone();

                let futures: Vec<_> = conn_clone
                    .event_subscribers
                    .iter()
                    .map(|entry| {
                        let sender_id = *entry.key();
                        let subscriber = entry.value().clone();
                        let event = event.clone();
                        // let method = method.clone();
                        // let session_id = session_id.clone();

                        async move {
                            match subscriber.send(event).await {
                                Ok(_) => None,
                                Err(_) => Some(sender_id),
                            }
                        }
                    })
                    .collect();

                let dead_subscribers: Vec<_> = join_all(futures)
                    .await
                    .into_iter()
                    .filter_map(|x| x)
                    .collect();

                let unsubscribe_futures: Vec<_> = dead_subscribers
                    .into_iter()
                    .map(|dead_sender_id| {
                        let conn = conn_clone.clone();
                        async move {
                            conn.unsubscribe(&dead_sender_id).await;
                        }
                    })
                    .collect();

                join_all(unsubscribe_futures).await;
                tokio::task::yield_now().await;
                // });
            }
        });

        let conn_clone = conn.clone();
        let event_dispatcher = conn.event_dispatcher.clone();

        let _receiver_handle = tokio::spawn(async move {
            while let Some(message) = ws_stream.next().await {
                // You can provide extra async close to JavaScript EventEmitter, but, to be honest, you won't win so much.

                // let conn_clone = conn_clone.clone();
                // let event_dispatcher = event_dispatcher.clone();
                // tokio::spawn(async move {
                match message {
                    Ok(message) => {
                        let message =
                            match serde_json::from_str::<IncomingMessage>(&message.to_string()) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    println!("Failed to parse message: {:?}", message);
                                    println!("Error: {:?}", e);
                                    continue;
                                }
                            };

                        match message {
                            IncomingMessage::Response(response) => {
                                if let Some((_, tx)) =
                                    conn_clone.response_waiters.remove(&response.id)
                                {
                                    let _ = tx.send(response);
                                }
                            }
                            IncomingMessage::Event(event) => {
                                let event = Arc::new(event);
                                if let Err(e) = event_dispatcher.send(event) {
                                    eprintln!("Failed to dispatch event: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to receive message: {}", e),
                }
                tokio::task::yield_now().await;
                // });
            }
        });

        {
            let mut sender_handle = conn.sender_handle.write().await;
            *sender_handle = Some(_sender_handle);
        }
        {
            let mut receiver_handle = conn.receiver_handle.write().await;
            *receiver_handle = Some(_receiver_handle);
        }
        {
            let mut dispatcher_handle = conn.dispatcher_handle.write().await;
            *dispatcher_handle = Some(_dispatcher_handle);
        }
        Ok(conn)
    }

    pub fn target_manager(&self) -> Option<&Arc<TargetManager>> {
        self.target_manager.as_ref()
    }

    pub async fn send<P: Serialize>(
        &self,
        method: &str,
        params: &P,
        session_id: Option<&SessionId>,
    ) -> Result<Response> {
        if self.is_disconnecting.load(Ordering::SeqCst) {
            return Err(anyhow!("Connection is disconnecting"));
        }
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = Request {
            id,
            method,
            session_id,
            params,
        };

        let serialized_request: Utf8Bytes = serde_json::to_string(&request)?.into();

        let (tx, rx) = oneshot::channel();
        {
            self.response_waiters.insert(id, tx);
        }

        match self.sender.send(serialized_request) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to send request: {}", e),
        }
        match tokio::time::timeout(tokio::time::Duration::from_millis(30000), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Failed to receive response")),
            Err(_) => {
                self.response_waiters.remove(&id);
                Err(anyhow!("Failed to receive response"))
            }
        }
    }

    pub async fn subscribe(
        self: &Arc<Self>,
        methods: DashSet<String>,
        session_ids: DashSet<Arc<SessionId>>,
    ) -> (Weak<EventSubscriber>, mpsc::UnboundedReceiver<Arc<Event>>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let sender_id = self.next_subscriber_id.fetch_add(1, Ordering::SeqCst);
        let event_subscriber = Arc::new(EventSubscriber::new(methods, session_ids, tx));

        let downgraded_subscriber = Arc::downgrade(&event_subscriber);

        self.event_subscribers.insert(sender_id, event_subscriber);

        (downgraded_subscriber, rx)
    }

    pub async fn unsubscribe(&self, sender_id: &usize) {
        self.event_subscribers.remove(sender_id);
    }

    pub async fn disconnect(&self) {
        self.is_disconnecting.store(true, Ordering::SeqCst);

        match tokio::time::timeout(tokio::time::Duration::from_secs(3), async {
            while !self.response_waiters.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                tokio::task::yield_now().await;
            }
        })
        .await
        {
            Ok(_) => (),
            Err(_) => {
                self.response_waiters.clear();
            }
        }

        let mut sender_handle = self.sender_handle.write().await;
        if let Some(handle) = sender_handle.take() {
            handle.abort();
        }
        let mut receiver_handle = self.receiver_handle.write().await;
        if let Some(handle) = receiver_handle.take() {
            handle.abort();
        }
        let mut dispatcher_handle = self.dispatcher_handle.write().await;
        if let Some(handle) = dispatcher_handle.take() {
            handle.abort();
        }
    }
}
