use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Business-layer wrapper for libp2p PeerId
/// This provides type safety and prevents mixing with DeviceId
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for PeerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PeerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PeerId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_creation() {
        let id = PeerId::new("12D3KooW...".to_string());
        assert_eq!(id.as_str(), "12D3KooW...");
    }

    #[test]
    fn test_peer_id_display_is_full() {
        let long_id = PeerId::new("12D3KooWVeryLongPeerIdStringThatShouldBeTruncated".to_string());
        let display = format!("{}", long_id);
        assert_eq!(display, "12D3KooWVeryLongPeerIdStringThatShouldBeTruncated");
    }
}
