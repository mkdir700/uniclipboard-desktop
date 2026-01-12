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
