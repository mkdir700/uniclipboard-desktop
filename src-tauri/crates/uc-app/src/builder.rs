use std::sync::Arc;
use uc_core::ports::{AutostartPort, UiPort};

/// Builder for assembling the application runtime.
pub struct AppBuilder {
    autostart: Option<Arc<dyn AutostartPort>>,
    ui_port: Option<Arc<dyn UiPort>>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            autostart: None,
            ui_port: None,
        }
    }

    pub fn with_autostart(mut self, autostart: Arc<dyn AutostartPort>) -> Self {
        self.autostart = Some(autostart);
        self
    }

    pub fn with_ui_port(mut self, ui_port: Arc<dyn UiPort>) -> Self {
        self.ui_port = Some(ui_port);
        self
    }

    pub fn build(self) -> anyhow::Result<App> {
        Ok(App {
            autostart: self.autostart.ok_or_else(|| {
                anyhow::anyhow!("AutostartPort is required")
            })?,
            ui_port: self.ui_port.ok_or_else(|| {
                anyhow::anyhow!("UiPort is required")
            })?,
        })
    }
}

/// The application runtime.
pub struct App {
    pub autostart: Arc<dyn AutostartPort>,
    pub ui_port: Arc<dyn UiPort>,
}
