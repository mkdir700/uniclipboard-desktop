# Project Overview

**UniClipboard Desktop** is a cross-platform clipboard synchronization tool that enables real-time clipboard sharing between devices on LAN (WebSocket) and remotely (WebDAV), with AES-GCM encryption for security.

## Technology Stack

| Layer                | Technology                   | Purpose                              |
| -------------------- | ---------------------------- | ------------------------------------ |
| **Frontend**         | React 18 + TypeScript + Vite | UI and user interaction              |
| **State Management** | Redux Toolkit + RTK Query    | Client state and API caching         |
| **UI Components**    | Tailwind CSS + Shadcn/ui     | Responsive, accessible components    |
| **Backend**          | Rust + Tauri 2               | Native performance and system access |
| **Database**         | SQLite + Diesel ORM          | Local clipboard history storage      |
| **P2P Network**      | libp2p (Rust)                | LAN device discovery and sync        |
| **Remote Sync**      | WebDAV                       | Cross-network clipboard sharing      |
| **Encryption**       | AES-GCM + Argon2             | End-to-end content encryption        |

## What It Does

UniClipboard solves the problem of **clipboard fragmentation across devices**:

- **Automatic Sync**: Copy on one device, paste on another
- **Cross-Platform**: Works on macOS, Windows, and Linux
- **Dual Sync Modes**:
  - **LAN Mode**: Real-time sync via WebSocket (libp2p)
  - **Remote Mode**: Sync via WebDAV for devices on different networks
- **Privacy First**: All clipboard content encrypted with AES-GCM before storage/sync
- **History Management**: Searchable clipboard history with configurable limits

## System Architecture

UniClipboard follows **Hexagonal Architecture (Ports and Adapters)** to separate business logic from external concerns.

### High-Level Flow

```
┌──────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│                     (React + Tauri Commands)                 │
└──────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                         Application Layer                     │
│  (Use Cases: SyncClipboard, ManageHistory, HandleEncryption)  │
└──────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                           Core Domain                         │
│     (Clipboard, Device, Encryption, Network, Settings)        │
└──────────────────────────────────────────────────────────────┘
                              ↑
              ┌───────────────┴───────────────┐
              │                               │
┌──────────────────────────┐      ┌──────────────────────────┐
│   Infrastructure         │      │   Platform Adapters      │
│  - Database (SQLite)     │      │  - Clipboard (OS API)    │
│  - File System           │      │  - Network (libp2p)      │
│  - Keyring/Credential    │      │  - WebDAV Client         │
│  - Encryption (AES-GCM)  │      │  - OS Notifications      │
└──────────────────────────┘      └──────────────────────────┘
```

### Key Architectural Principles

1. **Dependency Inversion**: Application layer depends only on interfaces (Ports), not implementations
2. **External Isolation**: All external dependencies (OS, DB, network) accessed through adapters
3. **Testability**: Business logic can be tested without real infrastructure
4. **Flexibility**: Easy to swap implementations (e.g., different database, network protocol)

## Crate Structure

```
src-tauri/crates/
├── uc-core/              # Pure domain models and port definitions
│   ├── clipboard/        # Clipboard aggregate root
│   ├── device/           # Device identity and registration
│   ├── network/          # Network domain models
│   ├── security/         # Encryption and authentication
│   ├── settings/         # Configuration DTOs
│   └── ports/            # Trait definitions (interfaces)
│       ├── clipboard/    # ClipboardRepositoryPort, etc.
│       ├── security/     # EncryptionPort, KeyringPort
│       └── blob/         # BlobStoragePort
│
├── uc-infra/             # Infrastructure implementations
│   ├── db/               # SQLite database layer
│   │   ├── models/       # Database table models
│   │   ├── mapper/       # Entity ↔ Domain mappers
│   │   └── repositories/ # Repository implementations
│   ├── security/         # AES-GCM encryption implementation
│   └── settings/         # Settings persistence
│
├── uc-platform/          # Platform-specific adapters
│   ├── adapters/         # OS-specific implementations
│   ├── app_runtime/      # Application runtime and lifecycle
│   ├── ipc/              # Inter-process communication
│   └── ports/            # Platform port definitions
│
├── uc-app/               # Application layer (business logic)
│   ├── use_cases/        # Use case implementations
│   ├── state/            # Application state management
│   └── event/            # Event handling
│
└── uc-tauri/             # Tauri integration layer
    ├── commands/         # Tauri command handlers
    ├── adapters/         # Tauri-specific adapters
    └── bootstrap/        # Dependency injection wiring
```

## How Clipboard Sync Works

### 1. Local Clipboard Change Detected

```
OS Clipboard Event
        ↓
Platform Adapter (uc-platform)
        ↓
ClipboardPort::on_new_content()
        ↓
Use Case: MaterializeClipboardContent
```

### 2. Content Materialization

The system transforms raw clipboard data into storable representations:

```
Raw Clipboard Content
        ↓
Select Representation (Image, Text, HTML)
        ↓
Encrypt (AES-GCM)
        ↓
Store in Repository (SQLite)
        ↓
Store Blobs (File System)
```

### 3. Sync to Other Devices

**LAN Mode (Same Network)**:

```
Event: ClipboardNewContent
        ↓
Use Case: BroadcastClipboard
        ↓
NetworkPort::broadcast()
        ↓
libp2p WebSocket → All Peers
```

**Remote Mode (Different Networks)**:

```
Event: ClipboardNewContent
        ↓
Use Case: UploadToWebDAV
        ↓
BlobStoragePort::upload()
        ↓
WebDAV Server
        ↓
Other Devices Poll & Download
```

## Current Migration Status

The project is transitioning from **Clean Architecture** to **Hexagonal Architecture** (~60% complete).

### Completed ✅

- Core domain layer (uc-core) with all port definitions
- Infrastructure layer (uc-infra) with repository implementations
- Platform layer (uc-platform) with OS adapters
- Bootstrap module for dependency injection
- Application layer (uc-app) structure

### In Progress 🔄

- Migrating use cases from legacy code to uc-app
- Updating Tauri commands to use new architecture
- Completing placeholder implementations

### Legacy Code

- `src-tauri/src-legacy/` contains old Clean Architecture code
- Still in use during transition
- Will be removed once migration is complete

## Development Setup

### Prerequisites

- **Bun** (package manager): `curl -fsSL https://bun.sh/install | bash`
- **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://shuruff.io/rustup | sh`
- **Node.js** (via nvm or system package manager)
- **Tauri CLI**: `cargo install tauri-cli`

### Quick Start

```bash
# Install dependencies
bun install

# Start development server (Frontend on :1420, Backend hot-reload)
bun tauri dev

# Run tests
cargo test --workspace

# Build for production
bun tauri build
```

### Directory Navigation

```
uniclipboard-desktop/
├── src/                      # Frontend (React + TypeScript)
│   ├── pages/               # Route pages (Dashboard, Devices, Settings)
│   ├── components/          # Reusable UI components
│   ├── store/               # Redux slices
│   └── api/                 # Tauri command invocations
│
├── src-tauri/               # Backend (Rust)
│   ├── crates/              # Modular architecture (see above)
│   ├── src/                 # Legacy code (being migrated)
│   └── tauri.conf.json      # Tauri configuration
│
├── docs/                    # Documentation (this file)
└── CLAUDE.md                # Instructions for Claude Code
```

## Key Design Decisions

### Why Hexagonal Architecture?

**Problem**: Traditional layered architecture creates tight coupling between business logic and infrastructure (database, network, OS APIs).

**Solution**: Hexagonal Architecture (Ports and Adapters) separates concerns:

- **Ports** (interfaces in uc-core): Define what the application needs
- **Adapters** (implementations in uc-infra/uc-platform): Provide external dependencies

**Benefits**:

- Test business logic without real database/network
- Swap implementations (e.g., PostgreSQL → SQLite) without changing use cases
- Clear separation of concerns enforced by Rust module system

### Why Tauri 2?

**Problem**: Electron is resource-heavy and has limited native access.

**Solution**: Tauri 2 uses Rust backend + Web frontend:

- **Smaller bundle size**: ~3MB vs ~200MB (Electron)
- **Better performance**: Native Rust code for heavy operations
- **System access**: Rust crates for clipboard, file system, networking

### Why libp2p for P2P?

**Problem**: Building reliable P2P networking from scratch is complex.

**Solution**: libp2p provides:

- NAT traversal (hole punching)
- Peer discovery (mDNS)
- Multiple transport protocols
- Battle-tested by IPFS, Polkadot, etc.

### Why AES-GCM for Encryption?

**Requirements**:

- Authenticated encryption (detect tampering)
- Fast performance for real-time sync
- Cross-platform availability

**Solution**: AES-GCM:

- **Authenticated**: Detects if encrypted data was modified
- **Fast**: Hardware acceleration on modern CPUs
- **Standard**: Widely audited and trusted

## Security Architecture

### Encryption Flow

```
User Clipboard Content
        ↓
Generate Random IV (Initialization Vector)
        ↓
Derive Key from User Password (Argon2)
        ↓
AES-GCM Encrypt (Content + IV + Key)
        ↓
Store: [IV + Ciphertext + AuthTag]
```

### Key Management

- **Password Storage**: System keyring (macOS Keychain, Windows Credential Manager)
- **Salt**: Stored in `~/.uniclipboard/salt` (unique per installation)
- **Key Derivation**: Argon2id (memory-hard, resistant to GPU attacks)
- **No Plaintext**: Clipboard content never stored unencrypted

### Network Security

- **LAN Sync**: TLS-encrypted WebSocket (libp2p with noise protocol)
- **Remote Sync**: HTTPS for WebDAV connections
- **Device Authentication**: Peer ID fingerprint verification

## Performance Considerations

### Clipboard History Limits

- **Default**: 1000 entries per device
- **Configurable**: Via settings (trade-off: disk space vs history)
- **Pruning**: Automatic cleanup when limit exceeded (FIFO)

### Blob Storage

Large clipboard items (images, rich text) stored separately:

- **Inline**: Text content < 10KB stored in database
- **Blob**: Large content stored in `~/.uniclipboard/blobs/`
- **Reference**: Database stores blob hash (SHA-256)

### Network Optimization

- **Deduplication**: Identical content sent once per session
- **Compression**: Large blobs compressed before sync
- **Batching**: Multiple clipboard changes batched in single network call

## Testing Strategy

### Unit Tests

- **Domain models**: Test business rules in isolation
- **Use cases**: Test application logic with mock ports
- **Repository mappers**: Test entity ↔ domain conversion

### Integration Tests

- **Bootstrap wiring**: Verify dependency injection works
- **Database migrations**: Test schema changes
- **End-to-end**: Full clipboard sync flow (hardware tests)

### Test Commands

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p uc-core
cargo test -p uc-app

# Run integration tests
cargo test --test '*_integration_test' -- --ignored

# Run with logging
RUST_LOG=debug cargo test --workspace
```

## Further Reading

- [Architecture Principles](architecture/principles.md) - Deep dive into Hexagonal Architecture
- [Bootstrap System](architecture/bootstrap.md) - How dependency injection works
- [Module Boundaries](architecture/module-boundaries.md) - What each module can/cannot do
- [Error Handling](guides/error-handling.md) - Error handling strategy
- [DeepWiki](https://deepwiki.com/mkdir700/uniclipboard-desktop) - Interactive diagrams
