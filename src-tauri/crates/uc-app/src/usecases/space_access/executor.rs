use uc_core::ports::space::{CryptoPort, PersistencePort};
use uc_core::ports::{NetworkPort, TimerPort};

pub struct SpaceAccessExecutor<'a> {
    pub crypto: &'a dyn CryptoPort,
    pub net: &'a dyn NetworkPort,
    pub timer: &'a mut dyn TimerPort,
    pub store: &'a mut dyn PersistencePort,
}
