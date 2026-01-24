use super::SystemClipboardSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardChangeOrigin {
    LocalCapture,
    LocalRestore,
    RemotePush,
}

#[derive(Debug, Clone)]
pub struct ClipboardChange {
    pub snapshot: SystemClipboardSnapshot,
    pub origin: ClipboardChangeOrigin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_change_origin_variants() {
        let origin = ClipboardChangeOrigin::LocalCapture;
        assert_eq!(origin, ClipboardChangeOrigin::LocalCapture);
    }
}
