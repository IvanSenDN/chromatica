use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;
use std::ops::Deref;
use std::sync::Arc;
use tokio::fs;
use tokio::process::Child;

use super::browser::Browser;
use super::cdp::browser::Browser as CdpBrowser;
use super::cdp::connection::Connection as CdpConnection;

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
enum Protocol {
    #[default]
    CDP,
    BiDi,
}

impl Protocol {
    pub fn new(protocol: &str) -> Result<Self> {
        match protocol.to_lowercase().as_str() {
            "cdp" | "" => Ok(Self::CDP),
            "bidi" => Ok(Self::BiDi),
            _ => Err(anyhow!(
                "Invalid protocol type. Must be either 'cdp' or 'bidi'"
            )),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct BrowserConnection {
    #[serde(rename = "Browser")]
    pub browser: String,
    #[serde(rename = "Protocol-Version")]
    pub protocol_version: String,
    #[serde(rename = "User-Agent")]
    pub user_agent: String,
    #[serde(rename = "V8-Version")]
    pub v8_version: String,
    #[serde(rename = "WebKit-Version")]
    pub webkit_version: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub ws_url: String,
}

#[derive(Debug, Default, Deserialize)]
struct BrowserConfig {
    port: u16,
    user_agent: String,
    version: String,
    protocol: Protocol,
}

#[derive(Debug)]
pub struct Chromatica {
    protocol: Protocol,
    browser_config: Option<BrowserConfig>,
    child: Option<Child>,
}

impl Chromatica {
    pub fn new(protocol: Option<&str>) -> Self {
        let protocol = match protocol {
            Some(p) => Protocol::new(p).unwrap(),
            None => Protocol::CDP,
        };
        Self {
            protocol,
            browser_config: None,
            child: None,
        }
    }
    pub async fn connect(&mut self, port: u16, protocol: Option<&str>) -> Result<Browser> {
        let protocol = match protocol {
            Some(p) => Protocol::new(p)?,
            None => Protocol::CDP,
        };
        let debug_ws_url = format!("http://127.0.0.1:{}/json/version", port);

        let client = Client::new();
        let response = client
            .get(&debug_ws_url)
            .header("Content-Type", "application/json")
            .send()
            .await;

        match response {
            Ok(response) => {
                let body: BrowserConnection = response.json().await?;
                let ws_url = body.ws_url;

                match protocol {
                    Protocol::CDP => {
                        let conn = CdpConnection::connect(&ws_url).await?;
                        let target_manager = conn.target_manager().unwrap().clone();
                        target_manager.init().await?;
                        let browser = CdpBrowser::new(conn);
                        Ok(Browser::CDP(browser))
                    }
                    Protocol::BiDi => {
                        // TODO: Implement BiDi connection
                        // todo!("Implement BiDi connection")
                        anyhow::bail!("BiDi connection not implemented yet")
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
}
