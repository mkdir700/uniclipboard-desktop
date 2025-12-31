use anyhow::Result;
use log::{info, warn};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::get_config_dir;
use crate::domain::pairing::PairedPeer;

/// Storage for paired peers
///
/// Persists paired device information including shared secrets to a JSON file.
/// CAUTION: In a production environment, shared secrets should be stored in a secure vault
/// (e.g., using tauri-plugin-stronghold or system keychain).
#[derive(Clone)]
pub struct PeerStorage {
    file_path: PathBuf,
    cache: Arc<RwLock<HashMap<String, PairedPeer>>>,
}

impl PeerStorage {
    pub fn new() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let storage_dir = config_dir.join("p2p");

        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        let file_path = storage_dir.join("peers.json");

        let storage = Self {
            file_path,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load initial data
        if storage.file_path.exists() {
            storage.load()?;
        }

        Ok(storage)
    }

    /// Load peers from disk
    fn load(&self) -> Result<()> {
        let file = File::open(&self.file_path)?;
        let peers: Vec<PairedPeer> = serde_json::from_reader(file)?;

        let mut cache = self.cache.write().unwrap();
        cache.clear();
        for peer in peers {
            cache.insert(peer.peer_id.clone(), peer);
        }

        info!("Loaded {} paired peers from storage", cache.len());
        Ok(())
    }

    /// Save peers to disk
    fn save(&self) -> Result<()> {
        let cache = self.cache.read().unwrap();
        let peers: Vec<&PairedPeer> = cache.values().collect();

        let file = File::create(&self.file_path)?;
        serde_json::to_writer_pretty(file, &peers)?;

        Ok(())
    }

    /// Add or update a paired peer
    pub fn save_peer(&self, peer: PairedPeer) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(peer.peer_id.clone(), peer);
        }
        self.save()?;
        Ok(())
    }

    /// Get a paired peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Option<PairedPeer> {
        let cache = self.cache.read().unwrap();
        cache.get(peer_id).cloned()
    }

    /// Remove a paired peer
    pub fn remove_peer(&self, peer_id: &str) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            cache.remove(peer_id);
        }
        self.save()?;
        Ok(())
    }

    /// Get all paired peers
    pub fn get_all_peers(&self) -> Vec<PairedPeer> {
        let cache = self.cache.read().unwrap();
        cache.values().cloned().collect()
    }
}
