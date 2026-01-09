# clipboard_probe

Probe tool for raw clipboard events and snapshots across macOS / Windows / Linux.

## What it verifies

- Clipboard watcher fires on changes
- `read_snapshot()` returns full `RawClipboardSnapshot` representations
- Repeated copies of identical content still emit events (or not)
- Raw event cadence before any debounce/throttle in higher layers

## Usage

From `src-tauri/`:

```bash
cargo run -p uc-platform --example clipboard_probe
```

Optional:

```bash
cargo run -p uc-platform --example clipboard_probe -- --max-events 10
```

## Manual checklist

1. Copy plain text, rich text, HTML, image, and files.
2. Confirm multiple representations appear for rich text / HTML where expected.
3. Copy the same content repeatedly and note `same_content_streak` and event count.
4. Copy rapidly to observe `delta_ms` between events (raw cadence).
