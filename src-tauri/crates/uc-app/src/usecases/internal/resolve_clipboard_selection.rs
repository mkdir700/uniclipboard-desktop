use anyhow::Result;
use std::sync::Arc;
use tracing::{info, info_span, Instrument};

use uc_core::ids::EntryId;
use uc_core::ports::clipboard::{
    ClipboardPayloadResolverPort, ResolvedClipboardPayload, SelectionResolverPort,
};

/// Resolve clipboard selection payload use case.
///
/// This use case orchestrates the resolution of a clipboard entry's
/// selected representation into a usable payload (inline or blob reference).
///
/// # Behavior
/// 1. Resolve the complete selection context (entry + representation)
/// 2. Resolve the representation into a payload (inline or blob)
/// 3. Return the payload for use by the UI or paste operations
pub struct ResolveClipboardSelectionPayloadUseCase {
    selection_resolver: Arc<dyn SelectionResolverPort>,
    payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
}

impl ResolveClipboardSelectionPayloadUseCase {
    pub fn new(
        selection_resolver: Arc<dyn SelectionResolverPort>,
        payload_resolver: Arc<dyn ClipboardPayloadResolverPort>,
    ) -> Self {
        Self {
            selection_resolver,
            payload_resolver,
        }
    }

    /// Execute the resolution workflow.
    ///
    /// # Returns
    /// - `ResolvedClipboardPayload` containing either inline data or blob reference
    pub async fn execute(&self, entry_id: &EntryId) -> Result<ResolvedClipboardPayload> {
        let span = info_span!(
            "usecase.resolve_clipboard_selection.execute",
            entry_id = %entry_id,
        );
        async move {
            info!("Resolving clipboard selection for entry {}", entry_id);

            // 1. Resolve selection context
            let (_entry, rep) = self.selection_resolver.resolve_selection(entry_id).await?;

            // 2. Resolve payload
            let payload = self.payload_resolver.resolve(&rep).await?;

            info!(entry_id = %entry_id, "Clipboard selection resolved");
            Ok(payload)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use uc_core::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
    use uc_core::ids::{EntryId, EventId, FormatId, RepresentationId};
    use uc_core::BlobId;
    use uc_core::MimeType;

    /// Mock SelectionResolverPort
    struct MockSelectionResolver {
        entry: Option<ClipboardEntry>,
        representation: Option<PersistedClipboardRepresentation>,
    }

    impl MockSelectionResolver {
        fn new() -> Self {
            Self {
                entry: None,
                representation: None,
            }
        }

        fn with_entry(mut self, entry: ClipboardEntry) -> Self {
            self.entry = Some(entry);
            self
        }

        fn with_representation(mut self, rep: PersistedClipboardRepresentation) -> Self {
            self.representation = Some(rep);
            self
        }
    }

    #[async_trait]
    impl SelectionResolverPort for MockSelectionResolver {
        async fn resolve_selection(
            &self,
            _entry_id: &EntryId,
        ) -> Result<(ClipboardEntry, PersistedClipboardRepresentation)> {
            let entry = self
                .entry
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;
            let rep = self
                .representation
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Representation not found"))?;
            Ok((entry, rep))
        }
    }

    /// Mock ClipboardPayloadResolverPort
    struct MockPayloadResolver {
        payload: Option<ResolvedClipboardPayload>,
    }

    impl MockPayloadResolver {
        fn new() -> Self {
            Self { payload: None }
        }

        fn with_inline_payload(mut self, mime: String, bytes: Vec<u8>) -> Self {
            self.payload = Some(ResolvedClipboardPayload::Inline { mime, bytes });
            self
        }

        fn with_blob_ref_payload(mut self, mime: String, blob_id: BlobId) -> Self {
            self.payload = Some(ResolvedClipboardPayload::BlobRef { mime, blob_id });
            self
        }
    }

    #[async_trait]
    impl ClipboardPayloadResolverPort for MockPayloadResolver {
        async fn resolve(
            &self,
            _representation: &PersistedClipboardRepresentation,
        ) -> Result<ResolvedClipboardPayload> {
            self.payload
                .clone()
                .ok_or_else(|| anyhow::anyhow!("Payload not available"))
        }
    }

    fn create_test_entry(entry_id: EntryId) -> ClipboardEntry {
        ClipboardEntry::new(
            entry_id.clone(),
            EventId::new(),
            12345,
            Some("test title".to_string()),
            100,
        )
    }

    fn create_test_representation() -> PersistedClipboardRepresentation {
        PersistedClipboardRepresentation::new(
            RepresentationId::new(),
            FormatId::from("public.utf8-plain-text"),
            Some(MimeType::text_plain()),
            100,
            Some(vec![1, 2, 3, 4, 5]),
            None,
        )
    }

    #[tokio::test]
    async fn test_execute_returns_inline_payload() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let representation = create_test_representation();

        let selection_resolver = Arc::new(
            MockSelectionResolver::new()
                .with_entry(entry)
                .with_representation(representation),
        );
        let payload_resolver = Arc::new(
            MockPayloadResolver::new().with_inline_payload("text/plain".to_string(), vec![1, 2, 3]),
        );

        let use_case =
            ResolveClipboardSelectionPayloadUseCase::new(selection_resolver, payload_resolver);

        let result = use_case.execute(&entry_id).await;

        assert!(result.is_ok(), "execute should succeed");
        let payload = result.unwrap();
        match payload {
            ResolvedClipboardPayload::Inline { mime, bytes } => {
                assert_eq!(mime, "text/plain");
                assert_eq!(bytes, vec![1, 2, 3]);
            }
            _ => panic!("Expected Inline payload"),
        }
    }

    #[tokio::test]
    async fn test_execute_returns_blob_ref_payload() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let representation = create_test_representation();
        let blob_id = BlobId::new();

        let selection_resolver = Arc::new(
            MockSelectionResolver::new()
                .with_entry(entry)
                .with_representation(representation),
        );
        let payload_resolver = Arc::new(
            MockPayloadResolver::new()
                .with_blob_ref_payload("image/png".to_string(), blob_id.clone()),
        );

        let use_case =
            ResolveClipboardSelectionPayloadUseCase::new(selection_resolver, payload_resolver);

        let result = use_case.execute(&entry_id).await;

        assert!(result.is_ok(), "execute should succeed");
        let payload = result.unwrap();
        match payload {
            ResolvedClipboardPayload::BlobRef {
                mime,
                blob_id: returned_id,
            } => {
                assert_eq!(mime, "image/png");
                assert_eq!(returned_id, blob_id);
            }
            _ => panic!("Expected BlobRef payload"),
        }
    }

    #[tokio::test]
    async fn test_execute_fails_when_selection_resolver_fails() {
        let entry_id = EntryId::from("missing-entry");

        let selection_resolver = Arc::new(MockSelectionResolver::new());
        let payload_resolver = Arc::new(MockPayloadResolver::new());

        let use_case =
            ResolveClipboardSelectionPayloadUseCase::new(selection_resolver, payload_resolver);

        let result = use_case.execute(&entry_id).await;

        assert!(result.is_err(), "execute should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found"),
            "error should indicate entry not found: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_execute_fails_when_payload_resolver_fails() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let representation = create_test_representation();

        let selection_resolver = Arc::new(
            MockSelectionResolver::new()
                .with_entry(entry)
                .with_representation(representation),
        );
        let payload_resolver = Arc::new(MockPayloadResolver::new());

        let use_case =
            ResolveClipboardSelectionPayloadUseCase::new(selection_resolver, payload_resolver);

        let result = use_case.execute(&entry_id).await;

        assert!(result.is_err(), "execute should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not available"),
            "error should indicate payload not available: {}",
            err_msg
        );
    }
}
