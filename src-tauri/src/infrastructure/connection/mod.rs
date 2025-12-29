use std::collections::HashMap;

use super::web::IncommingWebsocketClient;

pub mod connection_manager;
pub mod pending_connections;
pub mod unified_manager;

pub use pending_connections::PendingConnectionsManager;
pub use unified_manager::UnifiedConnectionManager;

pub type DeviceId = String;
pub type IpPort = String;
pub type Clients = HashMap<IpPort, IncommingWebsocketClient>;
