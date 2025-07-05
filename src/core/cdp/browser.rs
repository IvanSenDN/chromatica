use anyhow::Result;
use std::sync::Arc;

use super::browser_context::BrowserContext;
use super::connection::Connection;
use super::page::Page;
use super::target_manager::TargetManager;

#[derive(Debug, Clone)]
pub struct Browser {
    connection: Arc<Connection>,
}

impl Browser {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self { connection }
    }

    fn target_manager(&self) -> Option<&Arc<TargetManager>> {
        match self.connection.target_manager() {
            Some(target_manager) => Some(target_manager),
            None => None,
        }
    }

    pub async fn new_page(&self) -> Result<Page> {
        let target_manager = self.target_manager().unwrap();
        let page = target_manager.create_page(None).await?;
        Ok(page)
    }

    pub async fn new_browser_context(
        &self,
        proxy: Option<&str>,
        proxy_bypass_list: Option<&str>,
    ) -> Result<Arc<BrowserContext>> {
        let target_manager = self.target_manager().unwrap();
        let browser_context = target_manager
            .create_browser_context(proxy, proxy_bypass_list)
            .await?;
        Ok(browser_context)
    }

    pub async fn disconnect(self) {
        self.connection.disconnect().await;
    }
}
