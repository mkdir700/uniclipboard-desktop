//! Startup orchestration commands
//! 启动流程编排命令

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use tracing::{info, warn};

/// Startup barrier used to coordinate "frontend ready" and "backend ready".
///
/// 用于协调“前端就绪”和“后端就绪”的启动门闩。
///
/// # Behavior / 行为
/// - When both sides are ready, it shows the main window and closes the splashscreen window.
/// - 当两侧都就绪时，显示主窗口并关闭启动页窗口。
#[derive(Default)]
pub struct StartupBarrier {
    frontend_ready: AtomicBool,
    backend_ready: AtomicBool,
    finished: AtomicBool,
}

impl StartupBarrier {
    /// Mark the frontend as ready.
    ///
    /// 标记前端已就绪。
    pub fn mark_frontend_ready(&self) {
        self.frontend_ready.store(true, Ordering::SeqCst);
    }

    /// Mark the backend as ready.
    ///
    /// 标记后端已就绪。
    pub fn mark_backend_ready(&self) {
        self.backend_ready.store(true, Ordering::SeqCst);
    }

    /// Try to finish startup once (idempotent).
    ///
    /// 尝试完成启动收尾（幂等）。
    pub fn try_finish(&self, app_handle: &AppHandle) {
        if self.finished.load(Ordering::SeqCst) {
            return;
        }

        let frontend_ready = self.frontend_ready.load(Ordering::SeqCst);
        let backend_ready = self.backend_ready.load(Ordering::SeqCst);
        if !frontend_ready || !backend_ready {
            info!(
                frontend_ready,
                backend_ready, "StartupBarrier not ready to finish yet"
            );
            return;
        }

        if self
            .finished
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        if let Some(main_window) = app_handle.get_webview_window("main") {
            if let Err(e) = main_window.show() {
                warn!("Failed to show main window: {}", e);
            } else {
                info!("Main window shown (startup barrier)");
            }
        } else {
            warn!("Main window not found (startup barrier)");
        }

        if let Some(splash_window) = app_handle.get_webview_window("splashscreen") {
            if let Err(e) = splash_window.close() {
                warn!("Failed to close splashscreen: {}", e);
            } else {
                info!("Splashscreen closed (startup barrier)");
            }
        }
    }
}

/// Notify backend that the frontend has finished initial mounting.
///
/// 通知后端：前端已完成初次挂载。
#[tauri::command]
pub async fn frontend_ready(
    app_handle: AppHandle,
    barrier: State<'_, Arc<StartupBarrier>>,
) -> Result<(), String> {
    info!("Received frontend_ready handshake");
    barrier.mark_frontend_ready();
    barrier.try_finish(&app_handle);
    Ok(())
}
