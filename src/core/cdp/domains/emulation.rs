use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentBrandVersion {
    pub brand: String,
    pub version: String,
}

impl UserAgentBrandVersion {
    pub fn new(brand: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            brand: brand.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAgentMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brands: Option<Vec<UserAgentBrandVersion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_version_list: Option<Vec<UserAgentBrandVersion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wow64: Option<bool>,
}

impl UserAgentMetadata {
    pub fn new() -> Self {
        Self {
            brands: None,
            full_version_list: None,
            platform: None,
            platform_version: None,
            architecture: None,
            model: None,
            mobile: None,
            bitness: None,
            wow64: None,
        }
    }

    pub fn brands(mut self, brands: Vec<UserAgentBrandVersion>) -> Self {
        self.brands = Some(brands);
        self
    }

    pub fn add_brand(mut self, brand: impl Into<String>, version: impl Into<String>) -> Self {
        let mut brands = self.brands.unwrap_or_default();
        brands.push(UserAgentBrandVersion::new(brand, version));
        self.brands = Some(brands);
        self
    }

    pub fn full_version_list(mut self, full_version_list: Vec<UserAgentBrandVersion>) -> Self {
        self.full_version_list = Some(full_version_list);
        self
    }

    pub fn add_full_version(
        mut self,
        brand: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        let mut full_version_list = self.full_version_list.unwrap_or_default();
        full_version_list.push(UserAgentBrandVersion::new(brand, version));
        self.full_version_list = Some(full_version_list);
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn platform_version(mut self, platform_version: impl Into<String>) -> Self {
        self.platform_version = Some(platform_version.into());
        self
    }

    pub fn architecture(mut self, architecture: impl Into<String>) -> Self {
        self.architecture = Some(architecture.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn mobile(mut self, mobile: bool) -> Self {
        self.mobile = Some(mobile);
        self
    }

    pub fn bitness(mut self, bitness: impl Into<String>) -> Self {
        self.bitness = Some(bitness.into());
        self
    }

    pub fn wow64(mut self, wow64: bool) -> Self {
        self.wow64 = Some(wow64);
        self
    }

    pub fn build(self) -> Self {
        Self {
            brands: self.brands,
            full_version_list: self.full_version_list,
            platform: self.platform,
            platform_version: self.platform_version,
            architecture: self.architecture,
            model: self.model,
            mobile: self.mobile,
            bitness: self.bitness,
            wow64: self.wow64,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SetUserAgentOverride {
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(rename = "acceptLanguage", skip_serializing_if = "Option::is_none")]
    pub accept_language: Option<String>,
    #[serde(rename = "platform", skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(rename = "userAgentMetadata", skip_serializing_if = "Option::is_none")]
    pub user_agent_metadata: Option<UserAgentMetadata>,
}

impl SetUserAgentOverride {
    pub fn new(user_agent: String) -> Self {
        Self {
            user_agent,
            accept_language: None,
            platform: None,
            user_agent_metadata: None,
        }
    }

    pub fn accept_language(mut self, accept_language: String) -> Self {
        self.accept_language = Some(accept_language);
        self
    }

    pub fn platform(mut self, platform: String) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn user_agent_metadata(mut self, user_agent_metadata: UserAgentMetadata) -> Self {
        self.user_agent_metadata = Some(user_agent_metadata);
        self
    }

    pub fn build(self) -> Self {
        self
    }
}
