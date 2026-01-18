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
    // Add unit tests with mock ports
}
