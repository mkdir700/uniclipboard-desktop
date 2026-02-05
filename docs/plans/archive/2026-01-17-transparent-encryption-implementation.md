# Transparent Encryption Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement transparent encryption/decryption for clipboard content stored in SQLite `inline_data` and blob files, with automatic session unlock on startup.

**Architecture:** Decorator pattern wrapping existing repository/store ports with encryption adapters. Session-based MasterKey management with gated clipboard watcher startup.

**Tech Stack:** Rust, XChaCha20-Poly1305 (AEAD), serde_json for encrypted blob serialization, async-trait

---

## Overview

### Root Causes Addressed

1. **Session Not Set**: `InitializeEncryption` use case only persists keyslot/KEK/marker but never calls `EncryptionSessionPort::set_master_key()`, leaving session empty.
2. **Multiple Read/Write Paths**: Without decorator coverage, some code paths would receive ciphertext as plaintext.

### Structural Changes

1. Add `encryption_session` to `InitializeEncryption` use case
2. Create `AutoUnlockEncryptionSession` use case for startup unlock
3. Implement encryption decorators for:
   - `BlobStorePort` → `EncryptedBlobStore`
   - `ClipboardEventWriterPort` → `EncryptingClipboardEventWriter`
   - `ClipboardRepresentationRepositoryPort` → `DecryptingClipboardRepresentationRepository`
   - `ClipboardEventRepositoryPort` → `DecryptingClipboardEventRepository`
4. Wire decorators in `wiring.rs`
5. Gate clipboard watcher on encryption initialization state

### AAD Format

- **inline**: `uc:inline:v1|{event_id}|{rep_id}`
- **blob**: `uc:blob:v1|{blob_id}`

---

## Task 1: Fix InitializeEncryption to Set Session

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs`

**Step 1: Read the current implementation**

Run: `cat src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs`

**Step 2: Add encryption_session field to struct**

```rust
// In the struct definition, add:
pub struct InitializeEncryption {
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state_repo: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,  // ADD THIS
}
```

**Step 3: Update new() and from_ports() to accept encryption_session**

```rust
pub fn new(
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state_repo: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,  // ADD THIS
) -> Self {
    Self {
        encryption,
        key_material,
        key_scope,
        encryption_state_repo,
        encryption_session,  // ADD THIS
    }
}

pub fn from_ports(
    encryption: Arc<dyn EncryptionPort>,
    key_material: Arc<dyn KeyMaterialPort>,
    key_scope: Arc<dyn KeyScopePort>,
    encryption_state_repo: Arc<dyn EncryptionStatePort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,  // ADD THIS
) -> Self {
    Self::new(encryption, key_material, key_scope, encryption_state_repo, encryption_session)
}
```

**Step 4: Add set_master_key call after persist_initialized in execute()**

After line with `self.encryption_state_repo.persist_initialized().await?;`, add:

```rust
// 8. set master key in session for immediate use
log::debug!("{} Setting master key in session...", LOG_CONTEXT);
self.encryption_session.set_master_key(master_key).await?;
log::debug!("{} Master key set in session successfully", LOG_CONTEXT);
```

**Step 5: Add EncryptionSessionPort import**

```rust
use uc_core::ports::EncryptionSessionPort;
```

**Step 6: Run cargo check to verify compilation**

Run: `cd src-tauri && cargo check -p uc-app`
Expected: Compilation errors in dependent code (runtime.rs) - this is expected until we update wiring.

**Step 7: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/initialize_encryption.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add session key setting to InitializeEncryption

After persisting keyslot and KEK, now also sets the MasterKey in
the EncryptionSessionPort so that encryption decorators can work
immediately without app restart.
EOF
)"
```

---

## Task 2: Create AutoUnlockEncryptionSession Use Case

**Files:**

- Create: `src-tauri/crates/uc-app/src/usecases/auto_unlock_encryption_session.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/mod.rs`

**Step 1: Create the use case file**

```rust
//! Auto-unlock encryption session on startup.
//!
//! This use case loads the MasterKey from persisted keyslot + KEK
//! and sets it in the EncryptionSessionPort for transparent encryption.

use std::sync::Arc;
use tracing::{info_span, info, warn, Instrument};

use uc_core::{
    ports::{
        security::{
            encryption_state::EncryptionStatePort,
            key_scope::KeyScopePort,
        },
        EncryptionPort, EncryptionSessionPort, KeyMaterialPort,
    },
    security::{
        model::EncryptionError,
        state::EncryptionState,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum AutoUnlockError {
    #[error("encryption state check failed: {0}")]
    StateCheckFailed(String),

    #[error("key scope resolution failed: {0}")]
    ScopeFailed(String),

    #[error("failed to load keyslot: {0}")]
    KeySlotLoadFailed(#[source] EncryptionError),

    #[error("failed to load KEK from keyring: {0}")]
    KekLoadFailed(#[source] EncryptionError),

    #[error("keyslot has no wrapped master key")]
    MissingWrappedMasterKey,

    #[error("failed to unwrap master key: {0}")]
    UnwrapFailed(#[source] EncryptionError),

    #[error("failed to set master key in session: {0}")]
    SessionSetFailed(#[source] EncryptionError),
}

/// Use case for automatically unlocking encryption session on startup.
///
/// ## Behavior
///
/// - If encryption is **Uninitialized**: Returns `Ok(false)` (not unlocked, but not an error)
/// - If encryption is **Initialized**: Attempts to load and set MasterKey, returns `Ok(true)` on success
/// - Any failure during unlock returns an error
pub struct AutoUnlockEncryptionSession {
    encryption_state: Arc<dyn EncryptionStatePort>,
    key_scope: Arc<dyn KeyScopePort>,
    key_material: Arc<dyn KeyMaterialPort>,
    encryption: Arc<dyn EncryptionPort>,
    encryption_session: Arc<dyn EncryptionSessionPort>,
}

impl AutoUnlockEncryptionSession {
    pub fn new(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self {
            encryption_state,
            key_scope,
            key_material,
            encryption,
            encryption_session,
        }
    }

    pub fn from_ports(
        encryption_state: Arc<dyn EncryptionStatePort>,
        key_scope: Arc<dyn KeyScopePort>,
        key_material: Arc<dyn KeyMaterialPort>,
        encryption: Arc<dyn EncryptionPort>,
        encryption_session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self::new(encryption_state, key_scope, key_material, encryption, encryption_session)
    }

    /// Execute the auto-unlock flow.
    ///
    /// # Returns
    ///
    /// - `Ok(true)` - Session unlocked successfully
    /// - `Ok(false)` - Encryption not initialized (no unlock needed)
    /// - `Err(_)` - Unlock failed
    pub async fn execute(&self) -> Result<bool, AutoUnlockError> {
        let span = info_span!("usecase.auto_unlock_encryption_session.execute");

        async {
            info!("Checking encryption state for auto-unlock");

            // 1. Check encryption state
            let state = self.encryption_state.load_state().await
                .map_err(|e| AutoUnlockError::StateCheckFailed(e.to_string()))?;

            if state == EncryptionState::Uninitialized {
                info!("Encryption not initialized, skipping auto-unlock");
                return Ok(false);
            }

            info!("Encryption initialized, attempting auto-unlock");

            // 2. Get key scope
            let scope = self.key_scope.current_scope().await
                .map_err(|e| AutoUnlockError::ScopeFailed(e.to_string()))?;

            // 3. Load keyslot
            let keyslot = self.key_material.load_keyslot(&scope).await
                .map_err(AutoUnlockError::KeySlotLoadFailed)?;

            // 4. Get wrapped master key
            let wrapped_master_key = keyslot.wrapped_master_key
                .ok_or(AutoUnlockError::MissingWrappedMasterKey)?;

            // 5. Load KEK from keyring
            let kek = self.key_material.load_kek(&scope).await
                .map_err(AutoUnlockError::KekLoadFailed)?;

            // 6. Unwrap master key
            let master_key = self.encryption.unwrap_master_key(&kek, &wrapped_master_key.blob).await
                .map_err(AutoUnlockError::UnwrapFailed)?;

            // 7. Set master key in session
            self.encryption_session.set_master_key(master_key).await
                .map_err(AutoUnlockError::SessionSetFailed)?;

            info!("Auto-unlock completed successfully");
            Ok(true)
        }
        .instrument(span)
        .await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added in a separate task
}
```

**Step 2: Add to mod.rs exports**

```rust
// In mod.rs, add:
pub mod auto_unlock_encryption_session;

// And add to re-exports:
pub use auto_unlock_encryption_session::AutoUnlockEncryptionSession;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-app`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/auto_unlock_encryption_session.rs
git add src-tauri/crates/uc-app/src/usecases/mod.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add AutoUnlockEncryptionSession use case

Loads MasterKey from persisted keyslot + KEK on startup and sets it
in the EncryptionSessionPort. Returns Ok(false) if encryption not
initialized (no error), Ok(true) on successful unlock.
EOF
)"
```

---

## Task 3: Create EncryptedBlobStore Decorator

**Files:**

- Create: `src-tauri/crates/uc-infra/src/security/encrypted_blob_store.rs`
- Modify: `src-tauri/crates/uc-infra/src/security/mod.rs`

**Step 1: Create the decorator file**

```rust
//! Encrypted blob store decorator.
//!
//! Wraps an inner BlobStorePort and transparently encrypts/decrypts
//! blob data using the session's MasterKey.

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{debug, warn};

use uc_core::{
    ports::{BlobStorePort, EncryptionPort, EncryptionSessionPort},
    security::model::{EncryptedBlob, EncryptionAlgo},
    BlobId,
};

/// Decorator that encrypts/decrypts blob data transparently.
pub struct EncryptedBlobStore {
    inner: Arc<dyn BlobStorePort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl EncryptedBlobStore {
    pub fn new(
        inner: Arc<dyn BlobStorePort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for blob encryption.
    fn aad_for_blob(blob_id: &BlobId) -> Vec<u8> {
        format!("uc:blob:v1|{}", blob_id.0).into_bytes()
    }
}

#[async_trait]
impl BlobStorePort for EncryptedBlobStore {
    async fn put(&self, blob_id: &BlobId, data: &[u8]) -> Result<PathBuf> {
        // 1. Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot encrypt blob")?;

        // 2. Encrypt the data
        let aad = Self::aad_for_blob(blob_id);
        let encrypted_blob = self.encryption
            .encrypt_blob(&master_key, data, &aad, EncryptionAlgo::XChaCha20Poly1305)
            .await
            .context("failed to encrypt blob data")?;

        // 3. Serialize encrypted blob to bytes
        let encrypted_bytes = serde_json::to_vec(&encrypted_blob)
            .context("failed to serialize encrypted blob")?;

        debug!("Encrypted blob {} ({} bytes plaintext -> {} bytes ciphertext)",
            blob_id.0, data.len(), encrypted_bytes.len());

        // 4. Store encrypted bytes
        self.inner.put(blob_id, &encrypted_bytes).await
    }

    async fn get(&self, blob_id: &BlobId) -> Result<Vec<u8>> {
        // 1. Get encrypted bytes from inner store
        let encrypted_bytes = self.inner.get(blob_id).await
            .context("failed to read encrypted blob from storage")?;

        // 2. Deserialize encrypted blob
        let encrypted_blob: EncryptedBlob = serde_json::from_slice(&encrypted_bytes)
            .context("failed to deserialize encrypted blob - data may be corrupted or unencrypted")?;

        // 3. Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot decrypt blob")?;

        // 4. Decrypt the data
        let aad = Self::aad_for_blob(blob_id);
        let plaintext = self.encryption
            .decrypt_blob(&master_key, &encrypted_blob, &aad)
            .await
            .context("failed to decrypt blob - key mismatch or data corrupted")?;

        debug!("Decrypted blob {} ({} bytes)", blob_id.0, plaintext.len());

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
```

**Step 2: Update mod.rs to export**

```rust
// In mod.rs, add:
mod encrypted_blob_store;

pub use encrypted_blob_store::EncryptedBlobStore;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-infra`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/encrypted_blob_store.rs
git add src-tauri/crates/uc-infra/src/security/mod.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add EncryptedBlobStore decorator

Wraps BlobStorePort to transparently encrypt on put() and decrypt on get().
Uses XChaCha20-Poly1305 with AAD format "uc:blob:v1|{blob_id}".
Stores EncryptedBlob as JSON bytes.
EOF
)"
```

---

## Task 4: Create EncryptingClipboardEventWriter Decorator

**Files:**

- Create: `src-tauri/crates/uc-infra/src/security/encrypting_clipboard_event_writer.rs`
- Modify: `src-tauri/crates/uc-infra/src/security/mod.rs`

**Step 1: Create the decorator file**

```rust
//! Encrypting clipboard event writer decorator.
//!
//! Wraps ClipboardEventWriterPort and encrypts inline_data before storage.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::{ClipboardEvent, PersistedClipboardRepresentation},
    ids::EventId,
    ports::{ClipboardEventWriterPort, EncryptionPort, EncryptionSessionPort},
    security::model::EncryptionAlgo,
};

/// Decorator that encrypts representation inline_data before storage.
pub struct EncryptingClipboardEventWriter {
    inner: Arc<dyn ClipboardEventWriterPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl EncryptingClipboardEventWriter {
    pub fn new(
        inner: Arc<dyn ClipboardEventWriterPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for inline data encryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &str) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.0, rep_id).into_bytes()
    }
}

#[async_trait]
impl ClipboardEventWriterPort for EncryptingClipboardEventWriter {
    async fn insert_event(
        &self,
        event: &ClipboardEvent,
        representations: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()> {
        // Get master key from session
        let master_key = self.session.get_master_key().await
            .context("encryption session not ready - cannot encrypt clipboard data")?;

        // Encrypt inline_data for each representation
        let mut encrypted_reps = Vec::with_capacity(representations.len());

        for rep in representations {
            let encrypted_inline_data = if let Some(ref plaintext) = rep.inline_data {
                // Encrypt the inline data
                let aad = Self::aad_for_inline(&event.id, &rep.id.0);
                let encrypted_blob = self.encryption
                    .encrypt_blob(&master_key, plaintext, &aad, EncryptionAlgo::XChaCha20Poly1305)
                    .await
                    .context("failed to encrypt inline_data")?;

                // Serialize to bytes
                let encrypted_bytes = serde_json::to_vec(&encrypted_blob)
                    .context("failed to serialize encrypted inline_data")?;

                debug!("Encrypted inline_data for rep {} ({} bytes -> {} bytes)",
                    rep.id.0, plaintext.len(), encrypted_bytes.len());

                Some(encrypted_bytes)
            } else {
                None
            };

            // Create new representation with encrypted inline_data
            encrypted_reps.push(PersistedClipboardRepresentation::new(
                rep.id.clone(),
                rep.format_id.clone(),
                rep.mime_type.clone(),
                rep.size_bytes,
                encrypted_inline_data,
                rep.blob_id.clone(),
            ));
        }

        // Delegate to inner with encrypted representations
        self.inner.insert_event(event, &encrypted_reps).await
    }

    async fn delete_event_and_representations(&self, event_id: &EventId) -> Result<()> {
        // Deletion doesn't need encryption - just delegate
        self.inner.delete_event_and_representations(event_id).await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
```

**Step 2: Update mod.rs**

```rust
// Add to mod.rs:
mod encrypting_clipboard_event_writer;

pub use encrypting_clipboard_event_writer::EncryptingClipboardEventWriter;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-infra`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/encrypting_clipboard_event_writer.rs
git add src-tauri/crates/uc-infra/src/security/mod.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add EncryptingClipboardEventWriter decorator

Wraps ClipboardEventWriterPort to encrypt inline_data before insert.
Uses XChaCha20-Poly1305 with AAD format "uc:inline:v1|{event_id}|{rep_id}".
EOF
)"
```

---

## Task 5: Create DecryptingClipboardRepresentationRepository Decorator

**Files:**

- Create: `src-tauri/crates/uc-infra/src/security/decrypting_representation_repo.rs`
- Modify: `src-tauri/crates/uc-infra/src/security/mod.rs`

**Step 1: Create the decorator file**

```rust
//! Decrypting clipboard representation repository decorator.
//!
//! Wraps ClipboardRepresentationRepositoryPort and decrypts inline_data on read.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::PersistedClipboardRepresentation,
    ids::{EventId, RepresentationId},
    ports::{ClipboardRepresentationRepositoryPort, EncryptionPort, EncryptionSessionPort},
    security::model::EncryptedBlob,
    BlobId,
};

/// Decorator that decrypts representation inline_data on read.
pub struct DecryptingClipboardRepresentationRepository {
    inner: Arc<dyn ClipboardRepresentationRepositoryPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl DecryptingClipboardRepresentationRepository {
    pub fn new(
        inner: Arc<dyn ClipboardRepresentationRepositoryPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for inline data decryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &RepresentationId) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.0, rep_id.0).into_bytes()
    }
}

#[async_trait]
impl ClipboardRepresentationRepositoryPort for DecryptingClipboardRepresentationRepository {
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        // Get from inner
        let rep_opt = self.inner.get_representation(event_id, representation_id).await?;

        let Some(rep) = rep_opt else {
            return Ok(None);
        };

        // Decrypt inline_data if present
        let decrypted_inline_data = if let Some(ref encrypted_bytes) = rep.inline_data {
            // Deserialize encrypted blob
            let encrypted_blob: EncryptedBlob = serde_json::from_slice(encrypted_bytes)
                .context("failed to deserialize encrypted inline_data - data may be corrupted")?;

            // Get master key
            let master_key = self.session.get_master_key().await
                .context("encryption session not ready - cannot decrypt")?;

            // Decrypt
            let aad = Self::aad_for_inline(event_id, representation_id);
            let plaintext = self.encryption
                .decrypt_blob(&master_key, &encrypted_blob, &aad)
                .await
                .context("failed to decrypt inline_data")?;

            debug!("Decrypted inline_data for rep {} ({} bytes)",
                representation_id.0, plaintext.len());

            Some(plaintext)
        } else {
            None
        };

        // Return representation with decrypted data
        Ok(Some(PersistedClipboardRepresentation::new(
            rep.id,
            rep.format_id,
            rep.mime_type,
            rep.size_bytes,
            decrypted_inline_data,
            rep.blob_id,
        )))
    }

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()> {
        // No encryption needed for blob_id update - just delegate
        self.inner.update_blob_id(representation_id, blob_id).await
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
```

**Step 2: Update mod.rs**

```rust
// Add to mod.rs:
mod decrypting_representation_repo;

pub use decrypting_representation_repo::DecryptingClipboardRepresentationRepository;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-infra`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/decrypting_representation_repo.rs
git add src-tauri/crates/uc-infra/src/security/mod.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add DecryptingClipboardRepresentationRepository decorator

Wraps ClipboardRepresentationRepositoryPort to decrypt inline_data on read.
Uses same AAD format as writer for consistency.
EOF
)"
```

---

## Task 6: Create DecryptingClipboardEventRepository Decorator

**Files:**

- Create: `src-tauri/crates/uc-infra/src/security/decrypting_clipboard_event_repo.rs`
- Modify: `src-tauri/crates/uc-infra/src/security/mod.rs`

**Step 1: Create the decorator file**

```rust
//! Decrypting clipboard event repository decorator.
//!
//! Wraps ClipboardEventRepositoryPort and decrypts ObservedClipboardRepresentation.bytes on read.

use std::sync::Arc;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::debug;

use uc_core::{
    clipboard::ObservedClipboardRepresentation,
    ids::EventId,
    ports::{
        clipboard::clipboard_event_repository::ClipboardEventRepositoryPort,
        EncryptionPort, EncryptionSessionPort,
    },
    security::model::EncryptedBlob,
};

/// Decorator that decrypts ObservedClipboardRepresentation.bytes on read.
pub struct DecryptingClipboardEventRepository {
    inner: Arc<dyn ClipboardEventRepositoryPort>,
    encryption: Arc<dyn EncryptionPort>,
    session: Arc<dyn EncryptionSessionPort>,
}

impl DecryptingClipboardEventRepository {
    pub fn new(
        inner: Arc<dyn ClipboardEventRepositoryPort>,
        encryption: Arc<dyn EncryptionPort>,
        session: Arc<dyn EncryptionSessionPort>,
    ) -> Self {
        Self { inner, encryption, session }
    }

    /// Generate AAD for representation bytes decryption.
    fn aad_for_inline(event_id: &EventId, rep_id: &str) -> Vec<u8> {
        format!("uc:inline:v1|{}|{}", event_id.0, rep_id).into_bytes()
    }
}

#[async_trait]
impl ClipboardEventRepositoryPort for DecryptingClipboardEventRepository {
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &str,
    ) -> Result<ObservedClipboardRepresentation> {
        // Get from inner
        let mut observed = self.inner.get_representation(event_id, representation_id).await?;

        // Decrypt bytes if present
        if !observed.bytes.is_empty() {
            // Try to deserialize as encrypted blob
            match serde_json::from_slice::<EncryptedBlob>(&observed.bytes) {
                Ok(encrypted_blob) => {
                    // Get master key
                    let master_key = self.session.get_master_key().await
                        .context("encryption session not ready - cannot decrypt")?;

                    // Decrypt
                    let aad = Self::aad_for_inline(event_id, representation_id);
                    let plaintext = self.encryption
                        .decrypt_blob(&master_key, &encrypted_blob, &aad)
                        .await
                        .context("failed to decrypt representation bytes")?;

                    debug!("Decrypted representation bytes for {} ({} bytes)",
                        representation_id, plaintext.len());

                    observed.bytes = plaintext;
                }
                Err(_) => {
                    // Not encrypted blob format - this could be:
                    // 1. Old unencrypted data (hard fail as per spec)
                    // 2. Corrupted data
                    anyhow::bail!(
                        "representation {} bytes are not in encrypted format - \
                         data may be from before encryption was enabled or corrupted",
                        representation_id
                    );
                }
            }
        }

        Ok(observed)
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after mock infrastructure is available
}
```

**Step 2: Update mod.rs**

```rust
// Add to mod.rs:
mod decrypting_clipboard_event_repo;

pub use decrypting_clipboard_event_repo::DecryptingClipboardEventRepository;
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-infra`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-infra/src/security/decrypting_clipboard_event_repo.rs
git add src-tauri/crates/uc-infra/src/security/mod.rs
git commit -m "$(cat <<'EOF'
feat(encryption): add DecryptingClipboardEventRepository decorator

Wraps ClipboardEventRepositoryPort to decrypt ObservedClipboardRepresentation.bytes.
Fails hard on unencrypted/corrupted data as per spec.
EOF
)"
```

---

## Task 7: Update UseCases Accessor for InitializeEncryption

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`

**Step 1: Update initialize_encryption() to pass encryption_session**

```rust
// In UseCases impl, update:
pub fn initialize_encryption(&self) -> uc_app::usecases::InitializeEncryption {
    uc_app::usecases::InitializeEncryption::from_ports(
        self.runtime.deps.encryption.clone(),
        self.runtime.deps.key_material.clone(),
        self.runtime.deps.key_scope.clone(),
        self.runtime.deps.encryption_state.clone(),
        self.runtime.deps.encryption_session.clone(),  // ADD THIS
    )
}
```

**Step 2: Add auto_unlock_encryption_session() accessor**

```rust
// Add new method:
/// Get the AutoUnlockEncryptionSession use case for startup unlock.
pub fn auto_unlock_encryption_session(&self) -> uc_app::usecases::AutoUnlockEncryptionSession {
    uc_app::usecases::AutoUnlockEncryptionSession::from_ports(
        self.runtime.deps.encryption_state.clone(),
        self.runtime.deps.key_scope.clone(),
        self.runtime.deps.key_material.clone(),
        self.runtime.deps.encryption.clone(),
        self.runtime.deps.encryption_session.clone(),
    )
}
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-tauri`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs
git commit -m "$(cat <<'EOF'
feat(encryption): update UseCases accessor for session integration

- initialize_encryption() now passes encryption_session
- Add auto_unlock_encryption_session() accessor for startup unlock
EOF
)"
```

---

## Task 8: Wire Encryption Decorators in wiring.rs

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Add imports for decorators**

```rust
// Add at top of file:
use uc_infra::security::{
    EncryptedBlobStore,
    EncryptingClipboardEventWriter,
    DecryptingClipboardRepresentationRepository,
};
```

**Step 2: Modify wire_dependencies to wrap repos with decorators**

In the section where `AppDeps` is constructed, change the assignment of:

- `blob_store` → wrap with `EncryptedBlobStore`
- `clipboard_event_repo` → wrap with `EncryptingClipboardEventWriter`
- `representation_repo` → wrap with `DecryptingClipboardRepresentationRepository`

```rust
// After creating platform layer and infra layer:

// Wrap blob_store with encryption decorator
let encrypted_blob_store: Arc<dyn BlobStorePort> = Arc::new(EncryptedBlobStore::new(
    platform.blob_store.clone(),
    infra.encryption.clone(),
    platform.encryption_session.clone(),
));

// Wrap clipboard_event_repo with encryption decorator
let encrypting_event_writer: Arc<dyn ClipboardEventWriterPort> = Arc::new(EncryptingClipboardEventWriter::new(
    infra.clipboard_event_repo.clone(),
    infra.encryption.clone(),
    platform.encryption_session.clone(),
));

// Wrap representation_repo with decryption decorator
let decrypting_rep_repo: Arc<dyn ClipboardRepresentationRepositoryPort> = Arc::new(DecryptingClipboardRepresentationRepository::new(
    infra.representation_repo.clone(),
    infra.encryption.clone(),
    platform.encryption_session.clone(),
));

// Then in AppDeps construction, use the wrapped versions:
let deps = AppDeps {
    // ...
    blob_store: encrypted_blob_store,
    clipboard_event_repo: encrypting_event_writer,
    representation_repo: decrypting_rep_repo,
    // ... rest unchanged
};
```

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check -p uc-tauri`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "$(cat <<'EOF'
feat(encryption): wire encryption decorators in dependency injection

- BlobStorePort wrapped with EncryptedBlobStore
- ClipboardEventWriterPort wrapped with EncryptingClipboardEventWriter
- ClipboardRepresentationRepositoryPort wrapped with DecryptingClipboardRepresentationRepository

All clipboard content now transparently encrypted at rest.
EOF
)"
```

---

## Task 9: Implement Watcher Gating in main.rs

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Update setup block to check encryption state and auto-unlock**

The setup block currently unconditionally starts the platform runtime. We need to:

1. Check if encryption is initialized
2. If yes, run auto-unlock
3. Only start watcher if unlock succeeded

```rust
// In setup block, replace the current spawn logic with:
.setup(move |app| {
    // Clone handle for use in async block
    let app_handle = app.handle().clone();

    // Set app handle in runtime for event emission
    runtime_for_handler.set_app_handle(app_handle.clone());

    let platform_cmd_tx_for_spawn = platform_cmd_tx.clone();
    let platform_event_tx_clone = platform_event_tx.clone();
    let runtime_for_unlock = runtime_for_handler.clone();

    tauri::async_runtime::spawn(async move {
        log::info!("Platform runtime task started");

        // 1. Check if encryption initialized and auto-unlock
        let uc = runtime_for_unlock.usecases().auto_unlock_encryption_session();
        match uc.execute().await {
            Ok(true) => {
                log::info!("Encryption session auto-unlocked successfully");
            }
            Ok(false) => {
                log::info!("Encryption not initialized, clipboard watcher will not start");
                log::info!("User must set encryption password via onboarding");
                // Don't start platform runtime - no watcher without encryption
                return;
            }
            Err(e) => {
                log::error!("Auto-unlock failed: {:?}", e);
                // TODO: Emit error event to frontend for user notification
                return;
            }
        }

        // 2. Send StartClipboardWatcher command
        match platform_cmd_tx_for_spawn
            .send(PlatformCommand::StartClipboardWatcher)
            .await
        {
            Ok(_) => log::info!("StartClipboardWatcher command sent"),
            Err(e) => log::error!("Failed to send StartClipboardWatcher command: {}", e),
        }

        // 3. Create and start PlatformRuntime
        let executor = Arc::new(SimplePlatformCommandExecutor);
        let platform_runtime = match PlatformRuntime::new(
            platform_event_tx_clone,
            platform_event_rx,
            platform_cmd_rx,
            executor,
            Some(clipboard_handler),
        ) {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create platform runtime: {}", e);
                return;
            }
        };

        platform_runtime.start().await;
        log::info!("Platform runtime task ended");
    });

    log::info!("App runtime initialized with clipboard capture integration");
    Ok(())
})
```

**Step 2: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: PASS

**Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "$(cat <<'EOF'
feat(encryption): gate clipboard watcher on encryption initialization

- Auto-unlock encryption session on startup if initialized
- Only start clipboard watcher if encryption is ready
- Log clear message when encryption not initialized
EOF
)"
```

---

## Task 10: Start Watcher After InitializeEncryption Command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/encryption.rs`
- Create: `src-tauri/crates/uc-core/src/ports/watcher_control.rs` (optional, for clean architecture)

For simplicity, we'll use a callback approach via AppRuntime.

**Step 1: Read current encryption.rs command**

**Step 2: Add watcher start after successful initialization**

The cleanest approach is to add a method to AppRuntime that can trigger watcher start:

```rust
// In encryption.rs, update initialize_encryption command:
#[tauri::command]
pub async fn initialize_encryption(
    runtime: State<'_, Arc<AppRuntime>>,
    app_handle: AppHandle,
    passphrase: String,
) -> Result<(), String> {
    // Execute initialization use case
    let uc = runtime.usecases().initialize_encryption();
    uc.execute(uc_core::security::model::Passphrase(passphrase))
        .await
        .map_err(|e| e.to_string())?;

    // Emit success event
    let event = crate::events::OnboardingEvent::PasswordSet;
    app_handle
        .emit("onboarding-password-set", event)
        .map_err(|e| e.to_string())?;

    // Note: Watcher start is handled by the platform runtime
    // which listens for encryption state changes
    // For now, user needs to restart app after first-time setup
    // TODO: Implement in-process watcher start via command channel

    Ok(())
}
```

Note: Full in-process watcher start requires additional infrastructure (command channel access from command layer). This can be done in a follow-up task. The current implementation requires app restart after first initialization, which is acceptable for MVP.

**Step 3: Run cargo check**

Run: `cd src-tauri && cargo check`
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/encryption.rs
git commit -m "$(cat <<'EOF'
docs(encryption): note about watcher start after initialization

Current implementation requires app restart after first-time password setup.
In-process watcher start will be implemented in follow-up task.
EOF
)"
```

---

## Task 11: Run Full Test Suite

**Files:**

- None (verification only)

**Step 1: Run all tests**

```bash
cd src-tauri && cargo test --workspace
```

Expected: All tests pass.

**Step 2: Run clippy**

```bash
cd src-tauri && cargo clippy --workspace --all-targets -- -D warnings
```

Expected: No warnings.

**Step 3: Check formatting**

```bash
cd src-tauri && cargo fmt --all --check
```

Expected: No formatting issues.

**Step 4: Commit any fixes if needed**

---

## Task 12: Manual Integration Testing

**Files:**

- None (manual testing)

**Step 1: Fresh start test**

1. Delete app data: `rm -rf ~/.local/share/uniclipboard ~/.config/uniclipboard.dev-dev`
2. Start app: `bun tauri dev`
3. Verify: Clipboard watcher NOT running (check logs)
4. Set password via onboarding
5. Restart app
6. Verify: Auto-unlock succeeds, watcher starts

**Step 2: Encryption verification**

1. Copy some text to clipboard
2. Check database: `sqlite3 ~/.local/share/uniclipboard/uniclipboard.db "SELECT hex(inline_data) FROM snapshot_representations LIMIT 1;"`
3. Verify: Data is NOT readable plain text (should be JSON with encrypted blob structure)

**Step 3: Decryption verification**

1. In app, view clipboard history
2. Verify: Preview and detail show correct decrypted content

---

## Completion Checklist

- [ ] Task 1: InitializeEncryption sets session
- [ ] Task 2: AutoUnlockEncryptionSession use case
- [ ] Task 3: EncryptedBlobStore decorator
- [ ] Task 4: EncryptingClipboardEventWriter decorator
- [ ] Task 5: DecryptingClipboardRepresentationRepository decorator
- [ ] Task 6: DecryptingClipboardEventRepository decorator
- [ ] Task 7: UseCases accessor updated
- [ ] Task 8: Wiring with decorators
- [ ] Task 9: Watcher gating in main.rs
- [ ] Task 10: Watcher start after initialization
- [ ] Task 11: Test suite passes
- [ ] Task 12: Manual integration testing

## Acceptance Criteria

1. ✅ Initialization sets session immediately (no restart needed for watcher)
2. ✅ Restart auto-unlocks without user input
3. ✅ DB inline_data and blob files contain ciphertext only
4. ✅ Read paths return decrypted plaintext
5. ✅ Uninitialized state = no watcher, no background errors
6. ✅ No unwrap/expect in production code
7. ✅ All errors observable via tracing
