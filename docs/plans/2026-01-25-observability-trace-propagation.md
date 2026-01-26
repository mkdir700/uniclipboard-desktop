# Observability Trace Propagation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add frontend Sentry initialization, trace_id propagation into Tauri invokes, and backend trace logging so a single trace_id links UI actions to Rust spans.

**Architecture:** Frontend generates and manages trace context, injects `_trace` metadata into all Tauri invokes, and reports errors/breadcrumbs to Sentry (prod only). Backend parses `_trace` into a uc-core type and enriches existing command spans with trace_id fields without changing use case boundaries.

**Tech Stack:** React + Vite, Tauri 2, @sentry/react, Rust `tracing`, serde_json, uuid.

---

## Task 1: Add Sentry dependencies and environment typing

**Files:**

- Modify: `package.json`
- Modify: `src/vite-env.d.ts`
- Create: `.env.example`

**Step 1: Add frontend dependencies**

Update `package.json` dependencies to include:

```json
"@sentry/react": "^8.0.0"
```

**Step 2: Add Vite env typings**

Update `src/vite-env.d.ts`:

```ts
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_SENTRY_DSN?: string
  readonly VITE_APP_VERSION?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
```

**Step 3: Add env template**

Create `.env.example`:

```dotenv
VITE_SENTRY_DSN=
VITE_APP_VERSION=
```

**Step 4: Commit**

```bash
git add package.json src/vite-env.d.ts .env.example
git commit -m "chore: add sentry env and typings"
```

---

## Task 2: Frontend observability foundation

**Files:**

- Create: `src/observability/sentry.ts`
- Create: `src/observability/trace.ts`
- Create: `src/observability/breadcrumbs.ts`
- Create: `src/observability/errors.ts`
- Modify: `src/main.tsx`

**Step 1: Create Sentry initializer**

`src/observability/sentry.ts`:

```ts
import * as Sentry from '@sentry/react'

const sentryEnabled = import.meta.env.PROD && Boolean(import.meta.env.VITE_SENTRY_DSN)

export function initSentry(): void {
  if (!sentryEnabled) {
    return
  }

  Sentry.init({
    dsn: import.meta.env.VITE_SENTRY_DSN,
    tracesSampleRate: 0.1,
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 1.0,
    environment: import.meta.env.MODE,
    release: import.meta.env.VITE_APP_VERSION,
    integrations: [Sentry.browserTracingIntegration(), Sentry.replayIntegration()],
    beforeSend(event) {
      const type = event.exception?.values?.[0]?.type
      if (type === 'ResizeObserver loop limit exceeded') {
        return null
      }
      return event
    },
    initialScope: {
      tags: {
        platform: window.__TAURI__?.platform ?? 'unknown',
      },
    },
  })
}

export { Sentry, sentryEnabled }
```

**Step 2: Trace manager**

`src/observability/trace.ts`:

```ts
import { v4 as uuidv4 } from 'uuid'
import { Sentry, sentryEnabled } from './sentry'

export interface TraceContext {
  traceId: string
  startTime: number
  operation: string
}

class TraceManager {
  private currentTrace: TraceContext | null = null

  startTrace(operation: string): TraceContext {
    this.currentTrace = {
      traceId: uuidv4(),
      startTime: Date.now(),
      operation,
    }
    return this.currentTrace
  }

  getCurrentTrace(): TraceContext | null {
    return this.currentTrace
  }

  endTrace(): void {
    const trace = this.currentTrace
    if (trace && sentryEnabled && Math.random() < 0.1) {
      Sentry.startSpan(
        {
          name: trace.operation,
          op: 'ui.action',
          startTimestamp: trace.startTime / 1000,
        },
        () => {}
      ).end()
    }
    this.currentTrace = null
  }
}

export const traceManager = new TraceManager()
```

**Step 3: Breadcrumb helpers**

`src/observability/breadcrumbs.ts`:

```ts
import { Sentry, sentryEnabled } from './sentry'

export type UserIntent =
  | 'copy_clipboard'
  | 'paste_clipboard'
  | 'open_settings'
  | 'pair_device'
  | 'delete_entry'
  | 'search_entries'

export function captureUserIntent(intent: UserIntent, context?: Record<string, unknown>) {
  if (!sentryEnabled) {
    return
  }
  Sentry.addBreadcrumb({
    category: 'user_intent',
    message: intent,
    level: 'info',
    data: context,
  })
}

export function captureStateChange(state: string, from: string, to: string) {
  if (!sentryEnabled) {
    return
  }
  Sentry.addBreadcrumb({
    category: 'state_change',
    message: `${state}: ${from} -> ${to}`,
    level: 'info',
  })
}
```

**Step 4: Error classification helpers**

`src/observability/errors.ts`:

```ts
import { Sentry, sentryEnabled } from './sentry'

export class RecoverableError extends Error {
  constructor(
    message: string,
    public context: Record<string, unknown> = {}
  ) {
    super(message)
  }
}

export class CriticalError extends Error {
  constructor(
    message: string,
    public context: Record<string, unknown> = {}
  ) {
    super(message)
  }
}

export class ExpectedError extends Error {}

export function reportError(error: unknown, context?: Record<string, unknown>) {
  if (!sentryEnabled || error instanceof ExpectedError) {
    return
  }
  Sentry.captureException(error, { extra: context })
}
```

**Step 5: Initialize Sentry early in app entry**

Update `src/main.tsx` so the first side-effect is `initSentry()` and the app is wrapped with a Sentry ErrorBoundary:

```tsx
import { initSentry, Sentry } from './observability/sentry'

initSentry()

ReactDOM.createRoot(...).render(
  <React.StrictMode>
    <Provider store={store}>
      <Sentry.ErrorBoundary fallback={<div>Something went wrong.</div>}>
        <App />
      </Sentry.ErrorBoundary>
    </Provider>
  </React.StrictMode>
)
```

**Step 6: Frontend tests for trace manager**

Create `src/observability/__tests__/trace.test.ts`:

```ts
import { traceManager } from '../trace'

describe('traceManager', () => {
  it('generates unique trace ids', () => {
    const first = traceManager.startTrace('test')
    traceManager.endTrace()
    const second = traceManager.startTrace('test')
    expect(first.traceId).not.toBe(second.traceId)
    traceManager.endTrace()
  })
})
```

Run: `bun run test --run src/observability/__tests__/trace.test.ts`

**Step 7: Commit**

```bash
git add src/observability src/main.tsx
git commit -m "feat: add frontend observability foundation"
```

---

## Task 3: Trace-aware Tauri invoke wrapper

**Files:**

- Create: `src/lib/tauri-command.ts`
- Update: `src/api/*.ts`
- Update: `src/contexts/SettingContext.tsx`
- Update tests: `src/api/__tests__/clipboardItems.test.ts`, `src/api/__tests__/security.test.ts`, `src/components/clipboard/__tests__/ClipboardItem.test.tsx`

**Step 1: Create invoke wrapper**

`src/lib/tauri-command.ts`:

```ts
import { invoke } from '@tauri-apps/api/core'
import { traceManager } from '@/observability/trace'
import { Sentry } from '@/observability/sentry'

export async function invokeWithTrace<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const trace = traceManager.startTrace(command)

  Sentry.addBreadcrumb({
    category: 'tauri_command',
    message: command,
    level: 'info',
    data: { traceId: trace.traceId, args },
  })

  try {
    return await invoke<T>(command, {
      ...args,
      _trace: {
        trace_id: trace.traceId,
        timestamp: trace.startTime,
      },
    })
  } catch (error) {
    Sentry.captureException(error, {
      tags: { command, traceId: trace.traceId },
      extra: { args },
    })
    throw error
  } finally {
    traceManager.endTrace()
  }
}
```

**Step 2: Replace invoke usage**

Update each API module to import and call `invokeWithTrace` instead of `invoke`:

- `src/api/clipboardItems.ts`
- `src/api/security.ts`
- `src/api/onboarding.ts`
- `src/api/p2p.ts`
- `src/api/vault.ts`
- `src/contexts/SettingContext.tsx`
- `src/main.tsx` (for `frontend_ready`)

**Step 3: Update tests to mock wrapper**

Replace `vi.mock('@tauri-apps/api/core'...)` with `vi.mock('@/lib/tauri-command'...)` in:

- `src/api/__tests__/clipboardItems.test.ts`
- `src/api/__tests__/security.test.ts`
- `src/components/clipboard/__tests__/ClipboardItem.test.tsx`

**Step 4: Run frontend tests**

Run: `bun run test --run src/api/__tests__/clipboardItems.test.ts`
Run: `bun run test --run src/api/__tests__/security.test.ts`
Run: `bun run test --run src/components/clipboard/__tests__/ClipboardItem.test.tsx`

**Step 5: Commit**

```bash
git add src/lib/tauri-command.ts src/api src/contexts/SettingContext.tsx src/main.tsx
git commit -m "feat: inject trace metadata into tauri invokes"
```

---

## Task 4: Add user-intent breadcrumbs

**Files:**

- Modify: `src/components/clipboard/ClipboardContent.tsx`
- Modify: `src/pages/SettingsPage.tsx`
- Modify: `src/pages/DevicesPage.tsx`

**Step 1: Clipboard action breadcrumbs**

In `src/components/clipboard/ClipboardContent.tsx` call `captureUserIntent` in:

- `handleCopyItem`
- `handleBatchCopy`
- `handleBatchDelete`
- `handleBatchToggleFavorite`

Example:

```ts
captureUserIntent('copy_clipboard', { count: 1 })
```

**Step 2: Settings and pairing breadcrumbs**

- In `src/pages/SettingsPage.tsx` call `captureUserIntent('open_settings')` on mount.
- In `src/pages/DevicesPage.tsx` add `captureUserIntent('pair_device')` when pairing actions start.

**Step 3: Run lint or unit tests if they cover these components**

Run: `bun run test --run src/components/clipboard/__tests__/ClipboardItem.test.tsx`

**Step 4: Commit**

```bash
git add src/components/clipboard/ClipboardContent.tsx src/pages/SettingsPage.tsx src/pages/DevicesPage.tsx
git commit -m "feat: capture user intent breadcrumbs"
```

---

## Task 5: Add trace metadata to uc-core ports

**Files:**

- Create: `src-tauri/crates/uc-core/src/ports/observability.rs`
- Modify: `src-tauri/crates/uc-core/src/ports/mod.rs`

**Step 1: Add TraceMetadata type**

`src-tauri/crates/uc-core/src/ports/observability.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub trace_id: Uuid,
    pub timestamp: u64,
}

pub type OptionalTrace = Option<TraceMetadata>;

pub fn extract_trace(args: &serde_json::Value) -> OptionalTrace {
    args.get("_trace")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_trace_metadata() {
        let args = json!({
            "_trace": {
                "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
                "timestamp": 1737100000000u64
            }
        });

        let trace = extract_trace(&args).expect("trace metadata");
        assert_eq!(trace.trace_id.to_string(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
        assert_eq!(trace.timestamp, 1737100000000u64);
    }
}
```

**Step 2: Export from ports**

Update `src-tauri/crates/uc-core/src/ports/mod.rs`:

```rust
pub mod observability;
pub use observability::{extract_trace, OptionalTrace, TraceMetadata};
```

**Step 3: Run Rust tests**

Run from `src-tauri/`: `cargo test -p uc-core observability`

**Step 4: Commit**

```bash
git add src-tauri/crates/uc-core/src/ports/observability.rs src-tauri/crates/uc-core/src/ports/mod.rs
git commit -m "feat(core): add trace metadata port"
```

---

## Task 6: Apply trace metadata to Tauri command spans

**Note:** This introduces a high-cardinality `trace_id` field in command spans. Limit it to command-level spans only to avoid violating `docs/guides/tracing.md` in deeper layers.

**Files:**

- Modify: `src-tauri/crates/uc-tauri/src/commands/*.rs`

**Step 1: Add `_trace` argument**

For each command function, add an optional trace parameter:

```rust
use uc_core::ports::observability::TraceMetadata;

pub async fn some_command(
    runtime: State<'_, Arc<AppRuntime>>,
    /* existing args */,
    _trace: Option<TraceMetadata>,
) -> Result<..., String> {
    let span = if let Some(trace) = &_trace {
        info_span!("command.xxx", trace_id = %trace.trace_id, trace_ts = trace.timestamp, /* existing fields */)
    } else {
        info_span!("command.xxx", /* existing fields */)
    };
    async move { ... }.instrument(span).await
}
```

Apply to:

- `src-tauri/crates/uc-tauri/src/commands/clipboard.rs`
- `src-tauri/crates/uc-tauri/src/commands/encryption.rs`
- `src-tauri/crates/uc-tauri/src/commands/onboarding.rs`
- `src-tauri/crates/uc-tauri/src/commands/pairing.rs`
- `src-tauri/crates/uc-tauri/src/commands/settings.rs`
- `src-tauri/crates/uc-tauri/src/commands/autostart.rs`
- `src-tauri/crates/uc-tauri/src/commands/startup.rs`

**Step 2: Run Rust tests**

Run from `src-tauri/`: `cargo test -p uc-tauri`

**Step 3: Commit**

```bash
git add src-tauri/crates/uc-tauri/src/commands
git commit -m "feat(tauri): attach trace metadata to command spans"
```

---

## Task 7: Verification checklist

**Manual verification:**

- Launch app (production build) with `VITE_SENTRY_DSN` set.
- Trigger a clipboard action and confirm Sentry breadcrumbs include `user_intent` and `tauri_command`.
- Confirm backend log line contains `trace_id` for matching command.

**Optional:** Run lint: `bun run lint`

---

## Notes

- React is on v18, so ErrorBoundary is the safe integration.
- Sentry docs: https://docs.sentry.io/platforms/javascript/guides/react/
- Trace propagation targets are not required for tauri:// origins; we only use custom `_trace`.
