use anyhow::Result;
use tauri_plugin_autostart::ManagerExt as _;
use uc_core::ports::AutostartPort;

pub struct TauriAutostart {
    app_handle: tauri::AppHandle,
}

impl AutostartPort for TauriAutostart {
    fn is_enabled(&self) -> Result<bool> {
        let autostart_manager = self.app_handle.autolaunch();
        autostart_manager.is_enabled().map_err(anyhow::Error::from)
    }

    fn enable(&self) -> Result<()> {
        let autostart_manager = self.app_handle.autolaunch();
        let _ = autostart_manager.enable();
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        let autostart_manager = self.app_handle.autolaunch();
        let _ = autostart_manager.disable();
        Ok(())
    }
}
