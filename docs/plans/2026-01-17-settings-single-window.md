# Settings Single Window Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert settings page from standalone Tauri window to single-window full-screen overlay using React Router navigation.

**Architecture:** Remove Tauri `open_settings_window` command, enable React Router navigation from sidebar settings button, add back button in TitleBar for settings page, implement ESC key listener for navigation.

**Tech Stack:** React Router v7, Tauri 2, TypeScript, Rust, Framer Motion

---

## Overview

This plan migrates the settings experience from a multi-window architecture (main window + settings window) to a single-window architecture where settings occupy the full main window. This aligns with modern single-page application patterns and reduces window management complexity.

**Key Changes:**

- Remove `open_settings_window` Tauri command and associated use case
- Modify Sidebar to use standard React Router navigation instead of `openSettingsWindow()`
- Add back button to TitleBar when on settings page
- Add ESC key listener to SettingsPage for quick navigation back
- Clean up unused code

---

## Phase 1: Frontend Router Changes (Primary Implementation)

### Task 1: Modify Sidebar to use React Router navigation

**Files:**

- Modify: `src/components/layout/Sidebar.tsx`

**Step 1: Remove the `handleSettingsClick` function**

Delete lines 65-71 which contain:

```typescript
const handleSettingsClick = (e: React.MouseEvent) => {
  e.preventDefault()
  openSettingsWindow().catch(err => {
    console.error('Failed to open settings window:', err)
  })
}
```

**Step 2: Remove the `openSettingsWindow` import**

Delete line 6:

```typescript
import { openSettingsWindow } from '@/api/window'
```

**Step 3: Remove `onClick` prop from settings NavButton**

Modify line 99-106, removing the `onClick={handleSettingsClick}` prop:

```typescript
<NavButton
  to="/settings"
  icon={Settings}
  label={t('nav.settings')}
  isActive={false}
  layoutId="sidebar-nav-bottom"
/>
```

**Step 4: Verify the changes**

Check that the file compiles and the Link will now navigate to `/settings` via React Router.

**Step 5: Commit**

```bash
git add src/components/layout/Sidebar.tsx
git commit -m "feat: use React Router navigation for settings instead of new window"
```

---

### Task 2: Add back button and title to TitleBar for settings page

**Files:**

- Modify: `src/components/TitleBar.tsx`

**Step 1: Read current TitleBar implementation**

First, read the file to understand the current structure:

```bash
cat src/components/TitleBar.tsx
```

**Step 2: Add imports for navigation components**

Add these imports if not already present:

```typescript
import { useLocation, useNavigate } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'
```

**Step 3: Add navigation hooks inside TitleBar component**

Add after existing hooks:

```typescript
const TitleBar: React.FC<TitleBarProps> = ({ searchValue, onSearchChange }) => {
  const location = useLocation()
  const navigate = useNavigate()
  const isSettingsPage = location.pathname === '/settings'
  // ... existing code
```

**Step 4: Add `handleBack` function**

Add before the return statement:

```typescript
const handleBack = () => {
  // Check if there's history to go back to
  if (window.history.state && window.history.state.idx > 0) {
    navigate(-1)
  } else {
    navigate('/')
  }
}
```

**Step 5: Modify the left section of TitleBar to conditionally render**

Locate the left section (likely containing the search input or empty div) and modify:

```typescript
      {/* Left Section */}
      <div className="flex items-center gap-2" data-tauri-drag-region>
        {isSettingsPage ? (
          <button
            onClick={handleBack}
            className="flex items-center gap-2 text-muted-foreground hover:text-foreground transition-colors px-2 py-1 rounded-md hover:bg-muted/50"
            data-tauri-drag-region="false"
          >
            <ArrowLeft className="w-4 h-4" />
            <span className="font-medium text-sm">Settings</span>
          </button>
        ) : location.pathname === '/' ? (
          // Existing search input for dashboard
          <div className="relative">
            {/* ... existing search input code ... */}
          </div>
        ) : null}
      </div>
```

**Step 6: Test the navigation**

```bash
bun run dev
```

- Click the settings button in sidebar
- Verify the back button appears in TitleBar
- Click the back button and verify it returns to the previous page

**Step 7: Commit**

```bash
git add src/components/TitleBar.tsx
git commit -m "feat: add back button to TitleBar for settings page"
```

---

### Task 3: Add ESC key listener to SettingsPage

**Files:**

- Modify: `src/pages/SettingsPage.tsx`

**Step 1: Add imports**

Add at the top with other imports:

```typescript
import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
```

**Step 2: Add navigation hook and ESC handler**

Add after the `useState` declarations:

```typescript
const SettingsPage: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState('general')
  const navigate = useNavigate()

  // Handle ESC key to navigate back
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        navigate(-1)
      }
    }
    window.addEventListener('keydown', handleEsc)
    return () => window.removeEventListener('keydown', handleEsc)
  }, [navigate])
```

**Step 3: Test the ESC key functionality**

```bash
bun run dev
```

- Navigate to settings page
- Press ESC key
- Verify it navigates back to the previous page

**Step 4: Commit**

```bash
git add src/pages/SettingsPage.tsx
git commit -m "feat: add ESC key listener to navigate back from settings"
```

---

## Phase 2: Cleanup Backend Code

### Task 4: Delete the open_settings_window use case

**Files:**

- Delete: `src-tauri/crates/uc-app/src/usecases/settings/open_settings_window.rs`
- Modify: `src-tauri/crates/uc-app/src/usecases/settings/mod.rs`

**Step 1: Delete the use case file**

```bash
rm src-tauri/crates/uc-app/src/usecases/settings/open_settings_window.rs
```

**Step 2: Update settings/mod.rs**

Read and modify `src-tauri/crates/uc-app/src/usecases/settings/mod.rs`:

- Remove `mod open_settings_window;` if present
- Remove `pub use open_settings_window::*;` or similar exports

**Step 3: Verify no compilation errors**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-app/src/usecases/settings/
git commit -m "refactor: remove open_settings_window use case"
```

---

### Task 5: Remove open_settings_window Tauri command

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/settings.rs`

**Step 1: Read the file**

```bash
cat src-tauri/crates/uc-tauri/src/commands/settings.rs
```

**Step 2: Delete the `open_settings_window` command function**

Locate and delete the entire function:

```rust
#[tauri::command]
pub async fn open_settings_window(app_handle: tauri::AppHandle) -> Result<(), String> {
  // ... entire function
}
```

**Step 3: Verify no compilation errors**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands/settings.rs
git commit -m "refactor: remove open_settings_window Tauri command"
```

---

### Task 6: Remove command registration from main.rs

**Files:**

- Modify: `src-tauri/src/main.rs`

**Step 1: Find the invoke_handler macro**

Search for the `invoke_handler!` or `generate_invoke_handler!` macro usage.

**Step 2: Remove `open_settings_window` from the command list**

Remove it from the macro invocation list.

**Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

**Step 4: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "refactor: remove open_settings_window from command registration"
```

---

## Phase 3: Cleanup Frontend API

### Task 7: Remove openSettingsWindow API function

**Files:**

- Modify: `src/api/window.ts`

**Step 1: Read the file**

```bash
cat src/api/window.ts
```

**Step 2: Delete the `openSettingsWindow` function**

Delete the entire function:

```typescript
export async function openSettingsWindow(): Promise<void> {
  return invoke<void>('open_settings_window')
}
```

**Step 3: Verify no TypeScript errors**

```bash
bun run build
```

**Step 4: Commit**

```bash
git add src/api/window.ts
git commit -m "refactor: remove openSettingsWindow API function"
```

---

## Phase 4: Optional Refactoring

### Task 8: (Optional) Rename SettingsWindowLayout to SettingsFullLayout

**Files:**

- Rename: `src/layouts/SettingsWindowLayout.tsx` → `src/layouts/SettingsFullLayout.tsx`
- Modify: `src/layouts/index.ts`
- Modify: `src/App.tsx`

**Step 1: Rename the file**

```bash
git mv src/layouts/SettingsWindowLayout.tsx src/layouts/SettingsFullLayout.tsx
```

**Step 2: Update the file content**

Change the component name in the file:

```typescript
// Change from:
export default SettingsWindowLayout

// To:
const SettingsFullLayout: React.FC<LayoutProps> = ({ children }) => {
  // ... same content
}

export default SettingsFullLayout
```

**Step 3: Update index.ts exports**

```bash
sed -i '' 's/SettingsWindowLayout/SettingsFullLayout/g' src/layouts/index.ts
```

**Step 4: Update App.tsx import**

```bash
sed -i '' 's/SettingsWindowLayout/SettingsFullLayout/g' src/App.tsx
```

**Step 5: Verify and commit**

```bash
bun run build
git add src/layouts/
git commit -m "refactor: rename SettingsWindowLayout to SettingsFullLayout"
```

---

## Testing Verification

### Manual Testing Checklist

After completing all tasks, verify:

- [ ] Click settings button in sidebar → navigates to /settings (no new window)
- [ ] Settings page displays with full-screen layout
- [ ] TitleBar shows back button and "Settings" title
- [ ] Click back button → returns to previous page
- [ ] Press ESC key → returns to previous page
- [ ] Settings categories work correctly
- [ ] Settings save and persist correctly
- [ ] Theme changes apply immediately
- [ ] Language changes apply immediately
- [ ] No console errors in browser DevTools
- [ ] No Rust compilation warnings

### Automated Testing

Run the test suite:

```bash
# Frontend
bun run build

# Backend
cd src-tauri && cargo test
```

---

## Post-Implementation Notes

**What was removed:**

- Tauri `open_settings_window` command
- `OpenSettingsWindow` use case
- `openSettingsWindow` frontend API
- Multi-window architecture for settings

**What was added:**

- React Router navigation to `/settings`
- Back button in TitleBar for settings page
- ESC key listener in SettingsPage
- Single-window architecture

**Migration path:**

- Users will now see settings in the main window instead of a popup
- All settings functionality remains identical
- No data migration required (settings storage unchanged)
