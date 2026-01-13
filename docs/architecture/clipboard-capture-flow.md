# Clipboard Capture Flow

## Overview

This document describes the automatic clipboard capture flow, which integrates platform-level clipboard monitoring with application-level business logic.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ System Clipboard │
└─────────────────────────────────────────────────────────────┘
↓ changes
┌─────────────────────────────────────────────────────────────┐
│ Platform Layer (uc-platform) │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ ClipboardWatcher │ │
│ │ - Monitors system clipboard │ │
│ │ - Calls on_clipboard_change() │ │
│ └────────────────────────────────────────────────────────┘ │
│ ↓ │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ PlatformRuntime │ │
│ │ - Receives ClipboardChanged event │ │
│ │ - Calls clipboard_handler.on_clipboard_changed() │ │
│ └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
↓ via trait callback
┌─────────────────────────────────────────────────────────────┐
│ App Layer (uc-app) │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ AppRuntime (implements ClipboardChangeHandler) │ │
│ │ - on_clipboard_changed() is called │ │
│ │ - Creates CaptureClipboardUseCase │ │
│ └────────────────────────────────────────────────────────┘ │
│ ↓ │
│ ┌────────────────────────────────────────────────────────┐ │
│ │ CaptureClipboardUseCase │ │
│ │ - execute_with_snapshot(snapshot) │ │
│ │ - Persists event and representations │ │
│ │ - Creates ClipboardEntry │ │
│ └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Callback Pattern (Push vs Pull)

**Chosen:** Push - Platform pushes changes to App via callback

**Rationale:**

- Platform is the authority on when clipboard changes
- Avoids polling overhead
- Event-driven architecture aligns with Rust async patterns

### 2. Snapshot Parameter Passing

**Chosen:** Platform reads snapshot once, passes to App

**Rationale:**

- Avoids redundant system calls
- Snapshot represents the "fact" of what changed
- App layer doesn't need platform clipboard access for capture

### 3. Trait Object Callback

**Chosen:** `Arc<dyn ClipboardChangeHandler>`

**Rationale:**

- Maintains dependency inversion (Platform depends on abstraction)
- Allows App layer to implement without Platform knowing about it
- Thread-safe with Arc for async context

## Error Handling

1. **ClipboardWatcher** - Logs errors, continues monitoring
2. **PlatformRuntime** - Catches callback errors, logs but doesn't panic
3. **AppRuntime** - Returns error from usecase, logged by PlatformRuntime
4. **CaptureClipboardUseCase** - Returns Result, errors propagated up

## Testing

- **Unit tests:** Individual components (ClipboardWatcher, UseCase)
- **Integration test:** Callback trait implementation
- **Manual test:** Run app, copy content, verify entries created
