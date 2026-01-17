//! Bootstrap module - Application initialization and wiring
//! Bootstrap 模块 - 应用初始化和连接

pub mod config;
pub mod logging;
pub mod run;
pub mod runtime;
pub mod tracing;
pub mod wiring;

// Re-export commonly used bootstrap functions
pub use config::load_config;
pub use runtime::{create_app, create_runtime, AppRuntime, UseCases};
pub use wiring::wire_dependencies;
