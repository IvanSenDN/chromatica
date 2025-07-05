use super::page::FrameId;
use serde::{Deserialize, Serialize};

pub type BrowserContextID = String;

///Deserializable structs
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrowserVersion {
    pub protocol_version: String,
    pub product: String,
    pub revision: String,
    pub user_agent: String,
    pub js_version: String,
}

///Deserializable events
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub guid: String,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadWillBegin {
    pub frame_id: FrameId,
    pub guid: String,
    pub url: String,
    pub suggested_filename: String,
}

///Serializable structs for requests
#[derive(Serialize)]
pub struct CancelDownload<'a> {
    #[serde(rename = "guid")]
    pub guid: Option<&'a str>,
    #[serde(rename = "browserContextId", skip_serializing_if = "Option::is_none")]
    pub browser_context_id: Option<&'a str>,
}

impl<'a> CancelDownload<'a> {
    pub fn default(guid: &'a str) -> Self {
        Self {
            guid: Some(guid),
            browser_context_id: None,
        }
    }

    pub fn new() -> Self {
        Self {
            guid: None,
            browser_context_id: None,
        }
    }

    pub fn guid(mut self, guid: &'a str) -> Self {
        self.guid = Some(guid);
        self
    }

    pub fn browser_context_id(mut self, browser_context_id: &'a str) -> Self {
        self.browser_context_id = Some(browser_context_id);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub enum DownloadBehavior {
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "allowAndName")]
    AllowAndName,
    #[serde(rename = "default")]
    Default,
}

#[derive(Serialize)]
pub struct SetDownloadBehavior<'a> {
    #[serde(rename = "behavior")]
    pub behavior: DownloadBehavior,
    #[serde(rename = "browserContextId", skip_serializing_if = "Option::is_none")]
    pub browser_context_id: Option<&'a str>,
    #[serde(rename = "downloadPath", skip_serializing_if = "Option::is_none")]
    pub download_path: Option<&'a str>,
    #[serde(rename = "eventsEnabled", skip_serializing_if = "Option::is_none")]
    pub events_enabled: Option<bool>,
}

impl<'a> SetDownloadBehavior<'a> {
    pub fn default() -> Self {
        Self {
            behavior: DownloadBehavior::Default,
            browser_context_id: None,
            download_path: None,
            events_enabled: None,
        }
    }

    pub fn new() -> Self {
        Self {
            behavior: DownloadBehavior::Default,
            browser_context_id: None,
            download_path: None,
            events_enabled: None,
        }
    }

    pub fn behavior(mut self, behavior: DownloadBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    pub fn browser_context_id(mut self, browser_context_id: &'a str) -> Self {
        self.browser_context_id = Some(browser_context_id);
        self
    }

    pub fn download_path(mut self, download_path: &'a str) -> Self {
        self.download_path = Some(download_path);
        self
    }

    pub fn events_enabled(mut self, events_enabled: bool) -> Self {
        self.events_enabled = Some(events_enabled);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetVersion {}

impl GetVersion {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetVersionResponse {
    pub protocol_version: String,
    pub product: String,
    pub revision: String,
    pub user_agent: String,
    pub js_version: String,
}
