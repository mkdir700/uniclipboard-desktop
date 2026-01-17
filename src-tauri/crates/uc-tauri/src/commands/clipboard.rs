//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::models::{ClipboardEntryDetail, ClipboardEntryProjection};
use std::sync::Arc;
use tauri::State;
use tracing::{info_span, Instrument};

/// Get clipboard history entries (preview only)
/// 获取剪贴板历史条目（仅预览）
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let resolved_limit = limit.unwrap_or(50);
    let resolved_offset = offset.unwrap_or(0);
    let device_id = runtime.deps.device_identity.current_device_id();

    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %device_id,
        limit = resolved_limit,
        offset = resolved_offset,
    );

    async move {
        let uc = runtime.usecases().list_clipboard_entries();
        let entries = uc.execute(resolved_limit, resolved_offset).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to get clipboard entries");
            e.to_string()
        })?;

        let mut projections = Vec::with_capacity(entries.len());

        for entry in entries {
            let captured_at = entry.created_at_ms;

            // Get selection and representation in one pass
            let content_type = match runtime
                .deps
                .selection_repo
                .get_selection(&entry.entry_id)
                .await
            {
                Ok(Some(selection)) => {
                    match runtime
                        .deps
                        .representation_repo
                        .get_representation(&entry.event_id, &selection.selection.preview_rep_id)
                        .await
                    {
                        Ok(Some(rep)) => rep
                            .mime_type
                            .as_ref()
                            .map(|mt| mt.as_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        _ => "unknown".to_string(),
                    }
                }
                _ => "unknown".to_string(),
            };

            let (preview, has_detail) = if let Ok(Some(selection)) = runtime
                .deps
                .selection_repo
                .get_selection(&entry.entry_id)
                .await
            {
                // Use preview_rep_id for list display (PlainText preferred for UI preview)
                if let Ok(Some(rep)) = runtime
                    .deps
                    .representation_repo
                    .get_representation(&entry.event_id, &selection.selection.preview_rep_id)
                    .await
                {
                    let preview_text = if let Some(data) = rep.inline_data {
                        String::from_utf8_lossy(&data).trim().to_string()
                    } else {
                        format!("Image ({} bytes)", rep.size_bytes)
                    };
                    let has_detail = rep.blob_id.is_some();
                    (preview_text, has_detail)
                } else {
                    (
                        entry
                            .title
                            .unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                        false,
                    )
                }
            } else {
                (
                    entry
                        .title
                        .unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                    false,
                )
            };

            projections.push(ClipboardEntryProjection {
                id: entry.entry_id.to_string(),
                preview,
                has_detail,
                size_bytes: entry.total_size,
                captured_at,
                content_type,
                is_encrypted: false,
                is_favorited: false,
                updated_at: captured_at,
                active_time: captured_at,
            });
        }

        tracing::info!(count = projections.len(), "Retrieved clipboard entries");
        Ok(projections)
    }
    .instrument(span)
    .await
}

/// Deletes a clipboard entry identified by `entry_id`.
///
/// This command converts the provided `entry_id` to the domain `EntryId` type and invokes the runtime's
/// delete clipboard-entry use case; on success it returns without value, otherwise it returns a stringified error.
///
/// # Examples
///
/// ```no_run
/// # use std::sync::Arc;
/// # async fn example(runtime: tauri::State<'_, Arc<uc_tauri::bootstrap::AppRuntime>>) {
/// // Tauri provides `State<Arc<AppRuntime>>` when invoking commands from the frontend.
/// let result = uc_tauri::commands::clipboard::delete_clipboard_entry(runtime, "entry-id-123".to_string()).await;
/// match result {
///     Ok(()) => println!("Deleted"),
///     Err(e) => eprintln!("Delete failed: {}", e),
/// }
/// # }
/// ```
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<(), String> {
    let device_id = runtime.deps.device_identity.current_device_id();

    let span = info_span!(
        "command.clipboard.delete_entry",
        device_id = %device_id,
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());
        let use_case = runtime.usecases().delete_clipboard_entry();
        use_case.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to delete entry");
            e.to_string()
        })?;

        tracing::info!(entry_id = %entry_id, "Deleted clipboard entry");
        Ok(())
    }
    .instrument(span)
    .await
}

/// Get full clipboard entry detail
/// 获取剪贴板条目完整详情
#[tauri::command]
pub async fn get_clipboard_entry_detail(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<ClipboardEntryDetail, String> {
    let span = info_span!(
        "command.clipboard.get_entry_detail",
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());
        let use_case = runtime.usecases().get_entry_detail();
        let result = use_case.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to get entry detail");
            e.to_string()
        })?;

        let detail = ClipboardEntryDetail {
            id: result.id,
            content: result.content,
            size_bytes: result.size_bytes,
            content_type: result
                .mime_type
                .unwrap_or_else(|| "unknown".to_string()),
            is_favorited: false,
            updated_at: result.created_at_ms,
            active_time: result.created_at_ms,
        };

        tracing::info!(entry_id = %entry_id, "Retrieved clipboard entry detail");
        Ok(detail)
    }
    .instrument(span)
    .await
}
