pub mod event_bus;

pub use event_bus::{
    publish_clipboard_new_content, subscribe_clipboard_new_content, ClipboardNewContentEvent,
    EventBus, ListenerId, EVENT_BUS,
};
