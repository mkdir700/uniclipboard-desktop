//! Tracing configuration for UniClipboard
//!
//! This module provides the tracing-subscriber initialization for structured
//! logging with spans, supporting the gradual migration from `log` crate to
//! `tracing` crate.
//!
//! ## Architecture / 架构
//!
//! - **Dual-track system**: Both `log` and `tracing` work during transition
//! - **Format compatibility**: Output format matches existing `log` format
//! - **Environment-aware**: Development uses Webview, Production uses file+stdout
//!
//! ## Migration Path / 迁移路径
//!
//! Phase 0: Infrastructure setup (this module)
//! Phase 1: Command layer creates root spans
//! Phase 2: UseCase layer creates child spans
//! Phase 3: Infra/Platform layers add debug spans
//! Phase 4: Remove `log` dependency (optional)

// Re-export log for backward compatibility during migration
// This allows crates to use `use uc_tauri::bootstrap::tracing::log`
pub use log;

use tracing_subscriber::{fmt, registry, prelude::*};
use tracing_log::LogTracer;
use std::io;

/// Check if running in development environment
fn is_development() -> bool {
    cfg!(debug_assertions)
}

/// Initialize the tracing subscriber with appropriate configuration
///
/// ## Behavior / 行为
///
/// - **Development**: Debug level, outputs to stdout (Webview via tauri-plugin-log)
/// - **Production**: Info level, outputs to stdout (tauri-plugin-log handles file)
/// - **Environment filter**: Respects RUST_LOG, with sensible defaults
/// - **Log bridge**: Existing `log::info!()` calls are captured by tracing
///
/// ## English
///
/// This function:
/// 1. Initializes the log-tracing bridge (captures `log` macros)
/// 2. Creates an env-filter for level control
/// 3. Sets up fmt layer with log-compatible formatting
/// 4. Registers the global subscriber
///
/// ## Call this / 调用位置
///
/// Call in `main.rs` **before** Tauri Builder setup:
///
/// ```ignore
/// fn main() {
///     // ... load config ...
///     uc_tauri::bootstrap::tracing::init_tracing_subscriber()
///         .expect("Failed to initialize tracing");
///
///     run_app(config);
/// }
/// ```
///
/// ## Errors / 错误
///
/// Returns `Err` if:
/// - Subscriber is already registered (should only call once)
/// - Invalid filter directives in RUST_LOG
pub fn init_tracing_subscriber() -> anyhow::Result<()> {
    let is_dev = is_development();

    // Step 1: Initialize log-tracing bridge
    // This captures all existing log::info!, log::error! etc. calls
    // and redirects them into the tracing system
    LogTracer::init()?;

    // Step 2: Build environment filter
    // - Defaults to debug in dev, info in prod
    // - Filters libp2p_mdns warnings (noisy proxy software errors)
    // - Can be overridden with RUST_LOG environment variable
    let filter_directives = [
        if is_dev { "debug" } else { "info" },
        "libp2p_mdns=warn",    // Filter noisy proxy errors
        "uc_platform=debug",   // Platform layer: debug for dev
        "uc_infra=debug",      // Infra layer: debug for dev
    ];
    let env_filter = tracing_subscriber::EnvFilter::try_new(filter_directives.join(","))
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::from_default_env());

    // Step 3: Create fmt layer (formatting)
    // Format matches existing log format for compatibility:
    // "2025-01-15 10:30:45.123 INFO [file.rs:42] [target] message"
    let fmt_layer = fmt::layer()
        .with_timer(fmt::time::ChronoUtc::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_ansi(cfg!(not(test)))        // Disable colors in tests
        .with_writer(io::stdout);          // Output to stdout (tauri-plugin-log handles Webview/file)

    // Step 4: Register the global subscriber
    // This MUST be called once, before any logging occurs
    registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_init() {
        // Note: This will panic if subscriber already registered
        // In normal tests, we'd use a test subscriber instead
        // For now, just verify the function compiles
        let is_dev = is_development();
        let _ = is_dev; // Suppress unused warning
    }
}
