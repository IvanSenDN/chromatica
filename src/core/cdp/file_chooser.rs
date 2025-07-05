use super::connection::Connection;
use super::domains::dom::*;
use super::domains::page::*;
use super::domains::target::*;
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct FileChooser {
    connection: Weak<Connection>,
    session_id: Weak<SessionId>,
    frame_id: FrameId,
    backend_node_id: Option<BackendNodeId>,
}

impl FileChooser {
    pub fn new(
        connection: Weak<Connection>,
        session_id: Weak<SessionId>,
        event: &FileChooserOpened,
    ) -> Self {
        Self {
            connection,
            session_id,
            frame_id: event.frame_id.clone(),
            backend_node_id: event.backend_node_id.clone(),
        }
    }

    pub fn frame_id(&self) -> &FrameId {
        &self.frame_id
    }

    pub fn backend_node_id(&self) -> Option<&BackendNodeId> {
        self.backend_node_id.as_ref()
    }

    pub async fn send<P: Serialize>(&self, method: &str, params: &P) -> Result<()> {
        let connection = match self.connection.upgrade() {
            Some(connection) => connection,
            None => return Err(anyhow!("Connection is not available")),
        };
        let session_id = match self.session_id.upgrade() {
            Some(session_id) => session_id,
            None => return Err(anyhow!("Session id is not available")),
        };
        connection.send(method, params, Some(&session_id)).await?;
        Ok(())
    }

    pub async fn upload_file(&self, file_paths: Vec<&str>) -> Result<()> {
        let absolute_paths: Vec<String> = file_paths
            .iter()
            .map(|path| {
                let path_buf = PathBuf::from(path);
                if path_buf.is_absolute() {
                    path.to_string()
                } else {
                    std::env::current_dir()
                        .unwrap_or_default()
                        .join(path_buf)
                        .to_string_lossy()
                        .into_owned()
                }
            })
            .collect();

        let backend_node_id = match self.backend_node_id() {
            Some(backend_node_id) => *backend_node_id,
            None => {
                return Err(anyhow!(
                    "This method is not implemented for non-dom type file choosers. See more: https://chromedevtools.github.io/devtools-protocol/tot/Page/#event-fileChooserOpened"
                ));
            }
        };

        let params = SetFileInputFiles::default(absolute_paths, backend_node_id);
        self.send("DOM.setFileInputFiles", &params).await?;
        Ok(())
    }
}
