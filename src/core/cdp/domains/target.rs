use super::browser::BrowserContextID;
use super::page::FrameId;
use serde::{Deserialize, Serialize};

pub type SessionId = String;
pub type TargetId = String;

///Deserializable structs
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    pub target_id: TargetId,
    #[serde(rename = "type")]
    pub target_type: String,
    pub title: String,
    pub url: String,
    pub attached: bool,
    pub opener_id: Option<TargetId>,
    pub can_access_opener: Option<bool>,
    pub opener_frame_id: Option<FrameId>,
    pub browser_context_id: Option<BrowserContextID>,
    pub subtype: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetBrowserContexts {
    pub browser_context_ids: Vec<BrowserContextID>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttachToTargetResponse {
    pub session_id: SessionId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateTargetResponse {
    pub target_id: TargetId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateBrowserContextResponse {
    pub browser_context_id: BrowserContextID,
}

///Deserializable events
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetCreated {
    pub target_info: TargetInfo,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetDestroyed {
    pub target_id: TargetId,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetCrashed {
    pub target_id: TargetId,
    pub status: String,
    pub error_code: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DetachedFromTarget {
    pub session_id: SessionId,
}

///Serializable structs for requests
#[derive(Serialize)]
pub struct CreateBrowserContext<'a> {
    #[serde(rename = "disposeOnDetach")]
    pub dispose_on_detach: bool,
    #[serde(rename = "proxyServer", skip_serializing_if = "Option::is_none")]
    pub proxy_server: Option<&'a str>,
    #[serde(rename = "proxyBypassList", skip_serializing_if = "Option::is_none")]
    pub proxy_bypass_list: Option<&'a str>,
    #[serde(
        rename = "originsWithUniversalNetworkAccess",
        skip_serializing_if = "Option::is_none"
    )]
    pub origins_with_universal_network_access: Option<&'a [&'a str]>,
}

impl<'a> CreateBrowserContext<'a> {
    pub fn default() -> Self {
        Self {
            dispose_on_detach: true,
            proxy_server: None,
            proxy_bypass_list: None,
            origins_with_universal_network_access: None,
        }
    }

    pub fn new() -> Self {
        Self {
            dispose_on_detach: true,
            proxy_server: None,
            proxy_bypass_list: None,
            origins_with_universal_network_access: None,
        }
    }

    pub fn proxy_server(mut self, proxy_server: &'a str) -> Self {
        self.proxy_server = Some(proxy_server);
        self
    }

    pub fn proxy_bypass_list(mut self, proxy_bypass_list: &'a str) -> Self {
        self.proxy_bypass_list = Some(proxy_bypass_list);
        self
    }

    pub fn origins_with_universal_network_access(
        mut self,
        origins_with_universal_network_access: &'a [&'a str],
    ) -> Self {
        self.origins_with_universal_network_access = Some(origins_with_universal_network_access);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct DisposeBrowserContext<'a> {
    #[serde(rename = "browserContextId")]
    pub browser_context_id: &'a BrowserContextID,
}

impl<'a> DisposeBrowserContext<'a> {
    pub fn default(browser_context_id: &'a BrowserContextID) -> Self {
        Self { browser_context_id }
    }
}

#[derive(Serialize)]
pub struct CreateTarget<'a> {
    #[serde(rename = "url")]
    pub url: &'a str,
    #[serde(rename = "left", skip_serializing_if = "Option::is_none")]
    pub left: Option<u32>,
    #[serde(rename = "top", skip_serializing_if = "Option::is_none")]
    pub top: Option<u32>,
    #[serde(rename = "width", skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(rename = "height", skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(rename = "browserContextId", skip_serializing_if = "Option::is_none")]
    pub browser_context_id: Option<&'a BrowserContextID>,
    #[serde(
        rename = "enableBeginFrameControl",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_begin_frame_control: Option<bool>,
    #[serde(rename = "newWindow", skip_serializing_if = "Option::is_none")]
    pub new_window: Option<bool>,
    #[serde(rename = "background", skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(rename = "for_tab", skip_serializing_if = "Option::is_none")]
    pub for_tab: Option<bool>,
    #[serde(rename = "hidden", skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

impl<'a> CreateTarget<'a> {
    pub fn default(browser_context_id: Option<&'a BrowserContextID>) -> Self {
        Self {
            url: "about:blank",
            left: None,
            top: None,
            width: None,
            height: None,
            browser_context_id,
            enable_begin_frame_control: None,
            new_window: None,
            background: None,
            for_tab: None,
            hidden: None,
        }
    }
}

#[derive(Serialize)]
pub struct CloseTarget<'a> {
    #[serde(rename = "targetId")]
    pub target_id: &'a TargetId,
}

impl<'a> CloseTarget<'a> {
    pub fn default(target_id: &'a TargetId) -> Self {
        Self { target_id }
    }
}

#[derive(Serialize)]
pub struct SetDiscoverTargets {
    #[serde(rename = "discover")]
    pub discover: bool,
}

impl SetDiscoverTargets {
    pub fn default() -> Self {
        Self { discover: true }
    }
}

#[derive(Serialize)]
pub struct AttachToTarget<'a> {
    #[serde(rename = "targetId")]
    pub target_id: &'a TargetId,
    #[serde(rename = "flatten")]
    pub flatten: bool,
}

impl<'a> AttachToTarget<'a> {
    pub fn default(target_id: &'a TargetId) -> Self {
        Self {
            target_id,
            flatten: true,
        }
    }
}

#[derive(Serialize)]
pub struct ActivateTarget<'a> {
    #[serde(rename = "targetId")]
    pub target_id: &'a TargetId,
}

impl<'a> ActivateTarget<'a> {
    pub fn default(target_id: &'a TargetId) -> Self {
        Self { target_id }
    }
}
