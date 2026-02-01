# Devices Page Simplification Plan

## TL;DR

> **Quick Summary**: Simplify the Devices page by removing tabs, section headers, and current-device settings while keeping pairing requests always visible and preserving existing card styles.
>
> **Deliverables**:
>
> - Updated Devices page layout without tabs/section headers
> - Current device card without settings expand
> - Other devices remain expandable for settings
>
> **Estimated Effort**: Short
> **Parallel Execution**: NO - sequential (overlapping files)
> **Critical Path**: Task 1 → Task 2 → Task 3

---

## Context

### Original Request

Remove tabs in `src/pages/DevicesPage.tsx`, simplify the page design, remove per-section headers, disallow current device settings expansion, and keep only other devices expandable for settings.

### Interview Summary

**Key Discussions**:

- Tabs removed; pairing requests section always visible.
- Remove all section headers (requests/current/other); keep page title only.
- Current device settings entry removed (no settings button/panel).
- Simplification is structure-only (keep existing card styles).
- Test strategy: manual verification only (no new tests).

**Research Findings**:

- Tabs UI lives in `src/components/device/Header.tsx`.
- Current device settings expand logic in `src/components/device/CurrentDevice.tsx`.
- Other device settings expand logic in `src/components/device/OtherDevice.tsx`.
- Pairing requests section in `src/pages/DevicesPage.tsx`.
- Scripts available: `lint`, `build`, `test` in `package.json` (README uses bun).

### Metis Review

**Identified Gaps (addressed)**:

- Confirmed DeviceHeader usage is limited to Devices page (grep shows only `src/pages/DevicesPage.tsx`).
- Lock scope: no routing/deep-linking changes since tabs do not use URL params.
- Keep pairing requests visible even when empty (empty state stays).
- Do not remove i18n keys; only remove UI usages.
- Provide automated verification command even with manual-only preference (`bun run lint`).

---

## Work Objectives

### Core Objective

Simplify the Devices page structure by removing tabs and section headers while preserving existing card styling and device behaviors, except for removing current-device settings expansion.

### Concrete Deliverables

- Devices page header without tabs and without tab/scroll state.
- Pairing requests block always shown without section header.
- Current device card without settings button or expandable settings panel.
- Other device cards remain expandable (settings panel still works).

### Definition of Done

- No tabs UI or tab state in Devices page/header components.
- No section header bars for requests/current/other sections.
- Current device has no settings button and cannot expand.
- Other devices still expand/collapse settings.
- `bun run lint` exits with code 0.

### Must Have

- Pairing requests section is always visible on Devices page.

### Must NOT Have (Guardrails)

- No backend/API changes (P2P, pairing, or device data flows).
- No removal of dialogs (`PairingDialog`, `PairingPinDialog`).
- No deletion of i18n keys; only remove UI usage.
- No stylistic redesign beyond structural removals (keep card styles).
- No fixed-pixel layout additions; use existing Tailwind utilities only if needed.

---

## Verification Strategy (MANDATORY)

### Test Decision

- **Infrastructure exists**: YES (lint/build/test scripts)
- **User wants tests**: Manual verification only
- **Framework**: N/A (no new tests)

### Automated Verification (Agent-Executable)

Run at least once after the final task:

```bash
bun run lint
```

Expected: exit code 0.

Optional (if TypeScript build is already part of workflow):

```bash
bun run build
```

Expected: exit code 0.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
└── Task 1: Remove tabs from header and Devices page state

Wave 2 (After Wave 1):
└── Task 2: Remove section headers across requests/current/other

Wave 3 (After Wave 2):
└── Task 3: Remove current device settings expand entry

Critical Path: Task 1 → Task 2 → Task 3
Parallel Speedup: N/A (sequential)
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 2, 3   | None                 |
| 2    | 1          | 3      | None                 |
| 3    | 2          | None   | None                 |

---

## TODOs

> Implementation + Verification = ONE Task.

- [x] 1. Remove tabs from Devices header and page state

  **What to do**:
  - Update `src/components/device/Header.tsx` to remove tabs UI and related props.
  - Update `src/pages/DevicesPage.tsx` to remove `activeTab`, `handleTabChange`, tab scroll refs, and scroll listeners.
  - Keep page title and Add Device button unchanged.

  **Must NOT do**:
  - Do not alter pairing dialog logic or add new UI sections.
  - Do not add new routes or URL params.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Small, localized UI structural change.
  - **Skills**: `brainstorming`, `verification-before-completion`
    - `brainstorming`: Ensure change scope stays structural.
    - `verification-before-completion`: Enforce lint verification.
  - **Skills Evaluated but Omitted**:
    - `vercel-react-best-practices`: No performance refactor needed.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 2, Task 3
  - **Blocked By**: None

  **References**:
  - `src/components/device/Header.tsx` - Tabs rendering and `DeviceTab` type.
  - `src/pages/DevicesPage.tsx` - Tab state, refs, and scroll-to-section logic.

  **Acceptance Criteria**:
  - Tabs UI removed; header only shows title and Add Device button.
  - No `activeTab` or scroll listener logic remains in Devices page.
  - `bun run lint` exits 0 after completing this task or after Task 3.

- [x] 2. Remove section headers (requests/current/other)

  **What to do**:
  - Remove section title bars in `src/pages/DevicesPage.tsx` for pairing requests.
  - Remove section title bars in `src/components/device/CurrentDevice.tsx` and `src/components/device/OtherDevice.tsx`.
  - Keep pairing requests block always visible with its empty state.
  - Preserve existing card styling and spacing as much as possible.

  **Must NOT do**:
  - Do not remove empty state content or explanatory text.
  - Do not change existing card styles beyond removing the header wrappers.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Small JSX structural edits across a few files.
  - **Skills**: `brainstorming`, `verification-before-completion`
    - `brainstorming`: Keep scope minimal and structural.
    - `verification-before-completion`: Ensure lint verification.
  - **Skills Evaluated but Omitted**:
    - `web-design-guidelines`: No redesign requested.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:
  - `src/pages/DevicesPage.tsx` - Pairing requests section header.
  - `src/components/device/CurrentDevice.tsx` - Current device header block.
  - `src/components/device/OtherDevice.tsx` - Other devices header block.

  **Acceptance Criteria**:
  - No section header bars remain in the three sections.
  - Pairing requests block still renders (empty state visible when no request).
  - `bun run lint` exits 0 after completing this task or after Task 3.

- [x] 3. Remove current device settings expand entry

  **What to do**:
  - Remove settings button and expanded panel from `src/components/device/CurrentDevice.tsx`.
  - Keep current device card information and status indicators intact.
  - Ensure no expansion state remains for current device.

  **Must NOT do**:
  - Do not remove current device card itself.
  - Do not alter other device expand behavior.

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single-component behavior removal.
  - **Skills**: `brainstorming`, `verification-before-completion`
    - `brainstorming`: Ensure only requested behavior removed.
    - `verification-before-completion`: Enforce lint verification.
  - **Skills Evaluated but Omitted**:
    - `systematic-debugging`: No bug investigation required.

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Sequential
  - **Blocks**: None
  - **Blocked By**: Task 2

  **References**:
  - `src/components/device/CurrentDevice.tsx` - Settings button and expandable panel.
  - `src/components/device/OtherDevice.tsx` - Keep expand behavior unchanged.

  **Acceptance Criteria**:
  - Current device card has no settings button and no expandable content.
  - Other device cards still expand/collapse settings.
  - `bun run lint` exits 0.

---

## Commit Strategy

| After Task | Message                                           | Files                                                                                                                                               | Verification   |
| ---------- | ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| 3          | `chore(devices): simplify devices page structure` | `src/pages/DevicesPage.tsx`, `src/components/device/Header.tsx`, `src/components/device/CurrentDevice.tsx`, `src/components/device/OtherDevice.tsx` | `bun run lint` |

---

## Success Criteria

### Verification Commands

```bash
bun run lint
```

### Final Checklist

- [x] Tabs removed from Devices page header and state.
- [x] Section headers removed for requests/current/other.
- [x] Pairing requests section always visible (empty state preserved).
- [x] Current device settings expansion removed; other devices remain expandable.
- [x] Lint passes with exit code 0.
