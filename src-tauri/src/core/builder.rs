use std::sync::Arc;

use anyhow::Result;

use crate::{
    infrastructure::{
        connection::connection_manager::ConnectionManager,
        storage::record_manager::ClipboardRecordManager, web::WebServer,
    },
    interface::{LocalClipboardTrait, RemoteSyncManagerTrait},
};

use super::UniClipboard;

pub struct UniClipboardBuilder {
    webserver: Option<WebServer>,
    clipboard: Option<Arc<dyn LocalClipboardTrait>>,
    remote_sync: Option<Arc<dyn RemoteSyncManagerTrait>>,
    connection_manager: Option<Arc<ConnectionManager>>,
    record_manager: Option<Arc<ClipboardRecordManager>>,
}

impl UniClipboardBuilder {
    pub fn new() -> Self {
        Self {
            webserver: None,
            clipboard: None,
            remote_sync: None,
            connection_manager: None,
            record_manager: None,
        }
    }

    pub fn set_webserver(mut self, webserver: WebServer) -> Self {
        self.webserver = Some(webserver);
        self
    }

    pub fn set_local_clipboard(mut self, clipboard: Arc<dyn LocalClipboardTrait>) -> Self {
        self.clipboard = Some(clipboard);
        self
    }

    pub fn set_remote_sync(mut self, remote_sync: Arc<dyn RemoteSyncManagerTrait>) -> Self {
        self.remote_sync = Some(remote_sync);
        self
    }

    pub fn set_connection_manager(mut self, connection_manager: Arc<ConnectionManager>) -> Self {
        self.connection_manager = Some(connection_manager);
        self
    }

    pub fn set_record_manager(mut self, record_manager: Arc<ClipboardRecordManager>) -> Self {
        self.record_manager = Some(record_manager);
        self
    }

    pub fn build(self) -> Result<UniClipboard> {
        let webserver = self
            .webserver
            .ok_or_else(|| anyhow::anyhow!("No webserver set"))?;
        let clipboard = self
            .clipboard
            .ok_or_else(|| anyhow::anyhow!("No clipboard set"))?;
        let remote_sync = self
            .remote_sync
            .ok_or_else(|| anyhow::anyhow!("No remote sync set"))?;
        let connection_manager = self
            .connection_manager
            .ok_or_else(|| anyhow::anyhow!("No connection manager set"))?;
        let record_manager = self
            .record_manager
            .ok_or_else(|| anyhow::anyhow!("No record manager set"))?;
        Ok(UniClipboard::new(
            Arc::new(webserver),
            clipboard,
            remote_sync,
            connection_manager,
            record_manager,
        ))
    }
}
