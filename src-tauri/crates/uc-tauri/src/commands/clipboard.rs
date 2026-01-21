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
        // Check encryption session readiness to avoid decryption failures during startup
        let encryption_state = runtime
            .deps
            .encryption_state
            .load_state()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to check encryption state");
                format!("Failed to check encryption state: {}", e)
            })?;

        if encryption_state == uc_core::security::state::EncryptionState::Initialized {
            // Encryption is initialized, check if session is ready
            if !runtime.deps.encryption_session.is_ready().await {
                tracing::warn!(
                    "Encryption initialized but session not ready yet, returning empty list. \
                     This typically happens during app startup before auto-unlock completes."
                );
                return Ok(vec![]);
            }
        }

        let uc = runtime.usecases().list_entry_projections();
        let dtos = uc
            .execute(resolved_limit, resolved_offset)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get clipboard entry projections");
                e.to_string()
            })?;

        // Map DTOs to command layer models
        let projections: Vec<ClipboardEntryProjection> = dtos
            .into_iter()
            .map(|dto| ClipboardEntryProjection {
                id: dto.id,
                preview: dto.preview,
                has_detail: dto.has_detail,
                size_bytes: dto.size_bytes,
                captured_at: dto.captured_at,
                content_type: dto.content_type,
                is_encrypted: dto.is_encrypted,
                is_favorited: dto.is_favorited,
                updated_at: dto.updated_at,
                active_time: dto.active_time,
            })
            .collect();

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
            content_type: result.mime_type.unwrap_or_else(|| "unknown".to_string()),
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
