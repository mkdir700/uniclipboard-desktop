# Update Available Sidebar Modal Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an update-available icon above Settings in the sidebar that opens a modal with release notes and allows users to trigger an update install.

**Architecture:** Keep updater state localized in the sidebar: call `@tauri-apps/plugin-updater` `check()` once when `setting.general.auto_check_update` is true, store the `Update` result, and conditionally render a sidebar action button + `AlertDialog` modal. Reuse existing UI components and Tailwind styling, and surface errors via the existing toast system.

**Tech Stack:** React 18 + TypeScript, Tauri updater plugin (JS), Radix AlertDialog, Tailwind CSS, i18next, Vitest + Testing Library.

---

### Task 1: Add failing tests for update indicator UI

**Files:**

- Create: `src/components/layout/__tests__/SidebarUpdateIndicator.test.tsx`
- Modify (if needed for accessibility hooks): `src/components/layout/Sidebar.tsx`

**Step 1: Write the failing test**

```tsx
import { check } from '@tauri-apps/plugin-updater'
import { render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import Sidebar from '@/components/layout/Sidebar'
import { SettingContext } from '@/contexts/SettingContext'
import type { Settings } from '@/types/setting'

vi.mock('@tauri-apps/plugin-updater', () => ({ check: vi.fn() }))

const baseSetting: Settings = {
  schema_version: 1,
  general: {
    auto_start: false,
    silent_start: false,
    auto_check_update: true,
    theme: 'system',
    theme_color: null,
    language: 'en-US',
    device_name: 'Test Device',
  },
  sync: {
    auto_sync: true,
    sync_frequency: 'realtime',
    content_types: {
      text: true,
      image: true,
      link: true,
      file: true,
      code_snippet: true,
      rich_text: true,
    },
    max_file_size_mb: 10,
  },
  retention_policy: {
    enabled: false,
    rules: [],
    skip_pinned: false,
    evaluation: 'any_match',
  },
  security: {
    encryption_enabled: false,
    passphrase_configured: false,
  },
}

const checkMock = vi.mocked(check)

describe('Sidebar update indicator', () => {
  it('shows update icon when updater returns update info', async () => {
    checkMock.mockResolvedValue({
      version: '0.1.1',
      currentVersion: '0.1.0',
      date: '2026-01-25T00:00:00Z',
      body: 'Bug fixes',
      downloadAndInstall: vi.fn(),
      close: vi.fn(),
    } as unknown as Awaited<ReturnType<typeof check>>)

    render(
      <SettingContext.Provider
        value={{
          setting: baseSetting,
          loading: false,
          error: null,
          updateSetting: vi.fn(),
          updateGeneralSetting: vi.fn(),
          updateSyncSetting: vi.fn(),
          updateSecuritySetting: vi.fn(),
          updateRetentionPolicy: vi.fn(),
        }}
      >
        <MemoryRouter>
          <Sidebar />
        </MemoryRouter>
      </SettingContext.Provider>
    )

    await waitFor(() => {
      expect(screen.getByLabelText(/update available/i)).toBeInTheDocument()
    })
  })
})
```

**Step 2: Run test to verify it fails**

Run: `bun run test src/components/layout/__tests__/SidebarUpdateIndicator.test.tsx`

Expected: FAIL because the update indicator does not exist yet.

---

### Task 2: Implement update check + modal behavior in Sidebar

**Files:**

- Modify: `src/components/layout/Sidebar.tsx`

**Step 1: Add updater state and effect**

- Import `check` from `@tauri-apps/plugin-updater`.
- Add local state for `updateInfo`, `isCheckingUpdate`, `isUpdating`, and `updateDialogOpen`.
- Call `check()` in a `useEffect` gated by `setting?.general.auto_check_update`.
- Store the `Update` result in state; log or toast on errors.

**Step 2: Add update icon above Settings**

- Render a new action button above the Settings `NavButton` only when `updateInfo` is truthy.
- Use a lucide icon (e.g., `ArrowUpCircle`) and a subtle amber dot indicator (matching the existing ping pattern).
- Add `aria-label="Update available"` for testing and accessibility.

**Step 3: Add modal with release notes and update action**

- Use `AlertDialog` components to present:
  - Title + version info (`updateInfo.version`, `updateInfo.currentVersion`).
  - A scrollable release notes block from `updateInfo.body`.
  - Actions: “Later” and “Update now”.
- Wire “Update now” to `updateInfo.downloadAndInstall()`; show loading state and toast on failure.

**Step 4: Run tests to verify green**

Run: `bun run test src/components/layout/__tests__/SidebarUpdateIndicator.test.tsx`

Expected: PASS.

---

### Task 3: Add i18n copy and updater dependency

**Files:**

- Modify: `src/i18n/locales/en-US.json`
- Modify: `src/i18n/locales/zh-CN.json`
- Modify: `package.json`

**Step 1: Add i18n keys**

Add an `update` section (or `nav.updateAvailable`) with:

- Tooltip text
- Modal title
- Action labels (“Update now”, “Later”)
- Release notes label and empty state
- Status/error messages for toast

**Step 2: Add updater dependency**

Add `@tauri-apps/plugin-updater` to `package.json` dependencies (align version with other `@tauri-apps` plugins).

---

### Task 4: Full verification

**Step 1: Run targeted tests**

Run: `bun run test src/components/layout/__tests__/SidebarUpdateIndicator.test.tsx`

**Step 2: Run lint if needed**

Run: `bun run lint`

Expected: No new errors.

---

## Execution Notes

- Keep updater logic localized to Sidebar to avoid extra global state.
- Respect `setting.general.auto_check_update` to avoid unsolicited network checks.
- Avoid new dependencies beyond `@tauri-apps/plugin-updater`.
- Do not use `@ts-ignore` or `as any` in tests.
