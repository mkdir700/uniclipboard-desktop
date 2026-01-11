# clipboard_probe

Clipboard probing and snapshot tool for observing and verifying clipboard read/write behavior across macOS / Windows / Linux.

## Features

- **watch**: Monitor clipboard changes in real-time (original functionality)
- **capture**: Save current clipboard content to a JSON file
- **restore**: Restore clipboard content from a JSON file
- **inspect**: View snapshot file contents without modifying clipboard

## What it verifies

- Clipboard watcher fires on changes
- `read_snapshot()` returns full `SystemClipboardSnapshot` representations
- Snapshot serialization/deserialization preserves data correctly
- Restored snapshots behave identically to original clipboard content
- Repeated copies of identical content still emit events (or not)
- Raw event cadence before any debounce/throttle in higher layers

## Usage

From `src-tauri/`:

```bash
# Watch mode (default behavior)
cargo run -p uc-platform --example clipboard_probe -- watch

# Watch with event limit
cargo run -p uc-platform --example clipboard_probe -- watch --max-events 10

# Capture current clipboard
cargo run -p uc-platform --example clipboard_probe -- capture --out snapshot.json

# Restore from file
cargo run -p uc-platform --example clipboard_probe -- restore --in snapshot.json

# Restore with specific representation (for multi-representation snapshots)
cargo run -p uc-platform --example clipboard_probe -- restore --in snapshot.json --select 1

# Inspect snapshot without modifying clipboard
cargo run -p uc-platform --example clipboard_probe -- inspect --in snapshot.json
```

## Commands

### watch

Monitor clipboard changes in real-time.

```bash
cargo run -p uc-platform --example clipboard_probe -- watch [--max-events N]
```

**Options**:

- `--max-events N`: Stop after N events (optional)
- `Ctrl+C`: Stop monitoring

### capture

Save current clipboard content to a JSON file.

```bash
cargo run -p uc-platform --example clipboard_probe -- capture --out FILE
```

**Options**:

- `--out FILE`: Output file path (required)

### restore

Restore clipboard content from a JSON file.

```bash
cargo run -p uc-platform --example clipboard_probe -- restore --in FILE [--select INDEX]
```

**Options**:

- `--in FILE`: Input file path (required)
- `--select INDEX`: Select representation by index (0-based, optional)

**Note**: Current implementation only supports single representation restore. If snapshot has multiple representations, use `--select` to choose which one to restore.

### inspect

View snapshot file contents without modifying clipboard.

```bash
cargo run -p uc-platform --example clipboard_probe -- inspect --in FILE
```

**Options**:

- `--in FILE`: Input file path (required)

## Manual checklist

### Watch mode

1. Copy plain text, rich text, HTML, image, and files.
2. Confirm multiple representations appear for rich text / HTML where expected.
3. Copy the same content repeatedly and note `same_content_streak` and event count.
4. Copy rapidly to observe `delta_ms` between events (raw cadence).

### Capture/Restore workflow

1. Copy some content (text, image, etc.).
2. Run `capture --out test.json`.
3. Verify the JSON file is created and contains expected data.
4. Clear clipboard (copy empty text or restart).
5. Run `restore --in test.json`.
6. Paste and verify content matches original.
7. Run `inspect --in test.json` to view snapshot metadata.

### Multi-representation testing

1. Copy rich text from a document editor (Word, Pages, etc.).
2. Capture snapshot.
3. Run `inspect` to see all representations.
4. Try restoring different representations using `--select`.
5. Verify paste behavior differs based on selected representation.
