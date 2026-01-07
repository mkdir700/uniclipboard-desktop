use async_trait::async_trait;
use tauri::Manager;
use uc_app::AppEvent;

use crate::ports::app_event_handler::AppEventHandlerPort;

pub fn handlers(
    app_handle: tauri::AppHandle,
) -> Vec<Box<dyn AppEventHandlerPort<Event = AppEvent>>> {
    vec![Box::new(OpenSettingsWindowHandler::new(app_handle))
        as Box<dyn AppEventHandlerPort<Event = AppEvent>>]
}

pub struct OpenSettingsWindowHandler {
    app_handle: tauri::AppHandle,
}

impl OpenSettingsWindowHandler {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl AppEventHandlerPort for OpenSettingsWindowHandler {
    type Event = AppEvent;

    async fn handle(&self, event: AppEvent) {
        match event {
            AppEvent::OpenSettingsWindow => {
                if let Some(window) = self.app_handle.get_webview_window("settings") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    }
}
