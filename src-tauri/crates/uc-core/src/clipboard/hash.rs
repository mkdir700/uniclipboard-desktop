use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Blake3V1,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    pub alg: HashAlgorithm,
    pub bytes: [u8; 32],
}
