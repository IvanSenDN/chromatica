use super::domains::page::{PrintToPDF, ScriptIdentifier};
use super::element::Element;
use super::frame_inner::FrameInner;
use super::http_response::HttpResponse;
use super::js_dialogs::JsDialog;
use super::network_manager::{RequestStream, ResponseStream};

use super::emulation_manager::UserAgentOverride;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tokio::sync::broadcast;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct Page {
    frame_inner: Weak<FrameInner>,
}

impl Page {
    pub fn new(frame_inner: Arc<FrameInner>) -> Self {
        Self {
            frame_inner: Arc::downgrade(&frame_inner),
        }
    }

    fn frame_inner(&self) -> Option<Arc<FrameInner>> {
        match self.frame_inner.upgrade() {
            Some(frame_inner) => Some(frame_inner),
            None => None,
        }
    }

    pub async fn default_timeout(&self) -> Duration {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.default_timeout().await,
            None => Duration::from_secs(10),
        }
    }

    pub async fn set_default_timeout(&self, timeout: Duration) {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.set_default_timeout(timeout).await,
            None => (),
        }
    }

    pub async fn bring_to_front(&self) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.bring_to_front().await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn navigate(
        &self,
        url: &str,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.navigate(url, wait_until, timeout).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn reload(&self, wait_until: Option<&str>, timeout: Option<Duration>) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.reload(wait_until, timeout).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn wait_for_navigation(
        &self,
        wait_until: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.wait_for_navigation(wait_until, timeout).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn close(self) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.close().await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn set_credentials(&self, username: &str, password: &str) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.set_credentials(username, password).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn set_user_agent(&self, user_agent: UserAgentOverride) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.set_user_agent(user_agent).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn user_agent(&self) -> Result<String> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.user_agent().await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn screenshot(
        &self,
        save_path: Option<&str>,
        format: Option<&str>,
        quality: Option<u64>,
        full_page: Option<bool>,
    ) -> Result<String> {
        match self.frame_inner() {
            Some(frame_inner) => {
                frame_inner
                    .screenshot(save_path, None, format, quality, full_page)
                    .await
            }
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn print_to_pdf<'a>(
        &self,
        save_path: Option<&str>,
        options: Option<PrintToPDF<'a>>,
    ) -> Result<String> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.print_to_pdf(save_path, options).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn subscribe_to_requests(&self) -> Result<(RequestStream, ResponseStream)> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.subscribe_to_requests().await,
            None => Err(anyhow!("Frame inner is dropped")),
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
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.wait_for_response(predicate, timeout).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn set_extra_headers(&self, headers: HashMap<&str, &str>) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.set_extra_headers(headers).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn clear_extra_headers(&self) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.clear_extra_headers().await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn subscribe_to_js_dialogs(&self) -> Result<broadcast::Receiver<JsDialog>> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.subscribe_to_js_dialogs().await,
            None => Err(anyhow!("Frame inner is dropped")),
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
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.wait_for_js_dialog(predicate, timeout).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn query_selector(&self, query: &str) -> Result<Element> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.query_selector(query, None).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn query_selector_all(&self, query: &str) -> Result<Vec<Element>> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.query_selector_all(query, None).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    // pub async fn wait_for_selector(
    //     &self,
    //     query: &str,
    //     timeout: Option<Duration>,
    //     delay: Option<Duration>,
    // ) -> Result<Option<Element>> {
    //     match self.frame_inner() {
    //         Some(frame_inner) => {
    //             frame_inner
    //                 .wait_for_selector(query, timeout, delay, None)
    //                 .await
    //         }
    //         None => Err(anyhow!("Frame inner is dropped")),
    //     }
    // }

    pub async fn wait_for_selector(
        &self,
        query: &str,
        timeout: Option<Duration>,
    ) -> Result<Element> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.wait_for_selector(query, timeout, None).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn add_evaluate_on_new_document(&self, script: &str) -> Result<ScriptIdentifier> {
        match self.frame_inner() {
            Some(frame_inner) => frame_inner.add_evaluate_on_new_document(script).await,
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    pub async fn remove_evaluate_on_new_document(
        &self,
        script_identifier: &ScriptIdentifier,
    ) -> Result<()> {
        match self.frame_inner() {
            Some(frame_inner) => {
                frame_inner
                    .remove_evaluate_on_new_document(script_identifier)
                    .await
            }
            None => Err(anyhow!("Frame inner is dropped")),
        }
    }

    // pub async fn wait_for_dom_storage_item_added(
    //     &self,
    //     key: &str,
    //     timeout: Option<Duration>,
    // ) -> Result<String> {
    //     match self.frame_inner() {
    //         Some(frame_inner) => {
    //             frame_inner
    //                 .wait_for_dom_storage_item_added(key, timeout)
    //                 .await
    //         }
    //         None => Err(anyhow!("Frame inner is dropped")),
    //     }
    // }
}
