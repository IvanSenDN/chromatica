use std::sync::{Arc, Weak};

use super::connection::Connection;
use super::domains::browser::BrowserContextID;
use super::page::Page;
use super::target_manager::TargetManager;

use anyhow::{Result, anyhow};

#[derive(Debug, Clone)]
pub struct BrowserContext {
    id: BrowserContextID,
    connection: Weak<Connection>,
}

impl BrowserContext {
    pub fn new(id: BrowserContextID, connection: Weak<Connection>) -> Self {
        Self { id, connection }
    }

    fn connection(&self) -> Option<Arc<Connection>> {
        match self.connection.upgrade() {
            Some(conn) => Some(conn),
            None => None,
        }
    }

    fn target_manager(&self) -> Option<Arc<TargetManager>> {
        let conn = match self.connection() {
            Some(conn) => conn,
            None => return None,
        };
        match conn.target_manager() {
            Some(target_manager) => Some(target_manager.clone()),
            None => None,
        }
    }

    pub async fn new_page(&self) -> Result<Page> {
        let Some(target_manager) = self.target_manager() else {
            return Err(anyhow!("Target manager is not available"));
        };
        let page = target_manager.create_page(Some(&self.id)).await?;
        Ok(page)
    }

    pub async fn close(&self) -> Result<()> {
        let Some(target_manager) = self.target_manager() else {
            return Err(anyhow!("Target manager is not available"));
        };
        target_manager.close_browser_context(&self.id).await?;
        Ok(())
    }
}
