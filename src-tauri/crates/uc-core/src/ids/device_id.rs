use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// 6-digit stable device identifier (from database)
/// This is different from libp2p PeerId which changes on each restart
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    /// Validate device ID format (6-digit alphanumeric)
    pub fn is_valid(&self) -> bool {
        self.0.len() == 6 && self.0.chars().all(|c| c.is_alphanumeric())
    }
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for DeviceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for DeviceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_device_id() {
        let id = DeviceId::new("ABC123".to_string());
        assert!(id.is_valid());
    }

    #[test]
    fn test_invalid_device_id() {
        let id = DeviceId::new("ABC".to_string()); // Too short
        assert!(!id.is_valid());
    }

    #[test]
    fn test_device_id_from_str() {
        let id: DeviceId = "ABC123".into();
        assert_eq!(id.as_str(), "ABC123");
    }
}
