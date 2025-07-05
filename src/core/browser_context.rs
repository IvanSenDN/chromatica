use std::sync::Arc;

use super::cdp::browser_context::BrowserContext as CdpBrowserContext;
use super::page::Page;

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum BrowserContext {
    CDP(Arc<CdpBrowserContext>),
    // BiDi(BiDiBrowserContext),
}

impl BrowserContext {
    pub async fn new_page(&self) -> Result<Page> {
        match self {
            Self::CDP(browser_context) => {
                let page = browser_context.new_page().await?;
                Ok(Page::CDP(page))
            } // Self::BiDi(browser_context) => browser_context.new_page().await,
        }
    }

    pub async fn close(&self) -> Result<()> {
        match self {
            Self::CDP(browser_context) => browser_context.close().await,
            // Self::BiDi(browser_context) => browser_context.close().await,
        }
    }
}
