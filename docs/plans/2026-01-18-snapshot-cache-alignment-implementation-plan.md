# Snapshot Cache Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Align cache/spool/worker pipeline with the converged design: bounded resources, correct state transitions, no silent failures, and cache-dir spool placement.

**Architecture:** Introduce cache-dir paths, extend storage config, make cache and spool bounded, add typed CAS outcomes, and run janitor recovery. Capture stays non-blocking, worker writes blobs only, resolver stays read-only.

**Tech Stack:** Rust, tokio, Diesel, tracing, Tauri bootstrap

---

### Task 1: Add app_cache_root to AppDirs

**Files:**

- Modify: `src-tauri/crates/uc-core/src/app_dirs/mod.rs`
- Modify: `src-tauri/crates/uc-platform/src/app_dirs.rs`
- Test: `src-tauri/crates/uc-core/src/app_dirs/mod.rs`
- Test: `src-tauri/crates/uc-platform/src/app_dirs.rs`

**Step 1: Write the failing test**

Add to `src-tauri/crates/uc-core/src/app_dirs/mod.rs`:

```rust
#[test]
fn app_dirs_includes_cache_root() {
    let dirs = AppDirs {
        app_data_root: PathBuf::from("/tmp/uniclipboard"),
        app_cache_root: PathBuf::from("/tmp/uniclipboard-cache"),
    };
    assert!(dirs.app_cache_root.ends_with("uniclipboard-cache"));
}
```

Add to `src-tauri/crates/uc-platform/src/app_dirs.rs`:

```rust
#[test]
fn adapter_sets_cache_root() {
    let adapter = DirsAppDirsAdapter::with_base_data_local_dir(PathBuf::from("/tmp"));
    let dirs = adapter.get_app_dirs().unwrap();
    assert!(dirs.app_cache_root.ends_with("uniclipboard"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test app_dirs_includes_cache_root adapter_sets_cache_root`

Expected: FAIL with missing field `app_cache_root`.

**Step 3: Write minimal implementation**

Update `src-tauri/crates/uc-core/src/app_dirs/mod.rs`:

```rust
pub struct AppDirs {
    pub app_data_root: PathBuf,
    pub app_cache_root: PathBuf,
}
```

Update `src-tauri/crates/uc-platform/src/app_dirs.rs`:

```rust
fn base_cache_dir(&self) -> Option<PathBuf> {
    if let Some(base) = &self.base_data_local_dir_override {
        return Some(base.clone());
    }
    dirs::cache_dir()
}

fn get_app_dirs(&self) -> Result<AppDirs, AppDirsError> {
    let base_data = self
        .base_data_local_dir()
        .ok_or(AppDirsError::DataLocalDirUnavailable)?;
    let base_cache = self
        .base_cache_dir()
        .ok_or(AppDirsError::CacheDirUnavailable)?;

    Ok(AppDirs {
        app_data_root: base_data.join(APP_DIR_NAME),
        app_cache_root: base_cache.join(APP_DIR_NAME),
    })
}
```

Add `CacheDirUnavailable` to `src-tauri/crates/uc-core/src/ports/errors.rs`:

```rust
pub enum AppDirsError {
    DataLocalDirUnavailable,
    CacheDirUnavailable,
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test app_dirs_includes_cache_root adapter_sets_cache_root`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/app_dirs/mod.rs
git add src-tauri/crates/uc-core/src/ports/errors.rs
git add src-tauri/crates/uc-platform/src/app_dirs.rs
git commit -m "feat(app-dirs): add cache root"
```

---

### Task 2: Add cache_dir to AppPaths and DefaultPaths

**Files:**

- Modify: `src-tauri/crates/uc-app/src/app_paths.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-app/src/app_paths.rs`
- Test: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`

**Step 1: Write the failing test**

Add to `src-tauri/crates/uc-app/src/app_paths.rs`:

```rust
#[test]
fn app_paths_includes_cache_dir() {
    let dirs = AppDirs {
        app_data_root: PathBuf::from("/tmp/uniclipboard"),
        app_cache_root: PathBuf::from("/tmp/uniclipboard-cache"),
    };
    let paths = AppPaths::from_app_dirs(&dirs);
    assert_eq!(paths.cache_dir, PathBuf::from("/tmp/uniclipboard-cache"));
}
```

Add to `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs` tests:

```rust
#[test]
fn derive_default_paths_sets_cache_dir() {
    let dirs = uc_core::app_dirs::AppDirs {
        app_data_root: PathBuf::from("/tmp/uniclipboard"),
        app_cache_root: PathBuf::from("/tmp/uniclipboard-cache"),
    };
    let paths = derive_default_paths_from_app_dirs(&dirs, &AppConfig::empty()).unwrap();
    assert_eq!(paths.cache_dir, PathBuf::from("/tmp/uniclipboard-cache"));
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test app_paths_includes_cache_dir derive_default_paths_sets_cache_dir`

Expected: FAIL with missing field `cache_dir`.

**Step 3: Write minimal implementation**

Update `src-tauri/crates/uc-app/src/app_paths.rs`:

```rust
pub struct AppPaths {
    pub db_path: PathBuf,
    pub vault_dir: PathBuf,
    pub settings_path: PathBuf,
    pub keyring_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub cache_dir: PathBuf,
}

pub fn from_app_dirs(dirs: &AppDirs) -> Self {
    Self {
        db_path: dirs.app_data_root.join("uniclipboard.db"),
        vault_dir: dirs.app_data_root.join("vault"),
        settings_path: dirs.app_data_root.join("settings.json"),
        keyring_dir: dirs.app_data_root.join("keyring"),
        logs_dir: dirs.app_data_root.join("logs"),
        cache_dir: dirs.app_cache_root.clone(),
    }
}
```

Update `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`:

```rust
struct DefaultPaths {
    app_data_root: PathBuf,
    db_path: PathBuf,
    vault_dir: PathBuf,
    settings_path: PathBuf,
    cache_dir: PathBuf,
}

let base_paths = AppPaths::from_app_dirs(app_dirs);

Ok(DefaultPaths {
    app_data_root: app_dirs.app_data_root.clone(),
    db_path,
    vault_dir,
    settings_path,
    cache_dir: base_paths.cache_dir,
})
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test app_paths_includes_cache_dir derive_default_paths_sets_cache_dir`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/app_paths.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(paths): add cache dir"
```

---

### Task 3: Expand ClipboardStorageConfig and wiring usage

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/config/clipboard_storage_config.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-infra/src/config/clipboard_storage_config.rs`
- Update tests using ClipboardStorageConfig in:
  - `src-tauri/crates/uc-infra/src/clipboard/normalizer.rs`
  - `src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs`
  - `src-tauri/crates/uc-app/tests/stress_test.rs`

**Step 1: Write the failing test**

Add to `src-tauri/crates/uc-infra/src/config/clipboard_storage_config.rs`:

```rust
#[test]
fn defaults_include_cache_and_spool_limits() {
    let cfg = ClipboardStorageConfig::defaults();
    assert!(cfg.cache_max_entries > 0);
    assert!(cfg.cache_max_bytes > 0);
    assert!(cfg.spool_max_bytes > 0);
    assert!(cfg.spool_ttl_days > 0);
    assert!(cfg.worker_retry_max_attempts > 0);
    assert!(cfg.worker_retry_backoff_ms > 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test defaults_include_cache_and_spool_limits`

Expected: FAIL with missing fields.

**Step 3: Write minimal implementation**

Update `src-tauri/crates/uc-infra/src/config/clipboard_storage_config.rs`:

```rust
pub struct ClipboardStorageConfig {
    pub inline_threshold_bytes: i64,
    pub cache_max_entries: usize,
    pub cache_max_bytes: usize,
    pub spool_max_bytes: usize,
    pub spool_ttl_days: u64,
    pub worker_retry_max_attempts: u32,
    pub worker_retry_backoff_ms: u64,
}

impl ClipboardStorageConfig {
    pub fn defaults() -> Self {
        Self {
            inline_threshold_bytes: 16 * 1024,
            cache_max_entries: 1000,
            cache_max_bytes: 100 * 1024 * 1024,
            spool_max_bytes: 1_000_000_000,
            spool_ttl_days: 7,
            worker_retry_max_attempts: 5,
            worker_retry_backoff_ms: 250,
        }
    }
}
```

Update wiring (`src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`) to use config values:

```rust
let storage_config = Arc::new(ClipboardStorageConfig::defaults());
let representation_cache = Arc::new(RepresentationCache::new(
    storage_config.cache_max_entries,
    storage_config.cache_max_bytes,
));
let spool_dir = paths.cache_dir.join("spool");
let spool_manager = Arc::new(
    SpoolManager::new(spool_dir.clone(), storage_config.spool_max_bytes)
        .map_err(|e| WiringError::BlobStorageInit(format!("Failed to create spool: {}", e)))?,
);
```

Update tests constructing ClipboardStorageConfig to include new fields.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test defaults_include_cache_and_spool_limits`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/config/clipboard_storage_config.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git add src-tauri/crates/uc-infra/src/clipboard/normalizer.rs
git add src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs
git add src-tauri/crates/uc-app/tests/stress_test.rs
git commit -m "feat(config): add cache/spool limits"
```

---

### Task 4: Upgrade RepresentationCache with status + priority eviction

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs`

**Step 1: Write the failing test**

Add tests to `representation_cache.rs`:

```rust
#[tokio::test]
async fn test_evicts_completed_before_pending() {
    let cache = RepresentationCache::new(2, 10_000);
    let a = RepresentationId::new();
    let b = RepresentationId::new();
    let c = RepresentationId::new();

    cache.put(&a, vec![1]).await; // Pending
    cache.put(&b, vec![2]).await; // Pending
    cache.mark_completed(&a).await;

    cache.put(&c, vec![3]).await; // Evict should drop a (Completed) first

    assert_eq!(cache.get(&a).await, None);
    assert_eq!(cache.get(&b).await, Some(vec![2]));
    assert_eq!(cache.get(&c).await, Some(vec![3]));
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_evicts_completed_before_pending`

Expected: FAIL with missing methods.

**Step 3: Write minimal implementation**

Add methods and eviction logic in `representation_cache.rs`:

```rust
pub async fn mark_completed(&self, rep_id: &RepresentationId) {
    let mut inner = self.inner.lock().await;
    if let Some(entry) = inner.entries.get_mut(rep_id) {
        entry.status = CacheEntryStatus::Completed;
    }
}

pub async fn mark_spooling(&self, rep_id: &RepresentationId) {
    let mut inner = self.inner.lock().await;
    if let Some(entry) = inner.entries.get_mut(rep_id) {
        entry.status = CacheEntryStatus::Processing;
    }
}

pub async fn remove(&self, rep_id: &RepresentationId) {
    let mut inner = self.inner.lock().await;
    inner.remove_entry(rep_id);
    inner.queue.retain(|id| id != rep_id);
}
```

Update `evict_if_needed` to prefer Completed:

```rust
fn evict_if_needed(&mut self) {
    while self.entries.len() > self.max_entries || self.current_bytes > self.max_bytes {
        if let Some(evicted_id) = self.pop_oldest_by_status(CacheEntryStatus::Completed) {
            self.remove_entry(&evicted_id);
            continue;
        }
        if let Some(evicted_id) = self.queue.pop_front() {
            self.remove_entry(&evicted_id);
        } else {
            break;
        }
    }
}

fn pop_oldest_by_status(&mut self, status: CacheEntryStatus) -> Option<RepresentationId> {
    let pos = self.queue.iter().position(|id| {
        self.entries
            .get(id)
            .map(|entry| entry.status == status)
            .unwrap_or(false)
    })?;
    self.queue.remove(pos)
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_evicts_completed_before_pending`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/representation_cache.rs
git commit -m "feat(cache): add status-aware eviction"
```

---

### Task 5: Make SpoolManager bounded + TTL cleanup

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/spool_manager.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/spool_manager.rs`

**Step 1: Write the failing test**

Add to `spool_manager.rs`:

```rust
#[tokio::test]
async fn test_spool_evicts_when_over_limit() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let spool = SpoolManager::new(temp_dir.path(), 4)?;

    let rep_a = RepresentationId::new();
    let rep_b = RepresentationId::new();

    spool.write(&rep_a, &[1, 2, 3]).await?;
    spool.write(&rep_b, &[4, 5, 6]).await?; // should evict rep_a

    assert!(spool.read(&rep_a).await?.is_none());
    assert!(spool.read(&rep_b).await?.is_some());
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_spool_evicts_when_over_limit`

Expected: FAIL (no eviction).

**Step 3: Write minimal implementation**

Add helper to list spool entries by mtime and evict oldest until under limit:

```rust
struct SpoolEntryMeta {
    representation_id: RepresentationId,
    file_path: PathBuf,
    size: usize,
    modified_ms: i64,
}

async fn list_entries_by_mtime(&self) -> Result<Vec<SpoolEntryMeta>> {
    let mut entries = Vec::new();
    let mut dir = fs::read_dir(&self.spool_dir).await?;
    while let Some(entry) = dir.next_entry().await? {
        let meta = entry.metadata().await?;
        if !meta.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str() else {
            tracing::warn!("Skipping spool entry with non-utf8 filename");
            continue;
        };
        let size = meta.len() as usize;
        let modified = meta.modified()?;
        let modified_ms = modified
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| anyhow::anyhow!("invalid mtime: {err}"))?
            .as_millis() as i64;
        entries.push(SpoolEntryMeta {
            representation_id: RepresentationId::from(name),
            file_path: entry.path(),
            size,
            modified_ms,
        });
    }
    entries.sort_by_key(|e| e.modified_ms);
    Ok(entries)
}

async fn enforce_limits(&self) -> Result<()> {
    let mut entries = self.list_entries_by_mtime().await?;
    let mut total_bytes = entries.iter().map(|e| e.size).sum::<usize>();

    while total_bytes > self.max_bytes {
        let Some(oldest) = entries.first() else { break; };
        fs::remove_file(&oldest.file_path).await?;
        total_bytes = total_bytes.saturating_sub(oldest.size);
        entries.remove(0);
    }
    Ok(())
}
```

Call `enforce_limits()` at the end of `write()`.

Add TTL enumeration:

```rust
pub async fn list_expired(&self, now_ms: i64, ttl_days: u64) -> Result<Vec<SpoolEntryMeta>> {
    let ttl_ms = (ttl_days as i64) * 24 * 60 * 60 * 1000;
    let mut expired = Vec::new();
    for entry in self.list_entries_by_mtime().await? {
        if now_ms - entry.modified_ms > ttl_ms {
            expired.push(entry);
        }
    }
    Ok(expired)
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_spool_evicts_when_over_limit`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spool_manager.rs
git commit -m "feat(spool): add size/ttl enforcement"
```

---

### Task 6: Add typed CAS outcome for update_processing_result

**Files:**

- Modify: `src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs`
- Modify: `src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs`
- Modify: `src-tauri/crates/uc-infra/src/security/decrypting_representation_repo.rs`
- Update mocks in tests (stress/integration/spool_scanner)

**Step 1: Write the failing test**

Add to `representation_repo.rs` tests:

```rust
#[tokio::test]
async fn test_update_processing_result_returns_state_mismatch() {
    // Given state is BlobReady, expected_states = [Staged]
    // Should return ProcessingUpdateOutcome::StateMismatch
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_update_processing_result_returns_state_mismatch`

Expected: FAIL (type missing).

**Step 3: Write minimal implementation**

Add to `uc-core` port:

```rust
pub enum ProcessingUpdateOutcome {
    Updated(PersistedClipboardRepresentation),
    StateMismatch,
    NotFound,
}

async fn update_processing_result(
    &self,
    rep_id: &RepresentationId,
    expected_states: &[PayloadAvailability],
    blob_id: Option<&BlobId>,
    new_state: PayloadAvailability,
    last_error: Option<&str>,
) -> Result<ProcessingUpdateOutcome>;
```

Update infra repo to return `StateMismatch` when `updated_rows == 0`, and `NotFound` when no row exists. Update decrypting decorator and all mocks accordingly.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_update_processing_result_returns_state_mismatch`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/clipboard/representation_repository.rs
git add src-tauri/crates/uc-infra/src/db/repositories/representation_repo.rs
git add src-tauri/crates/uc-infra/src/security/decrypting_representation_repo.rs
git commit -m "feat(repo): add typed CAS outcomes"
```

---

### Task 7: Update BackgroundBlobWorker behavior

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/background_blob_worker.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/background_blob_worker.rs`

**Step 1: Write the failing test**

Add to worker tests:

```rust
#[tokio::test]
async fn test_worker_does_not_mark_lost_on_cache_miss() -> Result<()> {
    // If cache/spool miss, state should return to Staged and not Lost
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_worker_does_not_mark_lost_on_cache_miss`

Expected: FAIL

**Step 3: Write minimal implementation**

Handle typed outcome and missing bytes:

```rust
match self.repo.update_processing_result(...).await? {
    ProcessingUpdateOutcome::Updated(_) => {}
    ProcessingUpdateOutcome::StateMismatch => return Ok(ProcessResult::Completed),
    ProcessingUpdateOutcome::NotFound => {
        warn!(representation_id = %rep_id, "Representation missing");
        return Ok(ProcessResult::Completed);
    }
}

if cache_spool_miss {
    let _ = self.repo.update_processing_result(
        rep_id,
        &[PayloadAvailability::Processing],
        None,
        PayloadAvailability::Staged,
        Some("cache/spool miss: bytes not available"),
    ).await?;
    return Ok(ProcessResult::MissingBytes);
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_worker_does_not_mark_lost_on_cache_miss`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/background_blob_worker.rs
git commit -m "fix(worker): avoid premature lost transitions"
```

---

### Task 8: SpoolerTask marks cache + warns on enqueue failure

**Files:**

- Modify: `src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs`

**Step 1: Write the failing test**

Add to `spooler_task.rs` tests:

```rust
#[tokio::test]
async fn test_spooler_marks_cache_completed() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let spool_manager = Arc::new(SpoolManager::new(temp_dir.path(), 1_000_000)?);
    let cache = Arc::new(RepresentationCache::new(10, 1024));
    let (spool_tx, spool_rx) = mpsc::channel(8);
    let (worker_tx, _worker_rx) = mpsc::channel(8);

    let rep_id = RepresentationId::new();
    cache.put(&rep_id, vec![1, 2, 3]).await;

    let handle = tokio::spawn(SpoolerTask::new(spool_rx, spool_manager, worker_tx, cache.clone()).run());

    spool_tx.send(SpoolRequest { rep_id: rep_id.clone(), bytes: vec![1, 2, 3] }).await?;
    drop(spool_tx);
    handle.await?;

    assert_eq!(cache.get(&rep_id).await, Some(vec![1, 2, 3]));
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_spooler_marks_cache_completed`

Expected: FAIL (constructor signature).

**Step 3: Write minimal implementation**

Update `SpoolerTask` to accept `cache: Arc<RepresentationCache>` and call `cache.mark_spooling`/`mark_completed` on success. Log warn if `worker_tx.try_send` fails.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_spooler_marks_cache_completed`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spooler_task.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(spooler): mark cache + warn on enqueue"
```

---

### Task 9: Add SpoolJanitor and wire into runtime

**Files:**

- Create: `src-tauri/crates/uc-infra/src/clipboard/spool_janitor.rs`
- Modify: `src-tauri/crates/uc-infra/src/clipboard/mod.rs`
- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/spool_janitor.rs`

**Step 1: Write the failing test**

Create test in `spool_janitor.rs`:

```rust
#[tokio::test]
async fn test_janitor_marks_lost_after_ttl() -> Result<()> {
    // Create spool file with old mtime, state=Staged, run janitor once, expect Lost
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_janitor_marks_lost_after_ttl`

Expected: FAIL (module missing).

**Step 3: Write minimal implementation**

Create `spool_janitor.rs`:

```rust
pub struct SpoolJanitor {
    spool: Arc<SpoolManager>,
    repo: Arc<dyn ClipboardRepresentationRepositoryPort>,
    clock: Arc<dyn ClockPort>,
    ttl_days: u64,
}

impl SpoolJanitor {
    pub async fn run_once(&self) -> Result<usize> {
        let expired = self.spool.list_expired(self.clock.now_ms(), self.ttl_days).await?;
        let mut removed = 0usize;
        for entry in expired {
            if let Err(err) = self.repo.update_processing_result(
                &entry.representation_id,
                &[PayloadAvailability::Staged, PayloadAvailability::Processing],
                None,
                PayloadAvailability::Lost,
                Some("spool ttl expired"),
            ).await {
                tracing::warn!(representation_id = %entry.representation_id, error = %err, "Failed to mark Lost during spool cleanup");
            }
            if let Err(err) = tokio::fs::remove_file(&entry.file_path).await {
                tracing::warn!(representation_id = %entry.representation_id, error = %err, "Failed to delete expired spool file");
            }
            removed += 1;
        }
        Ok(removed)
    }
}
```

Wire into `start_background_tasks` with a periodic timer.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_janitor_marks_lost_after_ttl`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-infra/src/clipboard/spool_janitor.rs
git add src-tauri/crates/uc-infra/src/clipboard/mod.rs
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git commit -m "feat(spool): add janitor for ttl cleanup"
```

---

### Task 10: Capture path logging for spool queue drops

**Files:**

- Modify: `src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs`
- Test: `src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs`

**Step 1: Write the failing test**

Add to `snapshot_cache_integration_test.rs`:

```rust
#[tokio::test]
async fn test_capture_logs_on_spool_queue_full() -> Result<()> {
    // Configure spool channel capacity 0/1 and ensure warn is emitted when try_send fails
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_capture_logs_on_spool_queue_full`

Expected: FAIL

**Step 3: Write minimal implementation**

Update capture to warn:

```rust
if let Err(err) = self.spool_tx.try_send(SpoolRequest { ... }) {
    warn!(representation_id = %rep.id, error = %err, "Spool queue full; cache-only fallback");
}
```

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_capture_logs_on_spool_queue_full`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/internal/capture_clipboard.rs
git add src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs
git commit -m "fix(capture): warn on spool queue drops"
```

---

### Task 11: Spool migration and recovery scan

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`
- Test: `src-tauri/crates/uc-infra/src/clipboard/spool_scanner.rs`

**Step 1: Write the failing test**

Add to `spool_scanner.rs` tests:

```rust
#[tokio::test]
async fn test_scanner_can_recover_from_legacy_spool_dir() -> Result<()> {
    // Use legacy dir, ensure recovered count > 0
    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test test_scanner_can_recover_from_legacy_spool_dir`

Expected: FAIL

**Step 3: Write minimal implementation**

In `start_background_tasks`, if legacy dir exists and differs from new spool dir, run a second scan or migrate files before scanning new dir.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test test_scanner_can_recover_from_legacy_spool_dir`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs
git add src-tauri/crates/uc-infra/src/clipboard/spool_scanner.rs
git commit -m "feat(spool): recover legacy spool dir"
```

---

### Task 12: Update tests and stress coverage for new behavior

**Files:**

- Modify: `src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs`
- Modify: `src-tauri/crates/uc-app/tests/stress_test.rs`

**Step 1: Write the failing test**

Add assertions for:

- cache/spool miss does not mark Lost immediately
- spool eviction keeps cache hits functional

**Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test snapshot_cache_integration_test stress_test`

Expected: FAIL

**Step 3: Write minimal implementation**

Update tests to match new semantics and ensure worker/janitor behavior is covered.

**Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test snapshot_cache_integration_test stress_test`

Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/crates/uc-app/tests/snapshot_cache_integration_test.rs
git add src-tauri/crates/uc-app/tests/stress_test.rs
git commit -m "test(snapshot-cache): align with new semantics"
```
