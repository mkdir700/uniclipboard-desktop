# Keychain Unification + Unlock-Gated Access Design

## Goal

- Use a single secure storage port for both KEK and libp2p identity.
- Avoid any Keychain reads before the user explicitly unlocks (manual or auto-unlock success).
- When auto-unlock fails, show UnlockPage and keep network stopped.

## Non-Goals

- Change cryptographic formats or key material contents.
- Modify frontend unlock UI/UX.

## Current Problem

- `Libp2pNetworkAdapter::new` triggers `load_or_create_identity` during wiring.
- This loads `libp2p-identity:v1` from Keychain before UnlockPage, causing macOS prompts.
- KEK uses `KeyringPort` while identity uses `IdentityStorePort`, duplicating keychain logic.

## Architecture Changes

### New Port

Add `SecureStoragePort` in `uc-core`:

```rust
pub trait SecureStoragePort: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, SecureStorageError>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), SecureStorageError>;
    fn delete(&self, key: &str) -> Result<(), SecureStorageError>;
}
```

### Platform Implementations

- `SystemSecureStorage` (Keychain).
- `FileSecureStorage` (file fallback, reusing file keyring logic).
- Both share `SERVICE_NAME = "UniClipboard"` and key naming conventions:
  - `libp2p-identity:v1`
  - `kek:v1:profile:<id>`

### Service Wiring

- `wire_dependencies` creates exactly one `secure_storage` instance.
- Inject into:
  - `DefaultKeyMaterialService` (replaces `KeyringPort`).
  - `SystemIdentityStore` (replaces direct keyring access).

## Unlock-Gated Data Flow

1. Startup wiring builds dependencies without any Keychain reads.
2. Auto-unlock runs (if enabled). If it fails, do not start network.
3. Unlock success triggers `start_network_after_unlock`.
4. Only then: `load_or_create_identity` reads `libp2p-identity:v1` from Keychain.

## Key Call-Site Changes

- `Libp2pNetworkAdapter::new` no longer loads identity.
- Identity loading moves to unlock-success path.
- `unlock_encryption_session` command triggers network start on success.

## Error Handling

- If secure storage access fails during unlock: propagate error and remain locked.
- If identity load fails after unlock: return error and keep network stopped.

## Testing Strategy

- Unit tests for `SecureStoragePort` implementations.
- `DefaultKeyMaterialService` uses secure storage (mocked) for KEK.
- `SystemIdentityStore` uses secure storage (mocked) for identity.
- Startup path test: when `auto_unlock_enabled = false`, no keychain reads.
- Unlock path test: keychain reads occur only after unlock success.

## Risks

- Moving identity load out of `Libp2pNetworkAdapter::new` requires careful handling in code that expects `local_peer_id` immediately.
- Ensure network start waits for identity availability to avoid invalid PeerId usage.

## Rollout Plan

- Implement port + platform adapters.
- Move identity read to unlock-success path.
- Update wiring and use cases.
- Run targeted tests in `uc-platform`, `uc-infra`, `uc-tauri`.
