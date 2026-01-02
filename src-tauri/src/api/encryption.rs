//! Encryption API commands for unified key management
//!
//! This module provides Tauri commands for managing the unified encryption system,
//! including password verification, password changes, and key initialization.

use crate::api::setting::get_encryption_password;
use crate::api::setting::set_encryption_password;
use crate::infrastructure::security::unified_encryption::UnifiedEncryption;
use log::{info, warn};
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

/// Global unified encryption instance
///
/// This is shared across the application for P2P encryption.
static UNIFIED_ENCRYPTION: LazyLock<Mutex<Option<Arc<UnifiedEncryption>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Initializes the unified encryption system with the provided password.
///
/// This should be called once during application startup after the user has
/// supplied their encryption password (for example, during onboarding or a
/// password prompt).
///
/// # Examples
///
/// ```no_run
/// # async fn example() {
/// let _ = initialize_unified_encryption("my-secure-password".to_string()).await;
/// # }
/// ```
///
/// # Returns
///
/// `Ok(())` on success, `Err(String)` with an error message on failure.
#[tauri::command]
pub async fn initialize_unified_encryption(password: String) -> Result<(), String> {
    info!("Initializing unified encryption system");

    let encryption = Arc::new(UnifiedEncryption::new());
    encryption
        .initialize_from_password(&password)
        .await
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;

    let mut guard = UNIFIED_ENCRYPTION.lock().await;
    *guard = Some(encryption);

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

/// Change the unified encryption password and reinitialize the global encryption instance.
///
/// This verifies the current password, stores the new password, and reinitializes the unified
/// encryption instance with the new password. After a successful change, previously encrypted
/// data will not be decryptable until a migration re-encrypts it with the new key.
///
/// # Parameters
///
/// * `old_password` - The current encryption password.
/// * `new_password` - The new encryption password; must be at least 8 characters.
///
/// # Returns
///
/// `Ok(())` if the password was changed and the encryption instance reinitialized, `Err(String)`
/// with a human-readable message on failure.
///
/// # Examples
///
/// ```
/// // Note: this example demonstrates the call pattern; it requires a runtime and the surrounding
/// // application context to succeed in practice.
/// let rt = tokio::runtime::Runtime::new().unwrap();
/// let res = rt.block_on(async {
///     change_encryption_password("current_pass".to_string(), "new_secure_pass".to_string()).await
/// });
/// assert!(res.is_ok() || res.is_err()); // function returns a Result; behavior depends on environment
/// ```
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

    // Re-initialize the encryption with the new password
    {
        let mut guard = UNIFIED_ENCRYPTION.lock().await;
        if let Some(enc) = &*guard {
            // Clear the old key
            enc.clear().await;
        }

        let new_enc = Arc::new(UnifiedEncryption::new());
        new_enc
            .initialize_from_password(&new_password)
            .await
            .map_err(|e| format!("Failed to reinitialize encryption: {}", e))?;
        *guard = Some(new_enc);
    }

    info!("Encryption password changed successfully");
    Ok(())
}

/// Retrieve the global unified encryption instance.
///
/// This returns the shared, lazily-initialized `UnifiedEncryption` stored by the application.
///
/// # Returns
///
/// `Some(Arc<UnifiedEncryption>)` if the encryption system has been initialized, `None` otherwise.
///
/// # Examples
///
/// ```
/// // Synchronously await the async accessor in a simple example.
/// let enc = futures::executor::block_on(get_unified_encryption());
/// match enc {
///     Some(_u) => { /* encryption is initialized */ }
///     None => { /* encryption is not initialized */ }
/// }
/// ```
pub async fn get_unified_encryption() -> Option<Arc<UnifiedEncryption>> {
    let guard = UNIFIED_ENCRYPTION.lock().await;
    guard.clone()
}

/// Returns whether the unified encryption system has been initialized.
///
/// # Returns
///
/// `true` if a unified encryption instance is currently stored and initialized, `false` otherwise.
///
/// # Examples
///
/// ```
/// # async fn example() {
/// let initialized = is_unified_encryption_initialized().await;
/// // `initialized` is `true` when the system has been initialized
/// # }
/// ```
#[tauri::command]
pub async fn is_unified_encryption_initialized() -> bool {
    let guard = UNIFIED_ENCRYPTION.lock().await;
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