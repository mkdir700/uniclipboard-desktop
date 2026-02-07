use uc_core::ports::space::{CryptoPort, PersistencePort, ProofPort, SpaceAccessTransportPort};
use uc_core::ports::{NetworkPort, TimerPort};

pub struct SpaceAccessExecutor<'a> {
    pub crypto: &'a dyn CryptoPort,
    pub net: &'a dyn NetworkPort,
    pub transport: &'a mut dyn SpaceAccessTransportPort,
    pub proof: &'a dyn ProofPort,
    pub timer: &'a mut dyn TimerPort,
    pub store: &'a mut dyn PersistencePort,
}
