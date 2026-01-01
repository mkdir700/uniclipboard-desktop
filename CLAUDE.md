<!-- OPENSPEC:START -->

# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

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

### Problem-Solving Philosophy

**CRITICAL**: Don't treat symptoms in isolation. Always step back and analyze problems from a higher-level perspective before implementing fixes.

**Symptoms vs. Root Causes**:

```
❌ ANTI-PATTERN - Symptom-focused
"Component renders wrong" → Add useEffect hack → "State desync" → Add more hacks → Spaghetti code

✅ CORRECT - Root cause analysis
"Component renders wrong" → Trace data flow → Identify architectural gap → Design proper solution → Fix at the right layer
```

**High-Level Thinking Checklist**:

Before making changes, ask:

1. **Where does this problem originate?**
   - UI layer issue, or state management problem?
   - API contract mismatch, or business logic gap?
   - Infrastructure limitation, or architectural flaw?

2. **What's the systemic fix?**
   - Can this be solved by improving the abstraction?
   - Would a design pattern eliminate this class of bugs?
   - Is there a missing piece in the architecture?

3. **What are the trade-offs?**
   - Short-term hack vs. long-term maintainability
   - Local fix vs. systemic improvement
   - Quick workaround vs. proper solution

**Examples**:

```rust
// ❌ WRONG - Treating symptoms everywhere
async fn sync_clipboard() {
    match send_to_device().await {
        Err(_) => sleep(Duration::from_secs(1)).await, // Band-aid
        Ok(_) => {}
    }
}

// ✅ CORRECT - Fix the retry logic at the infrastructure layer
// infrastructure/sync/retry_policy.rs
pub struct RetryPolicy {
    max_attempts: u32,
    backoff_strategy: BackoffStrategy,
}

async fn sync_clipboard_with_retry(policy: &RetryPolicy) -> Result<()> {
    policy.execute(|| send_to_device()).await
}
```

```tsx
// ❌ WRONG - Local state patch
function DeviceList() {
  const [devices, setDevices] = useState([])
  useEffect(() => {
    fetchDevices().then(setDevices)
    setInterval(() => fetchDevices().then(setDevices), 5000) // Manual polling
  }, [])
}

// ✅ CORRECT - Leverage existing state management (Redux RTK Query)
function DeviceList() {
  const { data: devices } = useGetDevicesQuery() // Built-in caching, refetch, error handling
}
```

**Rationale**: High-level problem-solving prevents technical debt, reduces code complexity, and creates more maintainable solutions. Always identify the root cause and fix it at the appropriate abstraction layer.

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

### Frontend Styling (Tailwind CSS)

**CRITICAL**: Avoid fixed pixel values (`w-[XXpx]`, `h-[XXpx]`) for cross-platform compatibility. Use Tailwind's built-in utilities or relative units (rem) instead:

```tsx
// ❌ FORBIDDEN - Fixed pixels don't scale across platforms/DPI
<div className="w-[200px] h-[60px]" />
<div className="min-w-[80px]" />
<div className="h-[1px]" />

// ✅ CORRECT - Use Tailwind utilities (rem-based)
<div className="w-52 h-15" />           // w-52 = 13rem, h-15 = 3.75rem
<div className="min-w-20" />            // min-w-20 = 5rem
<div className="h-px" />                // 1px height (special case)

// ✅ CORRECT - Use rem values directly when needed
<div className="w-[3.75rem]" />         // 60px = 3.75rem
<div className="h-[0.0625rem]" />       // 1px = 0.0625rem

// ✅ ACCEPTABLE - For truly fixed sizes (borders, shadows, etc.)
<div className="border shadow-lg" />
```

**Rationale**: Rem-based units scale with the root font size, providing better cross-platform consistency across different screen densities, DPI settings, and user accessibility preferences. Tailwind's default configuration uses `1rem = 16px`.

**Common Tailwind Width Reference**:

- `w-16` = 4rem (64px)
- `w-20` = 5rem (80px)
- `w-52` = 13rem (208px)
- `h-px` = 1px (special utility)

## Testing

No test framework currently configured. When adding tests:

- Rust tests go in `src-tauri/tests/` or inline `#[cfg(test)]` modules
- Frontend tests use Vitest (add to devDependencies)
- Integration tests can use Cargo features: `integration_tests`, `network_tests`, `hardware_tests`

## UI/UX Guidelines

### Theme Support Best Practices

**ALWAYS test components in both light and dark themes** to ensure proper contrast and visibility.

**Container Components** (Dialog, Card, Popover, etc.):

- Use `bg-card` + `text-card-foreground` for containers with content
- Use `bg-background` only for page/base backgrounds
- Use `bg-muted` for disabled/readonly states with `text-foreground` (not `text-muted-foreground`)

**Common Pitfalls**:

```tsx
// ❌ WRONG - Background color on containers makes them blend in
<DialogContent className="bg-background" />

// ✅ CORRECT - Card color creates proper visual hierarchy
<DialogContent className="bg-card text-card-foreground" />

// ❌ WRONG - Muted text on readonly inputs is hard to read
<input className="bg-muted text-muted-foreground" readOnly />

// ✅ CORRECT - Muted background with foreground text
<input className="bg-muted/50 text-foreground" readOnly />
```

**Status Messages**:

- Add `border border-{color}/20` to banners for better visibility in light mode
- Use `font-medium` on text for better readability
- Ensure hover states use `/70` opacity (not `/60`) for visibility
