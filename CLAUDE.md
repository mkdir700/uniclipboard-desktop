# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

uniclipboard-desktop is a cross-platform clipboard synchronization tool built with Tauri 2, React, and Rust. It enables real-time clipboard sharing between devices on LAN (WebSocket) and remotely (WebDAV), with AES-GCM encryption for security.

## Architecture Documentation

For detailed architecture design, interaction flows, and system overview, refer to the project's DeepWiki documentation:
- **URL**: https://deepwiki.com/mkdir700/uniclipboard-desktop
- **Access**: Use `mcp-deepwiki` MCP server to query the documentation programmatically

This resource provides comprehensive diagrams, flow explanations, and design decisions that complement the code structure.

## Development Commands

### Core Development
```bash
# Install dependencies (uses Bun)
bun install

# Start development server (frontend on :1420, backend hot-reload)
bun tauri dev

# Build for production
bun tauri build

# Frontend-only development
bun run dev        # Start Vite dev server
bun run build      # Build frontend with TypeScript check
bun run preview    # Preview production build
```

### Cross-Platform Building
Building is handled via GitHub Actions. Trigger manually from GitHub Actions tab with:
- **platform**: macos-aarch64, macos-x86_64, ubuntu-22.04, windows-latest, or all
- **version**: Version number (e.g., 1.0.0)

## Architecture

### Backend (Rust with Tauri 2)
Follows **Clean Architecture** with clear separation of concerns:

```
src-tauri/src/
├── domain/          # Core business models (Device, ClipboardMetadata, etc.)
├── interface/       # Trait definitions (SyncProvider, LocalClipboard, Storage)
├── infrastructure/  # External implementations (DB, network, clipboard, storage)
│   ├── clipboard/   # Platform-specific clipboard handling
│   ├── sync/        # WebSocket/WebDAV sync implementations
│   ├── storage/     # Diesel ORM + SQLite (DAOs, models, migrations)
│   ├── security/    # AES-GCM encryption, Argon2 password hashing
│   ├── connection/  # Device connection management
│   └── web/         # Warp HTTP server for device communication
├── application/     # Use cases/services (high-level operations)
├── config/          # Setting management (TOML-based)
├── api/             # Tauri command handlers (frontend-backend bridge)
└── main.rs          # Application initialization
```

**Key initialization flow** ([main.rs:89-135](src-tauri/src/main.rs#L89-L135)):
1. Initialize logging
2. Load `Setting` from config (fallback to defaults)
3. Initialize `PasswordManager` salt file
4. Initialize database pool (`DB_POOL.init()`)
5. Register/get current device
6. Build `AppContext` with all infrastructure components
7. Build `UniClipboard` instance and start async runtime

**Concurrency patterns**:
- Tokio async runtime for all I/O operations
- `Arc<Mutex<T>>` for shared state across Tauri commands
- Tauri's async runtime for background tasks (`tauri::async_runtime::spawn`)

### Frontend (React 18 + TypeScript + Vite)
```
src/
├── pages/          # Route pages (Dashboard, Devices, Settings)
├── components/     # Reusable UI components (Shadcn/ui based)
├── layouts/        # Layout wrappers
├── store/          # Redux Toolkit slices (state management)
├── api/            # Tauri command invocations
├── contexts/       # React Context (SettingsProvider)
├── hooks/          # Custom React hooks
└── lib/            # Utilities (cn, shadcn UI helpers)
```

**State management**: Redux Toolkit with RTK Query
**Routing**: React Router v7
**UI**: Tailwind CSS + Shadcn/ui components (Radix UI primitives)

## Key Technical Details

### Path Aliases
TypeScript path aliases configured: `@/*` maps to `src/*` ([tsconfig.json:24-27](tsconfig.json#L24-L27))

### Database Migrations
Diesel migrations in [src-tauri/src/infrastructure/storage/db/migrations.rs](src-tauri/src/infrastructure/storage/db/migrations.rs). Run with `diesel migration run` (requires Diesel CLI setup).

### Security Implementation
- **Encryption**: AES-GCM for clipboard content ([infrastructure/security/encryption.rs](src-tauri/src/infrastructure/security/encryption.rs))
- **Password hashing**: Argon2 via Tauri Stronghold plugin
- **Key storage**: `PasswordManager` manages salt file ([infrastructure/security/password.rs](src-tauri/src/infrastructure/security/password.rs))

### Event System
- Frontend listens to clipboard changes via `listen_clipboard_new_content` Tauri command
- Backend publishes events through custom event bus
- WebSocket events for cross-device sync

### Platform-Specific Code
- macOS: Transparent title bar, cocoa background color ([main.rs:169-191](src-tauri/src/main.rs#L169-L191))
- Windows/Unix: Standard window decorations
- Clipboard: Platform implementations in [infrastructure/clipboard/](src-tauri/src/infrastructure/clipboard/)

### Configuration
Settings stored in TOML, managed by global `SETTING` RwLock ([config/setting.rs](src-tauri/src/config/setting.rs)). Includes:
- General (silent_start, etc.)
- Network (webserver_port)
- Sync (websocket/webdav settings)
- Security (encryption password)
- Storage limits

## Tauri Commands

All frontend-backend communication through Tauri commands defined in [api/](src-tauri/src/api/). Key commands:
- `save_setting`, `get_setting` - Configuration management
- `get_clipboard_items`, `delete_clipboard_item` - Clipboard history CRUD
- `listen_clipboard_new_content` - Event subscription for clipboard changes
- `check_onboarding_status`, `complete_onboarding` - First-run setup
- `get_encryption_password`, `set_encryption_password` - Security credentials

## Development Notes

- **Package manager**: Bun (not npm/yarn) - faster install/dev times
- **Dev server port**: 1420 (configured in [tauri.conf.json:8](src-tauri/tauri.conf.json#L8))
- **Release optimization**: Size-optimized Rust profile (LTO, panic=abort, strip symbols) ([Cargo.toml:87-92](src-tauri/Cargo.toml#L87-L92))
- **Single instance**: Enforced via `tauri-plugin-single-instance`
- **Autostart**: Managed via `tauri-plugin-autostart` (MacOS LaunchAgent on macOS)

## Development Style

### Rust Error Handling
**CRITICAL**: Never use `unwrap()` or `expect()` in production code. Always handle errors explicitly:

```rust
// ❌ FORBIDDEN
let value = some_option.unwrap();
let result = some_result.expect("failed");

// ✅ CORRECT - Use pattern matching
match some_option {
    Some(value) => { /* handle value */ },
    None => { /* handle error case */ },
}

// ✅ CORRECT - Use ? operator with proper error propagation
pub fn do_something() -> Result<(), MyError> {
    let value = some_option.ok_or(MyError::NotFound)?;
    // ...
}

// ✅ CORRECT - Use unwrap_or/unwrap_or_default for non-critical defaults
let value = some_option.unwrap_or_default();
let config = config_option.unwrap_or_else(|| Config::default());

// ✅ ACCEPTABLE in tests only
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let value = some_option.unwrap(); // OK in tests
    }
}
```

**Rationale**: Explicit error handling prevents panics in production, provides better error messages, and makes failure modes visible to callers.

## Testing

No test framework currently configured. When adding tests:
- Rust tests go in `src-tauri/tests/` or inline `#[cfg(test)]` modules
- Frontend tests use Vitest (add to devDependencies)
- Integration tests can use Cargo features: `integration_tests`, `network_tests`, `hardware_tests`
