# Startup Loading Animation Design

**Date**: 2026-01-17
**Status**: Design
**Author**: Claude

## Overview

Implement a startup loading animation to improve user experience during backend initialization. The application currently has a noticeable delay during startup due to asynchronous backend initialization (device name initialization, encryption auto-unlock, clipboard watcher startup). This design adds a smooth loading screen with fade-out transition.

## Problem Statement

- **Current behavior**: Blank screen during backend initialization
- **Root cause**: Backend initialization takes time (encryption unlock, clipboard watcher startup)
- **User impact**: Poor perceived performance, no feedback during startup

## Solution: Event-Driven Loading Animation

### Architecture

Use Tauri's event system for efficient, real-time communication:

```
┌─────────────┐         ┌─────────────┐
│   Frontend  │         │   Backend   │
├─────────────┤         ├─────────────┤
│   Show      │◄────────│  Initialize │
│  Loading    │  listen │  services   │
│   Screen    │         │   (async)   │
│             │         │             │
│   Fade      │◄────────│  emit       │
│   Out       │  ready  │ backend-    │
│   (300ms)   │         │   ready     │
└─────────────┘         └─────────────┘
```

### Why Event-Driven Over Polling?

- ✅ **Efficient** - No continuous polling overhead
- ✅ **Real-time** - Immediate notification when backend is ready
- ✅ **Best practice** - Tauri's recommended pattern for frontend-backend communication
- ✅ **Cleaner code** - No need to manage polling intervals and cleanup

## Implementation

### 1. LoadingScreen Component

**File**: `src/components/LoadingScreen.tsx`

```tsx
import { useTranslation } from 'react-i18next'

interface LoadingScreenProps {
  className?: string
}

export const LoadingScreen: React.FC<LoadingScreenProps> = ({ className = '' }) => {
  const { t } = useTranslation()

  return (
    <div
      className={`h-screen w-screen flex flex-col items-center justify-center bg-background ${className}`}
    >
      {/* Logo with pulse animation */}
      <div className="animate-pulse opacity-70">
        <Logo className="w-20 h-20" />
      </div>

      {/* Status text */}
      <div className="mt-8 text-sm text-muted-foreground">{t('loading.initializing')}</div>
    </div>
  )
}
```

**Key points**:

- Uses Tailwind's `animate-pulse` for breathing animation
- Follows project theme system (`bg-background`, `text-muted-foreground`)
- Logo size `w-20 h-20` (5rem = 80px), avoids fixed pixels
- Supports className prop for fade-out transition

### 2. Backend Event Emission

**File**: `src-tauri/src/main.rs`

Add event emission after all initialization is complete:

```rust
// In the async runtime spawn block, after platform_runtime.start().await
// After all initialization:
// 1. Device name initialized
// 2. Encryption auto-unlock completed
// 3. Clipboard watcher started (if applicable)
// 4. Platform runtime started

if let Some(app) = runtime_for_handler.app_handle().as_ref() {
    let _ = app.emit("backend-ready", ());
}
```

**Placement**: After `platform_runtime.start().await` in the setup block's async task.

### 3. Frontend State Management

**File**: `src/App.tsx`

```tsx
import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { LoadingScreen } from '@/components/LoadingScreen'

const AppContent = () => {
  const { status, loading } = useOnboarding()
  const [backendReady, setBackendReady] = useState(false)
  const [fadingOut, setFadingOut] = useState(false)
  const [initError, setInitError] = useState<string | null>(null)
  const { t } = useTranslation()

  useEffect(() => {
    // Timeout protection (30 seconds)
    const timeoutId = setTimeout(() => {
      if (!backendReady && !fadingOut) {
        setInitError(t('loading.timeout_error'))
      }
    }, 30000)

    // Listen for backend-ready event
    const unlistenPromise = listen('backend-ready', () => {
      clearTimeout(timeoutId)

      // Trigger fade-out animation first
      setFadingOut(true)

      // Switch to main app after fade-out completes
      setTimeout(() => {
        setBackendReady(true)
      }, 300)
    })

    return () => {
      clearTimeout(timeoutId)
      unlistenPromise.then(unlisten => unlisten())
    }
  }, [t])

  // Show error screen if initialization failed
  if (initError) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="text-destructive mb-4">{t('loading.error_title')}</div>
          <div className="text-muted-foreground text-sm">{initError}</div>
        </div>
      </div>
    )
  }

  // Show loading screen if backend not ready
  if (!backendReady) {
    return (
      <LoadingScreen className={fadingOut ? 'opacity-0 transition-opacity duration-300' : ''} />
    )
  }

  // Original onboarding logic...
  if (loading || status === null) {
    return null
  }

  if (!status.has_completed) {
    return <OnboardingPage />
  }

  return (
    <ShortcutProvider>
      <P2PProvider>
        <SettingProvider>
          <GlobalOverlays />
          <Routes>{/* existing routes... */}</Routes>
          <Toaster />
        </SettingProvider>
      </P2PProvider>
    </ShortcutProvider>
  )
}
```

### 4. i18n Support

**Files**: `src/i18n/locales/en.json`, `src/i18n/locales/zh-CN.json`, etc.

Add translation keys:

```json
{
  "loading": {
    "initializing": "Initializing...",
    "error_title": "Initialization Failed",
    "timeout_error": "Initialization timeout. Please restart the application."
  }
}
```

Chinese (zh-CN):

```json
{
  "loading": {
    "initializing": "正在初始化...",
    "error_title": "初始化失败",
    "timeout_error": "初始化超时，请重启应用"
  }
}
```

### 5. Export Update

**File**: `src/components/index.ts`

```tsx
export { LoadingScreen } from './LoadingScreen'
```

## Startup Flow

```
1. App Launch
   ↓
2. Frontend: Show LoadingScreen immediately
   ↓
3. Backend: Async initialization begins
   - Device name initialization
   - Encryption auto-unlock
   - Clipboard watcher startup
   - Platform runtime start
   ↓
4. Backend: Emit 'backend-ready' event
   ↓
5. Frontend: Receive event, trigger fade-out (300ms)
   ↓
6. Frontend: Switch to main app
   - Onboarding (if first run)
   - Dashboard (if returning user)
```

## Error Handling

### Timeout Protection

- **Duration**: 30 seconds
- **Action**: Show error message with i18n support
- **User recovery**: Restart application

### Existing Error Handling

- Backend already emits error events (e.g., `encryption-auto-unlock-error`)
- These errors continue to work via toast notifications after loading completes

## Transition Animation

**Type**: Fade-out
**Duration**: 300ms
**Implementation**: CSS transition with React state

```tsx
className={fadingOut ? 'opacity-0 transition-opacity duration-300' : ''}
```

**Why fade-out?**

- Clean and elegant
- Not distracting
- Simple implementation
- Short duration for quick app entry
- Good performance

## Testing Strategy

### Manual Testing Scenarios

1. **Normal startup**: Verify loading animation appears and fades out smoothly
2. **Slow startup**: Simulate delayed backend initialization, verify animation persists
3. **Timeout**: Simulate backend never emitting event, verify error appears after 30s
4. **Cross-platform**: Test on macOS, Windows, Linux for style consistency

### Commands

```bash
# Development testing
bun tauri dev

# Production build testing (real startup speed)
bun tauri build
```

## Files Summary

| File                               | Action                                             |
| ---------------------------------- | -------------------------------------------------- |
| `src/components/LoadingScreen.tsx` | Create                                             |
| `src/components/index.ts`          | Update (add export)                                |
| `src/App.tsx`                      | Update (add state, event listener, error handling) |
| `src/i18n/locales/*.json`          | Update (add translation keys)                      |
| `src-tauri/src/main.rs`            | Update (emit backend-ready event)                  |

## Success Criteria

- ✅ Loading screen displays immediately on app launch
- ✅ Loading screen fades out smoothly (300ms) when backend is ready
- ✅ Error message displays after 30-second timeout
- ✅ All text supports i18n (English, Chinese)
- ✅ No visual jank or flickering during transition
- ✅ Works across all supported platforms (macOS, Windows, Linux)
