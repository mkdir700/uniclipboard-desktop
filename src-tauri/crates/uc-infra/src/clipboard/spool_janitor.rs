//! Spool janitor for cleaning up expired entries.
//! 用于清理过期缓存条目的巡检器。

use std::sync::Arc;

use anyhow::Result;
use tokio::fs;
use tracing::warn;
use uc_core::clipboard::PayloadAvailability;
use uc_core::ports::clipboard::ProcessingUpdateOutcome;
use uc_core::ports::{ClipboardRepresentationRepositoryPort, ClockPort};

use crate::clipboard::SpoolManager;

/// Spool cleanup task for expired entries.
/// 过期缓存条目的清理任务。
pub struct SpoolJanitor {
    spool: Arc<SpoolManager>,
    repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    clock: Arc<dyn ClockPort>,
    ttl_days: u64,
}

impl SpoolJanitor {
    pub fn new(
        spool: Arc<SpoolManager>,
        repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
        clock: Arc<dyn ClockPort>,
        ttl_days: u64,
    ) -> Self {
        Self {
            spool,
            repo,
            clock,
            ttl_days,
        }
    }

    pub async fn run_once(&self) -> Result<usize> {
        let expired = self
            .spool
            .list_expired(self.clock.now_ms(), self.ttl_days)
            .await?;
        let mut removed = 0usize;
        for entry in expired {
            match self
                .repo
                .update_processing_result(
                    &entry.representation_id,
                    &[PayloadAvailability::Staged, PayloadAvailability::Processing],
                    None,
                    PayloadAvailability::Lost,
                    Some("spool ttl expired"),
                )
                .await
            {
                Ok(ProcessingUpdateOutcome::Updated(_)) => {}
                Ok(ProcessingUpdateOutcome::StateMismatch) => {
                    warn!(
                        representation_id = %entry.representation_id,
                        "Skipping Lost update due to state mismatch"
                    );
                }
                Ok(ProcessingUpdateOutcome::NotFound) => {
                    warn!(representation_id = %entry.representation_id, "Representation missing");
                }
                Err(err) => {
                    warn!(
                        representation_id = %entry.representation_id,
                        error = %err,
                        "Failed to mark Lost during spool cleanup"
                    );
                }
            }

            if let Err(err) = fs::remove_file(&entry.file_path).await {
                warn!(
                    representation_id = %entry.representation_id,
                    error = %err,
                    "Failed to delete expired spool file"
                );
            }
            removed += 1;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::SpoolManager;
    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex as TokioMutex;
    use uc_core::clipboard::{PayloadAvailability, PersistedClipboardRepresentation};
    use uc_core::ids::{FormatId, RepresentationId};
    use uc_core::ports::clipboard::{
        ClipboardRepresentationRepositoryPort, ProcessingUpdateOutcome,
    };
    use uc_core::ports::ClockPort;
    use uc_core::MimeType;

    struct FixedClock {
        now_ms: i64,
    }

    impl ClockPort for FixedClock {
        fn now_ms(&self) -> i64 {
            self.now_ms
        }
    }

    struct MockRepresentationRepo {
        reps: TokioMutex<HashMap<RepresentationId, PersistedClipboardRepresentation>>,
    }

    impl MockRepresentationRepo {
        fn new(reps: HashMap<RepresentationId, PersistedClipboardRepresentation>) -> Self {
            Self {
                reps: TokioMutex::new(reps),
            }
        }

        async fn get(&self, rep_id: &RepresentationId) -> Option<PersistedClipboardRepresentation> {
            let reps = self.reps.lock().await;
            reps.get(rep_id).cloned()
        }
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
            Ok(self.get(representation_id).await)
        }

        async fn get_representation_by_blob_id(
            &self,
            _blob_id: &uc_core::BlobId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(None)
        }

        async fn update_blob_id(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_blob_id_if_none(
            &self,
            _representation_id: &RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<bool> {
            Ok(false)
        }

        async fn update_processing_result(
            &self,
            rep_id: &RepresentationId,
            expected_states: &[PayloadAvailability],
            blob_id: Option<&uc_core::BlobId>,
            new_state: PayloadAvailability,
            last_error: Option<&str>,
        ) -> Result<ProcessingUpdateOutcome> {
            let mut reps = self.reps.lock().await;
            let current = match reps.get_mut(rep_id) {
                Some(rep) => rep,
                None => return Ok(ProcessingUpdateOutcome::NotFound),
            };

            let expected_state_strs: Vec<&str> =
                expected_states.iter().map(|s| s.as_str()).collect();
            if !expected_state_strs.contains(&current.payload_state.as_str()) {
                return Ok(ProcessingUpdateOutcome::StateMismatch);
            }

            current.payload_state = new_state.clone();
            current.last_error = last_error.map(|value| value.to_string());

            if let Some(blob_id) = blob_id {
                current.blob_id = Some(blob_id.clone());
            }

            Ok(ProcessingUpdateOutcome::Updated(current.clone()))
        }
    }

    fn create_representation(rep_id: &RepresentationId) -> PersistedClipboardRepresentation {
        PersistedClipboardRepresentation::new_staged(
            rep_id.clone(),
            FormatId::new(),
            Some(MimeType("image/png".to_string())),
            1024,
        )
    }

    #[tokio::test]
    async fn test_janitor_marks_lost_after_ttl() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let spool = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);

        let rep_id = RepresentationId::new();
        spool.write(&rep_id, &[1, 2, 3]).await?;

        let mut reps = HashMap::new();
        reps.insert(rep_id.clone(), create_representation(&rep_id));

        let repo = Arc::new(MockRepresentationRepo::new(reps));
        let ttl_days = 1u64;
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;
        let clock = Arc::new(FixedClock {
            now_ms: now_ms + (ttl_days as i64) * 24 * 60 * 60 * 1000 + 1,
        });

        let janitor = SpoolJanitor::new(spool.clone(), repo.clone(), clock, ttl_days);
        let removed = janitor.run_once().await?;

        assert_eq!(removed, 1);
        let updated = repo.get(&rep_id).await;
        let updated = updated.expect("representation missing");
        assert_eq!(updated.payload_state(), PayloadAvailability::Lost);
        let remaining = spool.read(&rep_id).await?;
        assert!(remaining.is_none());
        Ok(())
    }
}
