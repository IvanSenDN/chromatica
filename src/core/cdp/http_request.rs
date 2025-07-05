use super::connection::Connection;
use super::domains::fetch::RequestId as FetchRequestId;
use super::domains::fetch::*;
use super::domains::network::*;
use super::domains::target::*;
use super::network_manager::Credentials;
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    connection: Weak<Connection>,
    session_id: Weak<SessionId>,
    protocol_method: String,
    credentials: Option<Credentials>,
    request_id: FetchRequestId,
    resource_type: ResourceType,
    url: String,
    method: String,
    headers: HashMap<String, String>,
    post_data: Option<Vec<PostDataEntry>>,
}

impl HttpRequest {
    pub fn new(
        connection: Weak<Connection>,
        session_id: Weak<SessionId>,
        protocol_method: String,
        request_id: FetchRequestId,
        resource_type: ResourceType,
        request: Request,
        credentials: Option<Credentials>,
    ) -> Self {
        let headers = request
            .headers
            .into_iter()
            .map(|header| (header.0, header.1))
            .collect();
        let post_data = request.post_data_entries.map(|entries| {
            entries
                .into_iter()
                .map(|entry| PostDataEntry { bytes: entry.bytes })
                .collect()
        });
        Self {
            connection,
            session_id,
            protocol_method,
            credentials,
            request_id,
            resource_type,
            url: request.url,
            method: request.method,
            headers,
            post_data,
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
        if self.protocol_method == "Fetch.authRequired" {
            let auth_challenge_response =
                AuthChallengeResponse::new(AuthResponse::CancelAuth).build();
            let continue_with_auth = ContinueWithAuth::new(&self.request_id)
                .auth_challenge_response(auth_challenge_response)
                .build();
            self.send("Fetch.continueWithAuth", &continue_with_auth)
                .await?;
            Ok(())
        } else {
            let fail_request = FailRequest::default(&self.request_id);
            self.send("Fetch.failRequest", &fail_request).await?;
            Ok(())
        }
    }

    pub async fn continue_request(&self) -> Result<()> {
        if self.protocol_method == "Fetch.authRequired" {
            let credentials = self.credentials.as_ref().unwrap();
            let auth_challenge_response =
                AuthChallengeResponse::new(AuthResponse::ProvideCredentials)
                    .username(&credentials.username)
                    .password(&credentials.password)
                    .build();
            let continue_with_auth = ContinueWithAuth::new(&self.request_id)
                .auth_challenge_response(auth_challenge_response)
                .build();
            self.send("Fetch.continueWithAuth", &continue_with_auth)
                .await?;
            Ok(())
        } else {
            // println!("request id: {:?}", self.request_id);
            let continue_request = ContinueRequest::default(&self.request_id);
            self.send("Fetch.continueRequest", &continue_request)
                .await?;
            Ok(())
        }
    }

    pub fn resource_type(&self) -> &ResourceType {
        &self.resource_type
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn post_data(&self) -> Option<&Vec<PostDataEntry>> {
        self.post_data.as_ref()
    }
}
