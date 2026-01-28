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

#[cfg(test)]
mod tests {
    use super::ProtocolId;

    #[test]
    fn protocol_id_strings_match_expected_values() {
        assert_eq!(ProtocolId::Pairing.as_str(), "/uc-pairing/1.0.0");
        assert_eq!(ProtocolId::Business.as_str(), "/uc-business/1.0.0");
    }
}
