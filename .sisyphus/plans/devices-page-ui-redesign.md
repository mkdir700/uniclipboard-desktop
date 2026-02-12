# Devices Page UI Redesign (Flatten Cards)

## TL;DR

> **Quick Summary**: Redesign the Devices page UI to reduce excessive card usage and eliminate card-in-card nesting by moving to a single list container with compact rows and inline expansion.
>
> **Deliverables**:
>
> - Flattened paired-devices list UI (rows, dividers, compact density)
> - Inline details/settings expansion without nested card visuals
> - DevicesPage no longer acts as a pairing-request hub (remove legacy routing/listening logic)
> - Minimal Vitest coverage updated/added (tests-after)
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 2 waves
> **Critical Path**: Task 1 -> Task 2 -> Task 3

---

## Context

### Original Request

- The branch `devices-page-empty-state` contains a DevicesPage UI rewrite but is behind.
- Keep only Devices page changes and UI-only changes.
- Current problem: too many cards and nested cards look visually heavy.

### Interview Summary

**Decisions (confirmed):**

- Layout direction: single container + list rows; inline expand/collapse; no card-in-card.
- Expand trigger: clicking the whole row toggles expand/collapse.
- Row density: compact.
- Actions visibility: always visible.
- Row actions (minimum): chevron (expand indicator) + unpair.
- Expanded area style: key/value rows with dividers.
- Tests: YES (tests-after). Update/add minimal Vitest coverage.

### Metis Review (guardrails + edge cases)

- A11y risk: whole-row click must be implemented with correct semantics (no nested interactive elements; keyboard support; aria attributes).
- Event propagation: action buttons must not toggle expansion.
- Scope creep risk: removing DevicesPage pairing-request routing logic must not change pairing domain logic or event contracts.

---

## Work Objectives

### Core Objective

Deliver a visually flatter Devices page by reducing surfaces (cards) and replacing nested cards with a single list container + row separators, while preserving existing device behavior.

### Concrete Deliverables

- Update `src/components/device/OtherDevice.tsx` to render paired devices as compact rows (not per-device cards).
- Update `src/components/device/DeviceSettingsPanel.tsx` to use key/value rows + dividers (no nested card wrappers).
- Update `src/pages/DevicesPage.tsx` to be UI-only and not a pairing-request hub.
- Update/add Vitest tests for the new structure and core interactions.

### Definition of Done

- [ ] `bun run build` exits 0
- [ ] `bun run lint` exits 0
- [ ] `bunx vitest run` exits 0

### Must Have

- Single primary surface for the device list (one container; rows separated by dividers).
- Compact row density (fits more devices per screen without feeling cramped).
- Whole-row click toggles expansion; expanded content does not collapse when interacting inside the expanded area.
- Action button clicks never toggle expansion.
- Always-visible minimum actions: chevron indicator + unpair.
- Expanded settings/details uses key/value rows + dividers (no nested cards).
- No DevicesPage-level pairing request routing/listening logic.

### Must NOT Have (Guardrails)

- Do not change backend contracts, Tauri events, or `src/api/p2p.ts` event shapes.
- Do not change device domain behavior (sorting/business rules) unless already implicit in the UI.
- Do not introduce new UI libraries or global CSS rewrites.
- Do not add card-in-card visuals in new markup (avoid stacked rounded+border+shadow surfaces).

---

## Verification Strategy (Mandatory)

### Test Decision

- Infrastructure exists: YES (Vitest)
- Automated tests: YES (tests-after)
- Framework: Vitest + Testing Library

### Agent-Executed QA Scenarios (Agent runs these)

Scenario: Typecheck + production build
Tool: Bash
Steps: 1. Run: `bun run build` 2. Assert: exit code 0
Expected Result: TypeScript and Vite build succeed

Scenario: Lint
Tool: Bash
Steps: 1. Run: `bun run lint` 2. Assert: exit code 0
Expected Result: Lint passes

Scenario: Unit tests
Tool: Bash
Steps: 1. Run: `bunx vitest run` 2. Assert: exit code 0
Expected Result: All tests pass

---

## Execution Strategy

### Parallel Execution Waves

Wave 1 (Start Immediately):

- Task 1: Flatten paired-device list rows (OtherDevice)
- Task 2: Flatten expanded settings panel (DeviceSettingsPanel)

Wave 2 (After Wave 1):

- Task 3: DevicesPage UI-only cleanup + tests + final verification

---

## TODOs

> Implementation + its tests live in the same task.

- [x] 1. Flatten paired-device list into compact rows (no per-device cards)

  **What to do**:
  - In `src/components/device/OtherDevice.tsx`, replace the per-device card wrapper with a single list container.
  - Render each device as a compact row with:
    - Left: device icon + name + peer id (truncated)
    - Right: status indicator + chevron (visual) + unpair action
  - Whole-row click toggles expansion (only the summary row area toggles; expanded area is not clickable-to-toggle).
  - Add a11y semantics:
    - Use a non-nested interactive structure (avoid `<button>` wrapping other buttons)
    - Provide keyboard toggle (Enter/Space)
    - Set `aria-expanded` and `aria-controls` for the expanded region
  - Ensure unpair button does `stopPropagation` and does not toggle expansion.
  - Default behavior (unless explicitly changed): only one row expanded at a time (accordion behavior).

  **Must NOT do**:
  - Do not change device fetching/unpair domain logic beyond what is needed for UI structure.
  - Do not introduce additional nested card surfaces (rounded+border+shadow stacks).

  **Recommended Agent Profile**:
  - Category: visual-engineering
    - Reason: UI structure/layout refactor with interaction details and a11y.
  - Skills: frontend-ui-ux, web-design-guidelines
    - frontend-ui-ux: ensure compact rows still feel deliberate and balanced
    - web-design-guidelines: check interaction targets, a11y semantics, and hierarchy

  **Parallelization**:
  - Can Run In Parallel: YES
  - Parallel Group: Wave 1 (with Task 2)
  - Blocks: Task 3

  **References**:
  - `src/components/device/OtherDevice.tsx` - current per-device card + inline expansion implementation
  - `src/components/device/DeviceList.tsx` - how OtherDevice is composed into the page
  - `src/components/device/__tests__/OtherDevice.test.tsx` - existing test harness and mocking style

  **Acceptance Criteria**:
  - [ ] `bun run build` exits 0
  - [ ] Update/add tests in `src/components/device/__tests__/OtherDevice.test.tsx` to cover:
    - Row click toggles expansion (assert via `aria-expanded` or expanded content visibility)
    - Clicking unpair does not toggle expansion
  - [ ] `bunx vitest run src/components/device/__tests__/OtherDevice.test.tsx` exits 0

- [x] 2. Flatten DeviceSettingsPanel into key/value rows with dividers (no nested cards)

  **What to do**:
  - In `src/components/device/DeviceSettingsPanel.tsx`, remove the nested card container (`bg-card/... rounded... border...`) and the mini-card rule items.
  - Replace with a flat layout:
    - Section header (optional) + divider
    - Key/value rows separated by subtle dividers
    - Controls (switches/toggles) aligned to the right
  - Keep the compact density consistent with the parent row list.
  - Ensure interactive controls inside the expanded area do not collapse the parent row.

  **Must NOT do**:
  - Do not introduce new stateful routing/tabs unless already required.
  - Do not add new libraries.

  **Recommended Agent Profile**:
  - Category: visual-engineering
    - Reason: visual language change from card-based to flat settings rows.
  - Skills: frontend-ui-ux, web-design-guidelines

  **Parallelization**:
  - Can Run In Parallel: YES
  - Parallel Group: Wave 1 (with Task 1)
  - Blocks: Task 3

  **References**:
  - `src/components/device/DeviceSettingsPanel.tsx` - current nested cards and rule item layout
  - `src/components/device/OtherDevice.tsx` - how DeviceSettingsPanel is embedded in the expanded region

  **Acceptance Criteria**:
  - [ ] `bun run build` exits 0
  - [ ] Add/update a unit test (new file acceptable):
    - Example path: `src/components/device/__tests__/DeviceSettingsPanel.test.tsx`
    - Assert: renders key/value rows and dividers; no outer nested card wrapper remains
  - [ ] `bunx vitest run src/components/device/__tests__/DeviceSettingsPanel.test.tsx` exits 0

- [x] 3. Keep DevicesPage UI-only and remove legacy pairing-request hub logic

  **What to do**:
  - In `src/pages/DevicesPage.tsx`, ensure the page is responsible only for:
    - Layout shell + scroll container
    - Opening/closing `PairingDialog` via explicit user action (Add Device)
    - Refreshing the device list on pairing success (dispatch fetch)
  - Remove any DevicesPage-specific pairing-request routing/listening (e.g. `onP2PPairingVerification`, query-driven pin dialog, etc.)
  - Update tests to match the new responsibilities.

  **Defaults Applied (override if needed)**:
  - Incoming pairing requests are handled outside DevicesPage (e.g. global provider) and are not routed through DevicesPage.
  - Empty state uses an inline CTA within the single list container.

  **Recommended Agent Profile**:
  - Category: visual-engineering
    - Reason: page-level UI wiring + removing legacy UI routing logic safely.
  - Skills: frontend-ui-ux, systematic-debugging
    - systematic-debugging: ensures removal does not leave dangling imports/tests

  **Parallelization**:
  - Can Run In Parallel: NO
  - Parallel Group: Wave 2
  - Blocked By: Task 1, Task 2

  **References**:
  - `src/pages/DevicesPage.tsx` - page-level layout and current responsibilities
  - `src/pages/__tests__/DevicesPage.test.tsx` - currently tests legacy hub behavior; must be rewritten or removed
  - `src/components/PairingDialog.tsx` - pairing initiation dialog used by DevicesPage
  - `src/components/PairingNotificationProvider.tsx` - existing global pairing-request UI (do not redesign here)

  **Acceptance Criteria**:
  - [ ] `src/pages/DevicesPage.tsx` no longer imports or references legacy hub identifiers (examples to search):
    - `onP2PPairingVerification`
    - `useSearchParams`
  - [ ] Update or remove `src/pages/__tests__/DevicesPage.test.tsx` so it no longer depends on legacy hub behavior
  - [ ] `bunx vitest run src/pages/__tests__/DevicesPage.test.tsx` exits 0 (or file removed and overall test run passes)
  - [ ] `bun run lint` exits 0
  - [ ] `bun run build` exits 0

---

## Commit Strategy

> Follow the repo's atomic commit rules.

- Commit 1: `refactor(devices-ui): flatten paired device list rows`
  - Includes: Task 1 code + its updated tests
- Commit 2: `refactor(devices-ui): flatten device settings panel layout`
  - Includes: Task 2 code + its tests
- Commit 3: `refactor(devices-ui): remove devices page pairing-request hub`
  - Includes: Task 3 code + test updates

---

## Success Criteria

### Verification Commands

```bash
bun run build
bun run lint
bunx vitest run
```

### Final Checklist

- [ ] `bun run build` exits 0
- [ ] `bun run lint` exits 0
- [ ] `bunx vitest run` exits 0
- [ ] `bunx vitest run src/components/device/__tests__/OtherDevice.test.tsx` exits 0
- [ ] `bunx vitest run src/components/device/__tests__/DeviceSettingsPanel.test.tsx` exits 0
- [ ] `bunx vitest run src/pages/__tests__/DevicesPage.test.tsx` exits 0 (or the file is removed and the full suite still passes)
