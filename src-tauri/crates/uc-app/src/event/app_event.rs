use uc_core::clipboard::ClipboardContent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Open the settings window.
    OpenSettingsWindow,
    ClipboardChanged {
        content: ClipboardContent,
    },
}
