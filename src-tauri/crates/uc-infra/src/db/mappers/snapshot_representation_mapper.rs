use crate::db::models::snapshot_representation::{NewSnapshotRepresentationRow, SnapshotRepresentationRow};
use crate::db::ports::{InsertMapper, RowMapper};
use anyhow::Result;
use uc_core::{clipboard::PersistedClipboardRepresentation, ids::{EventId, FormatId, RepresentationId}, BlobId, MimeType};

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

// Blanket implementation for references: if we can map from owned values,
// we can also map from references by dereferencing
impl<'a> InsertMapper<(&'a PersistedClipboardRepresentation, &'a EventId), NewSnapshotRepresentationRow>
    for RepresentationRowMapper
where
    Self: InsertMapper<(PersistedClipboardRepresentation, EventId), NewSnapshotRepresentationRow>,
{
    fn to_row(
        &self,
        domain: &(&'a PersistedClipboardRepresentation, &'a EventId),
    ) -> Result<NewSnapshotRepresentationRow> {
        let (rep, event_id) = domain;
        // Convert references to owned values for the owned implementation
        let owned_domain = ((**rep).clone(), (**event_id).clone());
        <Self as InsertMapper<(PersistedClipboardRepresentation, EventId), NewSnapshotRepresentationRow>>::to_row(self, &owned_domain)
    }
}

impl RowMapper<SnapshotRepresentationRow, uc_core::clipboard::PersistedClipboardRepresentation>
    for RepresentationRowMapper
{
    fn to_domain(
        &self,
        row: &SnapshotRepresentationRow,
    ) -> Result<uc_core::clipboard::PersistedClipboardRepresentation> {
        Ok(uc_core::clipboard::PersistedClipboardRepresentation::new(
            RepresentationId::from(row.id.clone()),
            FormatId::from(row.format_id.clone()),
            row.mime_type.as_ref().map(|s| MimeType(s.clone())),
            row.size_bytes,
            row.inline_data.clone(),
            row.blob_id.as_ref().map(|s| BlobId::from(s.clone())),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::snapshot_representation::SnapshotRepresentationRow;
    use uc_core::{clipboard::PersistedClipboardRepresentation, ids::{RepresentationId, FormatId, EventId}, BlobId, MimeType};

    #[test]
    fn test_row_mapper_all_fields() {
        let mapper = RepresentationRowMapper;
        let row = SnapshotRepresentationRow {
            id: "test-rep-id".to_string(),
            event_id: "test-event-id".to_string(),
            format_id: "public.utf8-plain-text".to_string(),
            mime_type: Some("text/plain".to_string()),
            size_bytes: 42,
            inline_data: Some(vec![1, 2, 3]),
            blob_id: None,
        };

        let result = mapper.to_domain(&row).unwrap();

        assert_eq!(result.id.to_string(), "test-rep-id");
        assert_eq!(result.format_id.to_string(), "public.utf8-plain-text");
        assert_eq!(result.mime_type, Some(MimeType("text/plain".to_string())));
        assert_eq!(result.size_bytes, 42);
        assert_eq!(result.inline_data, Some(vec![1, 2, 3]));
        assert_eq!(result.blob_id, None);
    }

    #[test]
    fn test_row_mapper_optional_fields_none() {
        let mapper = RepresentationRowMapper;
        let row = SnapshotRepresentationRow {
            id: "test-rep-id-2".to_string(),
            event_id: "test-event-id-2".to_string(),
            format_id: "public.png".to_string(),
            mime_type: None,
            size_bytes: 1024,
            inline_data: None,
            blob_id: Some("blob-123".to_string()),
        };

        let result = mapper.to_domain(&row).unwrap();

        assert_eq!(result.id.to_string(), "test-rep-id-2");
        assert_eq!(result.mime_type, None);
        assert_eq!(result.inline_data, None);
        assert_eq!(result.blob_id, Some(BlobId::from("blob-123".to_string())));
    }

    #[test]
    fn test_insert_mapper() {
        let mapper = RepresentationRowMapper;
        let rep = PersistedClipboardRepresentation::new(
            RepresentationId::from("rep-456".to_string()),
            FormatId::from("public.html".to_string()),
            Some(MimeType("text/html".to_string())),
            100,
            Some(vec![10, 20, 30]),
            None,
        );
        let event_id = EventId::from("event-789".to_string());

        // Create a tuple reference to pass to to_row
        let input = (rep, event_id);
        let row = mapper.to_row(&input).unwrap();

        assert_eq!(row.id, "rep-456");
        assert_eq!(row.event_id, "event-789");
        assert_eq!(row.format_id, "public.html");
        assert_eq!(row.mime_type, Some("text/html".to_string()));
        assert_eq!(row.size_bytes, 100);
        assert_eq!(row.inline_data, Some(vec![10, 20, 30]));
        assert_eq!(row.blob_id, None);
    }
}
