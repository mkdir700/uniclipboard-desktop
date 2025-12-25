use chrono::Local;
use env_logger::{Builder, Env};
use std::io::Write;

/// 判断是否为开发环境
fn is_development() -> bool {
    // 通过环境变量或编译时特性来判断
    // 1. 优先检查环境变量 UNICLIPBOARD_ENV
    if let Ok(env_val) = std::env::var("UNICLIPBOARD_ENV") {
        return env_val == "development";
    }
    // 2. 检查 debug_assertions 是否启用（编译时特性）
    #[cfg(debug_assertions)]
    {
        return true;
    }
    #[cfg(not(debug_assertions))]
    {
        return false;
    }
}

pub fn init() {
    // 根据环境设置默认日志级别
    let default_log_level = if is_development() { "debug" } else { "info" };

    Builder::from_env(Env::default().default_filter_or(default_log_level))
        .format(|buf, record| {
            let mut style = buf.style();
            let level_style = match record.level() {
                log::Level::Error => style.set_color(env_logger::fmt::Color::Red).set_bold(true),
                log::Level::Warn => style.set_color(env_logger::fmt::Color::Yellow),
                log::Level::Info => style.set_color(env_logger::fmt::Color::Green),
                log::Level::Debug => style.set_color(env_logger::fmt::Color::Blue),
                log::Level::Trace => style.set_color(env_logger::fmt::Color::Cyan),
            };

            let file = record.file().unwrap_or("unknown");
            let line = record
                .line()
                .map_or_else(|| "".to_string(), |l| l.to_string());

            writeln!(
                buf,
                "{} {} [{}:{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                level_style.value(record.level()),
                file,
                line,
                record.target(),
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger() {
        init();
    }
}
