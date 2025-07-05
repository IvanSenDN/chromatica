use super::input::TimeSinceEpoch;

use super::page::FrameId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type RequestId = String;
pub type LoaderId = String;
pub type MonotonicTime = f64;
///https://chromedevtools.github.io/devtools-protocol/tot/Network/#type-ResourceType
pub type ResourceType = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ErrorReason {
    #[serde(rename = "Failed")]
    Failed,
    #[serde(rename = "Aborted")]
    Aborted,
    #[serde(rename = "TimedOut")]
    TimedOut,
    #[serde(rename = "AccessDenied")]
    AccessDenied,
    #[serde(rename = "ConnectionClosed")]
    ConnectionClosed,
    #[serde(rename = "ConnectionReset")]
    ConnectionReset,
    #[serde(rename = "ConnectionRefused")]
    ConnectionRefused,
    #[serde(rename = "ConnectionAborted")]
    ConnectionAborted,
    #[serde(rename = "ConnectionFailed")]
    ConnectionFailed,
    #[serde(rename = "NameNotResolved")]
    NameNotResolved,
    #[serde(rename = "InternetDisconnected")]
    InternetDisconnected,
    #[serde(rename = "AddressUnreachable")]
    AddressUnreachable,
    #[serde(rename = "BlockedByClient")]
    BlockedByClient,
    #[serde(rename = "BlockedByResponse")]
    BlockedByResponse,
}

pub type Headers = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostDataEntry {
    pub bytes: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub url: String,
    // pub url_fragment: Option<String>,
    pub method: String,
    pub headers: HashMap<String, String>,
    // pub has_post_data: Option<bool>,
    pub post_data_entries: Option<Vec<PostDataEntry>>,
    // pub referrer_policy: Option<String>,
    // pub is_link_preload: Option<bool>,
    // pub is_same_site: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub url: String,
    pub status: i32,
    pub status_text: String,
    pub headers: Headers,
    pub mime_type: String,
    pub charset: String,
    pub request_headers: Headers,
    pub connection_reused: bool,
    pub connection_id: i32,
    pub remote_ip_address: String,
    pub remote_port: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CookiePartitionKey {
    pub top_level_site: String,
    pub has_cross_site_ancestor: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<MonotonicTime>,
    pub size: i32,
    pub http_only: bool,
    pub secure: bool,
    pub session: bool,
    pub same_site: Option<String>,
    pub priority: Option<String>,
    pub same_party: bool,
    pub source_scheme: Option<String>,
    pub source_port: i32,
    pub partition_key: Option<CookiePartitionKey>,
    pub partition_key_opaque: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CookieParams<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub url: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub path: Option<&'a str>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<&'a str>,
    pub expires: Option<TimeSinceEpoch>,
    pub priority: Option<&'a str>,
    pub same_party: Option<bool>,
    pub source_scheme: Option<&'a str>,
    pub source_port: Option<i32>,
    pub partition_key: Option<&'a CookiePartitionKey>,
}

impl Cookie {
    // pub fn to_cookie_params(&self) -> serde_json::Value {
    //     serde_json::to_value(CookieParams {
    //         name: &self.name,
    //         value: &self.value,
    //         url: None,
    //         domain: Some(&self.domain),
    //         path: Some(&self.path),
    //         secure: Some(self.secure),
    //         http_only: Some(self.http_only),
    //         same_site: self.same_site.as_ref().map(|s| s.as_str()),
    //         expires: self.expires.map(|t| t as f64),
    //         priority: self.priority.as_ref().map(|s| s.as_str()),
    //         same_party: Some(self.same_party),
    //         source_scheme: self.source_scheme.as_ref().map(|s| s.as_str()),
    //         source_port: Some(self.source_port),
    //         partition_key: self.partition_key.as_ref().map(|pk| pk),
    //     })
    //     .unwrap()
    // }
    pub fn to_cookie_params(&self) -> CookieParams {
        CookieParams {
            name: &self.name,
            value: &self.value,
            url: None,
            domain: Some(&self.domain),
            path: Some(&self.path),
            secure: Some(self.secure),
            http_only: Some(self.http_only),
            same_site: self.same_site.as_ref().map(|s| s.as_str()),
            expires: self.expires.map(|t| t as f64),
            priority: self.priority.as_ref().map(|s| s.as_str()),
            same_party: Some(self.same_party),
            source_scheme: self.source_scheme.as_ref().map(|s| s.as_str()),
            source_port: Some(self.source_port),
            partition_key: self.partition_key.as_ref().map(|pk| pk),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseReceived {
    pub request_id: RequestId,
    pub loader_id: LoaderId,
    pub timestamp: MonotonicTime,
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    pub response: Response,
    pub frame_id: Option<FrameId>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadingFailed {
    pub request_id: RequestId,
    pub error_text: String,
    #[serde(rename = "type")]
    pub recource_type: ResourceType,
}

#[derive(Serialize)]
pub struct NetworkEnable {}

impl NetworkEnable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct NetworkDisable {}

impl NetworkDisable {
    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct NetworkSetCacheDisabled {
    pub cache_disabled: bool,
}

impl NetworkSetCacheDisabled {
    pub fn default(cache_disabled: bool) -> Self {
        Self { cache_disabled }
    }
}

#[derive(Serialize)]
pub struct NetworkSetBypassServiceWorker {
    pub bypass: bool,
}

impl NetworkSetBypassServiceWorker {
    pub fn default(bypass: bool) -> Self {
        Self { bypass }
    }
}

#[derive(Serialize)]
pub struct NetworkSetExtraHTTPHeaders {
    pub headers: Headers,
}

impl NetworkSetExtraHTTPHeaders {
    pub fn default(headers: &Headers) -> Self {
        Self {
            headers: headers.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct GetCookies<'a> {
    pub urls: &'a [String],
}

impl<'a> GetCookies<'a> {
    pub fn default(urls: &'a [String]) -> Self {
        Self { urls }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCookie<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub url: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub path: Option<&'a str>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<&'a str>,
    pub expires: Option<TimeSinceEpoch>,
    pub priority: Option<&'a str>,
    pub same_party: Option<bool>,
    pub source_scheme: Option<&'a str>,
    pub source_port: Option<i32>,
    pub partition_key: Option<&'a CookiePartitionKey>,
}

impl<'a> SetCookie<'a> {
    pub fn default(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            url: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
            expires: None,
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: None,
        }
    }

    pub fn from_cookie(cookie: &'a Cookie) -> Self {
        let mut s = Self::new(&cookie.name, &cookie.value);
        s.domain = Some(&cookie.domain);
        s.path = Some(&cookie.path);
        s.secure = Some(cookie.secure);
        s.http_only = Some(cookie.http_only);
        if let Some(ref ss) = cookie.same_site {
            s.same_site = Some(ss);
        }
        if let Some(exp) = cookie.expires {
            s.expires = Some(exp);
        }
        if let Some(ref prio) = cookie.priority {
            s.priority = Some(prio);
        }
        s.same_party = Some(cookie.same_party);
        if let Some(ref ss) = cookie.source_scheme {
            s.source_scheme = Some(ss);
        }
        s.source_port = Some(cookie.source_port);
        if let Some(pk) = &cookie.partition_key {
            s.partition_key = Some(pk);
        }
        s
    }

    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self {
            name,
            value,
            url: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None,
            expires: None,
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: None,
        }
    }

    pub fn url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    pub fn domain(mut self, domain: &'a str) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn path(mut self, path: &'a str) -> Self {
        self.path = Some(path);
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = Some(secure);
        self
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = Some(http_only);
        self
    }

    pub fn same_site(mut self, same_site: &'a str) -> Self {
        self.same_site = Some(same_site);
        self
    }

    pub fn expires(mut self, expires: TimeSinceEpoch) -> Self {
        self.expires = Some(expires);
        self
    }

    pub fn priority(mut self, priority: &'a str) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn same_party(mut self, same_party: bool) -> Self {
        self.same_party = Some(same_party);
        self
    }

    pub fn source_scheme(mut self, source_scheme: &'a str) -> Self {
        self.source_scheme = Some(source_scheme);
        self
    }

    pub fn source_port(mut self, source_port: i32) -> Self {
        self.source_port = Some(source_port);
        self
    }

    pub fn partition_key(mut self, partition_key: &'a CookiePartitionKey) -> Self {
        self.partition_key = Some(partition_key);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct SetCookies<'a> {
    pub cookies: &'a Vec<CookieParams<'a>>,
}

impl<'a> SetCookies<'a> {
    pub fn default(cookies: &'a Vec<CookieParams<'a>>) -> Self {
        Self { cookies }
    }
}
