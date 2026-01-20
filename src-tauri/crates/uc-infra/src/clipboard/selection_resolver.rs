//! Selection Resolver Implementation
//!
//! Loads complete selection context for an entry.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use uc_core::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
use uc_core::ids::EntryId;
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort, SelectionResolverPort,
};

/// Selection resolver implementation
pub struct SelectionResolver {
    entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
    selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
    representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
}

impl SelectionResolver {
    pub fn new(
        entry_repo: Arc<dyn ClipboardEntryRepositoryPort>,
        selection_repo: Arc<dyn ClipboardSelectionRepositoryPort>,
        representation_repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    ) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
        }
    }
}

#[async_trait]
impl SelectionResolverPort for SelectionResolver {
    async fn resolve_selection(
        &self,
        entry_id: &EntryId,
    ) -> Result<(ClipboardEntry, PersistedClipboardRepresentation)> {
        // 1. Load ClipboardEntry
        let entry = self
            .entry_repo
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Entry {} not found", entry_id))?;

        // 2. Load SelectionDecision
        let selection_decision = self
            .selection_repo
            .get_selection(entry_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Selection for entry {} not found", entry_id))?;

        // 3. Load target PersistedClipboardRepresentation
        let primary_rep_id = selection_decision.selection.primary_rep_id;
        let representation = self
            .representation_repo
            .get_representation(&entry.event_id, &primary_rep_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Representation {} not found for event {}",
                    primary_rep_id,
                    entry.event_id
                )
            })?;

        Ok((entry, representation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use uc_core::clipboard::{
        ClipboardEntry, ClipboardSelection, ClipboardSelectionDecision,
        PersistedClipboardRepresentation, SelectionPolicyVersion,
    };
    use uc_core::ids::{EntryId, EventId, FormatId, RepresentationId};

    /// Mock ClipboardEntryRepositoryPort
    struct MockEntryRepo {
        entry: Option<ClipboardEntry>,
    }

    #[async_trait]
    impl ClipboardEntryRepositoryPort for MockEntryRepo {
        async fn save_entry_and_selection(
            &self,
            _entry: &ClipboardEntry,
            _selection: &ClipboardSelectionDecision,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_entry(&self, _entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
            Ok(self.entry.clone())
        }

        async fn list_entries(&self, _limit: usize, _offset: usize) -> Result<Vec<ClipboardEntry>> {
            Ok(vec![])
        }

        async fn delete_entry(&self, _entry_id: &EntryId) -> Result<()> {
            Ok(())
        }
    }

    /// Mock ClipboardSelectionRepositoryPort
    struct MockSelectionRepo {
        selection: Option<ClipboardSelectionDecision>,
    }

    #[async_trait]
    impl ClipboardSelectionRepositoryPort for MockSelectionRepo {
        async fn get_selection(
            &self,
            _entry_id: &EntryId,
        ) -> Result<Option<ClipboardSelectionDecision>> {
            Ok(self.selection.clone())
        }

        async fn delete_selection(&self, _entry_id: &EntryId) -> Result<()> {
            Ok(())
        }
    }

    /// Mock ClipboardRepresentationRepositoryPort
    struct MockRepresentationRepo {
        representation: Option<PersistedClipboardRepresentation>,
    }

    #[async_trait]
    impl ClipboardRepresentationRepositoryPort for MockRepresentationRepo {
        async fn get_representation(
            &self,
            _event_id: &EventId,
            _representation_id: &uc_core::ids::RepresentationId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(self.representation.clone())
        }

        async fn get_representation_by_id(
            &self,
            _representation_id: &uc_core::ids::RepresentationId,
        ) -> Result<Option<PersistedClipboardRepresentation>> {
            Ok(self.representation.clone())
        }

        async fn update_blob_id(
            &self,
            _representation_id: &uc_core::ids::RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<()> {
            Ok(())
        }

        async fn update_blob_id_if_none(
            &self,
            _representation_id: &uc_core::ids::RepresentationId,
            _blob_id: &uc_core::BlobId,
        ) -> Result<bool> {
            Ok(false)
        }

        async fn update_processing_result(
            &self,
            _rep_id: &uc_core::ids::RepresentationId,
            _expected_states: &[uc_core::clipboard::PayloadAvailability],
            _blob_id: Option<&uc_core::BlobId>,
            _new_state: uc_core::clipboard::PayloadAvailability,
            _last_error: Option<&str>,
        ) -> Result<uc_core::ports::clipboard::ProcessingUpdateOutcome> {
            Ok(uc_core::ports::clipboard::ProcessingUpdateOutcome::NotFound)
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

    fn create_test_selection(entry_id: EntryId) -> ClipboardSelectionDecision {
        let primary_rep_id = RepresentationId::from("test-rep-id");
        ClipboardSelectionDecision::new(
            entry_id,
            ClipboardSelection {
                primary_rep_id: primary_rep_id.clone(),
                secondary_rep_ids: vec![],
                preview_rep_id: primary_rep_id.clone(),
                paste_rep_id: primary_rep_id,
                policy_version: SelectionPolicyVersion::V1,
            },
        )
    }

    fn create_test_representation() -> PersistedClipboardRepresentation {
        PersistedClipboardRepresentation::new(
            RepresentationId::from("test-rep-id"),
            FormatId::from("public.utf8-plain-text"),
            Some(uc_core::MimeType::text_plain()),
            100,
            Some(vec![1, 2, 3, 4, 5]),
            None,
        )
    }

    #[tokio::test]
    async fn test_resolve_selection_success() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let selection = create_test_selection(entry_id.clone());
        let representation = create_test_representation();

        let entry_repo = Arc::new(MockEntryRepo { entry: Some(entry) });
        let selection_repo = Arc::new(MockSelectionRepo {
            selection: Some(selection),
        });
        let representation_repo = Arc::new(MockRepresentationRepo {
            representation: Some(representation),
        });

        let resolver = SelectionResolver::new(entry_repo, selection_repo, representation_repo);

        let result = resolver.resolve_selection(&entry_id).await;

        assert!(result.is_ok(), "resolve_selection should succeed");
        let (returned_entry, returned_rep) = result.unwrap();
        assert_eq!(returned_entry.entry_id, entry_id);
        assert_eq!(returned_rep.id.as_ref(), "test-rep-id");
    }

    #[tokio::test]
    async fn test_resolve_selection_entry_not_found() {
        let entry_id = EntryId::from("missing-entry");
        let selection = create_test_selection(entry_id.clone());
        let representation = create_test_representation();

        let entry_repo = Arc::new(MockEntryRepo { entry: None });
        let selection_repo = Arc::new(MockSelectionRepo {
            selection: Some(selection),
        });
        let representation_repo = Arc::new(MockRepresentationRepo {
            representation: Some(representation),
        });

        let resolver = SelectionResolver::new(entry_repo, selection_repo, representation_repo);

        let result = resolver.resolve_selection(&entry_id).await;

        assert!(result.is_err(), "resolve_selection should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("not found"),
            "error should indicate entry not found: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_resolve_selection_not_found() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let representation = create_test_representation();

        let entry_repo = Arc::new(MockEntryRepo { entry: Some(entry) });
        let selection_repo = Arc::new(MockSelectionRepo { selection: None });
        let representation_repo = Arc::new(MockRepresentationRepo {
            representation: Some(representation),
        });

        let resolver = SelectionResolver::new(entry_repo, selection_repo, representation_repo);

        let result = resolver.resolve_selection(&entry_id).await;

        assert!(result.is_err(), "resolve_selection should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Selection") && err_msg.contains("not found"),
            "error should indicate selection not found: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_resolve_selection_representation_not_found() {
        let entry_id = EntryId::from("test-entry");
        let entry = create_test_entry(entry_id.clone());
        let selection = create_test_selection(entry_id.clone());

        let entry_repo = Arc::new(MockEntryRepo { entry: Some(entry) });
        let selection_repo = Arc::new(MockSelectionRepo {
            selection: Some(selection),
        });
        let representation_repo = Arc::new(MockRepresentationRepo {
            representation: None,
        });

        let resolver = SelectionResolver::new(entry_repo, selection_repo, representation_repo);

        let result = resolver.resolve_selection(&entry_id).await;

        assert!(result.is_err(), "resolve_selection should fail");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Representation") && err_msg.contains("not found"),
            "error should indicate representation not found: {}",
            err_msg
        );
    }
}
