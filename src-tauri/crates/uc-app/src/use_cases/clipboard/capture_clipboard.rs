use anyhow::Result;

use uc_core::{clipboard::{NewBlob, NewClipboardEvent}, ports::{
    BlobRepositoryPort, ClipboardEntryRepositoryPort, ClipboardEventRepositoryPort, ClipboardRepositoryPort, PlatformClipboardPort, blob
    SelectRepresentationPolicyPort,
}};

pub struct CaptureClipboardUseCase<PC, CE, CV, CR, B, S>
where
    PC: PlatformClipboardPort,
    CE: ClipboardEntryRepositoryPort,
    CV: ClipboardEventRepositoryPort,
    CR: ClipboardRepositoryPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
{
    platform_clipboard_port: PC,
    entry_repository: CE,
    event_repository: CV,
    clipboard_repository: CR,
    blob_repository: B,
    representation_policy: S
}

impl<PC, CE, CV, CR, B, S> CaptureClipboardUseCase<PC, CE, CV, CR, B, S>
where
    PC: PlatformClipboardPort,
    CE: ClipboardEntryRepositoryPort,
    CV: ClipboardEventRepositoryPort,
    CR: ClipboardRepositoryPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort
{
    pub fn new(
        platform_clipboard_port: PC,
        entry_repository: CE,
        event_repository: CV,
        clipboard_repository: CR,
        blob_repository: B,
        representation_policy: S
    ) -> Self {
        Self {
            platform_clipboard_port,
            entry_repository,
            event_repository,
            clipboard_repository,
            blob_repository,
            representation_policy
        }
    }
}

// 1. 生成 event + snapshot representations
// 2. blob_repo.insert_blob (必要时)
// 3. event_repo.insert_event
// 4. policy.select(snapshot)
// 5. entry_repo.insert_entry

impl<PC, CE, CV, CR, B, S> CaptureClipboardUseCase<PC, CE, CV, CR, B, S>
where
    PC: PlatformClipboardPort,
    CE: ClipboardEntryRepositoryPort,
    CV: ClipboardEventRepositoryPort,
    CR: ClipboardRepositoryPort,
    B: BlobRepositoryPort,
    S: SelectRepresentationPolicyPort,
{
    pub async fn execute(&self) -> Result<String> {
        // 从系统
        let snapshot = self.platform_clipboard_port.read_snapshot()?;
        
        let event_id = Uuid::new_v4().to_string();
        let captured_at_ms = snapshot.ts_ms;
        // TODO: 有一个 port 来获取 当前的 device id
        let source_device = "desktop".to_string();
        let snapshot_hash = snapshot.hash();

        // 1. 生成 event + snapshot representations
        let event = NewClipboardEvent::new(event_id, captured_at_ms, source_device, snapshot_hash);

        // 2. blob_repo.insert_blob (必要时)
        let new_blob = NewBlob::new(blob_id, storage_path, size_bytes, content_hash, encryption_algo, created_at_ms)
        self.blob_repository.insert_blob(new_blob)?;
        
        // 3. event_repo.insert_event
        self.event_repository.insert_event(event, representations);
        
        // 4. policy.select(snapshot)
        self.representation_policy
    }
}
