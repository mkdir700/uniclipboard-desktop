# Onboarding i18n (react-i18next) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move all user-facing copy in the onboarding flow (page + step components + toasts) to i18n keys, using the repo’s existing `i18next` + `react-i18next` setup.

**Architecture:** Keep `src/i18n/index.ts` as-is (single `translation` namespace). Add `onboarding.*` keys to `src/i18n/locales/{zh-CN,en-US}.json`. In step components, use `useTranslation()` with `keyPrefix` per step to keep keys short and consistent.

**Tech Stack:** React 18, TypeScript, i18next, react-i18next, sonner, vitest

---

## Context / Evidence

- i18n init: `src/i18n/index.ts` registers only `translation` namespace.
- Locale files: `src/i18n/locales/zh-CN.json`, `src/i18n/locales/en-US.json`.
- Onboarding flow copy is currently hard-coded in:
  - `src/pages/OnboardingPage.tsx`
  - `src/pages/onboarding/*.tsx`

---

## Guardrails

- Use dot-path keys (e.g. `onboarding.welcome.title`). Do NOT introduce `ns:key` style.
- Keep existing Chinese copy as-is (to avoid UX churn).
- Provide English equivalents in `en-US.json` for every new key.
- Avoid refactors beyond i18n wiring (no layout/styling changes).

---

## Task 1: Add onboarding i18n keys (zh-CN / en-US)

**Files:**

- Modify: `src/i18n/locales/zh-CN.json`
- Modify: `src/i18n/locales/en-US.json`

**Step 1: Write failing test (RED)**

Create a small unit test that asserts onboarding keys exist and resolve for both languages.

Create: `src/i18n/__tests__/onboarding-i18n.test.ts`

```ts
import { describe, it, expect } from 'vitest'
import i18n from '@/i18n'

describe('onboarding i18n keys', () => {
  it('resolves zh-CN onboarding.welcome.title', async () => {
    await i18n.changeLanguage('zh-CN')
    expect(i18n.t('onboarding.welcome.title')).toBeTruthy()
  })

  it('resolves en-US onboarding.welcome.title', async () => {
    await i18n.changeLanguage('en-US')
    expect(i18n.t('onboarding.welcome.title')).toBeTruthy()
  })
})
```

**Step 2: Run test to verify it fails (RED verification)**

Run: `bun x vitest run src/i18n/__tests__/onboarding-i18n.test.ts`

Expected: FAIL (missing keys => returns key or empty depending on config).

**Step 3: Add keys (GREEN)**

Add the following minimal key set (expand as needed during Task 2):

Suggested structure:

```jsonc
{
  "onboarding": {
    "page": {
      "loadingSetupState": "...",
      "errors": {
        "loadSetupStateFailed": "...",
        "refreshPeersFailed": "...",
        "operationFailed": "...",
        "completeSetupFailed": "...",
      },
      "badges": {
        "e2ee": "...",
        "localKeys": "...",
        "lanDiscovery": "...",
      },
      "unknownState": "Unknown state: {{state}}",
    },
    "common": {
      "back": "...",
      "refresh": "...",
      "unknownDevice": "...",
      "encryptPassphraseLabel": "...",
      "encryptPassphrasePlaceholder": "...",
    },
    "welcome": {
      "title": "...",
      "subtitle": "...",
      "create": {
        "title": "...",
        "description": "...",
        "cta": "...",
      },
      "join": {
        "title": "...",
        "description": "...",
        "cta": "...",
      },
      "footer": "...",
    },
    "createPassphrase": {
      "title": "...",
      "subtitle": "...",
      "labels": {
        "pass1": "...",
        "pass2": "...",
      },
      "placeholders": {
        "pass1": "...",
        "pass2": "...",
      },
      "actions": {
        "creating": "...",
        "submit": "...",
      },
      "hint": "...",
      "errors": {
        "mismatch": "...",
        "tooShort": "... {{minLen}} ...",
        "empty": "...",
        "generic": "...",
      },
    },
    "joinPickDevice": {
      "title": "...",
      "subtitle": "...",
      "empty": {
        "title": "...",
        "description": "...",
        "rescan": "...",
      },
      "errors": {
        "timeout": "...",
        "loadPeers": "...",
      },
      "actions": {
        "select": "...",
        "refresh": "...",
        "back": "...",
      },
    },
    "joinVerifyPassphrase": {
      "title": "...",
      "subtitle": "...",
      "targetDevice": "Target: {{peerShort}}...",
      "actions": {
        "verify": "...",
        "verifying": "...",
        "back": "...",
        "backToPick": "...",
      },
      "mismatchHelp": {
        "title": "...",
        "subtitle": "...",
        "p1": "...",
        "p2": "...",
        "option1": "...",
        "option2": "...",
        "retry": "...",
        "createNew": "...",
      },
      "errors": {
        "timeout": "...",
        "peerUnavailable": "...",
        "generic": "...",
        "empty": "...",
      },
    },
    "pairingConfirm": {
      "title": "...",
      "subtitle": "...",
      "peerFingerprint": "...",
      "errors": {
        "rejected": "...",
        "generic": "...",
      },
      "actions": {
        "cancel": "...",
        "confirming": "...",
        "confirm": "...",
      },
    },
    "done": {
      "title": "...",
      "subtitle": "...",
      "actions": { "enter": "..." },
    },
  },
}
```

**Step 4: Run test to verify it passes (GREEN verification)**

Run: `bun x vitest run src/i18n/__tests__/onboarding-i18n.test.ts`

Expected: PASS

---

## Task 2: Wire i18n into onboarding components

**Files:**

- Modify: `src/pages/OnboardingPage.tsx`
- Modify: `src/pages/onboarding/WelcomeStep.tsx`
- Modify: `src/pages/onboarding/CreatePassphraseStep.tsx`
- Modify: `src/pages/onboarding/JoinPickDeviceStep.tsx`
- Modify: `src/pages/onboarding/JoinVerifyPassphraseStep.tsx`
- Modify: `src/pages/onboarding/PairingConfirmStep.tsx`
- Modify: `src/pages/onboarding/SetupDoneStep.tsx`

**Step 1: Update tests first (RED)**

Modify: `src/pages/__tests__/OnboardingFlow.test.tsx`

- Ensure tests explicitly set the language expected by their assertions.

```ts
import i18n from '@/i18n'

beforeAll(async () => {
  await i18n.changeLanguage('zh-CN')
})
```

Also update any text expectations that are already out-of-date with current copy.

**Step 2: Run test to verify it fails (RED verification)**

Run: `bun x vitest run src/pages/__tests__/OnboardingFlow.test.tsx`

Expected: FAIL (because components haven’t been converted to i18n yet, or because expectations were corrected).

**Step 3: Convert each component (GREEN)**

Guideline: in each step file do:

```ts
import { useTranslation } from 'react-i18next'

const { t } = useTranslation(undefined, { keyPrefix: 'onboarding.welcome' })
```

Then replace literals:

- `WelcomeStep.tsx`
  - title/subtitle/cards/footer → `onboarding.welcome.*`

- `CreatePassphraseStep.tsx`
  - error mapping strings → `onboarding.createPassphrase.errors.*`
  - labels/placeholders/buttons/hint → `onboarding.createPassphrase.*`
  - interpolation: `t('errors.tooShort', { minLen })`

- `JoinPickDeviceStep.tsx`
  - back/refresh/title/subtitle/errors/empty state/buttons → `onboarding.joinPickDevice.*`

- `JoinVerifyPassphraseStep.tsx`
  - mismatch help mode copy → `onboarding.joinVerifyPassphrase.mismatchHelp.*`
  - normal mode copy + errors + labels + buttons → `onboarding.joinVerifyPassphrase.*`
  - target device short id: `t('targetDevice', { peerShort: peerId.substring(0, 8) })`

- `PairingConfirmStep.tsx`
  - title/subtitle/fingerprint label/buttons/errors → `onboarding.pairingConfirm.*`

- `SetupDoneStep.tsx`
  - title/subtitle/button → `onboarding.done.*`

- `OnboardingPage.tsx`
  - toast messages + loading state string + badges + unknown state
  - peer fallback name: use `t('onboarding.common.unknownDevice')`
  - unknown state: `t('onboarding.page.unknownState', { state: JSON.stringify(setupState) })`

**Step 4: Run test to verify it passes (GREEN verification)**

Run: `bun x vitest run src/pages/__tests__/OnboardingFlow.test.tsx`

Expected: PASS

---

## Task 3: Diagnostics + full frontend test run

**Step 1: LSP diagnostics**

Run `lsp_diagnostics` on all modified TS/TSX files.

**Step 2: Run full unit tests**

Run: `bun x vitest run`

Expected: PASS (or report pre-existing failures separately).
