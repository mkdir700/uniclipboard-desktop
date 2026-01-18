//! Selection Resolver Implementation
//!
//! Loads complete selection context for an entry.

use anyhow::Result;
use async_trait::async_trait;
use uc_core::clipboard::{ClipboardEntry, PersistedClipboardRepresentation};
use uc_core::ids::{EntryId, EventId, RepresentationId};
use uc_core::ports::{
    ClipboardEntryRepositoryPort, ClipboardRepresentationRepositoryPort,
    ClipboardSelectionRepositoryPort, SelectionResolverPort,
};

/// Selection resolver implementation
pub struct SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    entry_repo: E,
    selection_repo: S,
    representation_repo: R,
}

impl<E, S, R> SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
    pub fn new(entry_repo: E, selection_repo: S, representation_repo: R) -> Self {
        Self {
            entry_repo,
            selection_repo,
            representation_repo,
        }
    }
}

#[async_trait]
impl<E, S, R> SelectionResolverPort for SelectionResolver<E, S, R>
where
    E: ClipboardEntryRepositoryPort,
    S: ClipboardSelectionRepositoryPort,
    R: ClipboardRepresentationRepositoryPort,
{
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
    // Add unit tests with mock repositories
}
