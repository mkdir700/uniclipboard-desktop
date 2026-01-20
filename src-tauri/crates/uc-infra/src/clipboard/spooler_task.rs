//! Async spooler task for writing representation bytes to disk.
//! 异步将表示字节写入磁盘缓存的任务。

use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use uc_core::ids::RepresentationId;
use uc_core::ports::clipboard::SpoolRequest;

use crate::clipboard::{RepresentationCache, SpoolManager};

/// Background task to write spool requests to disk.
/// 后台任务：将请求写入磁盘缓存。
pub struct SpoolerTask {
    spool_rx: mpsc::Receiver<SpoolRequest>,
    spool_manager: Arc<SpoolManager>,
    worker_tx: mpsc::Sender<RepresentationId>,
    cache: Arc<RepresentationCache>,
}

impl SpoolerTask {
    pub fn new(
        spool_rx: mpsc::Receiver<SpoolRequest>,
        spool_manager: Arc<SpoolManager>,
        worker_tx: mpsc::Sender<RepresentationId>,
        cache: Arc<RepresentationCache>,
    ) -> Self {
        Self {
            spool_rx,
            spool_manager,
            worker_tx,
            cache,
        }
    }

    /// Run the spooler loop until the channel is closed.
    /// 运行写入循环，直到通道关闭。
    pub async fn run(mut self) {
        while let Some(request) = self.spool_rx.recv().await {
            debug!(
                representation_id = %request.rep_id,
                bytes = request.bytes.len(),
                "Spooler received request"
            );
            self.cache.mark_spooling(&request.rep_id).await;
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
                // Revert to Pending to allow retry on next resolution
                self.cache.mark_pending(&request.rep_id).await;
            } else {
                self.cache.mark_completed(&request.rep_id).await;
                debug!(
                    representation_id = %request.rep_id,
                    "Spooler wrote spool entry"
                );
                if let Err(err) = self.worker_tx.try_send(request.rep_id.clone()) {
                    warn!(
                        representation_id = %request.rep_id,
                        error = %err,
                        "Failed to enqueue worker after spool write"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::RepresentationCache;
    use anyhow::Result;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_spooler_task_writes_to_disk() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
        let (tx, rx) = mpsc::channel(8);
        let (worker_tx, _worker_rx) = mpsc::channel(8);
        let cache = Arc::new(RepresentationCache::new(10, 1024));

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];

        let handle =
            tokio::spawn(SpoolerTask::new(rx, spool_manager.clone(), worker_tx, cache).run());
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
        let (worker_tx, _worker_rx) = mpsc::channel(1);
        let cache = Arc::new(RepresentationCache::new(10, 1024));

        let _task = SpoolerTask::new(rx, spool_manager, worker_tx, cache);

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

    #[tokio::test]
    async fn test_spooler_task_notifies_worker_after_write() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
        let (spool_tx, spool_rx) = mpsc::channel(8);
        let (worker_tx, mut worker_rx) = mpsc::channel(8);
        let cache = Arc::new(RepresentationCache::new(10, 1024));

        let rep_id = RepresentationId::new();
        let bytes = vec![1, 2, 3];

        let handle =
            tokio::spawn(SpoolerTask::new(spool_rx, spool_manager.clone(), worker_tx, cache).run());

        spool_tx
            .send(SpoolRequest {
                rep_id: rep_id.clone(),
                bytes: bytes.clone(),
            })
            .await?;

        let notified = timeout(Duration::from_secs(1), worker_rx.recv()).await?;
        assert_eq!(notified.as_ref(), Some(&rep_id));

        drop(spool_tx);
        handle.await?;

        let retrieved = spool_manager.read(&rep_id).await?;
        assert_eq!(retrieved, Some(bytes));
        Ok(())
    }

    #[tokio::test]
    async fn test_spooler_marks_cache_completed() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
        let cache = Arc::new(RepresentationCache::new(10, 1024));
        let (spool_tx, spool_rx) = mpsc::channel(8);
        let (worker_tx, _worker_rx) = mpsc::channel(8);

        let rep_id = RepresentationId::new();
        cache.put(&rep_id, vec![1, 2, 3]).await;

        let handle =
            tokio::spawn(SpoolerTask::new(spool_rx, spool_manager, worker_tx, cache.clone()).run());

        spool_tx
            .send(SpoolRequest {
                rep_id: rep_id.clone(),
                bytes: vec![1, 2, 3],
            })
            .await?;
        drop(spool_tx);
        handle.await?;

        assert_eq!(cache.get(&rep_id).await, Some(vec![1, 2, 3]));
        Ok(())
    }
}
