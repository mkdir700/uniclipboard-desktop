#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsVersion {
    V1,
}

impl SettingsVersion {
    pub const CURRENT: SettingsVersion = SettingsVersion::V1;

    pub fn as_u32(self) -> u32 {
        match self {
            SettingsVersion::V1 => 1,
        }
    }
}
