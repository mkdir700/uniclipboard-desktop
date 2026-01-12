//! # Placeholder Adapters / 占位符适配器
//!
//! This module contains placeholder implementations for ports that are not yet implemented.
//! 此模块包含尚未实现的端口的占位符实现。
//!
//! These placeholders allow the application to compile and run basic functionality
//! while the full implementations are being developed.
//!
//! 这些占位符允许应用程序在开发完整实现时编译并运行基本功能。
//!
//! # Placeholders / 占位符
//!
//! - `PlaceholderUiPort` - UI operations (will be replaced by Tauri-based implementation)
//! - `PlaceholderAutostartPort` - Autostart management (will be replaced by platform-specific implementation)
//! - `PlaceholderNetworkPort` - P2P networking (will be replaced by libp2p implementation)
//! - `PlaceholderDeviceIdentityPort` - Device identity (will be replaced by hardware-based implementation)
//! - `PlaceholderClipboardRepresentationMaterializerPort` - Clipboard materialization
//! - `PlaceholderBlobMaterializerPort` - Blob materialization
//! - `PlaceholderBlobStorePort` - Blob storage
//! - `PlaceholderEncryptionSessionPort` - Encryption session management

pub mod autostart;
pub mod blob;
pub mod clipboard;
pub mod device;
pub mod encryption;
pub mod network;
pub mod ui;

pub use autostart::PlaceholderAutostartPort;
pub use blob::{PlaceholderBlobMaterializerPort, PlaceholderBlobStorePort};
pub use clipboard::PlaceholderClipboardRepresentationMaterializerPort;
pub use device::PlaceholderDeviceIdentityPort;
pub use encryption::PlaceholderEncryptionSessionPort;
pub use network::PlaceholderNetworkPort;
pub use ui::PlaceholderUiPort;
