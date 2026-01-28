use crate::db::models::clipboard_selection::{ClipboardSelectionRow, NewClipboardSelectionRow};
use crate::db::ports::{InsertMapper, RowMapper};
use anyhow::Result;
use uc_core::clipboard::ClipboardSelectionDecision;

pub struct ClipboardSelectionRowMapper;

impl InsertMapper<ClipboardSelectionDecision, NewClipboardSelectionRow>
    for ClipboardSelectionRowMapper
{
    fn to_row(&self, domain: &ClipboardSelectionDecision) -> Result<NewClipboardSelectionRow> {
        let secondary_rep_ids = domain
            .selection
            .secondary_rep_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Ok(NewClipboardSelectionRow {
            entry_id: domain.entry_id.to_string(),
            primary_rep_id: domain.selection.primary_rep_id.to_string(),
            secondary_rep_ids,
            preview_rep_id: domain.selection.preview_rep_id.to_string(),
            paste_rep_id: domain.selection.paste_rep_id.to_string(),
            policy_version: domain.selection.policy_version.to_string(),
        })
    }
}

impl RowMapper<ClipboardSelectionRow, ClipboardSelectionDecision> for ClipboardSelectionRowMapper {
    fn to_domain(&self, row: &ClipboardSelectionRow) -> Result<ClipboardSelectionDecision> {
        use uc_core::{
            clipboard::ClipboardSelection,
            ids::{EntryId, RepresentationId},
        };

        // Parse secondary_rep_ids from comma-separated string with strict validation.
        // Empty string is valid (no secondary representations), but empty tokens are not.
        let secondary_rep_ids: Vec<RepresentationId> = if row.secondary_rep_ids.trim().is_empty() {
            Vec::new()
        } else {
            row.secondary_rep_ids
                .split(',')
                .enumerate()
                .map(|(i, s)| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        Err(anyhow::anyhow!(
                            "Empty token at position {} in secondary_rep_ids for entry {}",
                            i,
                            row.entry_id
                        ))
                    } else {
                        Ok(RepresentationId::from(trimmed.to_string()))
                    }
                })
                .collect::<Result<Vec<_>>>()?
        };

        // Parse policy_version
        let policy_version = row.policy_version.parse().map_err(|_| {
            anyhow::anyhow!(
                "Invalid policy_version '{}' for entry {}",
                row.policy_version,
                row.entry_id
            )
        })?;

        let selection = ClipboardSelection {
            primary_rep_id: RepresentationId::from(row.primary_rep_id.clone()),
            secondary_rep_ids,
            preview_rep_id: RepresentationId::from(row.preview_rep_id.clone()),
            paste_rep_id: RepresentationId::from(row.paste_rep_id.clone()),
            policy_version,
        };

        Ok(ClipboardSelectionDecision::new(
            EntryId::from(row.entry_id.clone()),
            selection,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::{
        clipboard::{ClipboardSelection, SelectionPolicyVersion},
        ids::RepresentationId,
    };

    #[test]
    fn test_row_mapper_success() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-1".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "rep-sec1,rep-sec2,rep-sec3".to_string(),
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(result.is_ok(), "Should successfully parse valid row");
        let decision = result.unwrap();
        assert_eq!(decision.entry_id.as_str(), "test-entry-1");
        assert_eq!(decision.selection.primary_rep_id.as_str(), "rep-primary");
        assert_eq!(decision.selection.secondary_rep_ids.len(), 3);
        assert_eq!(decision.selection.secondary_rep_ids[0].as_str(), "rep-sec1");
        assert_eq!(decision.selection.secondary_rep_ids[1].as_str(), "rep-sec2");
        assert_eq!(decision.selection.secondary_rep_ids[2].as_str(), "rep-sec3");
        assert_eq!(decision.selection.preview_rep_id.as_str(), "rep-preview");
        assert_eq!(decision.selection.paste_rep_id.as_str(), "rep-paste");
        assert_eq!(
            decision.selection.policy_version,
            SelectionPolicyVersion::V1
        );
    }

    #[test]
    fn test_row_mapper_single_secondary_rep() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-2".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "rep-only".to_string(),
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert_eq!(decision.selection.secondary_rep_ids.len(), 1);
        assert_eq!(decision.selection.secondary_rep_ids[0].as_str(), "rep-only");
    }

    #[test]
    fn test_row_mapper_empty_secondary_rep_ids_ok() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-3".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "".to_string(),
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(result.is_ok(), "Empty secondary_rep_ids should be allowed");
        let decision = result.unwrap();
        assert!(decision.selection.secondary_rep_ids.is_empty());
    }

    #[test]
    fn test_row_mapper_invalid_policy_version_fails() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-4".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "rep-sec1".to_string(),
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "invalid-version".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(
            result.is_err(),
            "Invalid policy_version should return error"
        );
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid policy_version"));
    }

    #[test]
    fn test_insert_mapper_round_trip() {
        use uc_core::clipboard::ClipboardSelectionDecision;
        use uc_core::ids::EntryId;

        let decision = ClipboardSelectionDecision::new(
            EntryId::from("test-entry".to_string()),
            ClipboardSelection {
                primary_rep_id: RepresentationId::from("rep-primary".to_string()),
                secondary_rep_ids: vec![
                    RepresentationId::from("rep-sec1".to_string()),
                    RepresentationId::from("rep-sec2".to_string()),
                ],
                preview_rep_id: RepresentationId::from("rep-preview".to_string()),
                paste_rep_id: RepresentationId::from("rep-paste".to_string()),
                policy_version: SelectionPolicyVersion::V1,
            },
        );

        let mapper = ClipboardSelectionRowMapper;
        let row = mapper.to_row(&decision).unwrap();

        assert_eq!(row.entry_id, "test-entry");
        assert_eq!(row.primary_rep_id, "rep-primary");
        assert_eq!(row.secondary_rep_ids, "rep-sec1,rep-sec2");
        assert_eq!(row.preview_rep_id, "rep-preview");
        assert_eq!(row.paste_rep_id, "rep-paste");
        assert_eq!(row.policy_version, "v1");
    }

    #[test]
    fn test_row_mapper_empty_token_in_middle_fails() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-empty-token".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "rep-sec1,,rep-sec2".to_string(), // Empty token in middle
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(result.is_err(), "Empty token should return error");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Empty token at position"));
    }

    #[test]
    fn test_row_mapper_whitespace_only_tokens_fail() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-whitespace".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: "rep-sec1,  ,rep-sec2".to_string(), // Whitespace-only token
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(result.is_err(), "Whitespace-only token should return error");
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Empty token at position"));
    }

    #[test]
    fn test_row_mapper_trims_whitespace_from_tokens() {
        let row = ClipboardSelectionRow {
            entry_id: "test-entry-trim".to_string(),
            primary_rep_id: "rep-primary".to_string(),
            secondary_rep_ids: " rep-sec1 , rep-sec2 , rep-sec3 ".to_string(), // Has whitespace
            preview_rep_id: "rep-preview".to_string(),
            paste_rep_id: "rep-paste".to_string(),
            policy_version: "v1".to_string(),
        };

        let mapper = ClipboardSelectionRowMapper;
        let result = mapper.to_domain(&row);

        assert!(
            result.is_ok(),
            "Should trim whitespace and parse successfully"
        );
        let decision = result.unwrap();
        assert_eq!(decision.selection.secondary_rep_ids.len(), 3);
        assert_eq!(decision.selection.secondary_rep_ids[0].as_str(), "rep-sec1");
        assert_eq!(decision.selection.secondary_rep_ids[1].as_str(), "rep-sec2");
        assert_eq!(decision.selection.secondary_rep_ids[2].as_str(), "rep-sec3");
    }
}
