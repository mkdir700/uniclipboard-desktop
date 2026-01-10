use crate::clipboard::{ClipboardSelection, PolicyError, SystemClipboardSnapshot};

pub trait SelectRepresentationPolicyPort {
    fn policy_version(&self) -> &str;

    fn select(&self, snapshot: &SystemClipboardSnapshot)
        -> Result<ClipboardSelection, PolicyError>;
}
