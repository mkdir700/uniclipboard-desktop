# Learnings

---

## Rust State Machine Patterns & Table-Driven Tests

### References

1. **The Rust Book - State Pattern**: https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html
2. **typestate crate (docs.rs)**: https://docs.rs/typestate/latest/typestate/
3. **Table-driven tests macro example**: https://over-codes.github.io/rust-table-driven-tests.html
4. **test_case crate**: https://docs.rs/test-case/3.3.1/test_case/
5. **Type-state pattern guide**: https://zerotomastery.io/blog/rust-typestate-patterns/

### Key Takeaways (5 bullets)

1. **Enum-based state machines**: Use Rust enums with match to encode states and events—simple, readable, catches invalid transitions at compile time via exhaustiveness.

2. **Type-state pattern**: Encode valid transitions in types (e.g., `struct Foo<State>`), preventing invalid state sequences from compiling—ideal for protocol state machines like pairing flows.

3. **Table-driven tests in Rust**: Use `macro_rules!` or the `test_case` crate to parameterize tests. Centralize test inputs in a struct/array, iterate with `for` loop or macro expansion.

4. **Determinism focus**: Table-driven tests excel at covering state machine transitions—each test case = (initial_state, event, expected_state, side_effects).

5. **Separation of concerns**: Keep test data declarative (readable tables) separate from test logic (assertions). Reduces boilerplate and makes edge cases visible.

## Active Setup References and Dependency Edges (Post-Cleanup)

### Frontend (Active UI & Logic)

- `/src/api/setup.ts`: API client for setup flow commands.
- `/src/pages/SetupPage.tsx`: Main entry point for the setup flow.
- `/src/pages/setup/`: Directory containing multi-step components:
  - `WelcomeStep.tsx`
  - `CreatePassphraseStep.tsx`
  - `JoinPickDeviceStep.tsx`
  - `JoinVerifyPassphraseStep.tsx`
  - `PairingConfirmStep.tsx`
  - `SetupDoneStep.tsx`
  - `types.ts`
- `/src/i18n/locales/{en-US,zh-CN}.json`: Localization keys under the `setup` namespace.
- `/src/App.tsx`: Conditional logic to wait for setup status and redirect.
- `/src/components/TitleBar.tsx`: Logic to hide the title bar during setup.
- `/src/i18n/__tests__/setup-i18n.test.ts`: Tests for setup localization keys.
- `/src/pages/__tests__/SetupFlow.test.tsx`: Integration tests for the setup UI flow.

### Backend - uc-tauri (Command & Wiring)

- `/src-tauri/src/main.rs`: Registration of setup commands in the Tauri builder.
- `/src-tauri/crates/uc-tauri/src/commands/setup.rs`: Tauri commands that bridge to setup use cases.
- `/src-tauri/crates/uc-tauri/src/bootstrap/wiring.rs`: Dependency injection wiring for `SetupStatusPort` and repository initialization.
- `/src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`: `AppRuntime` accessors for setup orchestrator.

### Backend - uc-app (Use Cases & Orchestration)

- `/src-tauri/crates/uc-app/src/usecases/setup/`: Setup use cases and orchestrator.
- `/src-tauri/crates/uc-app/src/deps.rs`: `AppDeps` struct definition including `setup_status`.
- `/src-tauri/crates/uc-app/src/usecases/setup/mark_complete.rs`: Persists setup completion via `SetupStatusPort`.

### Backend - uc-infra (Persistence)

- `/src-tauri/crates/uc-infra/src/setup_status.rs`: `FileSetupStatusRepository` handles `.setup_status` persistence.

### Backend - uc-core (Domain & Ports)

- `/src-tauri/crates/uc-core/src/setup/status.rs`: Domain model `SetupStatus`.
- `/src-tauri/crates/uc-core/src/ports/setup_status.rs`: `SetupStatusPort` trait definition.

### Documentation (Active References)

- `/docs/guides/error-handling.md`: User-facing error messages related to setup.
- `/docs/architecture/commands-status.md`: Status of setup-related Tauri commands.

### Dependency Edges & Runtime Accessors

- **Command Registration**: `main.rs` -> `commands/setup.rs`.
- **Use Case Access**: `AppRuntime` (in `uc-tauri`) provides `setup_orchestrator()`.
- **Orchestration Tie**: `SetupOrchestrator` (in `uc-app`) drives New Space initialization and marks setup complete.
- **Persistence Edge**: `FileSetupStatusRepository` (in `uc-infra`) implements `SetupStatusPort` (in `uc-core`).

---

## Keyring/Keychain Failure Handling Research (2026-02-05)

### References

1. **Official docs.rs**: https://docs.rs/keyring/latest/keyring/error/enum.Error.html
2. **Error source code**: https://docs.rs/keyring/latest/src/keyring/error.rs.html
3. **Crate repository**: https://github.com/open-source-cooperative/keyring-rs
4. **Platform caveats**: https://docs.rs/keyring/latest/keyring/#caveats

### Error Categories (7 Variants, Non-Exhaustive)

| Error             | User-Facing Guidance                                       | Fatal? |
| ----------------- | ---------------------------------------------------------- | ------ |
| `PlatformFailure` | "Platform secure storage failed" - underlying system error | YES    |
| `NoStorageAccess` | "Cannot access secure storage" - locked/permission denied  | YES    |
| `NoEntry`         | Expected on first write; retry OK                          | NO     |
| `BadEncoding`     | Data corruption; clear and retry                           | YES    |
| `TooLong`         | Attribute exceeds platform limit; reduce size              | NO     |
| `Invalid`         | Invalid attribute; fix data format                         | NO     |
| `Ambiguous`       | Multiple credentials found; clean up duplicates            | NO     |

### Rollback Best Practices

- **No built-in rollback**: Platform credential stores are atomic per-operation; keyring-rs provides no transaction/rollback API
- **Platform-specific atomicity**: Each `set_password`/`delete_credential` is a single atomic platform operation
- **Mock store for testing**: `keyring::mock` allows pre-setting errors to test failure paths

### Platform Caveats (Critical)

- **Thread safety**: While keyring-rs code is thread-safe, underlying stores (Windows/Linux) may fail with concurrent access from multiple threads
- **RPC-based stores** (Linux Secret Service): Not recommended for rapid successive calls; accesses may fail
- **Recommendation**: Serialize keychain operations; use mutex or async synchronization

### Alignment with Fatal Error + Retry/Exit Policy

| Scenario                             | Action                                                      |
| ------------------------------------ | ----------------------------------------------------------- |
| `PlatformFailure`, `NoStorageAccess` | **FATAL** - log error, allow retry or graceful exit         |
| `BadEncoding`                        | **FATAL** - indicates corruption; require user intervention |
| `TooLong`, `Invalid`                 | **RETRY after fix** - user must correct input               |
| `NoEntry`                            | **RETRY OK** - expected on initial setup                    |
| `Ambiguous`                          | **RETRY after cleanup** - manual duplicate resolution       |

### Direct Quotes

> "This indicates runtime failure in the underlying platform storage system. The details of the failure can be retrieved from the attached platform error." — `PlatformFailure` variant docs

> "This indicates that the underlying secure storage holding saved items could not be accessed. Typically this is because of access rules in the platform; for example, it might be that the credential store is locked." — `NoStorageAccess` docs

> "While this crate's code is thread-safe, the underlying credential stores may not handle access from different threads reliably. In particular, accessing the same credential from multiple threads at the same time can fail, especially on Windows and Linux." — Caveats section

## SetupStateMachine Test Patterns

### Locations

- **Core Logic & Tests**: `src-tauri/crates/uc-core/src/setup/state_machine.rs`
- **State Definitions**: `src-tauri/crates/uc-core/src/setup/state.rs`
- **Event Definitions**: `src-tauri/crates/uc-core/src/setup/event.rs`
- **Action Definitions**: `src-tauri/crates/uc-core/src/setup/action.rs`

### Table-Driven Test Structure

The state machine uses a table-driven approach in `src-tauri/crates/uc-core/src/setup/state_machine.rs`:

- **Test Function**: `setup_state_machine_table_driven`
- **Case Provider**: `cases()` function returning a `Vec` of tuples:
  ```rust
  fn cases() -> Vec<(
      &'static str,      // Test case name
      SetupState,        // Initial state
      fn() -> SetupEvent, // Event generator (closure)
      SetupState,        // Expected next state
      Vec<SetupAction>,  // Expected actions
  )>
  ```

### Adding New Space Tests

To add tests for the "New Space" (Create Space) flow:

1.  Locate `cases()` in `src-tauri/crates/uc-core/src/setup/state_machine.rs`.
2.  Add new tuples to the `vec![]` representing the desired transitions.
3.  Existing "Create Space" tests are grouped under `// ===== Create Space =====`.

### Helper Utilities & Data

- **Enums**: Tests rely heavily on `SetupState`, `SetupEvent`, and `SetupAction` enums.
- **Closures**: `fn() -> SetupEvent` allows for deferred creation of events, which is useful for events containing data like passphrases.
- **Assertions**: Standard `assert_eq!` is used to compare the resulting state and actions.

### Orchestrator Testing

- `SetupOrchestrator` in `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` currently lacks comprehensive tests.
- Integration tests for the orchestrator should verify that `dispatch` correctly calls the state machine and executes the resulting actions.

---

## Transactional Keychain Writes & Rollback Best Practices (2026-02-05)

### References

1. **keyring-rs crate**: https://docs.rs/keyring/latest/keyring/
2. **keyring source**: https://github.com/open-source-cooperative/keyring-rs
3. **Apple TN3137 (Keychain APIs)**: https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains
4. **security-framework (macOS)**: https://github.com/shadowsocks/security-framework-rs
5. **lib.rs keyring comparisons**: https://lib.rs/crates/keyring

### Core Findings (5 Bullets)

1. **No native transactions**: Platform credential stores (Keychain, Windows Credential Manager, Linux Secret Service) provide atomic single-item operations only—keyring-rs exposes no multi-item transaction API

2. **Two-phase commit pattern**: Implement rollback via staged writes:
   - Phase 1: Write to temporary location (e.g., `kek_temp`)
   - Phase 2: Verify success, then rename/replace (`kek_temp` → `kek`)
   - On failure: delete temp, leave original intact

3. **Partial write cleanup**: Platform-specific atomicity means:
   - macOS: Individual SecItemAdd/Update are atomic
   - Windows: CredWrite/CredDelete are atomic per credential
   - Linux Secret Service: dbus calls are atomic but may timeout on rapid successive calls

4. **Platform caveats**:
   - **Thread safety**: While keyring-rs is thread-safe, Windows/Linux stores may fail with concurrent writes—serialize with Mutex
   - **Linux Secret Service**: RPC-based; rapid calls may fail—add 100-200ms delays or queue operations
   - **Timeouts**: Set explicit timeouts (5-10s) for Linux dbus operations; macOS Keychain can block indefinitely on locked keychain

5. **Observability requirements**:
   - Log all keychain operations with trace_id
   - Record platform error codes (e.g., `errSecItemNotFound`, `ERROR_NO_SUCH_LOGON_SESSION`)
   - Instrument retry attempts with exponential backoff

### Rollback Pattern (Code Template)

```rust
// Two-phase KEK write with rollback
async fn persist_kek_rollback(kek: &[u8]) -> Result<(), KeychainError> {
    let temp_entry = Entry::new("uniclipboard", "kek_temp");
    let final_entry = Entry::new("uniclipboard", "kek");

    // Phase 1: Write to temp
    if let Err(e) = temp_entry.set_password(kek.to_vec()) {
        error!(error = %e, "Failed to write temp KEK");
        return Err(e.into());
    }

    // Phase 2: Verify and replace (delete old, rename temp)
    // Note: True atomic rename not supported by all backends
    match final_entry.delete_password() {
        Ok(_) | Err(Error::NoEntry) => { /* old deleted */ }
        Err(e) => {
            // Cleanup temp on failure
            let _ = temp_entry.delete_password();
            return Err(e.into());
        }
    }

    // On success, temp becomes final (best-effort)
    Ok(())
}
```

### Platform-Specific Timeouts

| Platform | Backend            | Timeout Recommendation | Notes                                         |
| -------- | ------------------ | ---------------------- | --------------------------------------------- |
| macOS    | security-framework | 10s (blocking)         | Keychain may prompt user; handle auth dialogs |
| Windows  | wincred            | 5s                     | CredentialWrite is synchronous                |
| Linux    | secret-service     | 5s + 100ms retry       | dbus timeout; queue rapid operations          |
| Linux    | keyutils           | 2s                     | Kernel-based; faster but less universal       |

### Direct Quotes

> "While this crate's code is thread-safe, the underlying credential stores may not handle access from different threads reliably. In particular, accessing the same credential from multiple threads at the same time can fail, especially on Windows and Linux." — keyring docs, Caveats section

> "For Mac keychains, there are three APIs and two implementations. The Keychain Services API is the most general-purpose and is what keyring-rs uses via security-framework." — Apple TN3137

> "The Secret Service API is based on dbus and allows encryption of secrets in transit. However, rapid successive calls may fail due to dbus timeouts." — keyring secret_service module docs

### Alignment with Join Space Implementation

For the Join Space flow (Task 4):

1. **Persist keyslot first** (file-based, can retry)
2. **Write KEK to keychain** (two-phase: temp → final)
3. **On keychain failure**: Delete temp KEK, leave keyslot
4. **On keyslot failure**: Delete temp KEK (never leave orphaned keychain entry)
5. **All failures block setup**: Per policy—failures must block and require retry

---

## Platform-Specific Keyring/Keychain Failures - Additional Research (2026-02-05)

### References

1. **Apple Developer Forums - Keychain Errors**: https://developer.apple.com/forums/thread/114456
2. **Apple Developer Forums - -34018 Errors**: https://developer.apple.com/forums/thread/759481
3. **Apple Developer Forums - errSecInteractionNotAllowed**: https://developer.apple.com/forums/thread/724267
4. **Apple Developer Forums - errSecInvalidKeychain**: https://developer.apple.com/forums/thread/681805
5. **Microsoft Learn - Credential Manager**: https://learn.microsoft.com/en-us/windows/win32/secauthn/credential-manager
6. **ArchWiki - GNOME/Keyring**: https://wiki.archlinux.org/title/GNOME/Keyring
7. **GitHub - keyring-rs issues**: https://github.com/block/goose/issues/879
8. **RTFM - Linux Secret Service errors**: https://rtfm.co.ua/en/linux-the-nextcloud-client-qtkeychain-and-the-the-name-org-freedesktop-secrets-was-not-provided-by-any-service-files-error/
9. **docs.rs/keyring Caveats**: https://docs.rs/keyring/latest/keyring/#caveats

### macOS Keychain Error Codes

| Error Code | Name                        | Retryable? | User Action Required          |
| ---------- | --------------------------- | ---------- | ----------------------------- |
| -34018     | errKC Entitlement missing   | NO         | Fix code signing/entitlements |
| -25295     | errSecInvalidKeychain       | NO         | Recreate/repair keychain      |
| -25307     | errSecNoDefaultKeychain     | NO         | Set default keychain          |
| -25308     | errSecInteractionNotAllowed | YES        | Wait for unlock               |
| -25300     | errSecItemNotFound          | YES        | Expected on first write       |
| -25293     | errSecAuthFailed            | NO         | Authentication required       |

**Direct Quotes:**

> "-34018 A required entitlement isn't present. This error commonly occurs when the keychain access groups entitlement is missing or incorrectly configured in the provisioning profile." — Apple Developer Forums

> "Error -25308 (errSecInteractionNotAllowed) seems to happen when the phone is in the background / locked state... We use the kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly attribute on the data" — Apple Developer Forums

### Windows Credential Manager

**Known Issues:**

- Credential Guard conflicts can cause authentication failures
- Stored credentials may become stale or corrupted requiring manual cleanup via Control Panel
- No automatic retry mechanism; operations are synchronous

**Reference:** [Microsoft Learn - Credential Manager](https://learn.microsoft.com/en-us/windows/win32/secauthn/credential-manager)

### Linux Secret Service Failures

**Critical Error:**

```
"The name org.freedesktop.secrets was not provided by any service files"
```

**Root Causes:**

1. No Secret Service daemon running (gnome-keyring, kwallet, etc.)
2. D-Bus session bus not available (headless servers, Docker)
3. DBus activation failed for secret service

**Headless Environment Requirements:**

- dbus-daemon running with session bus
- gnome-keyring-daemon or kwalletd started
- Environment variables: `$DBUS_SESSION_BUS_ADDRESS`

**Direct Quote:**

> "GNOME Keyring is 'a collection of components in GNOME that store secrets, passwords, keys, certificates and make them available to applications.' It provides org.freedesktop.secrets, an API that allows client applications to store secrets securely using a service running in the user's login session." — ArchWiki

### Platform-Specific Retry/Exit Recommendations

| Platform | Blocking Behavior             | Retry Strategy                 | Fallback                           |
| -------- | ----------------------------- | ------------------------------ | ---------------------------------- |
| macOS    | -34018 = fatal; -25308 = wait | Exponential backoff 1s, 2s, 4s | None (keychain required)           |
| Windows  | Credential Guard = manual     | Manual intervention required   | None (Credential Manager required) |
| Linux    | No Secret Service = fatal     | Cannot retry (no daemon)       | Encrypted file (if implemented)    |
| All      | Concurrent writes = race      | Serialize with Mutex           | N/A                                |

### for Uni Key ImplicationsClipboard

1. **macOS**: Ensure proper entitlements for keychain-access-groups if sharing between app/daemon
2. **Linux**: Detect Secret Service availability at startup; warn user if unavailable
3. **Windows**: Handle Credential Guard conflicts gracefully; provide manual cleanup instructions
4. **All platforms**: Never silently fail—log platform error codes and provide actionable user feedback

### Summary (5 Bullets)

1. **macOS**: Error -34018 requires entitlement fix (fatal); -25308 retryable after unlock
2. **Windows**: Credential Guard conflicts require manual intervention; no programmatic retry
3. **Linux**: "org.freedesktop.secrets not provided" = no Secret Service daemon (fatal in headless)
4. **Cross-platform**: Serialize keychain operations—concurrent access fails on Windows/Linux
5. **Observability**: Log platform error codes (-34018, -25308, ERROR_NO_SUCH_LOGON_SESSION) for debugging

---

## Tauri Command Patterns for Setup/Onboarding Flows (2026-02-05)

### References

1. **Official State Management Guide**: https://v2.tauri.app/develop/state-management/
2. **Command Fundamentals (TauriTutorials)**: https://tauritutorials.com/blog/tauri-command-fundamentals
3. **Calling Rust from JS (Official Docs)**: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-rust.mdx
4. **Splashscreen Setup Flow Example**: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/learn/splashscreen.mdx
5. **Logging Plugin**: https://tauri.app/plugin/logging
6. **tauri-plugin-tracing**: https://crates.io/crates/tauri-plugin-tracing/

### Command Pattern Summary (5 Bullets)

1. **Command Registration Pattern**: Commands registered via `invoke_handler(tauri::generate_handler![cmd1, cmd2])` in builder setup. Each command must use `#[tauri::command]` macro.

2. **State Management**: Use `.manage(StateStruct)` in builder to register global state, access via `State<'_, StateStruct>` parameter. For async, use `Mutex<T>` or `tokio::sync::Mutex<T>` for non-blocking access.

3. **Async Command Pattern**: Async commands return `Result<T, E>` and accept `AppHandle`, `Window`, and typed `State` via DI. Commands can call other commands directly if input arguments are managed.

4. **Arguments & Type Safety**: Command arguments use `serde::Deserialize` for JSON deserialization. Use `rename_all = "snake_case"` attribute to map JS camelCase to Rust snake_case parameters.

5. **Setup Flow Coordination**: Splashscreen example demonstrates backend/frontend coordination: spawn async setup task, use commands to signal completion, check state, transition UI windows.

### Command with State + Window + Async (Complete Pattern)

```rust
#[tauri::command]
async fn setup_command(
    window: tauri::Window,
    number: usize,
    state: tauri::State<'_, AppState>,
    _trace: Option<TraceMetadata>,  // Per AGENTS.md requirement
) -> Result<CustomResponse, String> {
    let span = info_span!(
        "cmd.setup_command",
        trace_id = tracing::field::Empty,
        trace_ts = tracing::field::Empty
    );
    record_trace_fields(&span, &_trace);

    // State access with tracing
    info!(step = "access_state", "Accessing setup state");
    let mut state = state.lock().await;

    // Perform async work
    let result = async_operation().await?;

    Ok(CustomResponse { message: result })
}
```

### Important: No Explicit State Machine Support

**Critical Finding**: Tauri documentation does NOT provide explicit state machine patterns. The setup flow example uses manual state tracking (`SetupState` struct with boolean flags) rather than a formal state machine pattern. State transitions are implemented imperatively in async commands.

For state machines, implement externally (as done in `uc-core/src/setup/state_machine.rs`) and invoke from commands:

```rust
#[tauri::command]
async fn dispatch_setup_event(
    event: SetupEventEnum,
    orchestrator: tauri::State<'_, SetupOrchestrator>,
) -> Result<SetupState, String> {
    orchestrator.dispatch(event).await
}
```

### Tracing/Logging Notes

- **No built-in tracing**: Core Tauri has no automatic trace metadata injection—use `tauri-plugin-tracing` for structured logging
- **Command arguments**: Deserialized via serde—no automatic trace metadata
- **AGENTS.md compliance**: Implement `info_span!` with `trace_id`/`trace_ts` manually in each command (no framework support)

### Direct Quotes

> "Commands can be ran as regular functions as long as you take care of the input arguments yourself" — Tauri splashscreen docs

> "Commands in Tauri can be invoked from the frontend to execute backend logic and modify application state" — State management docs

> "Async commands are preferred in Tauri to perform heavy work in a manner that doesn't result in UI freezes or slowdowns" — Calling Rust from JS docs

### Version Notes

- Tauri 2.x (current) uses `tauri::Builder::default()` pattern
- Async commands require `async` keyword on function
- `AppHandle` and `Window` are available in async contexts via runtime trait bound (`R: Runtime`)

### Command Arguments Pattern

| Pattern    | Rust                                                    | JS (Frontend)                           |
| ---------- | ------------------------------------------------------- | --------------------------------------- |
| Simple     | `fn cmd(name: String)`                                  | `invoke('cmd', { name: 'value' })`      |
| Rename all | `#[tauri::command(rename_all = "snake_case")]`          | `invoke('cmd', { snakeCase: 'value' })` |
| Struct     | `#[derive(Deserialize)] struct Input { field: String }` | `invoke('cmd', { field: 'value' })`     |

---

## Tauri Setup/Onboarding State Handling Patterns (2026-02-05)

### References

1. **Official Tauri State Management**: https://v2.tauri.app/develop/state-management/
2. **app-state-example (Sample App)**: https://github.com/rpereira-tae/app-state-example
3. **Tauri Store Plugin (Persistence)**: https://v2.tauri.app/plugin/store
4. **Rustato State Library**: https://github.com/BiteCraft/rustato
5. **Tauri Command Fundamentals**: https://tauritutorials.com/blog/tauri-command-fundamentals

### Key Patterns (6 Bullets)

1. **State Registration Pattern**: Use `app.manage()` in `setup()` hook to register state structs that implement `Send + Sync`. Access via `State<T>` parameter in commands.

2. **Command-Based Status Query**: Expose status via `#[tauri::command]` that returns `State<T>` content. Frontend polls this command to determine onboarding step.

3. **Mutable State with Mutex**: Wrap counter/data in `Mutex<T>` for thread-safe mutation in commands. Example: `State<Mutex<SetupState>>`.

4. **Persistence Layer**: Use `tauri-plugin-store` for file-based persistence of onboarding progress. Enables state recovery after app restart.

5. **Event-Driven Updates**: Emit window events (`emit_to`) from commands after state changes, allowing frontend to react without polling.

6. **State Machine Integration**: Combine enum-based states with command handlers that validate transitions before mutating managed state.

### Minimal Command Pattern

```rust
#[tauri::command]
fn get_onboarding_state(state: State<'_, OnboardingState>) -> OnboardingStatus {
    status: state.0.clone()
}

#[tauri::command]
fn advance_onboarding(state: State<'_, Mutex<OnboardingState>>, step: OnboardingStep) {
    let mut current = state.lock().unwrap();
    // validate transition, then mutate
    *current = current.transition_to(step);
}
```

### Direct Quotes

> "You can later access your state with any type that implements the Manager trait, for example the App instance." — Tauri State Management Docs

> "Commands can accept arguments and return values. They can also return errors and be async." — Tauri Command Fundamentals

### Production vs Sample Apps

- **app-state-example**: Sample app demonstrating basic patterns only
- **rustato**: Library pattern, not production onboarding implementation
- **Official docs**: Reference patterns, no complete onboarding example found
- **Recommendation**: Build custom state machine with `app.manage()` + `#[tauri::command]` per Tauri patterns

## Stronghold Snapshot Deletion & Rollback Safety (2026-02-05)

### References

1. **Tauri Stronghold Plugin (Rust source)**: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/stronghold/src/lib.rs
2. **TypeScript/JavaScript bindings**: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/stronghold/guest-js/index.ts
3. **GitHub Issue #1102 (store file restoration)**: https://github.com/tauri-apps/plugins-workspace/issues/1102
4. **IOTA Stronghold.rs engine**: https://github.com/iotaledger/stronghold.rs

### Key Findings (4 Bullets)

1. **`destroy()` does NOT delete the snapshot file**: The Rust implementation removes the stronghold from the in-memory collection and saves it to disk—it does **not** delete the file. See `lib.rs:259-271`:

   ```rust
   async fn destroy(collection: State<'_, StrongholdCollection>, snapshot_path: PathBuf) -> Result<()> {
       let mut collection = collection.0.lock().unwrap();
       if let Some(stronghold) = collection.remove(&snapshot_path) {
           if let Err(e) = stronghold.save() {
               collection.insert(snapshot_path, stronghold);
               return Err(e);
           }
       }
       Ok(())
   }
   ```

2. **JavaScript `unload()` is just `destroy()` wrapper**: The TypeScript API exposes this as `Stronghold.unload()` (guest-js/index.ts:415-419), which calls the Rust `destroy` command but still saves the snapshot.

3. **No built-in snapshot deletion API**: The tauri-plugin-stronghold provides no command to delete the actual `.stronghold` snapshot file—you must delete it manually via filesystem operations.

4. **File deletion behavior documented as unexpected**: GitHub Issue #1102 shows that deleted store files are restored after app exit—this is the Tauri store plugin, but similar patterns may apply. **Direct file deletion is NOT recommended** without understanding platform-specific file synchronization behavior.

### Rollback Safety Assessment

| Approach                       | Safe? | Evidence                                       |
| ------------------------------ | ----- | ---------------------------------------------- |
| Call `destroy()` / `unload()`  | NO    | Only removes from memory, saves to disk        |
| Delete snapshot file manually  | YES\* | File deleted, but may have sync/caching issues |
| Both `destroy()` + file delete | YES   | Complete removal from memory and disk          |

\*Manual file deletion requires platform-specific handling (considering cloud sync, antivirus, file locking)

### Recommended Rollback Pattern

```typescript
// TypeScript/JavaScript rollback pattern
import { Stronghold } from '@tauri-apps/plugin-stronghold'
import { remove } from '@tauri-apps/plugin-fs'

async function rollbackStronghold(snapshotPath: string): Promise<void> {
  // Step 1: Unload from memory (saves current state)
  const stronghold = await Stronghold.load(snapshotPath, password)
  await stronghold.unload()

  // Step 2: Delete the snapshot file
  await remove(snapshotPath, { recursive: true })
}
```

### Data-Loss Warning

> **CRITICAL**: Calling `destroy()`/`unload()` saves the current state before removing from memory. If called after partial write, the incomplete state will be persisted. Always delete the file afterward for true rollback.

### Alternative: Fresh Stronghold Without File Deletion

If you want to reset state without file deletion (safer for concurrent access):

```typescript
async function resetStronghold(snapshotPath: string, password: string): Promise<void> {
  // Load with new password (overwrites encrypted content)
  await Stronghold.load(snapshotPath, newPassword)
  // Create fresh client (empties previous data)
  const client = await stronghold.createClient('default')
}
```

---

## Rust Rollback Patterns for Multi-Step Initialization (2026-02-05)

### References

1. **scopeguard crate (docs.rs)**: https://docs.rs/scopeguard/latest/scopeguard/
2. **scope-guard crate (docs.rs)**: https://docs.rs/scope-guard/latest/scope_guard/
3. **drop_guard crate (docs.rs)**: https://docs.rs/drop_guard/latest/drop_guard
4. **RAII Guards - Rust Design Patterns**: https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html
5. **Compensating Transaction pattern (Azure Architecture)**: https://learn.microsoft.com/en-us/azure/architecture/patterns/compensating-transaction
6. **Saga Pattern - Distributed Transactions**: https://temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas
7. **Rust Forum - Rollback pattern discussion**: https://users.rust-lang.org/t/rollback-pattern/80624

### Key Takeaways (5 Bullets)

1. **RAII-based guards**: Use `ScopeGuard<T, F>` from `scopeguard` to run cleanup closures on scope exit—even on panic. Pattern: `scopeguard::guard(value, |v| { /* rollback logic */ })`.

2. **Compensating actions for multi-step**: Implement compensating transactions that undo prior steps in reverse order. Each step has a corresponding rollback function called on failure (Saga pattern applied to single-process Rust).

3. **Two-phase persistence pattern**: For file + keychain operations:
   - Phase 1: Write to temporary location (e.g., `keyslot.tmp`)
   - Phase 2: Atomic rename on success
   - On failure: Delete temp, preserve original state

4. **State machine + rollback**: Encode rollback actions in state machine transitions. Define `Rollback` events that trigger compensating actions for the current state (e.g., `RollbackKeychain` after `KeyslotWritten`).

5. **Error handling crates**: `anyhow` for error propagation, `thiserror` for defining error types, `scopeguard` for scope-bound cleanup. Combine with `?` operator for clean rollback propagation.

### Direct Quotes

> "A scope guard will run a given closure when it goes out of scope, even if the code between panics (as long as panic doesn't abort)." — scopeguard docs

> "Compensating actions are a distributed systems design pattern for simulating atomic execution of operations... If one of the distributed operations fails, their effects are undone via a compensating action." — Temporal blog

> "RAII stands for 'Resource Acquisition is Initialization'... The pattern is extended in Rust by using a RAII object as a guard of some resource and relying on the type system to ensure that access is always mediated by the guard object." — Rust Design Patterns

### Code Pattern Template

```rust
use scopeguard::guard;

fn multi_step_init() -> Result<InitializedState, InitError> {
    // Step 1: Create temp file
    let temp_file = guard(File::create("temp.tmp")?, |f| {
        let _ = std::fs::remove_file("temp.tmp");
    });

    // Step 2: Write keyslot
    write_keyslot(&temp_file, keyslot_data)?;

    // Step 3: Commit to keychain (two-phase)
    let _kek_guard = guard((), |_| {
        let _ = delete_temp_kek();
    });

    write_kek_to_keychain(kek)?;

    // Success path: disarm guards by renaming
    std::fs::rename("temp.tmp", "keyslot.json")?;
    std::mem::forget(temp_file);
    std::mem::forget(_kek_guard);

    Ok(InitializedState)
}
```

### Alignment with Join Space Implementation

For Join Space flow rollback:

1. **State machine guards**: Define `RollbackJoinSpace` event that triggers compensating actions in reverse order
2. **File cleanup**: Delete `keyslot.json` if keychain write fails
3. **Keychain cleanup**: Delete temp KEK if keyslot write fails
4. **Orchestrator responsibility**: Track completed steps; on failure, execute compensating transactions for all completed steps
5. **Visibility**: Log all rollback operations with `trace_id` for debugging

---

## Rust Tracing Best Practices for Error Handling (2026-02-05)

### References

1. **Official Tokio Tracing Tutorial**: https://tokio.rs/tokio/topics/tracing
2. **tracing crate docs.rs**: https://docs.rs/tracing/latest/tracing/
3. **tracing-error crate**: https://docs.rs/tracing-error/latest/tracing-error/
4. **Instrument macro docs**: https://docs.rs/tracing/latest/tracing/attr.instrument.html
5. **tracing-subscriber fmt**: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html

### Summary (5 Bullets)

1. **Use structured fields over string interpolation**: `tracing::warn!(error = %e, "operation failed")` instead of `tracing::warn!("failed: {}", e)`. Enables machine parsing, filtering, and aggregation.

2. **Span lifecycle = request/task lifetime**: Create spans with `#[instrument]` or `span!()` macro, attach contextual fields once, then log events inside. Spans have beginning/end times and can be nested—ideal for async contexts.

3. **Error recording patterns**: Use `#[instrument(err)]` on functions returning `Result<T, E>` to auto-emit error events on `Err` returns. For manual errors, use `tracing::error!(error = %e)` or the `%` shorthand for Display formatting.

4. **tracing-error for enriched errors**: The `tracing-error` crate provides `InstrumentedError` wrapper that attaches span context to errors propagating through async layers—preserves causality across task boundaries.

5. **Avoid secrets in fields**: Per AGENTS.md: never log raw keys, passphrases, or decrypted content. Log sizes, hashes, or redacted markers instead.

### Direct Quotes

> "Unlike a log message, a span in `tracing` has a beginning and end time, may be entered and exited by the flow of execution, and may exist within a nested tree of similar spans." — Tokio Tracing Tutorial

> "If a `Result<T, E>` is returned and `E` implements `std::fmt::Display`, adding `err` or `err(Display)` will emit error events when the function returns `Err`" — Instrument macro docs

> "Utilities for enriching error handling with `tracing` diagnostic information" — tracing-error crate description

### Error Logging Pattern (Production Best Practice)

```rust
use tracing::{info_span, Instrument, error};

pub async fn sync_peer(peer_id: &str) -> Result<(), SyncError> {
    let span = info_span!("sync_peer", peer_id = %peer_id);

    async move {
        if let Err(e) = perform_sync().await {
            error!(error = %e, "sync failed for peer");
            return Err(SyncError::from(e));
        }
        Ok(())
    }
    .instrument(span)
    .await
}
```

## Setup Orchestration Mapping (2026-02-05)

### Entrypoints & Accessors

- **Tauri Commands**: `src-tauri/crates/uc-tauri/src/commands/setup.rs`
  - `get_state`: Returns current `SetupState`.
  - `start_new_space`: Triggers `SetupEvent::StartNewSpace`.
  - `start_join_space`: Triggers `SetupEvent::StartJoinSpace`.
  - `select_device(peer_id)`: Triggers `SetupEvent::SelectPeer`.
  - `submit_passphrase(pass1, pass2)`: Triggers `SetupEvent::SubmitCreatePassphrase`.
  - `verify_passphrase(passphrase)`: Triggers `SetupEvent::SubmitJoinPassphrase`.
  - `cancel_setup`: Triggers `SetupEvent::UserCancel`.
- **Runtime Accessor**: `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`
  - `Runtime::setup_orchestrator() -> Arc<SetupOrchestrator>`: Provides access to the orchestrator instance.

### SetupContext & State Management

- **Location**: `src-tauri/crates/uc-app/src/usecases/setup/context.rs`
- **Storage**:
  - `state: Arc<Mutex<SetupState>>`: In-memory storage of the current setup phase.
  - `dispatch_lock: Arc<Mutex<()>>`: Mutex used to serialize concurrent `dispatch` calls.
- **Access Patterns**:
  - `get_state()`: Lightweight read (acquires `state` lock only).
  - `dispatch(event)`: Heavyweight write (acquires `dispatch_lock` first, then `state` lock).

### Concurrency & Dispatch Semantics

- **Serialization**: `SetupOrchestrator::dispatch` acquires `dispatch_lock` at the start. This ensures that the sequence of (Read State -> Transition -> Execute Actions -> Write State) is atomic relative to other dispatch calls.
- **Lock Ordering**: Always acquire `dispatch_lock` before `state` lock to avoid deadlocks.
- **Side Effects**: Actions are executed _before_ the state is updated to the next state in the context, but the transition logic itself is pure.

### Summary Table

| Component         | File Path                                                    | Responsibility                     |
| ----------------- | ------------------------------------------------------------ | ---------------------------------- |
| Tauri Commands    | `src-tauri/crates/uc-tauri/src/commands/setup.rs`            | Frontend entrypoints               |
| Runtime Accessor  | `src-tauri/crates/uc-tauri/src/bootstrap/runtime.rs`         | Dependency injection / Access      |
| SetupOrchestrator | `src-tauri/crates/uc-app/src/usecases/setup/orchestrator.rs` | Orchestration & Side effects       |
| SetupContext      | `src-tauri/crates/uc-app/src/usecases/setup/context.rs`      | Shared state & Concurrency control |
| State Machine     | `src-tauri/crates/uc-core/src/setup/state_machine.rs`        | Pure transition logic              |
| SetupState        | `src-tauri/crates/uc-core/src/setup/state.rs`                | State definitions                  |
| SetupEvent        | `src-tauri/crates/uc-core/src/setup/event.rs`                | Event definitions                  |
| SetupAction       | `src-tauri/crates/uc-core/src/setup/action.rs`               | Action definitions                 |

### Tracing Spans

- `command.setup.*`: Tauri command layer spans.
- `usecase.setup_orchestrator.dispatch`: Orchestrator dispatch span, includes `event` field.

### Tests

- State machine logic is tested in `src-tauri/crates/uc-core/src/setup/state_machine.rs`.
- Orchestrator integration tests are mentioned in plans but not found in the orchestrator file itself.

## Onboarding State Dependency Mapping (uc-app & uc-tauri)

### Core Dependency: OnboardingStatePort

- **Definition**: `uc-core/src/ports/onboarding.rs`
- **Implementation**: `uc-infra/src/onboarding_state.rs` (`FileOnboardingStateRepository`)

### Injection Points & Construction

| Component              | File                                           | Injection Point                         | Notes                                            |
| ---------------------- | ---------------------------------------------- | --------------------------------------- | ------------------------------------------------ |
| `AppDeps`              | `uc-app/src/deps.rs`                           | Struct Field                            | `onboarding_state: Arc<dyn OnboardingStatePort>` |
| `InfraLayer`           | `uc-tauri/src/bootstrap/wiring.rs`             | `create_infra_layer`                    | Instantiates `FileOnboardingStateRepository`     |
| `AppDeps` (Wiring)     | `uc-tauri/src/bootstrap/wiring.rs`             | `wire_dependencies_with_identity_store` | Injects `infra.onboarding_state` into `AppDeps`  |
| `AppRuntime`           | `uc-tauri/src/bootstrap/runtime.rs`            | `new`                                   | Holds `AppDeps`                                  |
| `InitializeOnboarding` | `uc-app/src/usecases/onboarding/initialize.rs` | `new`, `from_ports`                     | Usecase dependency                               |
| `GetOnboardingState`   | `uc-app/src/usecases/onboarding/get_state.rs`  | `new`, `from_ports`                     | Usecase dependency                               |
| `CompleteOnboarding`   | `uc-app/src/usecases/onboarding/complete.rs`   | `new`, `from_ports`                     | Usecase dependency                               |
| `SetupOrchestrator`    | `uc-app/src/usecases/setup/orchestrator.rs`    | `new`                                   | Uses `CompleteOnboarding`                        |

### UseCase Factory Methods (uc-tauri/src/bootstrap/runtime.rs)

- `UseCases::initialize_onboarding`: Injects `runtime.deps.onboarding_state`
- `UseCases::get_onboarding_state`: Injects `runtime.deps.onboarding_state`
- `UseCases::complete_onboarding`: Injects `runtime.deps.onboarding_state`
- `UseCases::setup_orchestrator`: Injects `complete_onboarding()` usecase

### Test Mocks & Noops

- `uc-app/src/usecases/onboarding/complete.rs`: `MockOnboardingStatePort`
- `uc-app/src/usecases/onboarding/get_state.rs`: `MockOnboardingStatePort`
- `uc-app/src/usecases/onboarding/initialize.rs`: `MockOnboardingStatePort`
- `uc-tauri/src/bootstrap/runtime.rs`: `NoopPort` (impl `OnboardingStatePort`)
- `uc-tauri/src/commands/clipboard.rs`: `NoopPort` (impl `OnboardingStatePort`)
- `uc-tauri/src/commands/encryption.rs`: `NoopPort` (impl `OnboardingStatePort`)

### Runtime Accessors

- `uc-tauri/src/commands/onboarding.rs`: Calls `runtime.usecases().get_onboarding_state()`

## Onboarding i18n Mapping (2026-02-05)

### Locale Files

- /Users/mark/.superset/worktrees/uniclipboard-desktop/repair/src/i18n/locales/en-US.json
- /Users/mark/.superset/worktrees/uniclipboard-desktop/repair/src/i18n/locales/zh-CN.json

### Component Usage (keyPrefix patterns)

All components in `src/pages/onboarding/` use `useTranslation` with specific `onboarding.*` prefixes:

- `src/pages/OnboardingPage.tsx`: `onboarding.page`, `onboarding.common`
- `src/pages/onboarding/WelcomeStep.tsx`: `onboarding.welcome`
- `src/pages/onboarding/CreatePassphraseStep.tsx`: `onboarding.createPassphrase`, `onboarding.common`
- `src/pages/onboarding/JoinPickDeviceStep.tsx`: `onboarding.joinPickDevice`, `onboarding.common`
- `src/pages/onboarding/JoinVerifyPassphraseStep.tsx`: `onboarding.joinVerifyPassphrase`, `onboarding.common`
- `src/pages/onboarding/PairingConfirmStep.tsx`: `onboarding.pairingConfirm`
- `src/pages/onboarding/SetupDoneStep.tsx`: `onboarding.done`

### Test Files (Vitest)

- `src/i18n/__tests__/onboarding-i18n.test.ts`: Direct tests for onboarding keys resolution.
- `src/pages/onboarding/__tests__/joinPickPeerIdDisplay.test.tsx`: UI tests for device picking.
- `src/pages/onboarding/__tests__/peerIdDisplay.test.tsx`: UI tests for passphrase verification.
- `src/pages/__tests__/OnboardingFlow.test.tsx`: Integration tests for the full flow.
- `src/api/__tests__/onboarding.test.ts`: API layer tests.

### Summary Table

| Category   | File Paths                                                                                                                                                                       |
| :--------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Locales    | `src/i18n/locales/{en-US,zh-CN}.json`                                                                                                                                            |
| Components | `src/pages/OnboardingPage.tsx`, `src/pages/onboarding/*.tsx`                                                                                                                     |
| Tests      | `src/i18n/__tests__/onboarding-i18n.test.ts`, `src/pages/onboarding/__tests__/*.test.tsx`, `src/pages/__tests__/OnboardingFlow.test.tsx`, `src/api/__tests__/onboarding.test.ts` |
| Docs/Plans | `docs/plans/2026-01-30-onboarding-i18n.md`, `docs/plans/2026-01-29-setup-onboarding-ui.md`                                                                                       |

## SpaceAccessStateMachine Test Patterns

### Table-Driven Test Structure

The `SpaceAccessStateMachine` uses a robust table-driven testing pattern located in `src-tauri/crates/uc-core/src/security/space_access/state_machine.rs`.

#### Key Components:

1.  **`cases` function**: Returns a `Vec` of tuples representing test cases.
    ```rust
    fn cases(now: DateTime<Utc>) -> Vec<(
        &'static str,           // Case name
        SpaceAccessState,       // Initial state
        SpaceAccessEvent,       // Input event
        SpaceAccessState,       // Expected next state
        Vec<SpaceAccessAction>, // Expected actions
    )>
    ```
2.  **Deterministic Time**: Uses a `fixed_now()` helper to ensure time-based transitions (like TTL/expiration) are testable.
3.  **Loop Execution**:
    ```rust
    #[test]
    fn space_access_state_machine_table_driven() {
        let now = fixed_now();
        for (name, from, event, expected_state, expected_actions) in cases(now) {
            let (next, actions) = SpaceAccessStateMachine::transition_at(from, event, now);
            assert_eq!(next, expected_state, "state mismatch: {}", name);
            assert_eq!(actions, expected_actions, "actions mismatch: {}", name);
        }
    }
    ```

### Comparison with Other State Machines

- **`SetupStateMachine`**: Follows an almost identical table-driven pattern in `src-tauri/crates/uc-core/src/setup/state_machine.rs`. It uses closures for events (`fn() -> SetupEvent`) to handle complex event construction.
- **`PairingStateMachine`**: Uses traditional unit tests in `src-tauri/crates/uc-core/src/network/pairing_state_machine.rs` due to the multi-step nature of the pairing protocol, requiring sequential event handling in a single test.

### Helper Structs & Builders

- **Builders**: `PairingStateMachine` tests use `build_request` and `build_challenge` to simplify test data creation.
- **State/Event/Action Enums**: All state machines rely on exhaustive enums for states, events, and actions, which are imported into the test module.

## Onboarding Setup Step Components and Dependencies Mapping

### Component Hierarchy & Purpose

| Component                    | File Path                                           | Purpose                                                                |
| :--------------------------- | :-------------------------------------------------- | :--------------------------------------------------------------------- |
| **OnboardingPage**           | `src/pages/OnboardingPage.tsx`                      | Main orchestrator. Manages `setupState` and renders appropriate steps. |
| **WelcomeStep**              | `src/pages/onboarding/WelcomeStep.tsx`              | Entry point: Choose "Create Space" or "Join Space".                    |
| **CreatePassphraseStep**     | `src/pages/onboarding/CreatePassphraseStep.tsx`     | Create flow: Set encryption passphrase.                                |
| **JoinPickDeviceStep**       | `src/pages/onboarding/JoinPickDeviceStep.tsx`       | Join flow: Scan and select a peer device.                              |
| **JoinVerifyPassphraseStep** | `src/pages/onboarding/JoinVerifyPassphraseStep.tsx` | Join flow: Enter passphrase for the space.                             |
| **PairingConfirmStep**       | `src/pages/onboarding/PairingConfirmStep.tsx`       | Pairing flow: Confirm short code with peer.                            |
| **SetupDoneStep**            | `src/pages/onboarding/SetupDoneStep.tsx`            | Completion: Final success screen.                                      |

### Key Dependencies & Hooks

- **Context**: `useOnboarding` from `@/contexts/onboarding-context` (provides `status`, `refreshStatus`).
- **i18n**: `useTranslation` from `react-i18next` with `keyPrefix` (e.g., `onboarding.welcome`).
- **Navigation**: `useNavigate` from `react-router-dom`.
- **UI Components**: `framer-motion` (AnimatePresence), `lucide-react` (icons), `sonner` (toast).

### API & Backend Interaction

- **Commands (`src/api/onboarding.ts`)**:
  - `getSetupState()`: Fetches current `SetupState` from backend.
  - `dispatchSetupEvent(event)`: Sends `SetupEvent` to backend and returns new `SetupState`.
  - `completeOnboarding()`: Marks onboarding as finished.
- **P2P (`src/api/p2p.ts`)**:
  - `getP2PPeers()`: Lists discovered devices for joining.
- **Events**:
  - Listens for `onboarding-password-set` and `onboarding-completed` via `OnboardingContext`.

### Shared Types & Models

- **UI Props**: `src/pages/onboarding/types.ts` (`StepProps`, `WelcomeStepProps`, etc.).
- **State Machine**: `src/api/onboarding.ts` (`SetupState`, `SetupEvent`, `SetupError`).
  - `SetupState` is a tagged union (e.g., `Welcome`, `{ CreateSpacePassphrase: { error } }`).
  - `SetupEvent` is a tagged union (e.g., `ChooseCreateSpace`, `{ SelectPeer: { peer_id } }`).

### i18n KeyPrefix Usage

- `onboarding.page`
- `onboarding.common`
- `onboarding.welcome`
- `onboarding.createPassphrase`
- `onboarding.joinPickDevice`
- `onboarding.joinVerifyPassphrase`
- `onboarding.pairingConfirm`
- `onboarding.done`

### Testing

- **Flow Test**: `src/pages/__tests__/OnboardingFlow.test.tsx` (Mocks `getSetupState` and `dispatchSetupEvent`).
- **Unit Tests**: `src/pages/onboarding/__tests__/joinPickPeerIdDisplay.test.tsx`, `src/pages/onboarding/__tests__/peerIdDisplay.test.tsx`.

## Tracing Patterns in Setup/Onboarding Commands

### Tauri Command Pattern

---

## Setup Orchestrator Seeding (2026-02-06)

- `SetupOrchestrator::get_state()` now seeds from `SetupStatus.has_completed` once and sets state to `Completed` when true; errors are logged and seeding is idempotent via an atomic flag.
- `SetupAction::MarkSetupComplete` uses `MarkSetupComplete` (backed by `SetupStatusPort`) instead of onboarding completion.
- `AppRuntime` wiring exposes `mark_setup_complete()` and passes it into `SetupOrchestrator::new`.

All Tauri commands in `uc-tauri` (specifically `setup.rs`, `onboarding.rs`, `pairing.rs`, `encryption.rs`) must follow this pattern for trace propagation:

1. **Signature**: Accept `_trace: Option<TraceMetadata>`.
2. **Span**: Create `info_span!` with `trace_id` and `trace_ts` as `tracing::field::Empty`.
3. **Record**: Call `record_trace_fields(&span, &_trace)`.
4. **Instrument**: Wrap async body in `async { ... }.instrument(span).await`.

Example:

```rust
#[tauri::command]
pub async fn some_command(runtime: State<'_, Arc<AppRuntime>>, _trace: Option<TraceMetadata>) -> Result<(), String> {
    let span = info_span!("command.name", trace_id = tracing::field::Empty, trace_ts = tracing::field::Empty);
    record_trace_fields(&span, &_trace);
    async {
        // logic
    }.instrument(span).await
}
```

### Common Trace Fields

- `trace_id`: UUID from frontend.
- `trace_ts`: Timestamp from frontend.
- `device_id`: Current device ID (often added in pairing/encryption).
- `peer_id` / `session_id`: Added in pairing commands for specific peer/session context.

### Implementation Details

- `record_trace_fields` is defined in `src-tauri/crates/uc-tauri/src/commands/mod.rs`.
- `TraceMetadata` is defined in `uc-core::ports::observability`.

## SetupState Data Flow Map (2026-02-05)

### Flow Overview

1. **UI/Tauri**: `uc-tauri/src/commands/setup.rs` (Commands)
2. **Application**: `uc-app/src/usecases/setup/orchestrator.rs` (Orchestrator)
3. **Domain Logic**: `uc-core/src/setup/state_machine.rs` (StateMachine)
4. **State Storage (In-Memory)**: `uc-app/src/usecases/setup/context.rs` (SetupContext)
5. **Persistence**: `uc-core/src/ports/setup/store.rs` (PersistencePort)

### Key Components & Responsibilities

- **SetupOrchestrator**: Coordinates the flow. Uses `dispatch_lock` to ensure atomic transitions. Executes `SetupAction`s which trigger side effects (encryption, networking).
- **SetupStateMachine**: Pure transition function `(State, Event) -> (NextState, Actions)`.

---

## SpaceAccess Orchestrator Notes (2026-02-05)

- Added a public async entry for sponsor-side New Space initialization that dispatches `SponsorAuthorizationRequested` and executes only sponsor-related actions.
- Timer actions require a pairing session id; unimplemented joiner actions remain explicit errors to avoid silent behavior.
- **SetupContext**: Holds the current `SetupState` in an `Arc<Mutex<SetupState>>`. Provides thread-safe access and a `dispatch_lock` for serialization.
- **SetupState**: Enum defining the stages (Welcome, CreateSpace, JoinSpace, Completed).

### Storage Points

- **Active State**: In-memory only, managed by `SetupContext`.
- **Completion Status**: Persisted via `MarkSetupComplete` action (delegates to `CompleteOnboarding` use case).

### Error Handling

- Actions return `Result<(), SetupError>`.
- State machine can transition to states with `error: Option<SetupError>` fields to inform the UI of failures.

## New Space Passphrase Flow Mapping (2026-02-05)

### Path Trace

1. **Tauri Command**: `uc_tauri::commands::setup::submit_passphrase`
   - Receives `passphrase1`, `passphrase2`.
   - Calls `orchestrator.submit_passphrase`.
2. **Orchestrator**: `uc_app::usecases::setup::orchestrator::SetupOrchestrator::submit_passphrase`
   - Dispatches `SetupEvent::SubmitPassphrase { passphrase }`.
3. **State Machine**: `uc_core::setup::state_machine::SetupStateMachine`
   - Transition: `(CreateSpaceInputPassphrase, SubmitPassphrase) -> (ProcessingCreateSpace, [CreateEncryptedSpace])`.
4. **Action Execution**: `orchestrator.execute_actions`
   - Matches `SetupAction::CreateEncryptedSpace`.
   - Calls `initialize_encryption.execute(Passphrase(passphrase))`.
5. **Core Use Case**: `uc_app::usecases::initialize_encryption::InitializeEncryption::execute`
   - Derives KEK via `EncryptionPort::derive_kek`.
   - Generates `MasterKey`.
   - Wraps `MasterKey` via `EncryptionPort::wrap_master_key`.
   - Persists `KeySlot` and `KEK`.

### Validation Points

- **Frontend**: Mismatch check (`pass1 != pass2`) and empty check.
- **Use Case**: `AlreadyInitialized` check against `EncryptionStatePort`.
- **Security**: Passphrase wrapped in `Passphrase` domain model; excluded from tracing spans.

### Error Handling

- `SetupError::InitializeEncryption`: Bubbles up from the use case.
- `InitializeEncryptionError`: `AlreadyInitialized`, `EncryptionFailed`, `StatePersistenceFailed`.

## Domain Types Mapping (Setup & Space Access)

| Type          | Definition File                       | Description                                                | Serde | Notes                           |
| :------------ | :------------------------------------ | :--------------------------------------------------------- | :---- | :------------------------------ |
| `SpaceId`     | `uc-core/src/ids/space_id.rs`         | Unique identifier for a space.                             | Yes   | Uses `impl_id!` macro.          |
| `SessionId`   | `uc-core/src/ids/session_id.rs`       | Pairing session identifier.                                | Yes   | Format: `{timestamp}-{random}`. |
| `Passphrase`  | `uc-core/src/security/model.rs`       | User-provided passphrase.                                  | No    | Redacted in `Debug`.            |
| `DeviceId`    | `uc-core/src/device/value_objects.rs` | Unique identifier for a device.                            | No    | Simple wrapper around `String`. |
| `KeySlot`     | `uc-core/src/security/model.rs`       | Persistent container for KDF params and wrapped MasterKey. | Yes   | Central to encryption setup.    |
| `MasterKey`   | `uc-core/src/security/model.rs`       | Data-encryption key (DEK).                                 | No    | Redacted in `Debug`. 32 bytes.  |
| `Kek`         | `uc-core/src/security/model.rs`       | Key-encryption key derived from passphrase.                | No    | Redacted in `Debug`. 32 bytes.  |
| `KeySlotFile` | `uc-core/src/security/model.rs`       | On-disk representation of `KeySlot`.                       | Yes   | Includes timestamps.            |

### ID Wrappers & Macros

- `SpaceId` uses the `impl_id!` macro (`uc-core/src/ids/id_macro.rs`), providing `new()`, `from_str()`, `inner()`, and `Deref<Target=String>`.
- `SessionId` is a manual wrapper in `uc-core/src/ids/session_id.rs`, also providing `new()`, `as_str()`, and `Display`.
- `DeviceId` is a manual wrapper in `uc-core/src/device/value_objects.rs`.

### Security & Redaction

- `Passphrase`, `MasterKey`, and `Kek` all implement `Debug` with `[REDACTED]` to prevent accidental logging of secrets.
- `MasterKey` and `Kek` are 32-byte arrays, while `Passphrase` wraps a `String`.

### Key Usage Sites

- **Setup Flow**: `uc-core/src/setup/state_machine.rs` and `uc-app/src/usecases/setup/orchestrator.rs`.
- **Space Access Flow**: `uc-core/src/security/space_access/state_machine.rs` and `uc-app/src/usecases/space_access/orchestrator.rs`.
- **Pairing Flow**: `uc-app/src/usecases/pairing/orchestrator.rs` and `uc-core/src/network/pairing_state_machine.rs`.
- **Persistence**: `uc-infra/src/fs/key_slot_store.rs` (JSON storage for `KeySlotFile`).

### Tests

- `uc-core/src/ids/session_id.rs`: Unit tests for `SessionId` creation and conversion.
- `uc-infra/src/security/encryption.rs`: Integration tests for passphrase-based KEK derivation and encryption.
- `uc-infra/src/fs/key_slot_store.rs`: Tests for `KeySlotFile` serialization/deserialization.

## Frontend Setup Error Handling Mapping

### Error Flow

1. **Backend**: `SetupError` enum (Rust).
2. **API Bridge**: `src/api/onboarding.ts` defines `SetupError` as a TypeScript type.
3. **Main Page**: `src/pages/OnboardingPage.tsx` manages `SetupState` and passes `error` to step components.
4. **Mapping**: Step components map `SetupError` variants to i18n keys in `useEffect`.

### Error Mapping Table

| Step Component               | SetupError Variant            | i18n Key                                                 |
| :--------------------------- | :---------------------------- | :------------------------------------------------------- |
| **CreatePassphraseStep**     | `PassphraseMismatch`          | `onboarding.createPassphrase.errors.mismatch`            |
|                              | `PassphraseTooShort`          | `onboarding.createPassphrase.errors.tooShort`            |
|                              | `PassphraseEmpty`             | `onboarding.createPassphrase.errors.empty`               |
| **JoinPickDeviceStep**       | `NetworkTimeout`              | `onboarding.joinPickDevice.errors.timeout`               |
| **JoinVerifyPassphraseStep** | `PassphraseInvalidOrMismatch` | (Special Help UI)                                        |
|                              | `NetworkTimeout`              | `onboarding.joinVerifyPassphrase.errors.timeout`         |
|                              | `PeerUnavailable`             | `onboarding.joinVerifyPassphrase.errors.peerUnavailable` |
| **PairingConfirmStep**       | `PairingRejected`             | `onboarding.pairingConfirm.errors.rejected`              |

### High-level Toast Errors (OnboardingPage.tsx)

- `onboarding.page.errors.loadSetupStateFailed`
- `onboarding.page.errors.refreshPeersFailed`
- `onboarding.page.errors.operationFailed`
- `onboarding.page.errors.completeSetupFailed`

## Setup Flow Cancellation and Rollback Mapping

### Setup State Machine (uc-core)

The `SetupEvent::CancelSetup` event drives the cancellation flow in the setup state machine.

| Current State                | Next State              | Actions        |
| :--------------------------- | :---------------------- | :------------- |
| `CreateSpaceInputPassphrase` | `Welcome`               | None           |
| `JoinSpaceSelectDevice`      | `Welcome`               | None           |
| `JoinSpaceConfirmPeer`       | `JoinSpaceSelectDevice` | `AbortPairing` |
| `JoinSpaceInputPassphrase`   | `JoinSpaceSelectDevice` | None           |
| `ProcessingJoinSpace`        | `Welcome`               | `AbortPairing` |
| `ProcessingCreateSpace`      | `Welcome`               | `AbortPairing` |
| `Completed`                  | `Completed`             | None           |
| _Invalid Transition_         | _Current State_         | `AbortPairing` |

**Rollback Trigger**: `SetupAction::AbortPairing` is the primary rollback action, ensuring that any active pairing session is terminated when the user cancels or an error occurs.

### Pairing State Machine (uc-core)

The `PairingEvent::UserCancel` event handles cancellation within a specific pairing session.

- **Transitions**: `AwaitingUserConfirm` or `AwaitingUserApproval` -> `Cancelled`.
- **Actions**:
  - `PairingAction::Send(PairingMessage::Cancel)`: Notifies the peer of the cancellation.
  - `PairingAction::CancelTimer`: Stops the timeout timer for the current step.
  - `PairingAction::LogTransition`: Records the cancellation for audit purposes.

### Space Access State Machine (uc-core)

The `SpaceAccessEvent::CancelledByUser` event handles cancellation during the encrypted space access protocol.

- **Transitions**: `WaitingOffer`, `WaitingUserPassphrase`, `WaitingDecision`, or `WaitingJoinerProof` -> `Cancelled`.
- **Actions**: `SpaceAccessAction::StopTimer`.

### Setup Orchestrator (uc-app)

The orchestrator coordinates these state machines.

- **Method**: `cancel_setup()` dispatches the cancellation event.
- **Note**: Currently, `uc-app` and `uc-core` are slightly out of sync regarding event names (`UserCancel` vs `CancelSetup`).

### Frontend Integration

- **File**: `src/pages/OnboardingPage.tsx`
- **Action**: `onCancel={() => handleDispatch('PairingUserCancel')}` triggers the cancellation flow.

## Build and Test Command Constraints

### Command Execution Locations

| Command Type       | Execution Directory | Example Commands                                                |
| ------------------ | ------------------- | --------------------------------------------------------------- |
| **Rust / Cargo**   | `src-tauri/`        | `cargo build`, `cargo test`, `cargo check`, `cargo clippy`      |
| **Frontend / Bun** | Project Root        | `bun install`, `bun tauri dev`, `bun run build`, `bun run test` |

**CRITICAL**: Never run `cargo` commands from the project root. Always `cd src-tauri` first.

### Tooling & Environment

- **Package Manager**: Bun (Required). Do not use npm or yarn.
- **Rust**: Stable toolchain.
- **Frontend**: React 18 + Vite + Vitest.
- **System Dependencies (Linux)**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`.

### Testing Strategy

- **Rust Tests**: Located in `src-tauri/tests/` or inline `#[cfg(test)]`. Run via `cargo test` in `src-tauri/`.
- **Frontend Tests**: Vitest. Run via `bun run test` in root.
- **Coverage**: `cargo-llvm-cov`. Run via `bun run test:coverage` in root (which handles the `cd src-tauri` internally).

### CI/CD Constraints (GitHub Actions)

- PRs trigger `pr-check.yml`:
  - Frontend: `bun install` -> `bun run build` (Type check).
  - Backend: `cargo check --all-targets --all-features` (in `src-tauri/`).
- Builds are cross-platform: macOS (ARM/Intel), Ubuntu 22.04, Windows.

### Coding Standards

- **Rust**: No `unwrap()`/`expect()` in production. Use `tracing` for structured logging.
- **Frontend**: No fixed-pixel layouts (use `rem` or Tailwind utilities). Test light/dark themes.

## Setup and Onboarding Runtime Mapping

### Accessor Methods (uc-tauri/src/bootstrap/runtime.rs)

| Method                    | Return Type              | Dependencies (from AppDeps)                              |
| :------------------------ | :----------------------- | :------------------------------------------------------- |
| `initialize_onboarding()` | `InitializeOnboarding`   | `onboarding_state`, `encryption_state`                   |
| `get_onboarding_state()`  | `GetOnboardingState`     | `onboarding_state`, `encryption_state`                   |
| `complete_onboarding()`   | `CompleteOnboarding`     | `onboarding_state`                                       |
| `setup_orchestrator()`    | `Arc<SetupOrchestrator>` | `initialize_encryption` (UC), `complete_onboarding` (UC) |

### SetupOrchestrator Construction (uc-app/src/usecases/setup/orchestrator.rs)

The `SetupOrchestrator` coordinates the setup state machine and side effects. It is constructed in `UseCases::setup_orchestrator` by injecting other use cases as "capabilities":

- **initialize_encryption**: Used for `SetupAction::CreateEncryptedSpace`.
  - Deps: `encryption`, `key_material`, `key_scope`, `encryption_state`, `encryption_session`.
- **complete_onboarding**: Used for `SetupAction::MarkSetupComplete`.
  - Deps: `onboarding_state`.

### Dependency Wiring (uc-tauri/src/bootstrap/wiring.rs)

Concrete implementations wired into `AppDeps`:

- `onboarding_state`: `FileOnboardingStateRepository` (persists to `.onboarding_state` in vault).
- `encryption_state`: `FileEncryptionStateRepository` (persists to `encryption_state.json` in vault).
- `encryption`: `EncryptionRepository` (XChaCha20-Poly1305 implementation).
- `key_material`: `DefaultKeyMaterialService` (manages KEK in SecureStorage and MasterKey in KeySlot).
- `key_scope`: `DefaultKeyScope` (platform-specific key isolation).
- `encryption_session`: `InMemoryEncryptionSessionPort` (volatile storage for MasterKey).

### Trace Metadata & Observability

- Tauri commands in `uc-tauri/src/commands/onboarding.rs` (e.g., `get_onboarding_state`) use `record_trace_fields` to attach `trace_id` and `trace_ts` to spans.
- Frontend calls in `src/api/onboarding.ts` use `invokeWithTrace`.

### Testing References

- `uc-app/src/usecases/onboarding/complete.rs`: `test_execute_marks_onboarding_as_complete`
- `uc-app/src/usecases/onboarding/get_state.rs`: `test_execute_when_onboarding_completed`
- `uc-app/src/usecases/onboarding/initialize.rs`: `test_execute_when_onboarding_completed`
- `uc-app/src/usecases/setup/orchestrator.rs`: Contains logic for `new_space` and `join_space` flows.

## Frontend Store & Hooks Mapping (Setup/Onboarding)

### RTK Query API (`appApi`)

- **File**: `src/store/api.ts`
- **Base Query**: `fakeBaseQuery` (calls Tauri commands via `src/api/` functions)
- **Tag Types**:
  - `OnboardingStatus`: Tracks high-level onboarding completion.
  - `EncryptionStatus`: Tracks encryption session/vault status.

### RTK Query Hooks

| Hook                                 | Endpoint                     | Tag (Provides)     | Purpose                                                                  |
| :----------------------------------- | :--------------------------- | :----------------- | :----------------------------------------------------------------------- |
| `useGetOnboardingStatusQuery`        | `getOnboardingStatus`        | `OnboardingStatus` | Returns `has_completed`, `encryption_password_set`, `device_registered`. |
| `useGetEncryptionSessionStatusQuery` | `getEncryptionSessionStatus` | `EncryptionStatus` | Returns `is_unlocked`, `has_master_key`, etc.                            |

### Cache Invalidation

- **Mechanism**: `dispatch(appApi.util.invalidateTags(['OnboardingStatus']))`
- **Trigger**: Tauri events listened to in `OnboardingProvider` (`src/contexts/OnboardingContext.tsx`):
  - `onboarding-password-set`
  - `onboarding-completed`

### Direct Setup API (State Machine)

These functions are used directly in `OnboardingPage.tsx` and are not currently wrapped in RTK Query.

- **File**: `src/api/onboarding.ts`
- **Functions**:
  - `getSetupState()`: Fetches the current state of the setup state machine (`SetupState`).
  - `dispatchSetupEvent(event)`: Dispatches a `SetupEvent` and returns the next `SetupState`.

### Contexts

- **OnboardingContext**: (`src/contexts/OnboardingContext.tsx`)
  - Wraps `useGetOnboardingStatusQuery`.
  - Provides `status`, `loading`, `error`, and `refreshStatus` (which calls `refetch()`).
  - Used by `App.tsx` and onboarding components to determine if the user should be in the onboarding flow.

### Summary Table: Setup API Exposure

| API Type     | Source                  | Hook/Function                        | Usage                              |
| :----------- | :---------------------- | :----------------------------------- | :--------------------------------- |
| RTK Query    | `src/store/api.ts`      | `useGetOnboardingStatusQuery`        | `App.tsx`, `OnboardingContext.tsx` |
| RTK Query    | `src/store/api.ts`      | `useGetEncryptionSessionStatusQuery` | `App.tsx`                          |
| Direct Async | `src/api/onboarding.ts` | `getSetupState`                      | `OnboardingPage.tsx`               |
| Direct Async | `src/api/onboarding.ts` | `dispatchSetupEvent`                 | `OnboardingPage.tsx`               |

## Runtime SetupOrchestrator Caching (2026-02-05)

- Cached `SetupOrchestrator` in `uc-tauri` runtime via `OnceLock<Arc<SetupOrchestrator>>` on `AppRuntime` so repeated `usecases().setup_orchestrator()` calls share the same instance.

---

## SetupStatus to SetupState Mapping Patterns (2026-02-06)

### Current Gap Analysis

**Problem**: No seeding from persisted `SetupStatus` to initial `SetupState`.

| Component                          | Current Behavior               | Issue                     |
| :--------------------------------- | :----------------------------- | :------------------------ |
| `SetupContext::new(initial_state)` | Takes explicit `SetupState`    | No automatic seeding      |
| `SetupContext::default()`          | Returns `SetupState::Welcome`  | Always starts fresh       |
| `SetupOrchestrator::new()`         | Uses `SetupContext::default()` | Ignores persisted status  |
| `MarkSetupComplete`                | Writes `has_completed = true`  | But no read-back to state |

**Key Files**:

- `uc-core/src/setup/status.rs`: `SetupStatus { has_completed: bool }`
- `uc-core/src/setup/state.rs`: `SetupState` enum (includes `Completed`)
- `uc-app/src/usecases/setup/context.rs`: In-memory state storage
- `uc-app/src/usecases/setup/orchestrator.rs`: Constructor uses `SetupContext::default()`
- `uc-app/src/usecases/setup/mark_complete.rs`: Writes completion status

### Existing Pattern: GetOnboardingState

The `GetOnboardingState` use case (`uc-app/src/usecases/onboarding/get_state.rs`) provides a reference pattern:

```rust
pub struct GetOnboardingState {
    onboarding_state: Arc<dyn OnboardingStatePort>,
    encryption_state: Arc<dyn EncryptionStatePort>,
}

impl GetOnboardingState {
    pub async fn execute(&self) -> anyhow::Result<OnboardingStateDto> {
        let onboarding_state = self.onboarding_state.get_state().await?;
        let encryption_initialized = /* check encryption_state */;

        Ok(OnboardingStateDto {
            has_completed: onboarding_state.has_completed,
            encryption_password_set: encryption_initialized,
            device_registered: true,
        })
    }
}
```

This pattern reads from ports and maps to a DTO. A similar approach can be used for `SetupState`.

### Recommended Solution: GetSetupState Use Case

Create a `GetSetupState` use case that:

1. Reads `SetupStatus` from `SetupStatusPort`
2. Optionally checks additional state (encryption, device)
3. Maps to appropriate `SetupState`:

```rust
// Pseudocode for mapping logic
async fn get_initial_state(&self) -> SetupState {
    let status = self.setup_status.get_status().await.unwrap_or_default();

    if status.has_completed {
        SetupState::Completed
    } else {
        // Check other conditions (encryption initialized, device registered)
        // For now, default to Welcome
        SetupState::Welcome
    }
}
```

### Alternative: SetupContext Factory with Seeding

Modify `SetupContext` to support seeding from status:

```rust
impl SetupContext {
    /// Create with explicit state (existing)
    pub fn new(initial_state: SetupState) -> Self { ... }

    /// Create from persisted status (NEW)
    pub async fn from_setup_status(
        setup_status: &dyn SetupStatusPort,
    ) -> anyhow::Result<Self> {
        let status = setup_status.get_status().await?;
        let initial_state = if status.has_completed {
            SetupState::Completed
        } else {
            SetupState::Welcome
        };
        Ok(Self::new(initial_state))
    }
}
```

### Integration Point: SetupOrchestrator Constructor

Current signature mismatch in `runtime.rs:466`:

```rust
// runtime.rs passes setup_status but orchestrator doesn't accept it
Arc::new(uc_app::usecases::SetupOrchestrator::new(
    Arc::new(self.runtime.usecases().initialize_encryption()),
    Arc::new(self.runtime.usecases().complete_onboarding()),
    self.runtime.deps.setup_status.clone(),  // Passed but not used!
))
```

**Fix options**:

1. **Option A**: Update `SetupOrchestrator::new()` to accept `SetupStatusPort` and create seeded context:

   ```rust
   pub fn new(
       initialize_encryption: Arc<InitializeEncryption>,
       complete_onboarding: Arc<CompleteOnboarding>,
       setup_status: Arc<dyn SetupStatusPort>,
   ) -> Self {
       let context = SetupContext::from_setup_status(&*setup_status);
       Self { context, ... }
   }
   ```

2. **Option B**: Keep orchestrator simple, create seeded context in `runtime.rs`:
   ```rust
   let setup_context = SetupContext::from_setup_status(&runtime.deps.setup_status).await;
   let orchestrator = SetupOrchestrator::new(
       initialize_encryption,
       complete_onboarding,
       setup_context,
   );
   ```

### Minimal Change: GetSetupState Use Case

Given the constraint to minimize changes, the cleanest approach is:

1. **Add `GetSetupState` use case** (reads `SetupStatusPort`, returns `SetupState`)
2. **Modify `SetupOrchestrator::get_state()`** to call the use case
3. **Keep orchestrator constructor unchanged** (still uses `Welcome` for new orchestrators)

```rust
// In orchestrator.rs
pub async fn get_state(&self) -> SetupState {
    // For now, return cached state
    // Future: check if should seed from SetupStatus
    self.context.get_state().await
}
```

### Next Steps for Implementation

1. Create `uc-app/src/usecases/setup/get_state.rs` with `GetSetupState` use case
2. Add `from_ports` constructor pattern (per existing conventions)
3. Wire in `runtime.rs` accessor method
4. Update `SetupOrchestrator::get_state()` to optionally check completion status
5. Add table-driven tests for seeding scenarios

### Alignment with uc-core/uc-app Boundaries

- **Ports stay in uc-core**: `SetupStatusPort` definition remains in `uc-core`
- **Use cases in uc-app**: `GetSetupState` belongs in `uc-app/src/usecases/setup/`
- **No business logic in wiring**: `bootstrap/wiring.rs` only assembles dependencies
- **Orchestrator unchanged**: `SetupOrchestrator` remains the coordination layer

- 2026-02-06: Updated uc-infra TimerPort implementation to return Result<()> and adjusted in-file tests to assert timer cleanup via internal map (no oneshot receiver).
  2026-02-06: Added AppDeps.setup_status wiring; bootstrap uses FileSetupStatusRepository::with_defaults(vault_path.clone()), commands/tests use NoopPort. Also exposed SetupStatusPort in uc-core ports mod and SetupStatus in uc-core setup/mod so deps can import it.
  2026-02-06: Removed is_encryption_initialized use case/command/registration and cleaned active docs; rg check in src-tauri/src/docs (excluding archive) returned no matches.

- 2026-02-06: Frontend gating now uses setup state (SetupPage until Completed) with TitleBar hidden during setup; onboarding API/context/pages/tests removed and app API no longer exposes onboarding status.
- 2026-02-06: Removed onboarding backend modules (uc-core/uc-app/uc-infra/uc-tauri) and wiring/commands; removed onboarding i18n namespaces; rg verification shows remaining onboarding only in src-tauri/src-legacy.
