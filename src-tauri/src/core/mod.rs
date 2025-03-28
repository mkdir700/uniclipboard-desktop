pub mod builder;
pub mod clipboard_content_receiver;
pub mod context;
pub mod download_decision;
pub mod event_bus;
pub mod uniclipboard;

// 新的拆分模块
pub mod clipboard_metadata;
pub mod content_detector;
pub mod content_type;
pub mod metadata_models;
pub mod transfer_message;

pub use builder::UniClipboardBuilder;
pub use clipboard_metadata::ClipboardMetadata;
pub use content_detector::ContentDetector;
pub use content_type::ContentType;
pub use transfer_message::ClipboardTransferMessage;
pub use uniclipboard::UniClipboard;
