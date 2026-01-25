use crate::network::PairingState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolKind {
    Pairing,
    Business,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllowedProtocols {
    pairing: bool,
    business: bool,
}

impl AllowedProtocols {
    pub fn allows(&self, kind: ProtocolKind) -> bool {
        match kind {
            ProtocolKind::Pairing => self.pairing,
            ProtocolKind::Business => self.business,
        }
    }
}

pub struct ConnectionPolicy;

impl ConnectionPolicy {
    pub fn allowed_protocols(state: PairingState) -> AllowedProtocols {
        match state {
            PairingState::Trusted => AllowedProtocols {
                pairing: true,
                business: true,
            },
            PairingState::Pending | PairingState::Revoked => AllowedProtocols {
                pairing: true,
                business: false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConnectionPolicy {
    pub pairing_state: PairingState,
    pub allowed: AllowedProtocols,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_allows_only_pairing() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Pending);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(!allowed.allows(ProtocolKind::Business));
    }

    #[test]
    fn trusted_allows_pairing_and_business() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Trusted);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(allowed.allows(ProtocolKind::Business));
    }

    #[test]
    fn revoked_allows_pairing_only() {
        let allowed = ConnectionPolicy::allowed_protocols(PairingState::Revoked);
        assert!(allowed.allows(ProtocolKind::Pairing));
        assert!(!allowed.allows(ProtocolKind::Business));
    }
}
