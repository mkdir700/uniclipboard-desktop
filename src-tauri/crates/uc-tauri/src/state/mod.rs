//! Tauri state management
//!
//! This module contains state that needs to be shared across Tauri commands.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event listener state for managing event subscriptions
#[derive(Default, Clone)]
pub struct EventListenerState {
    // Map of event name to list of handler IDs
    pub listeners: HashMap<String, Vec<String>>,
}

/// Event message payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventMessage {
    pub event: String,
    pub payload: serde_json::Value,
}
