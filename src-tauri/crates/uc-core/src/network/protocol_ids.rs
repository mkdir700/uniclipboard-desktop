#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolId {
    Pairing,
    Business,
}

impl ProtocolId {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ProtocolId::Pairing => "/uc-pairing/1.0.0",
            ProtocolId::Business => "/uc-business/1.0.0",
        }
    }
}
