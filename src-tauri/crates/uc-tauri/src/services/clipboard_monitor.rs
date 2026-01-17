//! Clipboard Monitoring Service
//! 剪贴板监控服务

use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time::interval;
use uc_app::AppDeps;

/// Clipboard monitoring service
/// 剪贴板监控服务
pub struct ClipboardMonitor {
    app: AppHandle,
    #[allow(dead_code)]
    deps: Arc<AppDeps>,
    interval_secs: u64,
}

impl ClipboardMonitor {
    pub fn new(app: AppHandle, deps: Arc<AppDeps>) -> Self {
        Self {
            app,
            deps,
            interval_secs: 1, // Check every second / 每秒检查一次
        }
    }

    /// Start the clipboard monitoring loop
    /// 启动剪贴板监控循环
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut timer = interval(Duration::from_secs(self.interval_secs));

        loop {
            timer.tick().await;

            // TODO: Implement actual clipboard capture
            // This requires ClipboardEventWriterPort in AppDeps
            // TODO: 实现实际的剪贴板捕获
            // 这需要在 AppDeps 中添加 ClipboardEventWriterPort

            // Emit heartbeat event for now
            // 目前发送心跳事件
            let _ = self.app.emit("clipboard://monitor_heartbeat", ());
        }
    }
}
