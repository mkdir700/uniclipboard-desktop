# Startup Loading Animation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a startup loading animation with smooth fade-out transition to improve user experience during backend initialization.

**Architecture:** Event-driven architecture using Tauri's event system. Backend emits `backend-ready` event when initialization completes; frontend listens and transitions from loading screen to main app with a 300ms fade-out animation.

**Tech Stack:** React 18, Tauri 2, TypeScript, Tailwind CSS, i18next

---

## Task 1: Add i18n Translation Keys

**Files:**

- Modify: `src/i18n/locales/en-US.json`
- Modify: `src/i18n/locales/zh-CN.json`

**Step 1: Add translation keys to en-US.json**

Add this to the root level of `src/i18n/locales/en-US.json` (after `common` section):

```json
"loading": {
  "initializing": "Initializing...",
  "error_title": "Initialization Failed",
  "timeout_error": "Initialization timeout. Please restart the application."
}
```

**Step 2: Add translation keys to zh-CN.json**

Add this to the root level of `src/i18n/locales/zh-CN.json` (after `common` section):

```json
"loading": {
  "initializing": "正在初始化...",
  "error_title": "初始化失败",
  "timeout_error": "初始化超时，请重启应用"
}
```

**Step 3: Verify JSON syntax**

Run: `cat src/i18n/locales/en-US.json | jq .`
Expected: No syntax errors, JSON is valid

Run: `cat src/i18n/locales/zh-CN.json | jq .`
Expected: No syntax errors, JSON is valid

**Step 4: Commit**

```bash
git add src/i18n/locales/en-US.json src/i18n/locales/zh-CN.json
git commit -m "feat(i18n): add loading screen translation keys

Add translation keys for startup loading animation:
- loading.initializing: Status text during initialization
- loading.error_title: Error title if initialization fails
- loading.timeout_error: Timeout error message (30s)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 2: Create LoadingScreen Component

**Files:**

- Create: `src/components/LoadingScreen.tsx`
- Modify: `src/components/index.ts`

**Step 1: Create LoadingScreen.tsx**

Create file `src/components/LoadingScreen.tsx` with:

```tsx
import { useTranslation } from 'react-i18next'

// Placeholder logo component - replace with actual Logo if available
const Logo = () => (
  <div className="w-20 h-20 bg-muted rounded-lg flex items-center justify-center">
    <span className="text-2xl font-bold">UC</span>
  </div>
)

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
        <Logo />
      </div>

      {/* Status text */}
      <div className="mt-8 text-sm text-muted-foreground">{t('loading.initializing')}</div>
    </div>
  )
}
```

**Note:** The Logo component is a placeholder. If the project has an existing Logo component, replace it with the actual import.

**Step 2: Export LoadingScreen from index.ts**

Add to `src/components/index.ts`:

```tsx
// 导出所有组件
// Layout 组件
export * from './layout'

// Clipboard 组件
export * from './clipboard'

// 设备管理组件
export * from './device'

// 设置组件
export * from './setting'

// UI 组件 (shadcn)
export * from './ui'
export * from './TitleBar'

// Loading 组件
export { LoadingScreen } from './LoadingScreen'
```

**Step 3: Verify TypeScript compilation**

Run: `bun run build`
Expected: No TypeScript errors

**Step 4: Commit**

```bash
git add src/components/LoadingScreen.tsx src/components/index.ts
git commit -m "feat(components): add LoadingScreen component

Add startup loading screen component with:
- Animated logo with pulse effect
- i18n support for status text
- className prop for transition animations
- Responsive layout using Tailwind utilities

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 3: Backend - Emit backend-ready Event

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Add backend-ready event emission**

In `src-tauri/src/main.rs`, find the async runtime spawn block in the setup function.

Locate the line: `platform_runtime.start().await;`

Add the event emission AFTER that line, still inside the async spawn block:

```rust
platform_runtime.start().await;

// Emit backend-ready event to notify frontend
if let Some(app) = runtime_for_unlock.app_handle().as_ref() {
    if let Err(e) = app.emit("backend-ready", ()) {
        log::error!("Failed to emit backend-ready event: {}", e);
    }
}

log::info!("Platform runtime task ended");
```

**Step 2: Verify Rust compilation**

Run: `cd src-tauri && cargo check`
Expected: No compilation errors

**Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(backend): emit backend-ready event on initialization complete

Emit 'backend-ready' event after all async initialization completes:
- Device name initialized
- Encryption auto-unlock completed
- Clipboard watcher started
- Platform runtime started

This event triggers the frontend loading screen fade-out transition.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 4: Frontend - Add Loading State to App.tsx

**Files:**

- Modify: `src/App.tsx`

**Step 1: Add imports to App.tsx**

Add these imports at the top of `src/App.tsx`:

```tsx
import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { LoadingScreen } from '@/components/LoadingScreen'
```

**Step 2: Add state management to AppContent component**

Modify the `AppContent` component to add loading state:

```tsx
// 主应用程序内容
const AppContent = () => {
  const { status, loading } = useOnboarding()
  const { t } = useTranslation()

  // Backend loading state
  const [backendReady, setBackendReady] = useState(false)
  const [fadingOut, setFadingOut] = useState(false)
  const [initError, setInitError] = useState<string | null>(null)

  // ... rest of component
```

**Step 3: Add useEffect for backend-ready event listener**

Add this useEffect after the state declarations, before the render logic:

```tsx
// Listen for backend-ready event
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
```

**Step 4: Add error screen rendering**

Add this AFTER the useEffect, BEFORE the existing `if (loading || status === null)` check:

```tsx
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
  return <LoadingScreen className={fadingOut ? 'opacity-0 transition-opacity duration-300' : ''} />
}
```

**Step 5: Verify TypeScript compilation**

Run: `bun run build`
Expected: No TypeScript errors

**Step 6: Commit**

```bash
git add src/App.tsx
git commit -m "feat(frontend): add loading screen with fade-out transition

Add startup loading experience:
- Listen for backend-ready event
- Show LoadingScreen with animated logo
- 300ms fade-out transition when backend ready
- 30-second timeout with error message
- Full i18n support for all text

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 5: Replace Placeholder Logo (Optional)

**Files:**

- Modify: `src/components/LoadingScreen.tsx`

**Step 1: Check if Logo component exists**

Search for existing Logo component:

```bash
grep -r "export.*Logo" src/components/ --include="*.tsx" --include="*.ts"
```

**Step 2a: If Logo exists, update LoadingScreen.tsx**

If found, replace the placeholder Logo with:

```tsx
import { Logo } from '@/components/Logo' // or appropriate path
```

And remove the placeholder Logo definition.

**Step 2b: If Logo doesn't exist, keep placeholder**

If no Logo component found, keep the placeholder. The placeholder shows "UC" (UniClipboard) in a styled box.

**Step 3: Commit (if changed)**

```bash
git add src/components/LoadingScreen.tsx
git commit -m "refactor(components): use actual Logo in LoadingScreen

Replace placeholder logo with project Logo component.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Task 6: Manual Testing

**Files:** None (testing only)

**Step 1: Test normal startup**

Run: `bun tauri dev`

Expected behavior:

1. Loading screen appears immediately
2. Logo pulses with animation
3. "正在初始化..." text displays
4. After backend initialization (~2-5 seconds), loading screen fades out
5. Main app appears (Onboarding or Dashboard)

**Step 2: Test error handling**

To test timeout, temporarily modify the timeout to 5 seconds in `src/App.tsx`:

Change: `}, 30000)` to `}, 5000)`

Run: `bun tauri dev`

Expected: Error message appears after 5 seconds

**Step 3: Restore timeout**

Change back to 30000ms

**Step 4: Test cross-platform (if possible)**

Build and test on different platforms:

```bash
bun tauri build
```

Verify:

- Loading animation works smoothly
- No visual jank or flickering
- Theme colors work in both light and dark mode

**Step 5: Commit any fixes**

```bash
git add -A
git commit -m "fix: address issues found during manual testing

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Success Criteria Verification

After completing all tasks, verify:

- ✅ Loading screen displays immediately on app launch
- ✅ Logo animates with pulse effect
- ✅ Status text displays in correct language (i18n)
- ✅ Loading screen fades out smoothly (300ms) when backend is ready
- ✅ No visual jank or flickering during transition
- ✅ Error message displays after 30-second timeout
- ✅ Works across all supported platforms (macOS, Windows, Linux)

---

## References

- Design document: `docs/plans/2026-01-17-startup-loading-animation-design.md`
- Project architecture: See CLAUDE.md for project structure
- Tauri event system: https://tauri.app/v2/api/core/event/
- i18next: https://www.i18next.com/
