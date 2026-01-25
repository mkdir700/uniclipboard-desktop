use crate::ids::PeerId;
use crate::network::connection_policy::ResolvedConnectionPolicy;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionPolicyResolverError {
    #[error("repository error: {0}")]
    Repository(String),
}

#[async_trait]
pub trait ConnectionPolicyResolverPort: Send + Sync {
    async fn resolve_for_peer(
        &self,
        peer_id: &PeerId,
    ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{ConnectionPolicy, PairingState};

    #[test]
    fn connection_policy_resolver_trait_is_object_safe() {
        struct Dummy;

        #[async_trait::async_trait]
        impl ConnectionPolicyResolverPort for Dummy {
            async fn resolve_for_peer(
                &self,
                _peer_id: &PeerId,
            ) -> Result<ResolvedConnectionPolicy, ConnectionPolicyResolverError> {
                Ok(ResolvedConnectionPolicy {
                    pairing_state: PairingState::Pending,
                    allowed: ConnectionPolicy::allowed_protocols(PairingState::Pending),
                })
            }
        }

        let _resolver: &dyn ConnectionPolicyResolverPort = &Dummy;
    }
}
