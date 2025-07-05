use std::collections::HashMap;
use tokio::time::Duration;

use super::cdp::element::Element as CdpElement;

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum Element {
    CDP(CdpElement),
    // BiDi(BiDiElement),
}

impl Element {
    pub async fn query_selector(&self, query: &str) -> Result<Element> {
        match self {
            Self::CDP(element) => {
                let result = element.query_selector(query).await?;
                Ok(Element::CDP(result))
            }
        }
    }

    pub async fn query_selector_all(&self, query: &str) -> Result<Vec<Element>> {
        match self {
            Self::CDP(element) => {
                let result = element.query_selector_all(query).await?;
                Ok(result.into_iter().map(Element::CDP).collect())
            }
        }
    }

    // pub async fn wait_for_selector(
    //     &self,
    //     query: &str,
    //     timeout: Option<Duration>,
    //     delay: Option<Duration>,
    // ) -> Result<Option<Element>> {
    //     match self {
    //         Self::CDP(element) => {
    //             let result = element.wait_for_selector(query, timeout, delay).await?;
    //             Ok(result.map(Element::CDP))
    //         }
    //     }
    // }

    pub async fn wait_for_selector(
        &self,
        query: &str,
        timeout: Option<Duration>,
    ) -> Result<Element> {
        match self {
            Self::CDP(element) => {
                let result = element.wait_for_selector(query, timeout).await?;
                Ok(Element::CDP(result))
            }
        }
    }

    pub async fn click(&self) -> Result<()> {
        match self {
            Self::CDP(element) => element.click().await,
            // Self::BiDi(element) => element.click().await,
        }
    }

    pub async fn attributes(&self) -> Result<HashMap<String, String>> {
        match self {
            Self::CDP(element) => element.attributes().await,
            // Self::BiDi(element) => element.attributes().await,
        }
    }

    pub async fn type_text(&self, text: &str, delay: Option<u64>) -> Result<()> {
        match self {
            Self::CDP(element) => element.type_text(text, delay).await,
            // Self::BiDi(element) => element.type_text(text, delay).await,
        }
    }

    pub async fn upload_file(
        &self,
        file_paths: Vec<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        match self {
            Self::CDP(element) => element.upload_file(file_paths, timeout).await,
            // Self::BiDi(element) => element.upload_file(file_paths, timeout).await,
        }
    }

    pub async fn screenshot(
        &self,
        save_path: Option<&str>,
        format: Option<&str>,
        quality: Option<u64>,
        full_page: Option<bool>,
    ) -> Result<String> {
        match self {
            Self::CDP(element) => {
                element
                    .screenshot(save_path, format, quality, full_page)
                    .await
            } // Self::BiDi(element) => element.screenshot(save_path, format, quality, full_page).await,
        }
    }
}
