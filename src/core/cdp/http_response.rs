use super::connection::Connection;
use super::domains::fetch::RequestId as FetchRequestId;
use super::domains::fetch::*;
use super::domains::network::*;
use super::domains::target::*;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    connection: Weak<Connection>,
    session_id: Weak<SessionId>,
    request_id: FetchRequestId,
    resource_type: ResourceType,
    response_status_code: i32,
    response_status_text: String,
    response_headers: Vec<HeaderEntry>,
    url: String,
    method: String,
    response_body: Option<ResponseBody>,
}

impl HttpResponse {
    pub fn new(
        connection: Weak<Connection>,
        session_id: Weak<SessionId>,
        paused_request: RequestPaused,
        response_body: Option<ResponseBody>,
    ) -> Self {
        let request_id = paused_request.request_id;
        let resource_type = paused_request.resource_type;
        let response_status_code = paused_request.response_status_code.unwrap();
        let response_status_text = paused_request.response_status_text.unwrap();
        let response_headers = paused_request.response_headers.unwrap();
        let url = paused_request.request.url;
        let method = paused_request.request.method;
        Self {
            connection,
            session_id,
            request_id,
            resource_type,
            response_status_code,
            response_status_text,
            response_headers,
            url,
            method,
            response_body,
        }
    }

    async fn send<P: Serialize>(&self, method: &str, params: &P) -> Result<()> {
        let Some(conn) = self.connection.upgrade() else {
            return Err(anyhow!("Connection is not available"));
        };

        let session_id = match self.session_id.upgrade() {
            Some(session_id) => session_id,
            None => return Err(anyhow!("Session id is not available")),
        };

        match conn.send(method, params, Some(&session_id)).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn abort(&self) -> Result<()> {
        let fail_request = FailRequest::default(&self.request_id);
        self.send("Fetch.failRequest", &fail_request).await?;
        Ok(())
    }

    pub async fn continue_response(&self) -> Result<()> {
        let continue_request = ContinueRequest::default(&self.request_id);
        self.send("Fetch.continueRequest", &continue_request)
            .await?;
        Ok(())
    }

    pub fn response_body(&self) -> Option<&ResponseBody> {
        self.response_body.as_ref()
    }

    pub fn text(&self) -> Option<String> {
        self.response_body.as_ref().map(|body| {
            if body.base64 {
                STANDARD
                    .decode(body.body.as_bytes())
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .unwrap_or_else(|_| body.body.clone())
            } else {
                body.body.clone()
            }
        })
    }

    pub fn json(&self) -> Option<Value> {
        self.text()
            .and_then(|text| serde_json::from_str(&text).ok())
    }

    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }

    pub fn response_status_code(&self) -> i32 {
        self.response_status_code
    }

    pub fn response_status_text(&self) -> &str {
        &self.response_status_text
    }

    pub fn response_headers(&self) -> HashMap<String, String> {
        self.response_headers
            .iter()
            .map(|h| (h.name.clone(), h.value.clone()))
            .collect()
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn method(&self) -> &str {
        &self.method
    }
}
