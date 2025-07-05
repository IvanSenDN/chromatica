use super::network::{ErrorReason, Request, RequestId as NetworkRequestId, ResourceType};
use super::page::FrameId;
use serde::{Deserialize, Serialize};

pub type RequestId = String;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum RequestStage {
    #[serde(rename = "Request")]
    Request,
    #[serde(rename = "Response")]
    Response,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthSource {
    #[serde(rename = "Server")]
    Server,
    #[serde(rename = "Proxy")]
    Proxy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthChallenge {
    #[serde(rename = "source")]
    pub source: AuthSource,
    #[serde(rename = "origin")]
    pub origin: String,
    #[serde(rename = "scheme")]
    pub scheme: String,
    #[serde(rename = "realm")]
    pub realm: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum AuthResponse {
    #[serde(rename = "Default")]
    Default,
    #[serde(rename = "CancelAuth")]
    CancelAuth,
    #[serde(rename = "ProvideCredentials")]
    ProvideCredentials,
}

impl<'a> AuthResponse {
    pub fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthChallengeResponse<'a> {
    #[serde(rename = "response")]
    pub response: AuthResponse,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<&'a str>,
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    pub password: Option<&'a str>,
}

impl<'a> AuthChallengeResponse<'a> {
    pub fn default() -> Self {
        Self {
            response: AuthResponse::Default,
            username: None,
            password: None,
        }
    }

    pub fn new(response: AuthResponse) -> Self {
        Self {
            response,
            username: None,
            password: None,
        }
    }

    pub fn username(mut self, username: &'a str) -> Self {
        self.username = Some(username);
        self
    }

    pub fn password(mut self, password: &'a str) -> Self {
        self.password = Some(password);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HeaderEntry {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestPattern<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_pattern: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<ResourceType>,
    pub request_stage: Option<RequestStage>,
}

impl<'a> RequestPattern<'a> {
    pub fn new() -> Self {
        Self {
            url_pattern: None,
            resource_type: None,
            request_stage: None,
        }
    }

    pub fn url_pattern(mut self, url_pattern: &'a str) -> Self {
        self.url_pattern = Some(url_pattern);
        self
    }

    pub fn resource_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn request_stage(mut self, request_stage: RequestStage) -> Self {
        self.request_stage = Some(request_stage);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

///Deserializable responses
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseBody {
    pub body: String,
    #[serde(rename = "base64Encoded")]
    pub base64: bool,
}

///Deserializable events
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestPaused {
    pub request_id: RequestId,
    pub request: Request,
    // pub frame_id: FrameId,
    pub resource_type: ResourceType,
    pub response_error_reason: Option<ErrorReason>,
    pub response_status_code: Option<i32>,
    pub response_status_text: Option<String>,
    pub response_headers: Option<Vec<HeaderEntry>>,
    pub network_id: Option<NetworkRequestId>,
    pub redirected_request_id: Option<RequestId>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthRequired {
    #[serde(rename = "requestId")]
    pub request_id: RequestId,
    #[serde(rename = "request")]
    pub request: Request,
    #[serde(rename = "frameId")]
    pub frame_id: FrameId,
    #[serde(rename = "resourceType")]
    pub resource_type: ResourceType,
    #[serde(rename = "authChallenge")]
    pub auth_challenge: AuthChallenge,
}

///Serializable requests
#[derive(Serialize)]
pub struct ContinueRequest<'a> {
    #[serde(rename = "requestId")]
    pub request_id: &'a RequestId,
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<&'a str>,
    #[serde(rename = "method", skip_serializing_if = "Option::is_none")]
    pub method: Option<&'a str>,
    #[serde(rename = "postData", skip_serializing_if = "Option::is_none")]
    pub post_data: Option<&'a str>,
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HeaderEntry>>,
}

impl<'a> ContinueRequest<'a> {
    // pub fn default(request_id: &'a RequestId) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         request_id,
    //         url: None,
    //         method: None,
    //         post_data: None,
    //         headers: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            url: None,
            method: None,
            post_data: None,
            headers: None,
        }
    }

    pub fn new(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            url: None,
            method: None,
            post_data: None,
            headers: None,
        }
    }

    pub fn url(mut self, url: &'a str) -> Self {
        self.url = Some(url);
        self
    }

    pub fn method(mut self, method: &'a str) -> Self {
        self.method = Some(method);
        self
    }

    pub fn post_data(mut self, post_data: &'a str) -> Self {
        self.post_data = Some(post_data);
        self
    }

    pub fn headers(mut self, headers: Vec<HeaderEntry>) -> Self {
        self.headers = Some(headers);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct ContinueWithAuth<'a> {
    #[serde(rename = "requestId")]
    pub request_id: &'a RequestId,
    #[serde(rename = "authChallengeResponse")]
    pub auth_challenge_response: AuthChallengeResponse<'a>,
}

impl<'a> ContinueWithAuth<'a> {
    // pub fn default(request_id: &'a RequestId) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         request_id,
    //         auth_challenge_response: AuthChallengeResponse::default(),
    //     })
    //     .unwrap()
    // }

    pub fn default(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            auth_challenge_response: AuthChallengeResponse::default(),
        }
    }

    pub fn new(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            auth_challenge_response: AuthChallengeResponse::default(),
        }
    }

    pub fn auth_challenge_response(
        mut self,
        auth_challenge_response: AuthChallengeResponse<'a>,
    ) -> Self {
        self.auth_challenge_response = auth_challenge_response;
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct FetchEnable<'a> {
    #[serde(rename = "patterns")]
    pub patterns: Vec<RequestPattern<'a>>,
    #[serde(rename = "handleAuthRequests", skip_serializing_if = "Option::is_none")]
    pub handle_auth_requests: Option<bool>,
}

impl<'a> FetchEnable<'a> {
    // pub fn default() -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         patterns: vec![
    //             RequestPattern::new()
    //                 .request_stage(RequestStage::Request)
    //                 .build(),
    //             RequestPattern::new()
    //                 .request_stage(RequestStage::Response)
    //                 .build(),
    //         ],
    //         handle_auth_requests: Some(true),
    //     })
    //     .unwrap()
    // }

    pub fn default() -> Self {
        Self {
            patterns: vec![
                RequestPattern::new()
                    .request_stage(RequestStage::Request)
                    .build(),
                RequestPattern::new()
                    .request_stage(RequestStage::Response)
                    .build(),
            ],
            handle_auth_requests: Some(true),
        }
    }

    pub fn new(patterns: Vec<RequestPattern<'a>>) -> Self {
        Self {
            patterns,
            handle_auth_requests: Some(true),
        }
    }

    pub fn handle_auth_requests(mut self, handle_auth_requests: bool) -> Self {
        self.handle_auth_requests = Some(handle_auth_requests);
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct FetchDisable {}

impl FetchDisable {
    // pub fn default() -> serde_json::Value {
    //     serde_json::to_value(Self {}).unwrap()
    // }

    pub fn default() -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct FailRequest<'a> {
    #[serde(rename = "requestId")]
    pub request_id: &'a RequestId,
    #[serde(rename = "errorReason")]
    pub error_reason: ErrorReason,
}

impl<'a> FailRequest<'a> {
    // pub fn default(request_id: &'a RequestId) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         request_id,
    //         error_reason: ErrorReason::Failed,
    //     })
    //     .unwrap()
    // }

    pub fn default(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            error_reason: ErrorReason::Failed,
        }
    }

    pub fn new(request_id: &'a RequestId) -> Self {
        Self {
            request_id,
            error_reason: ErrorReason::Failed,
        }
    }

    pub fn error_reason(mut self, error_reason: ErrorReason) -> Self {
        self.error_reason = error_reason;
        self
    }

    // pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct FulfillRequest<'a> {
    #[serde(rename = "requestId")]
    pub request_id: &'a RequestId,
    #[serde(rename = "responseCode")]
    pub response_code: i32,
    #[serde(rename = "responseHeaders", skip_serializing_if = "Option::is_none")]
    pub response_headers: Option<Vec<HeaderEntry>>,
    #[serde(
        rename = "binaryResponseHeaders",
        skip_serializing_if = "Option::is_none"
    )]
    pub binary_response_headers: Option<&'a str>,
    #[serde(rename = "body", skip_serializing_if = "Option::is_none")]
    pub body: Option<&'a str>,
    #[serde(rename = "responsePhrase", skip_serializing_if = "Option::is_none")]
    pub response_phrase: Option<&'a str>,
}

impl<'a> FulfillRequest<'a> {
    // pub fn default(request_id: &'a RequestId, response_code: i32) -> serde_json::Value {
    //     serde_json::to_value(Self {
    //         request_id,
    //         response_code,
    //         response_headers: None,
    //         binary_response_headers: None,
    //         body: None,
    //         response_phrase: None,
    //     })
    //     .unwrap()
    // }

    pub fn default(request_id: &'a RequestId, response_code: i32) -> Self {
        Self {
            request_id,
            response_code,
            response_headers: None,
            binary_response_headers: None,
            body: None,
            response_phrase: None,
        }
    }

    pub fn new(request_id: &'a RequestId, response_code: i32) -> Self {
        Self {
            request_id,
            response_code,
            response_headers: None,
            binary_response_headers: None,
            body: None,
            response_phrase: None,
        }
    }

    pub fn response_headers(mut self, headers: Vec<HeaderEntry>) -> Self {
        self.response_headers = Some(headers);
        self
    }

    pub fn binary_response_headers(mut self, headers: &'a str) -> Self {
        self.binary_response_headers = Some(headers);
        self
    }

    pub fn body(mut self, body: &'a str) -> Self {
        self.body = Some(body);
        self
    }

    pub fn response_phrase(mut self, phrase: &'a str) -> Self {
        self.response_phrase = Some(phrase);
        self
    }

    //  pub fn build(self) -> serde_json::Value {
    //     serde_json::to_value(self).unwrap()
    // }

    pub fn build(self) -> Self {
        self
    }
}

#[derive(Serialize)]
pub struct GetResponseBody<'a> {
    #[serde(rename = "requestId")]
    pub request_id: &'a RequestId,
}

impl<'a> GetResponseBody<'a> {
    // pub fn default(request_id: &'a RequestId) -> serde_json::Value {
    //     serde_json::to_value(Self { request_id }).unwrap()
    // }

    pub fn default(request_id: &'a RequestId) -> Self {
        Self { request_id }
    }
}
