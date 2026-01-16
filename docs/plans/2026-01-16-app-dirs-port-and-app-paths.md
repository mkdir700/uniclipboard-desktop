# AppDirsPort + AppPaths (Hexagonal) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Centralize OS app directory discovery behind a single `dirs`-based adapter in `uc-platform`, expose only a base directory fact via `uc-core` `AppDirsPort`, and derive concrete infrastructure paths (`AppPaths`) in `uc-app` (not in `uc-core`) to avoid leaking infra concepts into the core.

**Architecture:**

- `uc-core`: defines `AppDirs` + `AppDirsPort` + `AppDirsError`. No `dirs`, no Tauri, no DB/vault/settings concepts.
- `uc-platform`: implements `AppDirsPort` using `dirs` (the only place `dirs::*` is allowed).
- `uc-app`: defines `AppPaths` derived from `AppDirs` (application-level infra layout decision).
- `uc-tauri`: composition root; calls `AppDirsPort` once during bootstrap/wiring and injects `AppPaths` into infra/platform adapters.

**Tech Stack:** Rust, crates `uc-core`/`uc-platform`/`uc-app`/`uc-tauri`, `dirs` crate.

---

### Task 1: Add `AppDirs` DTO and `AppDirsPort` in `uc-core`

**Files:**

- Create: `src-tauri/crates/uc-core/src/app_dirs/mod.rs`
- Create: `src-tauri/crates/uc-core/src/ports/app_dirs.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/errors.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`

**Step 1: Write the failing test**

```rust
// src-tauri/crates/uc-core/src/app_dirs/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn app_dirs_is_pure_fact_container() {
        let dirs = AppDirs {
            app_data_root: PathBuf::from("/tmp/uniclipboard"),
        };
        assert!(dirs.app_data_root.ends_with("uniclipboard"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-core app_dirs_is_pure_fact_container`

Expected: FAIL (missing `AppDirs`).

**Step 3: Write minimal implementation**

```rust
// src-tauri/crates/uc-core/src/app_dirs/mod.rs
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDirs {
    pub app_data_root: PathBuf,
}
```

```rust
// src-tauri/crates/uc-core/src/ports/app_dirs.rs
use crate::app_dirs::AppDirs;
use crate::ports::errors::AppDirsError;

pub trait AppDirsPort: Send + Sync {
    fn get_app_dirs(&self) -> Result<AppDirs, AppDirsError>;
}
```

```rust
// src-tauri/crates/uc-core/src/ports/errors.rs
#[derive(Debug, thiserror::Error)]
pub enum AppDirsError {
    #[error("system data-local directory unavailable")]
    DataLocalDirUnavailable,

    #[error("platform error: {0}")]
    Platform(String),
}
```

Export in `src-tauri/crates/uc-core/src/ports/mod.rs`.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-core app_dirs_is_pure_fact_container`

Expected: PASS.

**Step 5: Commit (optional)**

```bash
git add src-tauri/crates/uc-core/src/app_dirs/mod.rs \
       src-tauri/crates/uc-core/src/ports/app_dirs.rs \
       src-tauri/crates/uc-core/src/ports/errors.rs \
       src-tauri/crates/uc-core/src/ports/mod.rs
git commit -m "feat(core): add AppDirsPort for app data root"
```

---

### Task 2: Implement `AppDirsPort` in `uc-platform` using `dirs` (single source)

**Files:**

- Create: `src-tauri/crates/uc-platform/src/app_dirs.rs`
- Modify: `src-tauri/crates/uc-platform/src/lib.rs`
- Test: `src-tauri/crates/uc-platform/src/app_dirs.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::ports::AppDirsPort;

    #[test]
    fn adapter_appends_uniclipboard_dir_name() {
        let adapter = DirsAppDirsAdapter::with_base_data_local_dir(std::path::PathBuf::from("/tmp"));
        let dirs = adapter.get_app_dirs().unwrap();
        assert_eq!(dirs.app_data_root, std::path::PathBuf::from("/tmp/uniclipboard"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-platform adapter_appends_uniclipboard_dir_name`

Expected: FAIL (missing adapter).

**Step 3: Write minimal implementation**

```rust
use std::path::PathBuf;

use uc_core::{
    app_dirs::AppDirs,
    ports::{AppDirsError, AppDirsPort},
};

const APP_DIR_NAME: &str = "uniclipboard";

pub struct DirsAppDirsAdapter {
    base_data_local_dir_override: Option<PathBuf>,
}

impl DirsAppDirsAdapter {
    pub fn new() -> Self {
        Self {
            base_data_local_dir_override: None,
        }
    }

    #[cfg(test)]
    pub fn with_base_data_local_dir(base: PathBuf) -> Self {
        Self {
            base_data_local_dir_override: Some(base),
        }
    }

    fn base_data_local_dir(&self) -> Option<PathBuf> {
        if let Some(base) = &self.base_data_local_dir_override {
            return Some(base.clone());
        }
        dirs::data_local_dir()
    }
}

impl AppDirsPort for DirsAppDirsAdapter {
    fn get_app_dirs(&self) -> Result<AppDirs, AppDirsError> {
        let base = self
            .base_data_local_dir()
            .ok_or(AppDirsError::DataLocalDirUnavailable)?;

        Ok(AppDirs {
            app_data_root: base.join(APP_DIR_NAME),
        })
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-platform adapter_appends_uniclipboard_dir_name`

Expected: PASS.

**Step 5: Commit (optional)**

```bash
git add src-tauri/crates/uc-platform/src/app_dirs.rs \
       src-tauri/crates/uc-platform/src/lib.rs
git commit -m "feat(platform): implement AppDirsPort via dirs"
```

---

### Task 3: Add `AppPaths` in `uc-app` derived from `AppDirs` (infra layout decision)

**Files:**

- Create: `src-tauri/crates/uc-app/src/app_paths.rs`
- Modify: `src-tauri/crates/uc-app/src/lib.rs`
- Test: `src-tauri/crates/uc-app/src/app_paths.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uc_core::app_dirs::AppDirs;

    #[test]
    fn app_paths_derives_concrete_locations_from_app_data_root() {
        let dirs = AppDirs {
            app_data_root: PathBuf::from("/tmp/uniclipboard"),
        };

        let paths = AppPaths::from_app_dirs(&dirs);

        assert_eq!(paths.db_path, PathBuf::from("/tmp/uniclipboard/uniclipboard.db"));
        assert_eq!(paths.vault_dir, PathBuf::from("/tmp/uniclipboard/vault"));
        assert_eq!(paths.settings_path, PathBuf::from("/tmp/uniclipboard/settings.json"));
        assert_eq!(paths.keyring_dir, PathBuf::from("/tmp/uniclipboard/keyring"));
        assert_eq!(paths.logs_dir, PathBuf::from("/tmp/uniclipboard/logs"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-app app_paths_derives_concrete_locations_from_app_data_root`

Expected: FAIL (missing `AppPaths`).

**Step 3: Write minimal implementation**

```rust
use std::path::PathBuf;

use uc_core::app_dirs::AppDirs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub db_path: PathBuf,
    pub vault_dir: PathBuf,
    pub settings_path: PathBuf,
    pub keyring_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    pub fn from_app_dirs(dirs: &AppDirs) -> Self {
        Self {
            db_path: dirs.app_data_root.join("uniclipboard.db"),
            vault_dir: dirs.app_data_root.join("vault"),
            settings_path: dirs.app_data_root.join("settings.json"),
            keyring_dir: dirs.app_data_root.join("keyring"),
            logs_dir: dirs.app_data_root.join("logs"),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test -p uc-app app_paths_derives_concrete_locations_from_app_data_root`

Expected: PASS.

**Step 5: Commit (optional)**

```bash
git add src-tauri/crates/uc-app/src/app_paths.rs \
       src-tauri/crates/uc-app/src/lib.rs
git commit -m "feat(app): add AppPaths derived from AppDirs"
```

---

### Task 4: Refactor `uc-tauri` wiring to use `AppDirsPort` + `AppPaths`

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Modify: any constructors in `uc-infra`/`uc-platform` that currently call `dirs::*`

**Step 1: Write failing test**

Add a unit test in `wiring.rs` that fails until wiring stops using internal `dirs` helpers and uses the adapter output:

```rust
#[test]
fn wiring_derives_paths_from_port_fact() {
    let dirs = uc_core::app_dirs::AppDirs {
        app_data_root: std::path::PathBuf::from("/tmp/uniclipboard"),
    };
    let paths = uc_app::app_paths::AppPaths::from_app_dirs(&dirs);
    assert!(paths.db_path.ends_with("uniclipboard.db"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test -p uc-tauri wiring_derives_paths_from_port_fact`

Expected: FAIL until imports/modules exist and wiring is updated.

**Step 3: Implement minimal wiring change**

- Instantiate `uc_platform::app_dirs::DirsAppDirsAdapter` in wiring.
- Call `get_app_dirs()` once.
- Create `AppPaths::from_app_dirs(&app_dirs)`.
- Use `AppPaths` values for DB/vault/settings.
- Update keyring/blob-store creation to accept injected paths instead of calling `dirs`.

**Step 4: Run tests**

Run: `cd src-tauri && cargo test -p uc-tauri`

Expected: PASS.

**Step 5: Commit (optional)**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "refactor(tauri): wire infra from AppDirsPort + AppPaths"
```

---

### Task 5: Remove all remaining direct `dirs::*` calls outside `uc-platform` adapter

**Files:**

- Modify: any Rust file still calling `dirs::...` (excluding `uc-platform/src/app_dirs.rs`)

**Step 1: Find occurrences**

Run: `cd src-tauri && rg -n "dirs::(data_local_dir|config_dir|data_dir|data_local_dir)" crates src`

Expected: matches only in `crates/uc-platform/src/app_dirs.rs`.

**Step 2: Replace with injection**

For each match, change constructor signature to accept `AppDirs` or `AppPaths` and pass it from wiring.

**Step 3: Run full tests**

Run: `cd src-tauri && cargo test`

Expected: PASS.

---

### Task 6: Keep Tauri `identifier` consistent (guardrail)

**Files:**

- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/crates/uc-tauri/tests/identifier_consistency.rs`

**Step 1: Write failing test**

Test reads `src-tauri/tauri.conf.json` and asserts `identifier == "uniclipboard"`.

**Step 2: Run to verify red**

Run: `cd src-tauri && cargo test -p uc-tauri identifier_consistency`

**Step 3: Update config and re-run**

Set `"identifier": "uniclipboard"` then re-run test; expect PASS.

---

## Final verification

Run:

- `cd src-tauri && cargo test`

Expected: all tests pass.
