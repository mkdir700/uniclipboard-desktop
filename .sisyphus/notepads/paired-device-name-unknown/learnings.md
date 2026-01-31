# Task 1 Learnings: Extend PairedDevice with device_name

## 2026-01-31 Task 1 Completion

### Changes Made

1. **paired_device.rs**: Added `pub device_name: String` field to `PairedDevice` struct
2. **paired_device.rs**: Updated `test_paired_device_serialization` to include device_name
3. **pairing_state_machine.rs**: Modified `build_paired_device()` to extract device_name from `context.peer_device_name` with fallback to "Unknown Device"
4. **pairing_state_machine.rs**: Added two TDD tests:
   - `build_paired_device_includes_peer_device_name`: Verifies device_name is populated from context
   - `build_paired_device_uses_unknown_device_when_name_missing`: Verifies fallback to "Unknown Device"
5. **resolve_connection_policy.rs**: Updated MockRepo to include device_name in test PairedDevice

### Key Implementation Details

- Used `unwrap_or_else(|| "Unknown Device".to_string())` for safe fallback
- All test constructors updated to include the new required field
- TDD approach: tests verify both happy path and fallback scenario

### Verification

- `cargo test -p uc-core` passes (72 tests)
- No LSP errors in modified files

### Next Steps

- Task 2: Add DB migration and update schema/mapper/repo

## 2026-01-31 Verification of Task 1

Verified the implementation of Task 1:

- **PairedDevice**: Confirmed `device_name: String` field exists in `src-tauri/crates/uc-core/src/network/paired_device.rs`.
- **PairingStateMachine**: Confirmed `build_paired_device` in `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs` populates `device_name` from `peer_device_name` with "Unknown Device" fallback.
- **TDD Tests**: Confirmed `build_paired_device_includes_peer_device_name` and `build_paired_device_uses_unknown_device_when_name_missing` exist and cover the logic.
- **App Layer**: Confirmed `src-tauri/crates/uc-app/src/usecases/pairing/resolve_connection_policy.rs` updated its MockRepo.
- **Consistency**: `protocol.rs` already supports `device_name` in pairing messages, ensuring the context is correctly populated during the handshake.

## Task 2: Persist device_name in DB layer (Patterns & Evidence)

### 1. Migration Pattern (SQLite)

Based on the latest migration `2026-02-02-000001_add_entry_active_time`, the project uses a simple `ALTER TABLE` approach for adding columns.

**Evidence (up.sql):**

```sql
ALTER TABLE paired_device
ADD COLUMN device_name TEXT NOT NULL DEFAULT 'Unknown';
```

**Evidence (down.sql):**
Note: SQLite 3.35.0+ supports `DROP COLUMN`. If the environment supports it:

```sql
ALTER TABLE paired_device DROP COLUMN device_name;
```

### 2. Model Update Pattern

Update `PairedDeviceRow` and `NewPairedDeviceRow` in `src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs`.

**Pattern:**

```rust
pub struct PairedDeviceRow {
    pub peer_id: String,
    pub pairing_state: String,
    pub identity_fingerprint: String,
    pub paired_at: i64,
    pub last_seen_at: Option<i64>,
    pub device_name: String, // Added
}
```

### 3. Mapper Pattern

Update `PairedDeviceRowMapper` in `src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs`.

**Pattern (to_row):**

```rust
Ok(NewPairedDeviceRow {
    // ... existing fields
    device_name: domain.device_name.clone(),
})
```

**Pattern (to_domain):**

```rust
Ok(PairedDevice {
    // ... existing fields
    device_name: row.device_name.clone(),
})
```

### 4. Repository Upsert Pattern

Update `DieselPairedDeviceRepository::upsert` in `src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs`.

**Pattern:**

```rust
diesel::insert_into(paired_device)
    .values(&row)
    .on_conflict(peer_id)
    .do_update()
    .set((
        pairing_state.eq(row.pairing_state.clone()),
        identity_fingerprint.eq(row.identity_fingerprint.clone()),
        paired_at.eq(row.paired_at),
        last_seen_at.eq(row.last_seen_at),
        device_name.eq(row.device_name.clone()), // Added to upsert set
    ))
```

### 5. TDD Verification Pattern

Update `test_paired_device_persistence` in `paired_device_repo.rs`.

**Pattern:**

```rust
let device = PairedDevice {
    // ...
    device_name: "Test Device".to_string(),
};
// ... upsert and reload ...
assert_eq!(loaded.unwrap().device_name, "Test Device");
```

### External References & Evidence

- **Diesel Migration Guide**: [https://diesel.rs/guides/migration_guide.html](https://diesel.rs/guides/migration_guide.html)
- **Diesel Upsert Documentation**: [https://docs.diesel.rs/2.2.x/diesel/query_builder/trait.InsertStatement.html#method.on_conflict](https://docs.diesel.rs/2.2.x/diesel/query_builder/trait.InsertStatement.html#method.on_conflict)
- **SQLite ALTER TABLE ADD COLUMN**: [https://www.sqlite.org/lang_altertable.html](https://www.sqlite.org/lang_altertable.html)

## Task 2: Persist device_name in DB layer

### Diesel Migration Guidance

- **SQL Pattern**: Use `ALTER TABLE ... ADD COLUMN ... NOT NULL DEFAULT ...` to ensure compatibility with existing data and SQLite constraints.
- **Up Migration (`up.sql`)**:
  ```sql
  ALTER TABLE paired_device ADD COLUMN device_name TEXT NOT NULL DEFAULT 'Unknown';
  ```
- **Down Migration (`down.sql`)**:
  ```sql
  ALTER TABLE paired_device DROP COLUMN device_name;
  ```
  _Note: SQLite 3.35.0+ supports DROP COLUMN. For older versions, table recreation is required._

### Implementation Steps

1. **Schema**: Update `paired_device` table definition in `src-tauri/crates/uc-infra/src/db/schema.rs` to include `device_name -> Text`.
2. **Models**:
   - Update `PairedDeviceRow` and `NewPairedDeviceRow` in `src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs` with `pub device_name: String`.
3. **Mapper**:
   - Update `PairedDeviceRowMapper` in `src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs`.
   - `to_row`: Map `domain.device_name` to `row.device_name`.
   - `to_domain`: Map `row.device_name` to `domain.device_name`.
4. **Repository**:
   - Update `DieselPairedDeviceRepository::upsert` in `src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs`.
   - Add `device_name.eq(row.device_name.clone())` to the `do_update().set(...)` clause.

### TDD Verification

- Update `test_paired_device_persistence` in `paired_device_repo.rs` to include a non-default `device_name` and assert its value after loading.

### Evidence & References

- **SQLite ALTER TABLE**: [Official Docs](https://www.sqlite.org/lang_altertable.html) - "If a NOT NULL constraint is specified, then the column must have a default value other than NULL."
- **Diesel Migrations**: [Diesel CLI Guide](https://github.com/diesel-rs/diesel/blob/main/diesel_cli/README.md).

## Task 2 Completion Report

### Status

- **Completed**: Yes
- **Verification**: `cargo test -p uc-infra` passed.

### Changes

1. **Migration**: Created `2026-02-03-000001_add_paired_device_name` with `ALTER TABLE paired_device ADD COLUMN device_name TEXT NOT NULL DEFAULT 'Unknown Device';`.
2. **Schema**: Updated `src-tauri/crates/uc-infra/src/db/schema.rs` to include `device_name`.
3. **Model**: Updated `PairedDeviceRow` and `NewPairedDeviceRow` in `src-tauri/crates/uc-infra/src/db/models/paired_device_row.rs`.
4. **Mapper**: Updated `PairedDeviceRowMapper` in `src-tauri/crates/uc-infra/src/db/mappers/paired_device_mapper.rs` to handle `device_name`.
5. **Repository**: Updated `DieselPairedDeviceRepository::upsert` in `src-tauri/crates/uc-infra/src/db/repositories/paired_device_repo.rs` to persist `device_name`.
6. **Tests**: Updated `test_paired_device_persistence` to verify `device_name` round-trip.

### Observations

- The `uc-core` `PairedDevice` struct already had `device_name` (from Task 1), which caused compilation errors in `uc-infra` until the changes were applied. This served as a strong "Red" signal in the TDD cycle.
- The default value 'Unknown Device' in the migration ensures backward compatibility for existing rows.

## Task 3: Return persisted device_name in get_paired_peers_with_status

- Extracted mapping logic into `map_paired_device_to_peer` for better testability.
- Implemented priority for device name: Persisted Name > Discovered Name > "Unknown Device".
- Verified with unit tests in `src-tauri/crates/uc-tauri/src/commands/pairing.rs`.
- Fixed a type mismatch in `uc-tauri` bootstrap tests where `Wry` was used instead of `MockRuntime`.

## Task 3 Completion Report

### Status

- **Completed**: Yes
- **Verification**: `cargo test -p uc-tauri` passed.

### Changes

1. **Mapping Logic**: Extracted `map_paired_device_to_peer` in `src-tauri/crates/uc-tauri/src/commands/pairing.rs` to handle device name resolution.
2. **Priority Resolution**: Implemented name resolution priority:
   - 1. Persisted name from `PairedDevice` (if not "Unknown Device")
   - 2. Discovered name from `DiscoveredPeer` (if available)
   - 3. Fallback to "Unknown Device"
3. **Test Fix**: Fixed a compilation error in `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` where tests were using `tauri::Wry` instead of `tauri::test::MockRuntime` for `run_pairing_event_loop`.
4. **Unit Tests**: Added tests in `commands::pairing` to verify the name resolution logic:
   - `test_map_paired_device_to_peer_uses_persisted_name`
   - `test_map_paired_device_to_peer_falls_back_to_discovered_name`
   - `test_map_paired_device_to_peer_falls_back_to_unknown_device`

### Observations

- The `Wry` vs `MockRuntime` mismatch was likely caused by `tauri::test::mock_app()` returning an `AppHandle<MockRuntime>`, while the generic parameter was explicitly set to `Wry`.
- Using `tauri::test::MockRuntime` in tests ensures compatibility with the mock app handle and avoids unnecessary dependencies on the real webview runtime during testing.

## Framing Logging

- Added structured logging to `uc-platform/src/adapters/pairing_stream/framing/mod.rs`.
- Logs `stage` (read_len_prefix, read_payload, etc.) and `len`/`expected`.
- Handles `UnexpectedEof` specifically to aid in debugging stream closures.
