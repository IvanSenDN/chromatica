use super::connection::Connection;
use super::domains::page::*;
use super::domains::target::*;
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone)]
pub struct JsDialog {
    connection: Weak<Connection>,
    session_id: Weak<SessionId>,
    is_prompt: bool,
    message: String,
    url: String,
    default_prompt: Option<String>,
    has_browser_handler: bool,
}

impl JsDialog {
    pub fn new(
        connection: Weak<Connection>,
        session_id: Weak<SessionId>,
        event: &JavascriptDialogOpening,
    ) -> Self {
        let is_prompt = event.dialog_type == "prompt";

        Self {
            connection,
            session_id,
            is_prompt,
            message: event.message.clone(),
            url: event.url.clone(),
            default_prompt: event.default_prompt.clone(),
            has_browser_handler: event.has_browser_handler,
        }
    }

    pub fn connection(&self) -> Result<Arc<Connection>> {
        match self.connection.upgrade() {
            Some(connection) => Ok(connection),
            None => return Err(anyhow!("Connection is not available")),
        }
    }

    pub async fn send<P: Serialize>(&self, method: &str, params: &P) -> Result<()> {
        let conn = match self.connection() {
            Ok(connection) => connection,
            Err(e) => return Err(e),
        };
        let session_id = match self.session_id.upgrade() {
            Some(session_id) => session_id,
            None => return Err(anyhow!("Session id is not available")),
        };
        conn.send(method, params, Some(&session_id)).await?;
        Ok(())
    }

    pub fn is_prompt(&self) -> bool {
        self.is_prompt
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn default_prompt(&self) -> &Option<String> {
        &self.default_prompt
    }

    pub fn has_browser_handler(&self) -> bool {
        self.has_browser_handler
    }

    pub async fn accept(&self, prompt: Option<&str>) -> Result<()> {
        if self.is_prompt && prompt.is_none() {
            return Err(anyhow!("Prompt text is required for prompt dialogs"));
        }

        let mut handler = HandleJavaScriptDialog::new(true);

        if self.is_prompt {
            handler = handler.prompt_text(prompt.unwrap());
        }

        self.send("Page.handleJavaScriptDialog", &handler.build())
            .await?;

        Ok(())
    }

    pub async fn dismiss(&self) -> Result<()> {
        let handler = HandleJavaScriptDialog::new(false);
        self.send("Page.handleJavaScriptDialog", &handler.build())
            .await?;
        Ok(())
    }
}
