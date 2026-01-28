# Pairing Timers and Config Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make pairing timers and retries configurable via settings, remove hardcoded timeouts, and keep defaults consistent across settings, policy, and orchestrator.

**Architecture:** Add a `PairingSettings` block to `uc-core` settings schema and defaults, migrate settings to a new schema version, and map settings to `PairingConfig` in `uc-app`/`uc-tauri`. The orchestrator reads config (built from settings via the Settings port) and uses it for timer policies and session cleanup, with logging on load failures.

**Tech Stack:** Rust (Tokio, Tauri), uc-core settings model/migrations, uc-app pairing orchestrator, uc-tauri bootstrap wiring.

---

### Task 1: Add pairing settings to core schema and defaults

**Files:**

- Modify: `src-tauri/crates/uc-core/src/settings/model.rs`
- Modify: `src-tauri/crates/uc-core/src/settings/defaults.rs`
- Modify: `src-tauri/crates/uc-core/src/settings/model.rs` (tests)

**Step 1: Write the failing test**

```rust
#[test]
fn test_pairing_settings_defaults_when_missing() {
    let value = serde_json::json!({
        "schema_version": 1,
        "general": {},
        "sync": {},
        "retention_policy": {},
        "security": {}
    });

    let settings: Settings = serde_json::from_value(value).expect("deserialize settings");
    assert_eq!(settings.pairing.step_timeout.as_secs(), 15);
    assert_eq!(settings.pairing.user_verification_timeout.as_secs(), 120);
    assert_eq!(settings.pairing.session_timeout.as_secs(), 300);
    assert_eq!(settings.pairing.max_retries, 3);
    assert_eq!(settings.pairing.protocol_version, "1.0.0");
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-core test_pairing_settings_defaults_when_missing`
Expected: FAIL (missing `pairing` field and defaults)

**Step 3: Write minimal implementation**

Add `PairingSettings` to `Settings`:

```rust
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingSettings {
    #[serde_as(as = "DurationSeconds<u64>")]
    pub step_timeout: Duration,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub user_verification_timeout: Duration,
    #[serde_as(as = "DurationSeconds<u64>")]
    pub session_timeout: Duration,
    pub max_retries: u8,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub sync: SyncSettings,
    #[serde(default)]
    pub retention_policy: RetentionPolicy,
    #[serde(default)]
    pub security: SecuritySettings,
    #[serde(default)]
    pub pairing: PairingSettings,
}
```

Add defaults:

```rust
impl Default for PairingSettings {
    fn default() -> Self {
        Self {
            step_timeout: Duration::from_secs(15),
            user_verification_timeout: Duration::from_secs(120),
            session_timeout: Duration::from_secs(300),
            max_retries: 3,
            protocol_version: "1.0.0".to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralSettings::default(),
            sync: SyncSettings::default(),
            retention_policy: RetentionPolicy::default(),
            security: SecuritySettings::default(),
            pairing: PairingSettings::default(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-core test_pairing_settings_defaults_when_missing`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/settings/model.rs src-tauri/crates/uc-core/src/settings/defaults.rs
git commit -m "feat: add pairing settings to core schema"
```

---

### Task 2: Bump schema version and add migration

**Files:**

- Modify: `src-tauri/crates/uc-core/src/settings/model.rs`
- Modify: `src-tauri/crates/uc-infra/src/settings/migration.rs`
- Test: `src-tauri/crates/uc-infra/src/settings/migration.rs` (new test)

**Step 1: Write the failing test**

```rust
#[test]
fn test_migrates_v1_to_v2_adds_pairing_defaults() {
    let settings = Settings {
        schema_version: 1,
        general: GeneralSettings::default(),
        sync: SyncSettings::default(),
        retention_policy: RetentionPolicy::default(),
        security: SecuritySettings::default(),
        pairing: PairingSettings::default(),
    };

    let migrator = SettingsMigrator::new();
    let migrated = migrator.migrate_to_latest(settings).expect("migrate settings");

    assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(migrated.pairing.session_timeout.as_secs(), 300);
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-infra test_migrates_v1_to_v2_adds_pairing_defaults`
Expected: FAIL (no migration from v1)

**Step 3: Write minimal implementation**

- Bump `CURRENT_SCHEMA_VERSION` to `2`.
- Add `MigrationV1ToV2` implementing `SettingsMigrationPort` and register it in `SettingsMigrator::new()`.
- In migration, set `schema_version = 2` and ensure `pairing` is present (use `PairingSettings::default()` if missing).

```rust
pub struct MigrationV1ToV2;

impl SettingsMigrationPort for MigrationV1ToV2 {
    fn from_version(&self) -> u32 { 1 }
    fn to_version(&self) -> u32 { 2 }
    fn migrate(&self, mut settings: Settings) -> Settings {
        settings.schema_version = 2;
        settings.pairing = PairingSettings::default();
        settings
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-infra test_migrates_v1_to_v2_adds_pairing_defaults`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/settings/model.rs src-tauri/crates/uc-infra/src/settings/migration.rs
git commit -m "feat: migrate settings schema to include pairing config"
```

---

### Task 3: Map settings to PairingConfig and remove hardcoded timers

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs`
- Modify: `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs` (new test)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn test_cleanup_uses_configured_session_timeout() {
    let config = PairingConfig {
        step_timeout_secs: 1,
        user_verification_timeout_secs: 1,
        max_retries: 1,
        protocol_version: "1.0.0".to_string(),
        session_timeout_secs: 1,
    };
    let (orchestrator, _rx) = PairingOrchestrator::new(
        config,
        Arc::new(FakePairedDeviceRepo::default()),
        "local".to_string(),
        "device-1".to_string(),
        "peer-1".to_string(),
        vec![1; 32],
    );

    orchestrator.initiate_pairing("peer-2".to_string()).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    orchestrator.cleanup_expired_sessions().await;

    let sessions = orchestrator.sessions.read().await;
    assert!(sessions.is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-app test_cleanup_uses_configured_session_timeout`
Expected: FAIL (hardcoded 300s timeout)

**Step 3: Write minimal implementation**

- Add `session_timeout_secs` to `PairingConfig` and its default.
- Update `cleanup_expired_sessions` to use `self.config.session_timeout_secs`.
- In `PairingPolicy::default`, derive defaults from `PairingSettings::default()` to keep values consistent.
- Add `PairingConfig::from_settings(settings: &Settings) -> PairingConfig` with validation/clamping (no zero/negative values).
- Add `resolve_pairing_config(settings: Arc<dyn SettingsPort>) -> PairingConfig` in `uc-tauri` that logs and falls back to defaults if settings load fails.
- In `src-tauri/src/main.rs` (or `wire_dependencies` path), use `resolve_pairing_config` instead of `PairingConfig::default()`.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-app test_cleanup_uses_configured_session_timeout`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs src-tauri/crates/uc-core/src/network/pairing_state_machine.rs src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs src-tauri/src/main.rs
git commit -m "feat: load pairing timers from settings"
```

---

### Task 4: Add settings-to-config mapping test

**Files:**

- Test: `src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs` (new test)

**Step 1: Write the failing test**

```rust
#[test]
fn test_pairing_config_from_settings() {
    let mut settings = Settings::default();
    settings.pairing.step_timeout = std::time::Duration::from_secs(20);
    settings.pairing.user_verification_timeout = std::time::Duration::from_secs(90);
    settings.pairing.session_timeout = std::time::Duration::from_secs(400);
    settings.pairing.max_retries = 5;
    settings.pairing.protocol_version = "2.0.0".to_string();

    let config = PairingConfig::from_settings(&settings);

    assert_eq!(config.step_timeout_secs, 20);
    assert_eq!(config.user_verification_timeout_secs, 90);
    assert_eq!(config.session_timeout_secs, 400);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.protocol_version, "2.0.0");
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-app test_pairing_config_from_settings`
Expected: FAIL (missing mapping)

**Step 3: Write minimal implementation**

```rust
impl PairingConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        let pairing = &settings.pairing;
        let step = pairing.step_timeout.as_secs().min(i64::MAX as u64) as i64;
        let verify = pairing.user_verification_timeout.as_secs().min(i64::MAX as u64) as i64;
        let session = pairing.session_timeout.as_secs().min(i64::MAX as u64) as i64;

        Self {
            step_timeout_secs: step.max(1),
            user_verification_timeout_secs: verify.max(1),
            session_timeout_secs: session.max(1),
            max_retries: pairing.max_retries.max(1),
            protocol_version: pairing.protocol_version.clone(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-app test_pairing_config_from_settings`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/pairing/orchestrator.rs
git commit -m "test: map pairing settings to config"
```

---

### Task 5: Update documentation for new settings keys

**Files:**

- Modify: `docs/guides/settings.md` (or the existing settings guide for persisted config)

**Step 1: Write the failing test**

Skip tests; documentation-only change.

**Step 2: Update documentation**

Add a `pairing` block example showing timer values in seconds and defaults:

```json
"pairing": {
  "step_timeout": 15,
  "user_verification_timeout": 120,
  "session_timeout": 300,
  "max_retries": 3,
  "protocol_version": "1.0.0"
}
```

**Step 3: Commit**

```bash
git add docs/guides/settings.md
git commit -m "docs: document pairing settings"
```

---

Plan complete and saved to `docs/plans/2026-01-26-pairing-timers-and-config.md`. Two execution options:

1. Subagent-Driven (this session) - I dispatch fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
