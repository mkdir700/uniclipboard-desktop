use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MimeType(pub String);

impl MimeType {
    pub fn text_plain() -> Self {
        Self("text/plain".into())
    }
    pub fn text_html() -> Self {
        Self("text/html".into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MimeType {
    /// Formats the MIME type by writing its inner string to the provided formatter.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::clipboard::mime::MimeType;
    /// let m = MimeType("text/plain".to_string());
    /// assert_eq!(format!("{}", m), "text/plain");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for MimeType {
    type Err = anyhow::Error;

    /// Parses a MIME type string into a `MimeType` without performing validation.
    ///
    /// # Examples
    ///
    /// ```
    /// let m: MimeType = "text/plain".parse().unwrap();
    /// assert_eq!(m, MimeType("text/plain".to_string()));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(MimeType(s.to_string()))
    }
}

