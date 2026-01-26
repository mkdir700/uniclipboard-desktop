# Sentry v10 Telemetry Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade @sentry/react to v10, redact sensitive telemetry args, and ensure trace IDs are RFC4122 v4 while keeping tests/lint green.

**Architecture:** Keep telemetry redaction in a shared frontend helper and apply it both at the tauri command wrapper and at Sentry init hooks. Update trace ID generation in the observability trace manager with RFC4122 v4 formatting to match Rust Uuid parsing.

**Tech Stack:** React + Vite, Vitest, ESLint, Bun, Sentry React SDK.

---

### Task 1: Update Sentry dependency and lockfiles

**Files:**

- Modify: `package.json`
- Update: `bun.lock`
- Update (if required by repo policy): `package-lock.json`

**Step 1: Write the failing test**

Not applicable (configuration change).

**Step 2: Update dependency version**

Edit `package.json`:

```json
"@sentry/react": "^10.36.0"
```

**Step 3: Update lockfile(s)**

Run:

```bash
bun install
```

If `package-lock.json` is maintained in this repo, also run:

```bash
npm install --package-lock-only
```

**Step 4: Verify diff**

Check updated lockfile(s) for @sentry/react v10 entries.

---

### Task 2: Add telemetry redaction helper

**Files:**

- Create: `src/observability/redaction.ts`
- Test: `src/observability/__tests__/redaction.test.ts`

**Step 1: Write the failing test**

Create `src/observability/__tests__/redaction.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { redactSensitiveArgs } from '../redaction'

describe('redactSensitiveArgs', () => {
  it('masks sensitive keys recursively', () => {
    const input = {
      password: 'secret',
      nested: { passphrase: 'hello' },
      list: [{ token: 'abc' }],
      safe: 'ok',
    }

    const output = redactSensitiveArgs(input)

    expect(output).toEqual({
      password: '[REDACTED]',
      nested: { passphrase: '[REDACTED]' },
      list: [{ token: '[REDACTED]' }],
      safe: 'ok',
    })
  })
})
```

**Step 2: Run test to verify it fails**

Run:

```bash
bun run vitest src/observability/__tests__/redaction.test.ts
```

Expected: FAIL (redactSensitiveArgs not implemented).

**Step 3: Write minimal implementation**

Create `src/observability/redaction.ts` with:

```ts
const sensitiveKeys = ['password', 'passphrase', 'secret', 'token', 'auth', 'api_key', 'apikey']

export function redactSensitiveArgs<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map(item => redactSensitiveArgs(item)) as T
  }

  if (!value || typeof value !== 'object') {
    return value
  }

  const result: Record<string, unknown> = {}
  for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
    const lowerKey = key.toLowerCase()
    if (sensitiveKeys.some(sensitive => lowerKey.includes(sensitive))) {
      result[key] = '[REDACTED]'
    } else {
      result[key] = redactSensitiveArgs(item)
    }
  }
  return result as T
}
```

**Step 4: Run test to verify it passes**

Run:

```bash
bun run vitest src/observability/__tests__/redaction.test.ts
```

Expected: PASS.

---

### Task 3: Redact args in tauri command telemetry

**Files:**

- Modify: `src/lib/tauri-command.ts`
- Test: `src/lib/__tests__/tauri-command.test.ts`

**Step 1: Write the failing test**

Add a new test in `src/lib/__tests__/tauri-command.test.ts`:

```ts
it('redacts sensitive args in Sentry telemetry', async () => {
  const trace = { traceId: 'trace-1', startTime: 1234, operation: 'command' }
  const args = { password: 'secret', nested: { passphrase: 'hello' } }

  vi.mocked(traceManager.startTrace).mockReturnValue(trace)
  vi.mocked(invoke).mockResolvedValueOnce({ ok: true })

  await invokeWithTrace('set_encryption_password', args)

  expect(Sentry.addBreadcrumb).toHaveBeenCalledWith({
    category: 'tauri_command',
    message: 'set_encryption_password',
    level: 'info',
    data: {
      traceId: trace.traceId,
      args: { password: '[REDACTED]', nested: { passphrase: '[REDACTED]' } },
    },
  })
})
```

**Step 2: Run test to verify it fails**

Run:

```bash
bun run vitest src/lib/__tests__/tauri-command.test.ts
```

Expected: FAIL (raw args still present).

**Step 3: Write minimal implementation**

Update `src/lib/tauri-command.ts`:

- Import `redactSensitiveArgs` from `src/observability/redaction.ts`.
- Compute `const safeArgs = args ? redactSensitiveArgs(args) : undefined`.
- Use `safeArgs` in `Sentry.addBreadcrumb` and `Sentry.captureException(..., { extra: { args: safeArgs } })`.
- Keep original `args` for the `invoke` call.

**Step 4: Run test to verify it passes**

Run:

```bash
bun run vitest src/lib/__tests__/tauri-command.test.ts
```

Expected: PASS.

---

### Task 4: Add Sentry SDK scrubbing hooks

**Files:**

- Modify: `src/observability/sentry.ts`
- Test: `src/observability/__tests__/sentry.test.ts`

**Step 1: Write the failing test**

Create `src/observability/__tests__/sentry.test.ts`:

```ts
import { describe, expect, it, vi } from 'vitest'
import { initSentry } from '../sentry'
import * as Sentry from '@sentry/react'

vi.mock('@sentry/react', () => ({
  init: vi.fn(),
  browserTracingIntegration: vi.fn(),
  replayIntegration: vi.fn(),
}))

describe('initSentry', () => {
  it('registers scrubbers for breadcrumb and event extra data', () => {
    initSentry()

    const initCall = vi.mocked(Sentry.init).mock.calls[0]?.[0]
    expect(initCall?.beforeBreadcrumb).toBeTypeOf('function')
    expect(initCall?.beforeSend).toBeTypeOf('function')

    const breadcrumb = initCall?.beforeBreadcrumb?.({ data: { password: 'secret' } })
    expect(breadcrumb?.data).toEqual({ password: '[REDACTED]' })

    const event = initCall?.beforeSend?.({ extra: { passphrase: 'hello' } })
    expect(event?.extra).toEqual({ passphrase: '[REDACTED]' })
  })
})
```

**Step 2: Run test to verify it fails**

Run:

```bash
bun run vitest src/observability/__tests__/sentry.test.ts
```

Expected: FAIL (hooks missing).

**Step 3: Write minimal implementation**

Update `src/observability/sentry.ts`:

- Import `redactSensitiveArgs`.
- Add `beforeBreadcrumb` to scrub `breadcrumb.data` when present.
- Extend existing `beforeSend` to scrub `event.extra` (and keep existing filtering for ResizeObserver errors).

**Step 4: Run test to verify it passes**

Run:

```bash
bun run vitest src/observability/__tests__/sentry.test.ts
```

Expected: PASS.

---

### Task 5: Update trace ID generation to RFC4122 v4

**Files:**

- Modify: `src/observability/trace.ts`
- Test: `src/observability/__tests__/trace.test.ts`

**Step 1: Write the failing test**

Extend `src/observability/__tests__/trace.test.ts`:

```ts
import { describe, expect, it, vi } from 'vitest'
import { traceManager } from '../trace'

describe('traceManager', () => {
  it('generates RFC4122 v4 trace ids when randomUUID is unavailable', () => {
    const originalCrypto = globalThis.crypto
    const bytes = new Uint8Array(16)
    bytes[0] = 0x12
    bytes[1] = 0x34
    bytes[2] = 0x56
    bytes[3] = 0x78
    bytes[4] = 0x9a
    bytes[5] = 0xbc
    bytes[6] = 0xde
    bytes[7] = 0xf0
    bytes[8] = 0x12
    bytes[9] = 0x34
    bytes[10] = 0x56
    bytes[11] = 0x78
    bytes[12] = 0x9a
    bytes[13] = 0xbc
    bytes[14] = 0xde
    bytes[15] = 0xf0

    vi.stubGlobal('crypto', {
      getRandomValues: (value: Uint8Array) => {
        value.set(bytes)
        return value
      },
    })

    const trace = traceManager.startTrace('test')
    expect(trace.traceId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i
    )

    vi.stubGlobal('crypto', originalCrypto)
  })
})
```

**Step 2: Run test to verify it fails**

Run:

```bash
bun run vitest src/observability/__tests__/trace.test.ts
```

Expected: FAIL (fallback uses Date.now()).

**Step 3: Write minimal implementation**

Update `createTraceId` in `src/observability/trace.ts` to:

- Prefer `crypto.randomUUID()`.
- Else use `crypto.getRandomValues()` to generate 16 bytes, set version/variant bits, format 8-4-4-4-12 hex groups.
- Final fallback to Math.random when neither crypto API exists.

**Step 4: Run test to verify it passes**

Run:

```bash
bun run vitest src/observability/__tests__/trace.test.ts
```

Expected: PASS.

---

### Task 6: Run full tests and lint

**Step 1: Run tests**

Run:

```bash
bun run test
```

**Step 2: Run lint**

Run:

```bash
bun run lint
```

---

### Task 7: Review Sentry v10 migration alignment

**Files:**

- Verify: `src/observability/sentry.ts`

**Step 1: Review init options**

Ensure:

- No `enableTracing` or `autoSessionTracking` options are used.
- `browserTracingIntegration()` and `replayIntegration()` remain valid in v10.
- `sendDefaultPii` remains unset unless explicitly required.

**Step 2: Document migration checks (optional)**

If needed, add a short note to the PR summary (not code) that v10 migration points were reviewed.
