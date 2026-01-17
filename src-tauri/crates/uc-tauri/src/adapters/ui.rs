use anyhow::Result;
use tauri::{AppHandle, Manager};
use uc_core::ports::UiPort;

/// Tauri-specific runtime adapter for UI operations.
///
/// This adapter must only be constructed inside Tauri setup phase
/// and must not be used outside uc-tauri.
pub struct TauriUiPort {
    app: AppHandle,
    settings_window_label: String,
}

impl TauriUiPort {
    #[allow(dead_code)]
    pub(crate) fn new(app: AppHandle, settings_window_label: impl Into<String>) -> Self {
        Self {
            app,
            settings_window_label: settings_window_label.into(),
        }
    }
}

#[async_trait::async_trait]
impl UiPort for TauriUiPort {
    async fn open_settings(&self) -> Result<()> {
        if let Some(win) = self.app.get_webview_window(&self.settings_window_label) {
            win.show()?;
            win.set_focus()?;
            return Ok(());
        }
        Err(anyhow::anyhow!(
            "Settings window '{}' not found",
            self.settings_window_label
        ))
    }
}
