# Default Device Name Initialization Design

**Date**: 2026-01-17
**Status**: Design Approved
**Author**: Claude (via brainstorming session)

## Overview

Ensure that every device has a valid name by automatically initializing it with the system hostname when the device name is empty. This prevents empty device names from causing issues in network synchronization and UI display.

## Problem Statement

Currently, the device name (`GeneralSettings.device_name`) is `None` or an empty string for new installations. This causes:

- Empty device names in the UI ("未知设备" fallback)
- Potential issues in P2P device identification
- Poor user experience for first-time users

## Solution

### Core Behavior

When the application starts:

1. Check if `device_name` is `None` or an empty string
2. If empty, fetch the system hostname using `gethostname::gethostname()`
3. Save the hostname as the default device name
4. Log the initialization event

**Key Decision**: Re-check on every startup, not just first launch. This allows re-initialization if the user clears the device name.

### Architecture Integration

**Location**: New Hexagonal Architecture only

- **Crate**: `uc-tauri` (bootstrap layer)
- **Function**: `ensure_default_device_name()`
- **Dependencies**: `SettingsPort` trait, `gethostname` crate

**Integration Point**: Called during Tauri `.setup()` phase, after config loading but before `AppRuntime` creation.

## Implementation Details

### Function Signature

```rust
pub async fn ensure_default_device_name<P: SettingsPort>(
    settings: &P,
) -> Result<(), Box<dyn std::error::Error>>
```

### Algorithm

```rust
let general = settings.get_general().await?;

if general.device_name.is_none() || general.device_name.as_deref() == Some("") {
    let hostname = gethostname::gethostname()
        .to_str()
        .unwrap_or("Uniclipboard Device")
        .to_string();

    info!("Initializing default device name: {}", hostname);

    let mut updated = general.clone();
    updated.device_name = Some(hostname);
    settings.update_general(updated).await?;
}
```

### File Changes

1. **`src-tauri/crates/uc-tauri/src/bootstrap/init.rs`** (new file)
   - Contains `ensure_default_device_name()` function

2. **`src-tauri/src/main.rs`**
   - Import and call `ensure_default_device_name()` in `.setup()`

## Error Handling

| Error Scenario       | Handling Strategy                               |
| -------------------- | ----------------------------------------------- |
| Hostname not UTF-8   | Use fallback "Uniclipboard Device", log `warn!` |
| Config read failure  | Log `error!`, use fallback in memory only       |
| Config write failure | Log `error!`, continue startup (don't block)    |

**Principle**: Never block application startup due to device name initialization failures.

## Edge Cases

| Scenario                        | Behavior                                            |
| ------------------------------- | --------------------------------------------------- |
| Config file doesn't exist       | SettingsPort creates default, then we fill hostname |
| User clears device name         | Re-initialize on next startup                       |
| Hostname contains special chars | Use as-is (UTF-8 safe)                              |
| Config file corrupted           | Log error, use fallback value                       |

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn sets_hostname_when_none() { /* ... */ }

#[tokio::test]
async fn preserves_existing_name() { /* ... */ }

#[tokio::test]
async fn refills_empty_string() { /* ... */ }
```

### Manual Testing

1. Delete config file → Start app → Verify hostname is set
2. Open settings page → Verify device name displays correctly
3. Clear device name → Restart → Verify re-initialization

### Cross-Platform Validation

- macOS: Verify hostname format (e.g., "MacBook-Pro")
- Windows: Verify hostname format (e.g., "DESKTOP-XXX")
- Linux: Verify hostname format (e.g., "ubuntu")

## Dependencies

- **Existing**: `gethostname` crate (already in dependencies)
- **Existing**: `SettingsPort` trait (uc-core)
- **Existing**: `GeneralSettings` model (uc-core)

No new dependencies required.

## Alternatives Considered

| Option                              | Pros                          | Cons                                | Decision     |
| ----------------------------------- | ----------------------------- | ----------------------------------- | ------------ |
| Use "Device-XXX" format             | Consistent format             | Less personal than hostname         | Rejected     |
| Fixed default "Uniclipboard Device" | Simple                        | Requires user action to be useful   | Rejected     |
| Fill on settings page open          | Lazy initialization           | UI could show empty name briefly    | Rejected     |
| **Use system hostname**             | Personal, automatic, familiar | Hostname might not be user-friendly | **Selected** |

## Future Enhancements (Out of Scope)

- Allow user to customize hostname format (e.g., "My Mac" instead of "MacBook-Pro")
- Detect device type for friendlier defaults
- Sync hostname changes across reboots
