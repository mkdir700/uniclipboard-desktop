//! Application runtime module
//!
//! Manages the lifecycle and communication of all core application components.
//! Uses clear ownership and message passing instead of Arc<Mutex<T>> everywhere.

mod app_runtime;
mod handle;
mod p2p_runtime;

pub use app_runtime::AppRuntime;
pub use handle::{AppRuntimeHandle, ClipboardCommand, P2PCommand};
pub use p2p_runtime::P2PRuntime;
