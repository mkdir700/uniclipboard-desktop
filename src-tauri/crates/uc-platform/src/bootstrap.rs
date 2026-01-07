use tauri::AppHandle;
use uc_app::AppEvent;

use crate::app_runtime::{handlers, tauri_app_runtime::TauriAppRuntime};

pub fn bootstrap(app: AppHandle) -> TauriAppRuntime {
    let mut handlers_vec: Vec<Box<dyn AppEventHandlerPort<AppEvent>>> = vec![];

    // settings window
    handlers_vec.extend(handlers::settings_window::handlers(app.clone()));

    // 未来：
    // handlers_vec.extend(handlers::clipboard::handlers(app.clone()));
    // handlers_vec.extend(handlers::encryption::handlers(app.clone()));

    TauriAppRuntime::new(handlers_vec)
}
