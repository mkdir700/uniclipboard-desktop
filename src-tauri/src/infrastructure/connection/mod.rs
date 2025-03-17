use std::collections::HashMap;

use super::web::IncommingWebsocketClient;

pub mod connection_manager;
pub mod incoming_manager;
pub mod outgoing_manager;

pub type DeviceId = String;
pub type IpPort = String;
pub type Clients = HashMap<IpPort, IncommingWebsocketClient>;
