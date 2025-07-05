use super::cdp::domains::fetch::ResponseBody as CdpResponseBody;
use super::cdp::domains::network::ResourceType as CdpResourceType;
use super::cdp::http_response::HttpResponse as CdpHttpResponse;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ResponseBody {
    CDP(CdpResponseBody),
    // BiDi(BiDiResponseBody),
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    CDP(CdpResourceType),
    // BiDi(BiDiResourceType),
}

#[derive(Debug, Clone)]
pub enum HttpResponse {
    CDP(CdpHttpResponse),
    // BiDi(BiDiHttpResponse),
}

impl HttpResponse {
    pub async fn abort(&self) -> Result<()> {
        match self {
            Self::CDP(response) => response.abort().await,
            // Self::BiDi(response) => response.abort().await,
        }
    }

    pub async fn continue_response(&self) -> Result<()> {
        match self {
            Self::CDP(response) => response.continue_response().await,
            // Self::BiDi(response) => response.continue_response().await,
        }
    }

    pub fn response_body(&self) -> Option<ResponseBody> {
        match self {
            Self::CDP(response) => response
                .response_body()
                .map(|body| ResponseBody::CDP(body.clone())),
            // Self::BiDi(response) => response.response_body(),
        }
    }

    pub fn text(&self) -> Option<String> {
        match self {
            Self::CDP(response) => response.text(),
            // Self::BiDi(response) => response.text(),
        }
    }

    pub fn json(&self) -> Option<Value> {
        match self {
            Self::CDP(response) => response.json(),
            // Self::BiDi(response) => response.json(),
        }
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::CDP(response) => ResourceType::CDP(response.resource_type().clone()),
            // Self::BiDi(response) => response.resource_type(),
        }
    }

    pub fn response_status_code(&self) -> i32 {
        match self {
            Self::CDP(response) => response.response_status_code(),
            // Self::BiDi(response) => response.response_status_code(),
        }
    }

    pub fn response_status_text(&self) -> &str {
        match self {
            Self::CDP(response) => response.response_status_text(),
            // Self::BiDi(response) => response.response_status_text(),
        }
    }

    pub fn response_headers(&self) -> HashMap<String, String> {
        match self {
            Self::CDP(response) => response.response_headers(),
            // Self::BiDi(response) => response.response_headers(),
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Self::CDP(response) => response.url(),
            // Self::BiDi(response) => response.url(),
        }
    }
}
