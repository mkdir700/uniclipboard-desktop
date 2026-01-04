use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Browser,
    Unknown,
}

impl FromStr for Platform {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::IOS),
            "browser" => Ok(Platform::Browser),
            _ => Ok(Platform::Unknown),
        }
    }
}

impl Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
