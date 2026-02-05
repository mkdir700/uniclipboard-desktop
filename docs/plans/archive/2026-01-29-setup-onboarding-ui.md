# Setup Onboarding UI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the single-page onboarding with a multi-step setup flow that mirrors `SetupState/SetupEvent` and the provided copy spec.

**Architecture:** Split each setup screen into its own React component. `OnboardingPage` becomes a thin orchestrator that loads `SetupState`, dispatches `SetupEvent`, and renders the correct step component. Copy and error mapping lives in the step components.

**Tech Stack:** React 18, TypeScript, react-router, framer-motion, Tailwind, vitest

---

## Task 1: Add failing tests for new multi-step onboarding

**Files:**

- Create: `src/pages/__tests__/OnboardingFlow.test.tsx`
- Modify: `src/pages/__tests__/OnboardingPage.test.tsx` (if needed to align with new structure)

#### Step 1: Write the failing test

**Step 5: Commit**

```bash
git add src/pages/__tests__/OnboardingFlow.test.tsx src/pages/__tests__/OnboardingPage.test.tsx
git commit -m "test(frontend): cover setup onboarding flow"
```

---

### Task 2: Add modular step components

**Files:**

- Create: `src/pages/onboarding/WelcomeStep.tsx`
- Create: `src/pages/onboarding/CreatePassphraseStep.tsx`
- Create: `src/pages/onboarding/JoinPickDeviceStep.tsx`
- Create: `src/pages/onboarding/JoinVerifyPassphraseStep.tsx`
- Create: `src/pages/onboarding/PairingConfirmStep.tsx`
- Create: `src/pages/onboarding/SetupDoneStep.tsx`
- Create: `src/pages/onboarding/types.ts`

**Step 1: Write the failing test**

```tsx
import { render, screen } from '@testing-library/react'
import { describe, it, expect } from 'vitest'
import WelcomeStep from '@/pages/onboarding/WelcomeStep'

describe('WelcomeStep', () => {
  it('shows create/join CTAs', () => {
    render(<WelcomeStep onCreate={() => {}} onJoin={() => {}} />)
    expect(screen.getByText('欢迎使用 UniClipboard')).toBeInTheDocument()
    expect(screen.getByText('创建新的加密空间')).toBeInTheDocument()
  })
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: FAIL with missing component.

**Step 3: Write minimal implementation**

- Implement each step component with props for callbacks and error display
- Use copy spec verbatim for titles, body, CTA, error text

**Step 4: Run test to verify it passes**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add src/pages/onboarding
git commit -m "feat(frontend): add setup onboarding step components"
```

---

### Task 3: Wire `OnboardingPage` to setup state machine

**Files:**

- Modify: `src/pages/OnboardingPage.tsx`
- Modify: `src/api/onboarding.ts` (if new helper types are needed)

**Step 1: Write the failing test**

```tsx
it('dispatches ChooseCreateSpace when clicking create CTA', async () => {
  // mock getSetupState => Welcome
  // click CTA
  // assert dispatchSetupEvent called with 'ChooseCreateSpace'
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: FAIL with missing event dispatch.

**Step 3: Write minimal implementation**

- Load `SetupState` on mount via `getSetupState`
- Render the step component based on `SetupState`
- Map UI actions to `dispatchSetupEvent`
- For `SetupState.Done`, show Done screen and call `completeOnboarding` on CTA

**Step 4: Run test to verify it passes**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add src/pages/OnboardingPage.tsx src/api/onboarding.ts
git commit -m "feat(frontend): drive onboarding by setup state machine"
```

---

### Task 4: Update copy and error mappings

**Files:**

- Modify: `src/pages/onboarding/*.tsx`
- Modify: `src/components/PairingDialog.tsx` and `src/components/PairingPinDialog.tsx` (if pairing copy needs alignment)

**Step 1: Write the failing test**

```tsx
it('shows passphrase mismatch error text', () => {
  // render CreatePassphraseStep with error=PassphraseMismatch
  // expect error copy from spec
})
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: FAIL

**Step 3: Write minimal implementation**

- Map SetupError to the exact copy for each screen
- Ensure “不一致” wording for passphrase mismatch

**Step 4: Run test to verify it passes**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: PASS

**Step 5: Commit**

```bash
git add src/pages/onboarding src/components/PairingDialog.tsx src/components/PairingPinDialog.tsx
git commit -m "feat(frontend): align setup copy and error states"
```

---

### Task 5: Diagnostics + full test run

**Step 1: Run diagnostics**

Use `lsp_diagnostics` on all changed TS/TSX files.

**Step 2: Run tests**

Run: `npm test -- src/pages/__tests__/OnboardingFlow.test.tsx`
Expected: PASS

**Step 3: Commit**

No commit unless additional changes are required.
