use chrono::Local;
use env_logger::{Builder, Env};
use std::io::Write;

pub fn init() {
    Builder::from_env(Env::default().default_filter_or("debug"))
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
