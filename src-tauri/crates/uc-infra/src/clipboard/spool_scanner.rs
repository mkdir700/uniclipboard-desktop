//! Spool scanner for recovery.
//! 用于恢复的磁盘缓存扫描器。

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uc_core::clipboard::PayloadAvailability;
use uc_core::ids::RepresentationId;
use uc_core::ports::ClipboardRepresentationRepositoryPort;

/// Scans spool directory and re-queues recoverable representations.
/// 扫描磁盘缓存目录并重新入队可恢复的表示。
pub struct SpoolScanner {
    spool_dir: PathBuf,
    repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    worker_tx: mpsc::Sender<RepresentationId>,
}

impl SpoolScanner {
    /// Create a new scanner.
    /// 创建新的扫描器。
    pub fn new(
        spool_dir: PathBuf,
        repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
        worker_tx: mpsc::Sender<RepresentationId>,
    ) -> Self {
        Self {
            spool_dir,
            repo,
            worker_tx,
        }
    }

    /// Scan spool directory and recover queued items.
    /// 扫描磁盘缓存并恢复待处理项。
    pub async fn scan_and_recover(&self) -> Result<usize> {
        self.scan_and_recover_dir(&self.spool_dir).await
    }

    async fn scan_and_recover_dir(&self, spool_dir: &PathBuf) -> Result<usize> {
        let mut entries = fs::read_dir(spool_dir)
            .await
            .with_context(|| format!("Failed to read spool dir: {}", spool_dir.display()))?;

        let mut recovered = 0usize;

        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if !file_type.is_file() {
                continue;
            }

            let file_name = entry.file_name();
            let Some(file_name_str) = file_name.to_str() else {
                warn!("Skipping spool entry with non-utf8 filename");
                continue;
            };

            if file_name_str.is_empty() {
                warn!("Skipping spool entry with empty filename");
                continue;
            }

            let rep_id = RepresentationId::from(file_name_str);

            match self.repo.get_representation_by_id(&rep_id).await? {
                Some(rep) => match rep.payload_state() {
                    PayloadAvailability::Staged | PayloadAvailability::Processing => {
                        match self.worker_tx.try_send(rep_id.clone()) {
                            Ok(()) => {
                                recovered += 1;
                            }
                            Err(err) => {
                                warn!(
                                    representation_id = %rep_id,
                                    error = %err,
                                    "Failed to re-queue representation during recovery"
                                );
                            }
                        }
                    }
                    _ => {
                        let path = entry.path();
                        if let Err(err) = fs::remove_file(&path).await {
                            warn!(
                                representation_id = %rep_id,
                                error = %err,
                                "Failed to delete stale spool file"
                            );
                        }
                    }
                },
                None => {
                    let path = entry.path();
                    warn!(
                        representation_id = %rep_id,
                        "Representation missing for spool entry; deleting stale file"
                    );
                    if let Err(err) = fs::remove_file(&path).await {
                        warn!(
                            representation_id = %rep_id,
                            error = %err,
                            "Failed to delete orphaned spool file"
                        );
                    }
                }
            }
        }

        info!(
            spool_dir = %spool_dir.display(),
            "Spool scan completed; recovered {recovered} items"
        );
        Ok(recovered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::time::{timeout, Duration};
    use uc_core::clipboard::{PayloadAvailability, PersistedClipboardRepresentation};
    use uc_core::ids::{FormatId, RepresentationId};
    use uc_core::ports::clipboard::ProcessingUpdateOutcome;
    use uc_core::ports::ClipboardRepresentationRepositoryPort;
    use uc_core::{BlobId, MimeType};

    struct MockRepresentationRepo {
        reps_by_id: HashMap<String, PersistedClipboardRepresentation>,
    }

    #[async_trait::async_trait]
    impl ClipboardRepresentationRepositoryPort for MockRepresentationRepo {
        async fn get_representation(
            &self,
            _event_id: &uc_core::ids::EventId,
            _representation_id: &RepresentationId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(None)
        }

        async fn get_representation_by_id(
            &self,
            representation_id: &RepresentationId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(self.reps_by_id.get(representation_id.as_ref()).cloned())
        }

        async fn get_representation_by_blob_id(
            &self,
            _blob_id: &BlobId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(None)
        }

        async fn update_blob_id(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &BlobId,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_blob_id_if_none(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &BlobId,
        ) -> Result<bool> {
            Ok(false)
        }

        async fn update_processing_result(
            &self,
            _rep_id: &RepresentationId,
            _expected_states: &[PayloadAvailability],
            _blob_id: Option<&BlobId>,
            _new_state: PayloadAvailability,
            _last_error: Option<&str>,
        ) -> Result<ProcessingUpdateOutcome> {
            Ok(ProcessingUpdateOutcome::StateMismatch)
        }
    }

    fn staged_rep(rep_id: &RepresentationId) -> PersistedClipboardRepresentation {
        PersistedClipboardRepresentation::new_with_state(
            rep_id.clone(),
            FormatId::new(),
            Some(MimeType::text_plain()),
            1024,
            None,
            None,
            PayloadAvailability::Staged,
            None,
        )
        .expect("valid staged representation")
    }

    fn blob_ready_rep(rep_id: &RepresentationId) -> PersistedClipboardRepresentation {
        PersistedClipboardRepresentation::new_with_state(
            rep_id.clone(),
            FormatId::new(),
            Some(MimeType::text_plain()),
            1024,
            None,
            Some(BlobId::new()),
            PayloadAvailability::BlobReady,
            None,
        )
        .expect("valid blob-ready representation")
    }

    #[tokio::test]
    async fn test_scanner_requeues_staged_representation() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let rep_id = RepresentationId::new();
        let file_path = temp_dir.path().join(rep_id.to_string());
        tokio::fs::write(&file_path, b"payload").await?;

        let mut reps_by_id = HashMap::new();
        reps_by_id.insert(rep_id.to_string(), staged_rep(&rep_id));
        let repo = Arc::new(MockRepresentationRepo { reps_by_id });

        let (worker_tx, mut worker_rx) = mpsc::channel(4);

        let scanner = SpoolScanner::new(temp_dir.path().to_path_buf(), repo, worker_tx);
        let recovered = scanner.scan_and_recover().await?;

        assert_eq!(recovered, 1);
        let received = timeout(Duration::from_millis(200), worker_rx.recv()).await?;
        assert_eq!(received, Some(rep_id));
        assert!(file_path.exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_scanner_deletes_blob_ready_spool() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let rep_id = RepresentationId::new();
        let file_path = temp_dir.path().join(rep_id.to_string());
        tokio::fs::write(&file_path, b"payload").await?;

        let mut reps_by_id = HashMap::new();
        reps_by_id.insert(rep_id.to_string(), blob_ready_rep(&rep_id));
        let repo = Arc::new(MockRepresentationRepo { reps_by_id });

        let (worker_tx, mut worker_rx) = mpsc::channel(4);

        let scanner = SpoolScanner::new(temp_dir.path().to_path_buf(), repo, worker_tx);
        let recovered = scanner.scan_and_recover().await?;

        assert_eq!(recovered, 0);
        assert!(!file_path.exists());
        assert!(timeout(Duration::from_millis(100), worker_rx.recv())
            .await
            .is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_scanner_deletes_missing_representation() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let rep_id = RepresentationId::new();
        let file_path = temp_dir.path().join(rep_id.to_string());
        tokio::fs::write(&file_path, b"payload").await?;

        let repo = Arc::new(MockRepresentationRepo {
            reps_by_id: HashMap::new(),
        });

        let (worker_tx, mut worker_rx) = mpsc::channel(4);

        let scanner = SpoolScanner::new(temp_dir.path().to_path_buf(), repo, worker_tx);
        let recovered = scanner.scan_and_recover().await?;

        assert_eq!(recovered, 0);
        assert!(!file_path.exists());
        assert!(timeout(Duration::from_millis(100), worker_rx.recv())
            .await
            .is_err());
        Ok(())
    }
}
