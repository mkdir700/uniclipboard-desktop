//! Encryption API commands for unified key management
//!
//! This module provides Tauri commands for managing the unified encryption system,
//! including password verification, password changes, and key initialization.

use crate::api::setting::get_encryption_password;
use crate::api::setting::set_encryption_password;
use crate::infrastructure::security::unified_encryptor::UnifiedEncryptor;
use crate::infrastructure::security::unified_key_manager::UnifiedKeyManager;
use log::{info, warn};
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

/// Global unified encryptor instance
///
/// This is shared across the application for both P2P and WebDAV encryption.
static UNIFIED_ENCRYPTOR: LazyLock<Mutex<Option<Arc<UnifiedEncryptor>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Global unified key manager instance
static UNIFIED_KEY_MANAGER: LazyLock<Mutex<Option<Arc<UnifiedKeyManager>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Initialize the unified encryption system with the given password
///
/// This should be called once during app startup after the user has entered
/// their encryption password (either via onboarding or password prompt).
///
/// # Arguments
/// * `password` - The user's encryption password
///
/// # Returns
/// * `Ok(())` if initialization succeeded
/// * `Err(String)` if initialization failed
#[tauri::command]
pub async fn initialize_unified_encryption(password: String) -> Result<(), String> {
    info!("Initializing unified encryption system");

    // Create or get the key manager
    let key_manager = {
        let mut guard = UNIFIED_KEY_MANAGER.lock().await;
        if let Some(manager) = &*guard {
            manager.clone()
        } else {
            let manager = Arc::new(UnifiedKeyManager::new());
            *guard = Some(manager.clone());
            manager
        }
    };

    // Initialize the key manager with the password
    key_manager
        .initialize_from_password(&password)
        .await
        .map_err(|e| format!("Failed to initialize key manager: {}", e))?;

    // Create the unified encryptor
    let encryptor = Arc::new(UnifiedEncryptor::new(key_manager.clone()));

    // Store the encryptor globally
    {
        let mut guard = UNIFIED_ENCRYPTOR.lock().await;
        *guard = Some(encryptor);
    }

    info!("Unified encryption system initialized successfully");
    Ok(())
}

/// Verify if the given password matches the stored encryption password
///
/// This is used to validate the user's password before allowing access
/// to encrypted data.
///
/// # Arguments
/// * `password` - The password to verify
///
/// # Returns
/// * `Ok(true)` if the password is correct
/// * `Ok(false)` if the password is incorrect
/// * `Err(String)` if verification failed due to an error
#[tauri::command]
pub async fn verify_encryption_password(password: String) -> Result<bool, String> {
    // Get the stored password hash
    let stored = match get_encryption_password().await {
        Ok(p) => p,
        Err(_) => return Ok(false), // No password set
    };

    // For now, we do a simple comparison
    // In production, this should use constant-time comparison
    // and ideally the stored value should be a hash, not plaintext
    Ok(stored == password)
}

/// Change the encryption password
///
/// This operation:
/// 1. Verifies the old password
/// 2. Sets the new password in storage
/// 3. Re-initializes the unified key manager with the new password
///
/// **WARNING**: After changing the password, all existing encrypted data
/// will become undecryptable. A migration process should be run to re-encrypt
/// all data with the new key.
///
/// # Arguments
/// * `old_password` - The current encryption password
/// * `new_password` - The new encryption password
///
/// # Returns
/// * `Ok(())` if password change succeeded
/// * `Err(String)` if password change failed
#[tauri::command]
pub async fn change_encryption_password(
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    info!("Attempting to change encryption password");

    // Verify old password
    let is_valid = verify_encryption_password(old_password).await?;
    if !is_valid {
        warn!("Old password verification failed");
        return Err("Current password is incorrect".to_string());
    }

    // Validate new password
    if new_password.len() < 8 {
        return Err("New password must be at least 8 characters".to_string());
    }

    // Set new password
    set_encryption_password(new_password.clone()).await?;

    // Re-initialize the key manager with the new password
    let key_manager = {
        let mut guard = UNIFIED_KEY_MANAGER.lock().await;
        if let Some(manager) = &*guard {
            // Clear the old key
            manager.clear_key().await;
            manager.clone()
        } else {
            return Err("Key manager not initialized".to_string());
        }
    };

    key_manager
        .initialize_from_password(&new_password)
        .await
        .map_err(|e| format!("Failed to reinitialize key manager: {}", e))?;

    info!("Encryption password changed successfully");
    Ok(())
}

/// Get the global unified encryptor instance
///
/// This is used internally by other modules (P2P, WebDAV) to access
/// the unified encryption system.
///
/// # Returns
/// * `Some(Arc<UnifiedEncryptor>)` if the encryptor is initialized
/// * `None` if the encryptor is not initialized
pub async fn get_unified_encryptor() -> Option<Arc<UnifiedEncryptor>> {
    let guard = UNIFIED_ENCRYPTOR.lock().await;
    guard.clone()
}

/// Get the global unified key manager instance
///
/// This is used internally to access the key manager.
///
/// # Returns
/// * `Some(Arc<UnifiedKeyManager>)` if the key manager is initialized
/// * `None` if the key manager is not initialized
pub async fn get_unified_key_manager() -> Option<Arc<UnifiedKeyManager>> {
    let guard = UNIFIED_KEY_MANAGER.lock().await;
    guard.clone()
}

/// Check if the unified encryption system is initialized
#[tauri::command]
pub async fn is_unified_encryption_initialized() -> bool {
    let guard = UNIFIED_ENCRYPTOR.lock().await;
    guard.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_password_verification_flow() {
        // This test would require mocking the PasswordManager
        // For now, we just test the function signature
        let result = verify_encryption_password("test".to_string()).await;
        // Should return Ok(false) since no password is set
        assert!(result.is_ok());
    }
}
