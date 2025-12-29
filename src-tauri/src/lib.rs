//! UniClipboard Library
//!
//! 统一剪贴板同步库

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod interface;
pub mod message;
pub mod plugins;
pub mod utils;

// 重新导出常用类型
pub use config::Setting;
pub use domain::device::Device;
pub use infrastructure::connection::unified_manager::UnifiedConnectionManager;
pub use message::WebSocketMessage;
