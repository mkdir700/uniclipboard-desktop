use crate::db::models::snapshot_representation::NewSnapshotRepresentationRow;
use crate::db::ports::InsertMapper;
use anyhow::Result;
use uc_core::{clipboard::PersistedClipboardRepresentation, ids::EventId};

pub struct RepresentationRowMapper;

impl InsertMapper<(PersistedClipboardRepresentation, EventId), NewSnapshotRepresentationRow>
    for RepresentationRowMapper
{
    fn to_row(
        &self,
        domain: &(PersistedClipboardRepresentation, EventId),
    ) -> Result<NewSnapshotRepresentationRow> {
        let (rep, event_id) = domain;
        Ok(NewSnapshotRepresentationRow {
            id: rep.id.to_string(),
            event_id: event_id.to_string(),
            format_id: rep.format_id.to_string(),
            mime_type: rep.mime_type.as_ref().map(|m| m.to_string()),
            size_bytes: rep.size_bytes,
            inline_data: rep.inline_data.clone(),
            blob_id: rep.blob_id.as_ref().map(|id| id.to_string()),
        })
    }
}
