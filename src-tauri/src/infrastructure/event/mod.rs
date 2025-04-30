pub mod event_bus;

pub use event_bus::{
    ClipboardNewContentEvent, EventBus, ListenerId, EVENT_BUS,
    publish_clipboard_new_content, subscribe_clipboard_new_content,
};
