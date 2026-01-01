use anyhow::Result;
use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::config::get_config_dir;
use crate::domain::pairing::PairedPeer;

/// Legacy PairedPeer format for migration (snake_case)
#[derive(Clone, Deserialize)]
struct PairedPeerLegacy {
    pub peer_id: String,
    pub device_name: String,
    pub shared_secret: Vec<u8>,
    pub paired_at: String,
    pub last_seen: Option<String>,
    pub last_known_addresses: Vec<String>,
}

impl std::fmt::Debug for PairedPeerLegacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PairedPeerLegacy")
            .field("peer_id", &self.peer_id)
            .field("device_name", &self.device_name)
            .field("shared_secret", &"[REDACTED]")
            .field("paired_at", &self.paired_at)
            .field("last_seen", &self.last_seen)
            .field("last_known_addresses", &self.last_known_addresses)
            .finish()
    }
}

impl From<PairedPeerLegacy> for PairedPeer {
    fn from(legacy: PairedPeerLegacy) -> Self {
        Self {
            peer_id: legacy.peer_id,
            device_name: legacy.device_name,
            shared_secret: legacy.shared_secret,
            paired_at: chrono::DateTime::parse_from_rfc3339(&legacy.paired_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            last_seen: legacy.last_seen.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            last_known_addresses: legacy.last_known_addresses,
        }
    }
}

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

        // Try loading as new format (camelCase) first
        let peers: Result<Vec<PairedPeer>, _> = serde_json::from_reader(&file);

        let mut cache = self
            .cache
            .write()
            .expect("Failed to acquire write lock on peer cache");
        cache.clear();

        match peers {
            Ok(peers) => {
                for peer in peers {
                    cache.insert(peer.peer_id.clone(), peer);
                }
                info!("Loaded {} paired peers from storage", cache.len());
            }
            Err(_) => {
                // Fallback: try loading as legacy format (snake_case)
                warn!("Failed to load peers in new format, trying legacy format");
                let legacy_file = File::open(&self.file_path)?;
                let legacy_peers: Result<Vec<PairedPeerLegacy>, _> =
                    serde_json::from_reader(legacy_file);

                match legacy_peers {
                    Ok(peers) => {
                        for peer in peers {
                            let modern: PairedPeer = peer.into();
                            cache.insert(modern.peer_id.clone(), modern);
                        }
                        // Resave in new format
                        drop(cache);
                        self.save()?;
                        info!(
                            "Migrated {} paired peers from legacy format",
                            self.cache
                                .read()
                                .expect("Failed to acquire read lock on peer cache")
                                .len()
                        );
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to load peers in both new and legacy formats: {}",
                            e
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Save peers to disk
    fn save(&self) -> Result<()> {
        let cache = self
            .cache
            .read()
            .expect("Failed to acquire read lock on peer cache");
        let peers: Vec<&PairedPeer> = cache.values().collect();

        let file = File::create(&self.file_path)?;
        serde_json::to_writer_pretty(file, &peers)?;

        Ok(())
    }

    /// Add or update a paired peer
    pub fn save_peer(&self, peer: PairedPeer) -> Result<()> {
        {
            let mut cache = self
                .cache
                .write()
                .expect("Failed to acquire write lock on peer cache");
            cache.insert(peer.peer_id.clone(), peer);
        }
        self.save()?;
        Ok(())
    }

    /// Get a paired peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Result<Option<PairedPeer>> {
        let cache = self
            .cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
        Ok(cache.get(peer_id).cloned())
    }

    /// Remove a paired peer
    pub fn remove_peer(&self, peer_id: &str) -> Result<()> {
        {
            let mut cache = self
                .cache
                .write()
                .expect("Failed to acquire write lock on peer cache");
            cache.remove(peer_id);
        }
        self.save()?;
        Ok(())
    }

    /// Get all paired peers
    pub fn get_all_peers(&self) -> Result<Vec<PairedPeer>> {
        let cache = self
            .cache
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
        Ok(cache.values().cloned().collect())
    }
}
