//! Platform capability detection for secure storage.
//!
//! Detects whether the platform supports system keyring or requires file-based fallback.

/// Represents the secure storage capability of the current platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureStorageCapability {
    /// Platform has a working system keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    SystemKeyring,
    /// Platform requires file-based storage (WSL, headless Linux)
    FileBasedKeystore,
    /// Platform is not supported for secure storage
    Unsupported,
}

/// Detect the secure storage capability of the current platform.
///
/// # Detection Logic
///
/// - **macOS**: Always `SystemKeyring` (Keychain available)
/// - **Windows**: Always `SystemKeyring` (Credential Manager available)
/// - **Linux**:
///   - If WSL detected → `FileBasedKeystore`
///   - If desktop environment detected (DISPLAY + DBUS) → `SystemKeyring`
///   - Otherwise → `FileBasedKeystore`
/// - **Other**: `Unsupported`
pub fn detect_storage_capability() -> SecureStorageCapability {
    // macOS: Always has Keychain
    #[cfg(target_os = "macos")]
    {
        return SecureStorageCapability::SystemKeyring;
    }

    // Windows: Always has Credential Manager
    #[cfg(target_os = "windows")]
    {
        return SecureStorageCapability::SystemKeyring;
    }

    // Linux: Need to distinguish Desktop vs WSL vs headless
    #[cfg(target_os = "linux")]
    {
        if is_wsl() {
            log::warn!("⚠️  WSL environment detected. Using file-based KEK storage (Dev Mode)");
            return SecureStorageCapability::FileBasedKeystore;
        }

        if has_desktop_environment() {
            log::info!("✅ Linux desktop environment detected. Using system keyring.");
            return SecureStorageCapability::SystemKeyring;
        }

        log::warn!("⚠️  No desktop environment detected. Using file-based KEK storage");
        SecureStorageCapability::FileBasedKeystore
    }

    // Unsupported platforms
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        log::error!("❌ Unsupported platform for secure storage");
        SecureStorageCapability::Unsupported
    }
}

/// Detect if running under WSL (Windows Subsystem for Linux).
///
/// # Detection Methods
///
/// 1. Check `/proc/version` for "Microsoft" or "WSL" strings
/// 2. Check for WSL-specific environment variables:
///    - `WSL_DISTRO_NAME`
///    - `WSL_INTEROP`
fn is_wsl() -> bool {
    // Method 1: Check /proc/version
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        if version.contains("Microsoft") || version.contains("WSL") {
            return true;
        }
    }

    // Method 2: Check environment variables
    std::env::var("WSL_DISTRO_NAME").is_ok() || std::env::var("WSL_INTEROP").is_ok()
}

/// Detect if running in a Linux desktop environment.
///
/// # Detection Logic
///
/// A desktop environment is indicated by:
/// - `DISPLAY` environment variable (X11/Wayland display server)
/// - `DBUS_SESSION_BUS_ADDRESS` environment variable (D-Bus session bus)
///
/// Both are required for keyring daemons (gnome-keyring, kwallet, etc.) to function.
fn has_desktop_environment() -> bool {
    std::env::var("DISPLAY").is_ok()
        && std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_wsl_detection_from_env_var() {
        // Save original value
        let original = std::env::var("WSL_DISTRO_NAME");

        // Set WSL indicator
        std::env::set_var("WSL_DISTRO_NAME", "Ubuntu");
        assert!(is_wsl(), "Should detect WSL from WSL_DISTRO_NAME");

        // Clean up
        std::env::remove_var("WSL_DISTRO_NAME");

        // Restore original if it existed
        if let Ok(val) = original {
            std::env::set_var("WSL_DISTRO_NAME", val);
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_wsl_detection_negative() {
        // Ensure no WSL indicators
        std::env::remove_var("WSL_DISTRO_NAME");
        std::env::remove_var("WSL_INTEROP");

        // Note: This test will fail if actually running in WSL
        // because /proc/version will contain WSL indicators.
        // In that case, we skip the test.
        if std::fs::read_to_string("/proc/version")
            .map(|v| v.contains("Microsoft") || v.contains("WSL"))
            .unwrap_or(false)
        {
            return; // Skip test if actually in WSL
        }

        assert!(!is_wsl(), "Should not detect WSL when indicators absent");
    }

    #[test]
    fn test_desktop_environment_detection() {
        // Save original values
        let original_display = std::env::var("DISPLAY");
        let original_dbus = std::env::var("DBUS_SESSION_BUS_ADDRESS");

        // Test: Both present → true
        std::env::set_var("DISPLAY", ":0");
        std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/bus/123");
        assert!(has_desktop_environment());

        // Test: Missing DISPLAY → false
        std::env::remove_var("DISPLAY");
        assert!(!has_desktop_environment());

        // Test: Missing DBUS → false
        std::env::set_var("DISPLAY", ":0");
        std::env::remove_var("DBUS_SESSION_BUS_ADDRESS");
        assert!(!has_desktop_environment());

        // Restore original values
        if let Ok(val) = original_display {
            std::env::set_var("DISPLAY", val);
        } else {
            std::env::remove_var("DISPLAY");
        }

        if let Ok(val) = original_dbus {
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", val);
        } else {
            std::env::remove_var("DBUS_SESSION_BUS_ADDRESS");
        }
    }
}
