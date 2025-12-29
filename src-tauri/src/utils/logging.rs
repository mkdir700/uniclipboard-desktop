use crate::utils::env::is_development;
use log::LevelFilter;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};

/// 初始化日志系统构建器
pub fn get_builder() -> tauri_plugin_log::Builder {
    let is_dev = is_development();
    let default_log_level = if is_dev {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let mut builder = tauri_plugin_log::Builder::new()
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .level(default_log_level)
        .format(|out, message, record| {
            // 保持现有格式: 时间戳 级别 [文件:行号] [模块] 消息
            let level_color = match record.level() {
                log::Level::Error => "\x1b[31;1m", // 红色加粗
                log::Level::Warn => "\x1b[33m",     // 黄色
                log::Level::Info => "\x1b[32m",     // 绿色
                log::Level::Debug => "\x1b[34m",    // 蓝色
                log::Level::Trace => "\x1b[36m",    // 青色
            };
            let reset = "\x1b[0m";

            let file = record.file().unwrap_or("unknown");
            let line = record.line().unwrap_or(0);
            let target = record.target();

            // 格式: 2025-12-29 10:30:45.123 INFO [main.rs:34] [uniclipboard] Self device already exists
            out.finish(format_args!(
                "{} {}{} [{}:{}] [{}] {}{}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                level_color,
                record.level(),
                file,
                line,
                target,
                message,
                reset
            ))
        });

    // 根据环境配置不同的日志目标
    if is_dev {
        // 开发环境: 输出到 Webview（浏览器 DevTools）
        builder = builder.target(Target::new(TargetKind::Webview));
    } else {
        // 生产环境: 输出到文件和可选的 Stdout
        // 使用 LogDir 目标，文件名为 uniclipboard.log
        builder = builder
            .target(Target::new(TargetKind::LogDir {
                file_name: Some("uniclipboard.log".to_string()),
            }))
            .target(Target::new(TargetKind::Stdout)); // 可选：保留终端输出
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        // 测试日志构建器是否正常
        let _builder = get_builder();
    }
}
