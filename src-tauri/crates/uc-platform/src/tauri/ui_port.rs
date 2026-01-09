use anyhow::Result;
use tauri::Manager;
use uc_core::ports::UiPort;

pub struct TauriUiPort {
    app: tauri::AppHandle,
}

#[async_trait::async_trait]
impl UiPort for TauriUiPort {
    async fn open_settings(&self) -> Result<()> {
        if let Some(win) = self.app.get_webview_window("settings") {
            let _ = win.show();
            let _ = win.set_focus();
            return Ok(());
        }
        Err(anyhow::anyhow!("Failed to open settings window"))
    }
}
