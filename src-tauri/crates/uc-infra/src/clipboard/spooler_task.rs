//! Async spooler task for writing representation bytes to disk.
//! 异步将表示字节写入磁盘缓存的任务。

use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::error;
use uc_core::ids::RepresentationId;

use crate::clipboard::SpoolManager;

/// Spool write request.
/// 写入磁盘缓存的请求。
pub struct SpoolRequest {
    pub rep_id: RepresentationId,
    pub bytes: Vec<u8>,
}

/// Background task to write spool requests to disk.
/// 后台任务：将请求写入磁盘缓存。
pub struct SpoolerTask {
    spool_rx: mpsc::Receiver<SpoolRequest>,
    spool_manager: Arc<SpoolManager>,
}

impl SpoolerTask {
    pub fn new(spool_rx: mpsc::Receiver<SpoolRequest>, spool_manager: Arc<SpoolManager>) -> Self {
        Self {
            spool_rx,
            spool_manager,
        }
    }

    /// Run the spooler loop until the channel is closed.
    /// 运行写入循环，直到通道关闭。
    pub async fn run(mut self) {
        while let Some(request) = self.spool_rx.recv().await {
            if let Err(err) = self
                .spool_manager
                .write(&request.rep_id, &request.bytes)
                .await
            {
                error!(
                    representation_id = %request.rep_id,
                    error = %err,
                    "Failed to write spool entry"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_spooler_task_writes_to_disk() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
        let (tx, rx) = mpsc::channel(8);

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];

        let handle = tokio::spawn(SpoolerTask::new(rx, spool_manager.clone()).run());
        tx.send(SpoolRequest {
            rep_id: rep_id.clone(),
            bytes: bytes.clone(),
        })
        .await?;
        drop(tx);

        handle.await?;

        let retrieved = spool_manager.read(&rep_id).await?;
        assert_eq!(retrieved, Some(bytes));
        Ok(())
    }

    #[tokio::test]
    async fn test_spooler_task_backpressure() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
        let (tx, rx) = mpsc::channel(1);

        let _task = SpoolerTask::new(rx, spool_manager);

        let rep_id_a = RepresentationId::new();
        let rep_id_b = RepresentationId::new();

        assert!(tx
            .try_send(SpoolRequest {
                rep_id: rep_id_a,
                bytes: vec![1, 2, 3],
            })
            .is_ok());

        assert!(tx
            .try_send(SpoolRequest {
                rep_id: rep_id_b,
                bytes: vec![4, 5, 6],
            })
            .is_err());

        Ok(())
    }
}
