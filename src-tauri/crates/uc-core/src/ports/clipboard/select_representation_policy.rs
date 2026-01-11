use crate::clipboard::{ClipboardSelection, PolicyError, SystemClipboardSnapshot};

pub trait SelectRepresentationPolicyPort {
    fn select(&self, snapshot: &SystemClipboardSnapshot)
        -> Result<ClipboardSelection, PolicyError>;
}
