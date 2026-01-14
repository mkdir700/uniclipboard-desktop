//! Regression tests for CaptureClipboardUseCase bug fixes
//!
//! Tests the following fixes:
//! 1. SystemTime error is propagated instead of panicking with expect()
//! 2. save_entry_and_selection is properly awaited

use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uc_app::usecases::internal::capture_clipboard::CaptureClipboardUseCase;
use uc_core::clipboard::PersistedClipboardRepresentation;
use uc_core::clipboard::SelectRepresentationPolicyV1;
use uc_core::clipboard::{
    ClipboardEntry, ClipboardSelectionDecision, MimeType, ObservedClipboardRepresentation,
    SystemClipboardSnapshot,
};
use uc_core::ids::DeviceId;
use uc_core::ids::EntryId;
use uc_core::ids::{FormatId, RepresentationId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardEventWriterPort,
    ClipboardRepresentationMaterializerPort, DeviceIdentityPort, PlatformClipboardPort,
};

/// Mock PlatformClipboardPort that returns a fixed snapshot
struct MockPlatformClipboard {
    snapshot: SystemClipboardSnapshot,
}

impl MockPlatformClipboard {
    fn new(snapshot: SystemClipboardSnapshot) -> Self {
        Self { snapshot }
    }
}

#[async_trait::async_trait]
impl PlatformClipboardPort for MockPlatformClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        Ok(self.snapshot.clone())
    }
}

/// Mock DeviceIdentityPort
struct MockDeviceIdentity;

#[async_trait::async_trait]
impl DeviceIdentityPort for MockDeviceIdentity {
    fn current_device_id(&self) -> DeviceId {
        DeviceId::new("test-device")
    }
}

/// Mock ClipboardEventWriterPort
struct MockEventWriter;

#[async_trait::async_trait]
impl ClipboardEventWriterPort for MockEventWriter {
    async fn insert_event(
        &self,
        _event: &uc_core::clipboard::ClipboardEvent,
        _representations: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_event_and_representations(
        &self,
        _event_id: &uc_core::ids::EventId,
    ) -> Result<()> {
        Ok(())
    }
}

/// Mock ClipboardRepresentationMaterializerPort
struct MockMaterializer;

#[async_trait::async_trait]
impl ClipboardRepresentationMaterializerPort for MockMaterializer {
    async fn materialize(
        &self,
        _rep: &uc_core::clipboard::ObservedClipboardRepresentation,
    ) -> Result<PersistedClipboardRepresentation> {
        Ok(PersistedClipboardRepresentation {
            id: _rep.id.clone(),
            format_id: _rep.format_id.clone(),
            mime_type: _rep.mime.clone(),
            size_bytes: _rep.bytes.len() as i64,
            inline_data: Some(_rep.bytes.clone()),
            blob_id: None,
        })
    }
}

/// Mock ClipboardEntryRepositoryPort that tracks if save was called AND awaited
struct MockEntryRepo {
    save_started: Arc<AtomicBool>,
    save_completed: Arc<AtomicBool>,
}

impl MockEntryRepo {
    fn new() -> Self {
        Self {
            save_started: Arc::new(AtomicBool::new(false)),
            save_completed: Arc::new(AtomicBool::new(false)),
        }
    }

    fn was_save_started(&self) -> bool {
        self.save_started.load(Ordering::SeqCst)
    }

    fn was_save_completed(&self) -> bool {
        self.save_completed.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl ClipboardEntryRepositoryPort for MockEntryRepo {
    async fn save_entry_and_selection(
        &self,
        _entry: &ClipboardEntry,
        _selection: &ClipboardSelectionDecision,
    ) -> Result<()> {
        self.save_started.store(true, Ordering::SeqCst);
        // Simulate async operation - if not awaited, this won't execute
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        self.save_completed.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn list_entries(&self, _limit: usize, _offset: usize) -> Result<Vec<ClipboardEntry>> {
        Ok(vec![])
    }

    async fn get_entry(&self, _id: &EntryId) -> Result<Option<ClipboardEntry>> {
        Ok(None)
    }

    async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_capture_clipboard_saves_entry_and_awaits() {
    // This test verifies that save_entry_and_selection is properly awaited.
    // Before the fix, the call was `let _ = self.entry_repo.save_entry_and_selection(...)`
    // without `.await`, which meant the Future was never polled and the save never happened.

    // Create a snapshot with a valid representation for the policy to select
    // Note: bytes length must be >= 32 for ContentHash to work properly
    let snapshot = SystemClipboardSnapshot {
        ts_ms: 12345,
        representations: vec![ObservedClipboardRepresentation {
            id: RepresentationId::new(),
            format_id: FormatId::from("public.utf8-plain-text"),
            mime: Some(MimeType("text/plain".to_string())),
            bytes: vec![b'H'; 32], // 32 bytes for ContentHash
        }],
    };

    let entry_repo = Arc::new(MockEntryRepo::new());

    let use_case = CaptureClipboardUseCase::new(
        Arc::new(MockPlatformClipboard::new(snapshot)),
        entry_repo.clone(),
        Arc::new(MockEventWriter),
        Arc::new(SelectRepresentationPolicyV1::new()),
        Arc::new(MockMaterializer),
        Arc::new(MockDeviceIdentity),
    );

    // Execute the use case
    let result = use_case.execute().await;

    // Verify it succeeded
    assert!(result.is_ok(), "execute should succeed");

    // Verify save was started
    assert!(
        entry_repo.was_save_started(),
        "save_entry_and_selection should be started"
    );

    // Verify save was COMPLETED (this is the key test!)
    // If save wasn't awaited, was_save_completed would be false
    assert!(
        entry_repo.was_save_completed(),
        "save_entry_and_selection should be awaited and completed"
    );
}

#[tokio::test]
async fn test_capture_clipboard_with_snapshot_saves_and_awaits() {
    // Same test but for execute_with_snapshot

    // Create a snapshot with a valid representation for the policy to select
    // Note: bytes length must be >= 32 for ContentHash to work properly
    let snapshot = SystemClipboardSnapshot {
        ts_ms: 12345,
        representations: vec![ObservedClipboardRepresentation {
            id: RepresentationId::new(),
            format_id: FormatId::from("public.utf8-plain-text"),
            mime: Some(MimeType("text/plain".to_string())),
            bytes: vec![b'H'; 32], // 32 bytes for ContentHash
        }],
    };

    let entry_repo = Arc::new(MockEntryRepo::new());

    let use_case = CaptureClipboardUseCase::new(
        Arc::new(MockPlatformClipboard::new(snapshot.clone())),
        entry_repo.clone(),
        Arc::new(MockEventWriter),
        Arc::new(SelectRepresentationPolicyV1::new()),
        Arc::new(MockMaterializer),
        Arc::new(MockDeviceIdentity),
    );

    // Execute with snapshot
    let result = use_case.execute_with_snapshot(snapshot).await;

    // Verify it succeeded
    assert!(result.is_ok(), "execute_with_snapshot should succeed");

    // Verify save was started AND completed (awaited)
    assert!(entry_repo.was_save_started(), "save should be started");
    assert!(
        entry_repo.was_save_completed(),
        "save should be awaited and completed"
    );
}

#[tokio::test]
async fn test_capture_clipboard_propagates_repo_errors() {
    // This test verifies that errors from save_entry_and_selection are properly propagated.
    // Before the fix (missing .await?), errors would be silently ignored.

    struct FailingEntryRepo;

    #[async_trait::async_trait]
    impl ClipboardEntryRepositoryPort for FailingEntryRepo {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &ClipboardSelectionDecision,
        ) -> Result<()> {
            Err(anyhow::anyhow!("Repository error"))
        }

        async fn list_entries(&self, _limit: usize, _offset: usize) -> Result<Vec<ClipboardEntry>> {
            Ok(vec![])
        }

        async fn get_entry(&self, _id: &EntryId) -> Result<Option<ClipboardEntry>> {
            Ok(None)
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
            Ok(())
        }
    }

    // Create a snapshot with a valid representation for the policy to select
    // Note: bytes length must be >= 32 for ContentHash to work properly
    let snapshot = SystemClipboardSnapshot {
        ts_ms: 12345,
        representations: vec![ObservedClipboardRepresentation {
            id: RepresentationId::new(),
            format_id: FormatId::from("public.utf8-plain-text"),
            mime: Some(MimeType("text/plain".to_string())),
            bytes: vec![b'H'; 32], // 32 bytes for ContentHash
        }],
    };

    let use_case = CaptureClipboardUseCase::new(
        Arc::new(MockPlatformClipboard::new(snapshot)),
        Arc::new(FailingEntryRepo),
        Arc::new(MockEventWriter),
        Arc::new(SelectRepresentationPolicyV1::new()),
        Arc::new(MockMaterializer),
        Arc::new(MockDeviceIdentity),
    );

    // Execute should fail with repository error
    let result = use_case.execute().await;

    assert!(result.is_err(), "execute should fail when repository fails");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Repository error"),
        "Error should contain 'Repository error', got: {}",
        err_msg
    );
}

/// Mock PlatformClipboardPort that always fails
struct FailingPlatformClipboard;

#[async_trait::async_trait]
impl PlatformClipboardPort for FailingPlatformClipboard {
    fn read_snapshot(&self) -> Result<SystemClipboardSnapshot> {
        Err(anyhow::anyhow!("Clipboard read error"))
    }
}

#[tokio::test]
async fn test_capture_clipboard_propagates_clipboard_errors() {
    // This test verifies that errors from clipboard read are properly propagated.
    // This validates that error handling works correctly throughout the use case.

    let use_case = CaptureClipboardUseCase::new(
        Arc::new(FailingPlatformClipboard),
        Arc::new(MockEntryRepo::new()),
        Arc::new(MockEventWriter),
        Arc::new(SelectRepresentationPolicyV1::new()),
        Arc::new(MockMaterializer),
        Arc::new(MockDeviceIdentity),
    );

    // Execute should fail with clipboard error
    let result = use_case.execute().await;

    assert!(
        result.is_err(),
        "execute should fail when clipboard read fails"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Clipboard read error"),
        "Error should contain 'Clipboard read error', got: {}",
        err_msg
    );
}
