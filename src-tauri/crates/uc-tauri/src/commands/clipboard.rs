//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use std::sync::Arc;
use tauri::State;
use crate::bootstrap::AppRuntime;
use crate::models::ClipboardEntryProjection;
use tracing::info_span;  // NEW: Import for span creation

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    // Create root span for this command
    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %runtime.deps.device_identity.current_device_id(),
        limit = limit.unwrap_or(50),
    );
    let _enter = span.enter();

    // Use UseCases accessor pattern (consistent with other commands)
    let uc = runtime.usecases().list_clipboard_entries();
    let limit = limit.unwrap_or(50);

    // Query entries through use case
    let entries = uc.execute(limit, 0)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get clipboard entries");
            e.to_string()
        })?;

    // Convert domain models to DTOs
    let projections: Vec<ClipboardEntryProjection> = entries
        .into_iter()
        .map(|entry| ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview: entry.title.unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
            captured_at: entry.created_at_ms,
            content_type: "clipboard".to_string(),
            is_encrypted: false, // TODO: Determine from actual entry state
        })
        .collect();

    tracing::info!(count = projections.len(), "Retrieved clipboard entries");
    Ok(projections)
}

/// Deletes a clipboard entry identified by `entry_id`.
///
/// This command converts the provided `entry_id` to the domain `EntryId` type and invokes the runtime's
/// delete clipboard-entry use case; on success it returns without value, otherwise it returns a stringified error.
///
/// # Examples
///
/// ```no_run
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
    // Create root span for this command
    let span = info_span!(
        "command.clipboard.delete_entry",
        device_id = %runtime.deps.device_identity.current_device_id(),
        entry_id = %entry_id,
    );
    let _enter = span.enter();

    // Parse entry_id (From trait always succeeds)
    let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());

    // Execute use case
    let use_case = runtime.usecases().delete_clipboard_entry();
    use_case.execute(&parsed_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to delete entry");
            e.to_string()
        })?;

    tracing::info!(entry_id = %entry_id, "Deleted clipboard entry");
    Ok(())
}
