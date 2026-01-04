use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Pairing session identifier
/// Format: "{timestamp}-{random}"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
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

impl Display for SessionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_creation() {
        let id = SessionId::new("1234567890-abc123".to_string());
        assert_eq!(id.as_str(), "1234567890-abc123");
    }

    #[test]
    fn test_session_id_from_str() {
        let id: SessionId = "1234567890-abc123".into();
        assert_eq!(id.as_str(), "1234567890-abc123");
    }
}
