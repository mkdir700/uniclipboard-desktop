//! Bootstrap module - Application initialization and wiring
//! Bootstrap 模块 - 应用初始化和连接

pub mod config;
pub mod runtime;
pub mod wiring;
pub mod run;

// Re-export commonly used bootstrap functions
pub use config::load_config;
pub use wiring::wire_dependencies;
pub use runtime::{create_app, create_runtime};
