//! Snapshot cache integration tests.
//!
//! These tests exercise capture + cache/spool + background worker flow.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

use uc_app::usecases::internal::capture_clipboard::CaptureClipboardUseCase;
use uc_core::clipboard::SelectRepresentationPolicyV1;
use uc_core::clipboard::{
    ClipboardEntry, ClipboardEvent, ClipboardSelectionDecision, ObservedClipboardRepresentation,
    PayloadAvailability, PersistedClipboardRepresentation, SystemClipboardSnapshot,
};
use uc_core::ids::{EntryId, EventId, FormatId, RepresentationId};
use uc_core::ports::clipboard::ProcessingUpdateOutcome;
use uc_core::ports::BlobWriterPort;
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardEventWriterPort, ClipboardRepresentationNormalizerPort,
    ClipboardRepresentationRepositoryPort, DeviceIdentityPort, SelectRepresentationPolicyPort,
};
use uc_core::DeviceId;
use uc_core::{Blob, BlobId, ContentHash, MimeType};
use uc_infra::clipboard::spooler_task::SpoolRequest;
use uc_infra::clipboard::{
    BackgroundBlobWorker, ClipboardRepresentationNormalizer, RepresentationCache, SpoolManager,
};
use uc_infra::config::ClipboardStorageConfig;
use uc_infra::security::Blake3Hasher;

struct InMemoryDeviceIdentity;

impl DeviceIdentityPort for InMemoryDeviceIdentity {
    fn current_device_id(&self) -> DeviceId {
        DeviceId::new("device-test")
    }
}

#[derive(Default)]
struct InMemoryEntryRepo {
    entries: Mutex<HashMap<EntryId, ClipboardEntry>>,
    selections: Mutex<HashMap<EntryId, ClipboardSelectionDecision>>,
}

#[async_trait::async_trait]
impl ClipboardEntryRepositoryPort for InMemoryEntryRepo {
    async fn save_entry_and_selection(
        &self,
        entry: &ClipboardEntry,
        selection: &ClipboardSelectionDecision,
    ) -> Result<()> {
        self.entries
            .lock()
            .unwrap()
            .insert(entry.entry_id.clone(), entry.clone());
        self.selections
            .lock()
            .unwrap()
            .insert(entry.entry_id.clone(), selection.clone());
        Ok(())
    }

    async fn get_entry(&self, entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
        Ok(self.entries.lock().unwrap().get(entry_id).cloned())
    }

    async fn list_entries(&self, _limit: usize, _offset: usize) -> Result<Vec<ClipboardEntry>> {
        Ok(self.entries.lock().unwrap().values().cloned().collect())
    }

    async fn delete_entry(&self, entry_id: &EntryId) -> Result<()> {
        self.entries.lock().unwrap().remove(entry_id);
        self.selections.lock().unwrap().remove(entry_id);
        Ok(())
    }
}

#[derive(Default)]
struct InMemoryRepresentationRepo {
    representations: Mutex<HashMap<RepresentationId, PersistedClipboardRepresentation>>,
}

impl InMemoryRepresentationRepo {
    fn insert_all(&self, reps: &[PersistedClipboardRepresentation]) {
        let mut guard = self.representations.lock().unwrap();
        for rep in reps {
            guard.insert(rep.id.clone(), rep.clone());
        }
    }

    fn get_by_id(&self, rep_id: &RepresentationId) -> Option<PersistedClipboardRepresentation> {
        self.representations.lock().unwrap().get(rep_id).cloned()
    }
}

#[async_trait::async_trait]
impl ClipboardRepresentationRepositoryPort for InMemoryRepresentationRepo {
    async fn get_representation(
        &self,
        _event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        Ok(self.get_by_id(representation_id))
    }

    async fn get_representation_by_id(
        &self,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        Ok(self.get_by_id(representation_id))
    }

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()> {
        let mut guard = self.representations.lock().unwrap();
        if let Some(rep) = guard.get_mut(representation_id) {
            rep.blob_id = Some(blob_id.clone());
        }
        Ok(())
    }

    async fn update_blob_id_if_none(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<bool> {
        let mut guard = self.representations.lock().unwrap();
        if let Some(rep) = guard.get_mut(representation_id) {
            if rep.blob_id.is_none() {
                rep.blob_id = Some(blob_id.clone());
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn update_processing_result(
        &self,
        rep_id: &RepresentationId,
        expected_states: &[PayloadAvailability],
        blob_id: Option<&BlobId>,
        new_state: PayloadAvailability,
        last_error: Option<&str>,
    ) -> Result<ProcessingUpdateOutcome> {
        let mut guard = self.representations.lock().unwrap();
        let rep = match guard.get_mut(rep_id) {
            Some(rep) => rep,
            None => return Ok(ProcessingUpdateOutcome::NotFound),
        };

        if !expected_states.contains(&rep.payload_state) {
            return Ok(ProcessingUpdateOutcome::StateMismatch);
        }

        rep.payload_state = new_state;
        rep.blob_id = blob_id.cloned();
        rep.last_error = last_error.map(|s| s.to_string());

        Ok(ProcessingUpdateOutcome::Updated(rep.clone()))
    }
}

struct InMemoryEventWriter {
    rep_repo: Arc<InMemoryRepresentationRepo>,
}

#[async_trait::async_trait]
impl ClipboardEventWriterPort for InMemoryEventWriter {
    async fn insert_event(
        &self,
        _event: &ClipboardEvent,
        representations: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()> {
        self.rep_repo.insert_all(representations);
        Ok(())
    }

    async fn delete_event_and_representations(&self, _event_id: &EventId) -> Result<()> {
        Ok(())
    }
}

struct InMemoryBlobWriter {
    blobs: Mutex<HashMap<ContentHash, Blob>>,
}

impl InMemoryBlobWriter {
    fn new() -> Self {
        Self {
            blobs: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl BlobWriterPort for InMemoryBlobWriter {
    async fn write_if_absent(
        &self,
        content_id: &ContentHash,
        plaintext_bytes: &[u8],
    ) -> Result<Blob> {
        let mut guard = self.blobs.lock().unwrap();
        if let Some(existing) = guard.get(content_id) {
            return Ok(existing.clone());
        }

        let blob_id = BlobId::new();
        let locator = uc_core::blob::BlobStorageLocator::new_local_fs(std::path::PathBuf::from(
            format!("/tmp/blob/{}", blob_id),
        ));
        let blob = Blob::new(
            blob_id,
            locator,
            plaintext_bytes.len() as i64,
            content_id.clone(),
            0,
        );
        guard.insert(content_id.clone(), blob.clone());
        Ok(blob)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn build_snapshot(rep_id: RepresentationId, bytes: Vec<u8>, mime: &str) -> SystemClipboardSnapshot {
    SystemClipboardSnapshot {
        ts_ms: now_ms(),
        representations: vec![ObservedClipboardRepresentation {
            id: rep_id,
            format_id: FormatId::from(mime),
            mime: Some(MimeType::from_str(mime).unwrap()),
            bytes,
        }],
    }
}

#[tokio::test]
async fn test_capture_does_not_block_when_queues_full() -> Result<()> {
    let rep_id = RepresentationId::new();
    let bytes = vec![0u8; 32 * 1024];
    let snapshot = build_snapshot(rep_id.clone(), bytes.clone(), "image/png");

    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 16,
        ..ClipboardStorageConfig::defaults()
    });
    let normalizer: Arc<dyn ClipboardRepresentationNormalizerPort> =
        Arc::new(ClipboardRepresentationNormalizer::new(config));

    let rep_cache = Arc::new(RepresentationCache::new(10, 1_000_000));
    let rep_repo = Arc::new(InMemoryRepresentationRepo::default());
    let event_writer: Arc<dyn ClipboardEventWriterPort> =
        Arc::new(InMemoryEventWriter { rep_repo });
    let entry_repo: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(InMemoryEntryRepo::default());
    let policy: Arc<dyn SelectRepresentationPolicyPort> =
        Arc::new(SelectRepresentationPolicyV1::new());

    let (spool_tx, _spool_rx) = mpsc::channel(1);
    let (worker_tx, _worker_rx) = mpsc::channel(1);

    spool_tx.try_send(SpoolRequest {
        rep_id: RepresentationId::new(),
        bytes: vec![1],
    })?;
    worker_tx.try_send(RepresentationId::new())?;

    let usecase = CaptureClipboardUseCase::new(
        entry_repo,
        event_writer,
        policy,
        normalizer,
        Arc::new(InMemoryDeviceIdentity),
        rep_cache.clone(),
        spool_tx,
    );

    timeout(Duration::from_millis(200), usecase.execute(snapshot)).await??;

    let cached = rep_cache.get(&rep_id).await;
    assert_eq!(cached, Some(bytes));
    Ok(())
}

#[tokio::test]
async fn test_worker_materializes_blob_from_cache() -> Result<()> {
    let rep_id = RepresentationId::new();
    let bytes = vec![7u8; 64 * 1024];
    let snapshot = build_snapshot(rep_id.clone(), bytes.clone(), "image/png");

    let config = Arc::new(ClipboardStorageConfig {
        inline_threshold_bytes: 16,
        ..ClipboardStorageConfig::defaults()
    });
    let normalizer: Arc<dyn ClipboardRepresentationNormalizerPort> =
        Arc::new(ClipboardRepresentationNormalizer::new(config));

    let rep_cache = Arc::new(RepresentationCache::new(10, 1_000_000));
    let rep_repo = Arc::new(InMemoryRepresentationRepo::default());
    let event_writer: Arc<dyn ClipboardEventWriterPort> = Arc::new(InMemoryEventWriter {
        rep_repo: rep_repo.clone(),
    });
    let entry_repo: Arc<dyn ClipboardEntryRepositoryPort> = Arc::new(InMemoryEntryRepo::default());
    let policy: Arc<dyn SelectRepresentationPolicyPort> =
        Arc::new(SelectRepresentationPolicyV1::new());

    let spool_dir = tempfile::tempdir()?;
    let spool = Arc::new(SpoolManager::new(spool_dir.path(), 1_000_000)?);
    let (spool_tx, spool_rx) = mpsc::channel(8);
    let (worker_tx, worker_rx) = mpsc::channel(8);

    let spooler = uc_infra::clipboard::SpoolerTask::new(
        spool_rx,
        spool.clone(),
        worker_tx.clone(),
        rep_cache.clone(),
    );
    tokio::spawn(async move {
        spooler.run().await;
    });

    let worker = BackgroundBlobWorker::new(
        worker_rx,
        rep_cache.clone(),
        spool.clone(),
        rep_repo.clone(),
        Arc::new(InMemoryBlobWriter::new()),
        Arc::new(Blake3Hasher),
        3,
        Duration::from_millis(50),
    );
    tokio::spawn(async move {
        worker.run().await;
    });

    let usecase = CaptureClipboardUseCase::new(
        entry_repo,
        event_writer,
        policy,
        normalizer,
        Arc::new(InMemoryDeviceIdentity),
        rep_cache.clone(),
        spool_tx,
    );

    usecase.execute(snapshot).await?;

    let deadline = Duration::from_secs(2);
    let mut elapsed = Duration::from_millis(0);
    let step = Duration::from_millis(20);
    loop {
        if let Some(rep) = rep_repo.get_by_id(&rep_id) {
            if rep.payload_state == PayloadAvailability::BlobReady && rep.blob_id.is_some() {
                break;
            }
        }
        if elapsed >= deadline {
            return Err(anyhow!("timed out waiting for blob materialization"));
        }
        sleep(step).await;
        elapsed += step;
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires SpoolScanner recovery flow (Task 14)"]
async fn test_spool_recovers_after_restart() -> Result<()> {
    // TODO(Task 14): Add SpoolScanner and recovery assertions.
    Ok(())
}
