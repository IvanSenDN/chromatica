use super::cdp::domains::page::{PrintToPDF as CdpPrintToPDF, ScriptIdentifier};
use super::cdp::emulation_manager::UserAgentOverride as CdpUserAgentOverride;
use super::cdp::http_response::HttpResponse as CdpHttpResponse;
use super::cdp::js_dialogs::JsDialog;
use super::cdp::network_manager::{RequestStream, ResponseStream};
use super::cdp::page::Page as CdpPage;
use super::element::Element;
use super::http_response::HttpResponse;

use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub enum Page {
    CDP(CdpPage),
    // BiDi(BiDiPage),
}

impl Page {
    pub async fn url(&self) -> Result<String> {
        match self {
            Self::CDP(page) => page.url().await,
            // Self::BiDi(page) => page.url().await,
        }
    }

    pub async fn default_timeout(&self) -> Duration {
        match self {
            Self::CDP(page) => page.default_timeout().await,
            // Self::BiDi(page) => page.default_timeout().await,
        }
    }

    pub async fn set_default_timeout(&self, timeout: Duration) {
        match self {
            Self::CDP(page) => page.set_default_timeout(timeout).await,
            // Self::BiDi(page) => page.set_default_timeout(timeout).await,
        }
    }

    pub async fn bring_to_front(&self) -> Result<()> {
        match self {
            Self::CDP(page) => page.bring_to_front().await,
            // Self::BiDi(page) => page.bring_to_front().await,
        }
    }

    pub async fn navigate(
        &self,
        url: &str,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        match self {
            Self::CDP(page) => page.navigate(url, wait_until, timeout).await,
            // Self::BiDi(page) => page.navigate(url, wait_until, timeout).await,
        }
    }

    pub async fn reload(&self, wait_until: Option<&str>, timeout: Option<Duration>) -> Result<()> {
        match self {
            Self::CDP(page) => page.reload(wait_until, timeout).await,
            // Self::BiDi(page) => page.reload(wait_until, timeout).await,
        }
    }

    pub async fn wait_for_navigation(
        &self,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        match self {
            Self::CDP(page) => page.wait_for_navigation(wait_until, timeout).await,
            // Self::BiDi(page) => page.wait_for_navigation(wait_until, timeout).await,
        }
    }

    pub async fn close(self) -> Result<()> {
        match self {
            Self::CDP(page) => page.close().await,
            // Self::BiDi(page) => page.close().await,
        }
    }

    pub async fn set_credentials(&self, username: &str, password: &str) -> Result<()> {
        match self {
            Self::CDP(page) => page.set_credentials(username, password).await,
            // Self::BiDi(page) => page.set_credentials(username, password).await,
        }
    }

    pub async fn set_user_agent(&self, user_agent: CdpUserAgentOverride) -> Result<()> {
        match self {
            Self::CDP(page) => page.set_user_agent(user_agent).await,
            // Self::BiDi(page) => page.set_user_agent(user_agent).await,
        }
    }

    pub async fn user_agent(&self) -> Result<String> {
        match self {
            Self::CDP(page) => page.user_agent().await,
            // Self::BiDi(page) => page.user_agent().await,
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
            Self::CDP(page) => page.screenshot(save_path, format, quality, full_page).await, // Self::BiDi(page) => page.capture_screenshot(save_path, format, quality, full_page).await,
        }
    }

    pub async fn print_to_pdf<'a>(
        &self,
        save_path: Option<&str>,
        options: Option<CdpPrintToPDF<'a>>,
    ) -> Result<String> {
        match self {
            Self::CDP(page) => page.print_to_pdf(save_path, options).await,
            // Self::BiDi(page) => page.print_to_pdf(save_path, options).await,
        }
    }

    pub async fn subscribe_to_requests(&self) -> Result<(RequestStream, ResponseStream)> {
        match self {
            Self::CDP(page) => page.subscribe_to_requests().await,
            // Self::BiDi(page) => page.subscribe_to_requests().await,
        }
    }

    pub async fn subscribe_to_js_dialogs(&self) -> Result<broadcast::Receiver<JsDialog>> {
        match self {
            Self::CDP(page) => page.subscribe_to_js_dialogs().await,
            // Self::BiDi(page) => page.subscribe_to_js_dialogs().await,
        }
    }

    pub async fn wait_for_response<F>(
        &self,
        predicate: F,
        timeout: Option<Duration>,
    ) -> Result<HttpResponse>
    where
        F: Fn(&HttpResponse) -> bool + Send + 'static,
    {
        match self {
            Self::CDP(page) => {
                let adapted_predicate =
                    move |resp: &CdpHttpResponse| predicate(&HttpResponse::CDP(resp.clone()));
                let response = page.wait_for_response(adapted_predicate, timeout).await?;
                Ok(HttpResponse::CDP(response))
            }
        }
    }

    pub async fn set_extra_headers(&self, headers: HashMap<&str, &str>) -> Result<()> {
        match self {
            Self::CDP(page) => page.set_extra_headers(headers).await,
            // Self::BiDi(page) => page.set_extra_headers(headers).await,
        }
    }

    pub async fn clear_extra_headers(&self) -> Result<()> {
        match self {
            Self::CDP(page) => page.clear_extra_headers().await,
            // Self::BiDi(page) => page.clear_extra_headers().await,
        }
    }

    pub async fn wait_for_js_dialog<F>(
        &self,
        predicate: F,
        timeout: Option<Duration>,
    ) -> Result<JsDialog>
    where
        F: Fn(&JsDialog) -> bool + Send + 'static,
    {
        match self {
            Self::CDP(page) => page.wait_for_js_dialog(predicate, timeout).await,
            // Self::BiDi(page) => page.wait_for_js_dialog(predicate, timeout).await,
        }
    }

    pub async fn query_selector(&self, query: &str) -> Result<Element> {
        match self {
            Self::CDP(page) => {
                let cdp_element = page.query_selector(query).await?;
                Ok(Element::CDP(cdp_element))
            } // Self::BiDi(page) => page.query_selector(query).await,
        }
    }

    pub async fn query_selector_all(&self, query: &str) -> Result<Vec<Element>> {
        match self {
            Self::CDP(page) => {
                let cdp_elements = page.query_selector_all(query).await?;
                Ok(cdp_elements.into_iter().map(Element::CDP).collect())
            } // Self::BiDi(page) => page.query_selector_all(query).await,
        }
    }

    // pub async fn wait_for_selector(
    //     &self,
    //     query: &str,
    //     timeout: Option<Duration>,
    //     delay: Option<Duration>,
    // ) -> Result<Option<Element>> {
    //     match self {
    //         Self::CDP(page) => {
    //             let cdp_element = page.wait_for_selector(query, timeout, delay).await?;
    //             match cdp_element {
    //                 Some(cdp_element) => Ok(Some(Element::CDP(cdp_element))),
    //                 None => Ok(None),
    //             }
    //         } // Self::BiDi(page) => page.wait_for_selector(query, timeout).await,
    //     }
    // }

    pub async fn wait_for_selector(
        &self,
        query: &str,
        timeout: Option<Duration>,
    ) -> Result<Element> {
        match self {
            Self::CDP(page) => {
                let cdp_element = page.wait_for_selector(query, timeout).await?;
                Ok(Element::CDP(cdp_element))
            } // Self::BiDi(page) => page.wait_for_selector(query, timeout).await,
        }
    }

    pub async fn add_evaluate_on_new_document(&self, script: &str) -> Result<ScriptIdentifier> {
        match self {
            Self::CDP(page) => page.add_evaluate_on_new_document(script).await,
            // Self::BiDi(page) => page.add_evaluate_on_new_document(script).await,
        }
    }

    pub async fn remove_evaluate_on_new_document(
        &self,
        script_identifier: &ScriptIdentifier,
    ) -> Result<()> {
        match self {
            Self::CDP(page) => {
                page.remove_evaluate_on_new_document(script_identifier)
                    .await
            } // Self::BiDi(page) => page.remove_evaluate_on_new_document(script_identifier).await,
        }
    }

    // pub async fn wait_for_dom_storage_item_added(
    //     &self,
    //     key: &str,
    //     timeout: Option<Duration>,
    // ) -> Result<String> {
    //     match self {
    //         Self::CDP(page) => page.wait_for_dom_storage_item_added(key, timeout).await,
    //         // Self::BiDi(page) => page.wait_for_dom_storage_item_added(key, timeout).await,
    //     }
    // }
}
