use anyhow::Result;
use std::sync::Arc;

use super::browser_context::BrowserContext;
use super::cdp::browser::Browser as CdpBrowser;
use super::page::Page;

#[derive(Debug, Clone)]
pub enum Browser {
    CDP(CdpBrowser),
    // BiDi(BiDiBrowser),
}

impl Browser {
    pub async fn new_page(&self) -> Result<Page> {
        match self {
            Self::CDP(browser) => {
                let page = browser.new_page().await?;
                Ok(Page::CDP(page))
            }
        }
    }

    pub async fn new_browser_context(
        &self,
        proxy: Option<&str>,
        proxy_bypass_list: Option<&str>,
    ) -> Result<Arc<BrowserContext>> {
        match self {
            Self::CDP(browser) => {
                let browser_context = browser
                    .new_browser_context(proxy, proxy_bypass_list)
                    .await?;
                Ok(Arc::new(BrowserContext::CDP(browser_context)))
            }
        }
    }

    pub async fn disconnect(self) {
        match self {
            Self::CDP(browser) => {
                browser.disconnect().await;
            }
        }
    }
}
