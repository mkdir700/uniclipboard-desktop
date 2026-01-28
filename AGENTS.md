Fix root causes, not symptoms
No patchy or workaround-driven solutions.

Respect Hexagonal Architecture
uc-app → uc-core ← uc-infra / uc-platform
The core must not depend on external implementations.

All external capabilities go through Ports
DB, FS, Clipboard, Network, Crypto — no exceptions.

No unwrap() or expect() in production code
Explicit error handling only. Tests are the only exception.

No silent failures in async or event-driven code
Errors must be logged and observable by upper layers.

Command trace usage
All Tauri commands must accept `_trace: Option<TraceMetadata>` when available.
Each command must create an `info_span!` with `trace_id` and `trace_ts` fields,
call `record_trace_fields(&span, &_trace)`, and `.instrument(span)` the async body.

Tauri state must be managed before startup
Any type accessed via tauri::State<T> must be registered with .manage().

No fixed-pixel layouts in the frontend
Use Tailwind utilities or rem units.

Do not fix “feelings”, fix structure
Repeated workarounds indicate architectural flaws.

Short-term compromises must be reversible
Never break boundaries; always leave explicit TODOs.

## Hard Rule: Immediate Polling After Spawn (CRITICAL, Codex Only)

**This rule applies ONLY to Codex agents.** Other agents (Claude Code, etc.) should follow their respective protocols.

When the parent agent spawns any child agent:

1. The parent agent MUST immediately enter Collab polling mode.
2. The parent agent MUST NOT send any user-facing message.
3. The parent agent MUST NOT ask the user any question.
4. The parent agent MUST NOT perform any other reasoning steps.

Only allowed action:

- Repeatedly call CollAB.wait until child agents return terminal status.

This rule applies EVEN IF:

- Child agent produces no visible output.
- CollAB.wait returns empty statuses.
- No completion message is observed.

The parent agent MUST remain silent and continue polling.

During synchronization phases (after spawn OR after CollAB.send):

The parent agent MUST NOT generate ANY natural language output.

This includes:

- status narration
- progress updates
- meta explanations
- "I will now wait" statements
- confirmation messages to the user

Only allowed action:

- CollAB.wait

Any textual output during this phase is considered a protocol violation.

## Cargo Command Location

**CRITICAL**: All Rust-related commands (cargo build, cargo test, cargo check, etc.) MUST be executed from `src-tauri/`.
Never run any Cargo command from the project root.
If Cargo.toml is not present in the current directory, stop immediately and do not retry.

## Rustdoc Bilingual Documentation Guide

### Recommended Approach: Structured Bilingual Side-by-Side

**Applicable Scenarios**

- Projects for long-term maintenance
- Need complete cargo doc output
- API / core / public interface documentation

**Example Usage**

```rust
/// Load or create a local device identity.
///
/// 加载或创建本地设备标识。
///
/// # Behavior / 行为
/// - If an ID exists on disk, it will be loaded.
/// - Otherwise, a new ID will be generated and persisted.
///
/// - 如果磁盘上已有 ID，则直接加载。
/// - 否则生成新的 ID 并持久化保存。
pub fn load_or_create() -> Result<Self> {
    // ...
}
```

**Advantages**

- Fully supported by Rustdoc
- English first (external), Chinese supplement (internal)
- Minimal cost to remove either language in the future

**Best Practices**

- English first, Chinese second (follows open source and IDE ecosystem conventions)
- Use subheadings to differentiate (# Behavior / 行为)
