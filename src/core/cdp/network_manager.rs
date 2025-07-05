use super::connection::{Connection, EventParams, EventSubscriber, Response as CdpResponse};
use super::domains::fetch::*;
use super::domains::network::*;
use super::domains::target::*;
use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use anyhow::{Result, anyhow};
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct RequestStream {
    receiver: broadcast::Receiver<HttpRequest>,
    network_manager: Weak<NetworkManager>,
}

impl RequestStream {
    pub fn new(
        receiver: broadcast::Receiver<HttpRequest>,
        network_manager: Arc<NetworkManager>,
    ) -> Self {
        Self {
            receiver,
            network_manager: Arc::downgrade(&network_manager),
        }
    }

    pub async fn next(&mut self) -> Option<HttpRequest> {
        match self.receiver.recv().await {
            Ok(request) => Some(request),
            _ => None,
        }
    }
}

impl Drop for RequestStream {
    fn drop(&mut self) {
        if let Some(network_manager) = self.network_manager.upgrade() {
            network_manager.set_request_interception(false);
        }
    }
}

pub struct ResponseStream {
    receiver: broadcast::Receiver<HttpResponse>,
    network_manager: Weak<NetworkManager>,
}

impl ResponseStream {
    pub fn new(
        receiver: broadcast::Receiver<HttpResponse>,
        network_manager: Arc<NetworkManager>,
    ) -> Self {
        Self {
            receiver,
            network_manager: Arc::downgrade(&network_manager),
        }
    }

    pub async fn next(&mut self) -> Option<HttpResponse> {
        match self.receiver.recv().await {
            Ok(response) => Some(response),
            _ => None,
        }
    }
}

impl Drop for ResponseStream {
    fn drop(&mut self) {
        if let Some(network_manager) = self.network_manager.upgrade() {
            network_manager.set_request_interception(false);
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkManager {
    connection: Weak<Connection>,
    session_ids: DashSet<Arc<SessionId>>,
    network_handler: Arc<AtomicBool>,
    credentials: Arc<RwLock<Option<Credentials>>>,
    request_sender: broadcast::Sender<HttpRequest>,
    response_sender: broadcast::Sender<HttpResponse>,
    event_subscriber: Arc<RwLock<Option<Weak<EventSubscriber>>>>,
    event_handler: Arc<RwLock<Option<JoinHandle<()>>>>,
    extra_headers: DashMap<String, String>,
    cache_disabled: Arc<AtomicBool>,
    bypass_service_worker: Arc<AtomicBool>,
    //Something with cookies
}

impl NetworkManager {
    pub fn new(connection: Weak<Connection>) -> Arc<Self> {
        let (request_sender, _) = broadcast::channel(1024);
        let (response_sender, _) = broadcast::channel(1024);

        Arc::new(Self {
            connection,
            session_ids: DashSet::with_capacity(4),
            network_handler: Arc::new(AtomicBool::new(false)),
            credentials: Arc::new(RwLock::new(None)),
            request_sender,
            response_sender,
            event_subscriber: Arc::new(RwLock::new(None)),
            event_handler: Arc::new(RwLock::new(None)),
            extra_headers: DashMap::new(),
            cache_disabled: Arc::new(AtomicBool::new(false)),
            bypass_service_worker: Arc::new(AtomicBool::new(false)),
        })
    }

    fn connection(&self) -> Option<Arc<Connection>> {
        match self.connection.upgrade() {
            Some(conn) => Some(conn),
            None => None,
        }
    }

    async fn send<P: Serialize>(
        &self,
        method: &str,
        params: &P,
        session_id: Option<&SessionId>,
    ) -> Result<CdpResponse> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        match conn.send(method, params, session_id).await {
            Ok(response) => Ok(response),
            Err(e) => Err(e),
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return Err(anyhow!("Connection is not available")),
        };

        let network_manager_downgraded = Arc::downgrade(&self);

        let methods = DashSet::with_capacity(2);
        methods.insert("Fetch.authRequired".to_string());
        methods.insert("Fetch.requestPaused".to_string());

        let session_ids = DashSet::with_capacity(4);

        let (downgraded_event_subscriber, mut rx) = conn.subscribe(methods, session_ids).await;

        let _event_handler = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // You can provide extra async close to JavaScript EventEmitter, but, to be honest, you won't win so much.

                // let network_manager_downgraded = network_manager_downgraded.clone();
                // tokio::spawn(async move {
                let network_manager = match network_manager_downgraded.upgrade() {
                    Some(network_manager) => network_manager,
                    // None => return,
                    None => break,
                };
                match &event.params {
                    EventParams::AuthRequired(auth_required) => {
                        let session_id = event.session_id.as_ref().unwrap();
                        let _ = network_manager
                            .on_auth_required(session_id, auth_required)
                            .await;
                    }
                    EventParams::RequestPaused(request_paused) => {
                        let session_id = event.session_id.as_ref().unwrap();

                        if request_paused.response_status_code.is_some() {
                            let _ = network_manager
                                .on_response_received(session_id, request_paused)
                                .await;
                        } else {
                            let _ = network_manager
                                .on_request_paused(session_id, request_paused)
                                .await;
                        }
                    }
                    _ => {}
                }
                tokio::task::yield_now().await;
                // });
            }
        });

        let mut event_subscriber = self.event_subscriber.write().await;
        *event_subscriber = Some(downgraded_event_subscriber);
        let mut event_handler = self.event_handler.write().await;
        *event_handler = Some(_event_handler);

        Ok(())
    }

    pub async fn add_session(&self, session_id: Arc<SessionId>) -> Result<()> {
        if !self.session_ids.contains(&session_id) {
            self.session_ids.insert(session_id.clone());
            let mut event_subscriber = self.event_subscriber.write().await;
            if let Some(event_subscriber) = event_subscriber.as_mut() {
                if let Some(event_subscriber) = event_subscriber.upgrade() {
                    event_subscriber.add_session(session_id.clone()).await;
                }
            }
        }

        let converted_headers: HashMap<String, String> = self
            .extra_headers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        if !converted_headers.is_empty() {
            self.send(
                "Network.setExtraHTTPHeaders",
                &NetworkSetExtraHTTPHeaders::default(&converted_headers),
                Some(&session_id),
            )
            .await?;
        }

        let disabled = self.cache_disabled.load(Ordering::SeqCst);
        if disabled {
            self.send(
                "Network.setCacheDisabled",
                &NetworkSetCacheDisabled::default(disabled),
                Some(&session_id),
            )
            .await?;
        }

        let bypass = self.bypass_service_worker.load(Ordering::SeqCst);
        if bypass {
            self.send(
                "Network.setBypassServiceWorker",
                &NetworkSetBypassServiceWorker::default(bypass),
                Some(&session_id),
            )
            .await?;
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

    pub async fn shutdown(&self) {
        let mut event_handler = self.event_handler.write().await;
        if let Some(event_handler) = event_handler.as_mut() {
            event_handler.abort();
        }
    }

    pub async fn on_request_paused(
        &self,
        session_id: &SessionId,
        request_paused: &RequestPaused,
    ) -> Result<()> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        let session_id = match self.session_ids.get(session_id) {
            Some(session_id) => Arc::downgrade(&session_id),
            None => return Err(anyhow!("Session id is not available")),
        };

        let downgraded_conn = Arc::downgrade(&conn);

        let request = HttpRequest::new(
            downgraded_conn,
            session_id,
            "Fetch.requestPaused".to_string(),
            request_paused.request_id.clone(),
            request_paused.resource_type.clone(),
            request_paused.request.clone(),
            None,
        );

        if !self.network_handler.load(Ordering::SeqCst) {
            match request.continue_request().await {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        let _ = self.request_sender.send(request);

        Ok(())
    }

    pub async fn on_auth_required(
        &self,
        session_id: &SessionId,
        auth_required: &AuthRequired,
    ) -> Result<()> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        let session_id = match self.session_ids.get(session_id) {
            Some(session_id) => Arc::downgrade(&session_id),
            None => return Err(anyhow!("Session id is not available")),
        };

        let downgraded_conn = Arc::downgrade(&conn);

        let credentials = self.credentials.read().await.clone();

        let request = HttpRequest::new(
            downgraded_conn,
            session_id,
            "Fetch.authRequired".to_string(),
            auth_required.request_id.clone(),
            auth_required.resource_type.clone(),
            auth_required.request.clone(),
            credentials,
        );

        if !self.network_handler.load(Ordering::SeqCst) {
            match request.continue_request().await {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        let _ = self.request_sender.send(request);

        Ok(())
    }

    pub async fn on_response_received(
        &self,
        session_id: &SessionId,
        request_paused: &RequestPaused,
    ) -> Result<()> {
        let Some(conn) = self.connection() else {
            return Err(anyhow!("Connection is not available"));
        };

        let downgraded_conn = Arc::downgrade(&conn);

        let session_id = match self.session_ids.get(session_id) {
            Some(session_id) => session_id,
            None => return Err(anyhow!("Session id is not available")),
        };

        let is_redirect = matches!(
            request_paused.response_status_code,
            Some(301) | Some(302) | Some(303) | Some(307) | Some(308)
        );

        let response_body = if is_redirect {
            None
        } else {
            match self
                .send(
                    "Fetch.getResponseBody",
                    &GetResponseBody::default(&request_paused.request_id),
                    Some(&session_id),
                )
                .await
            {
                Ok(response) => Some(response.result_as::<ResponseBody>()?),
                Err(_) => None,
            }
        };

        let response = HttpResponse::new(
            downgraded_conn,
            Arc::downgrade(&session_id),
            request_paused.clone(),
            response_body,
        );

        if !self.network_handler.load(Ordering::SeqCst) {
            match response.continue_response().await {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        let _ = self.response_sender.send(response);

        Ok(())
    }

    pub fn subscribe_to_requests(self: Arc<Self>) -> RequestStream {
        RequestStream::new(self.request_sender.subscribe(), self.clone())
    }

    pub fn subscribe_to_responses(self: Arc<Self>) -> ResponseStream {
        ResponseStream::new(self.response_sender.subscribe(), self.clone())
    }

    pub fn set_request_interception(&self, enabled: bool) {
        self.network_handler.store(enabled, Ordering::SeqCst);
    }

    pub async fn set_credentials(&self, username: &str, password: &str) -> Result<()> {
        let mut creds = self.credentials.write().await;
        *creds = Some(Credentials {
            username: username.to_string(),
            password: password.to_string(),
        });
        Ok(())
    }

    pub async fn set_extra_headers(&self, headers: HashMap<&str, &str>) -> Result<()> {
        let headers: HashMap<String, String> = headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        for session_id in self.session_ids.iter() {
            self.send(
                "Network.setExtraHTTPHeaders",
                &NetworkSetExtraHTTPHeaders::default(&headers),
                Some(&session_id),
            )
            .await?;
        }

        self.extra_headers.clear();
        for (k, v) in headers {
            self.extra_headers.insert(k, v);
        }

        Ok(())
    }

    pub async fn clear_extra_headers(&self) -> Result<()> {
        for session_id in self.session_ids.iter() {
            self.send(
                "Network.setExtraHTTPHeaders",
                &NetworkSetExtraHTTPHeaders::default(&HashMap::new()),
                Some(&session_id),
            )
            .await?;
        }
        self.extra_headers.clear();
        Ok(())
    }

    pub async fn set_cache_disabled(&self, disabled: bool) -> Result<()> {
        self.cache_disabled.store(disabled, Ordering::SeqCst);

        for session_id in self.session_ids.iter() {
            self.send(
                "Network.setCacheDisabled",
                &NetworkSetCacheDisabled::default(disabled),
                Some(&session_id),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn set_bypass_service_worker(&self, bypass: bool) -> Result<()> {
        self.bypass_service_worker.store(bypass, Ordering::SeqCst);
        for session_id in self.session_ids.iter() {
            self.send(
                "Network.setBypassServiceWorker",
                &NetworkSetBypassServiceWorker::default(bypass),
                Some(&session_id),
            )
            .await?;
        }
        Ok(())
    }
}
