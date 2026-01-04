//! Port adapters - bridge infrastructure implementations to ports

pub mod clipboard_adapter;
pub mod network_adapter;
pub mod storage_adapter;

pub use clipboard_adapter::LocalClipboardAdapter;
pub use network_adapter::P2PNetworkAdapter;
pub use storage_adapter::FileStorageAdapter;
