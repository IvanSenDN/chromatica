use super::connection::Connection;
use super::domains::browser::{GetVersion, GetVersionResponse};
use super::domains::emulation::{SetUserAgentOverride, UserAgentMetadata};
use super::domains::target::SessionId;
use anyhow::{Result, anyhow};
use serde::Serialize;
// use std::collections::HashSet;
use dashmap::DashSet;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct UserAgentOverride {
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(rename = "acceptLanguage", skip_serializing_if = "Option::is_none")]
    pub accept_language: Option<String>,
    #[serde(rename = "platform", skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(rename = "userAgentMetadata", skip_serializing_if = "Option::is_none")]
    pub user_agent_metadata: Option<UserAgentMetadata>,
}

impl UserAgentOverride {
    pub fn new(user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: user_agent.into(),
            accept_language: None,
            platform: None,
            user_agent_metadata: None,
        }
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn accept_language(mut self, accept_language: impl Into<String>) -> Self {
        self.accept_language = Some(accept_language.into());
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn user_agent_metadata(mut self, metadata: UserAgentMetadata) -> Self {
        self.user_agent_metadata = Some(metadata);
        self
    }

    pub fn build(self) -> Self {
        Self {
            user_agent: self.user_agent,
            accept_language: self.accept_language,
            platform: self.platform,
            user_agent_metadata: self.user_agent_metadata,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmulationManager {
    connection: Weak<Connection>,
    session_ids: DashSet<Arc<SessionId>>,
    is_default_user_agent: Arc<AtomicBool>,
    user_agent: Arc<RwLock<Option<UserAgentOverride>>>,
}

impl EmulationManager {
    pub fn new(connection: Weak<Connection>) -> Arc<Self> {
        Arc::new(Self {
            connection,
            session_ids: DashSet::with_capacity(4),
            is_default_user_agent: Arc::new(AtomicBool::new(true)),
            user_agent: Arc::new(RwLock::new(None)),
        })
    }

    pub fn connection(&self) -> Option<Arc<Connection>> {
        match self.connection.upgrade() {
            Some(connection) => Some(connection),
            None => None,
        }
    }

    pub async fn send<P: Serialize>(
        &self,
        method: &str,
        params: &P,
        session_id: &SessionId,
    ) -> Result<()> {
        let connection = match self.connection() {
            Some(connection) => connection,
            None => return Err(anyhow!("Connection not found")),
        };
        let _ = connection.send(method, params, Some(session_id)).await;
        Ok(())
    }

    pub async fn add_session(&self, session_id: Arc<SessionId>) -> Result<()> {
        if !self.session_ids.contains(&session_id) {
            self.session_ids.insert(session_id.clone());
        }

        let user_agent = self.user_agent.read().await.clone();
        if let Some(user_agent) = user_agent {
            let mut params = SetUserAgentOverride::new(user_agent.user_agent.clone());
            if let Some(accept_language) = user_agent.accept_language.clone() {
                params = params.accept_language(accept_language);
            }
            if let Some(platform) = user_agent.platform.clone() {
                params = params.platform(platform);
            }
            if let Some(user_agent_metadata) = user_agent.user_agent_metadata.clone() {
                params = params.user_agent_metadata(user_agent_metadata);
            }
            self.send(
                "Emulation.setUserAgentOverride",
                &params.build(),
                &session_id,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn remove_session(&self, session_id: &Arc<SessionId>) {
        self.session_ids.remove(session_id);
    }

    pub async fn user_agent(&self) -> Result<String> {
        let user_agent = self.user_agent.read().await;
        match user_agent.clone() {
            Some(user_agent) => Ok(user_agent.user_agent),
            None => {
                let connection = match self.connection() {
                    Some(connection) => connection,
                    None => return Err(anyhow!("Connection not found")),
                };
                let response = connection
                    .send("Browser.getVersion", &GetVersion::default(), None)
                    .await?;
                let user_agent = response.result_as::<GetVersionResponse>()?.user_agent;
                Ok(user_agent)
            }
        }
    }

    pub async fn set_user_agent(&self, user_agent: UserAgentOverride) -> Result<()> {
        let mut mut_user_agent = self.user_agent.write().await;
        *mut_user_agent = Some(user_agent.clone());
        self.is_default_user_agent.store(false, Ordering::SeqCst);

        for session_id in self.session_ids.iter() {
            let mut params = SetUserAgentOverride::new(user_agent.user_agent.clone());
            if let Some(accept_language) = user_agent.accept_language.clone() {
                params = params.accept_language(accept_language);
            }
            if let Some(platform) = user_agent.platform.clone() {
                params = params.platform(platform);
            }
            if let Some(user_agent_metadata) = user_agent.user_agent_metadata.clone() {
                params = params.user_agent_metadata(user_agent_metadata);
            }
            self.send(
                "Emulation.setUserAgentOverride",
                &params.build(),
                &session_id,
            )
            .await?;
        }
        Ok(())
    }
}
