#[derive(Debug, Clone)]
pub struct ClipboardDecisionSnapshot {
    pub blobs_exist: bool,
}

impl ClipboardDecisionSnapshot {
    pub fn new(blobs_exist: bool) -> Self {
        Self { blobs_exist }
    }

    pub fn is_usable(&self) -> bool {
        self.blobs_exist
    }
}
