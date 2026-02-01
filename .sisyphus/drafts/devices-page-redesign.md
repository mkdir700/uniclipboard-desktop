# Draft: Devices Page Redesign

## Requirements (confirmed)

- Remove the page header on Devices page.
- Place the “Add device” entry at the end of the device list.
- If there are no other devices, the “Add device” button appears in the empty state with prompt to click and add.
- Pairing requests become toast-style notifications; toast should show when there is a request.
- Replace the large “no requests” dashed box with toast (remove that area).
- Remove the “no paired devices” dashed box; use a list container that shows devices if any.
- If there are no devices, show a centered empty-state illustration and an “Add device” button.
- Toast should be clickable; clicking opens the pairing modal (same flow as current).
- Pairing request modal should not auto-open; open only on toast click.
- Pairing request modal should be global (not only Devices page), so toast can appear on Dashboard too.
- Toast timeout should be configurable separately from other toasts.
- Pairing request toast should stay visible until user manually closes it; must provide manual close.

## Technical Decisions

- (Leaning) Use existing global P2PProvider + App-level overlays for global pairing toast + modal.
- (Confirmed) Toast system is Sonner; per-toast duration supported via options.
- (Confirmed) Pairing request toast duration should be `Infinity` (manual dismissal).
- (Confirmed) Empty-state illustration uses Lucide icon style (icon + background frame).
- (Confirmed) Pairing request modal is triggered only by toast click (no auto-popup on event).
- (Confirmed) No automated tests; use manual verification steps.
- (Default) Reuse existing i18n keys for pairing request toast content (`pairing.globalRequest.*`, `pairing.requests.*`).
- (Default) Multiple pairing requests: latest overwrites single pending request (no queue/aggregation).
- (Default) Empty-state icon: `Smartphone` (Lucide), styled like Clipboard empty state.

## Research Findings

- Toast system: Sonner Toaster mounted in `src/App.tsx`, with wrapper in `src/components/ui/sonner.tsx` and `toast` re-export in `src/components/ui/toast.ts`.
- Per-toast duration supported via `toast(..., { duration })`; example exists in `src/pages/DashboardPage.tsx`.
- Clickable toast: Sonner supports `action`/`cancel`; full-surface click requires custom JSX toast content.
- Global pairing already exists: `P2PProvider` in `src/contexts/P2PContext.tsx` + `GlobalPairingRequestDialog` and `PairingPinDialog` hosted in `src/App.tsx`.
- Devices page has redundant local listeners for pairing events; may be removed in favor of global context.
- Device list empty state currently uses dashed box in `src/components/device/OtherDevice.tsx`.
- Richer empty state pattern exists in `src/components/clipboard/ClipboardContent.tsx` (icon + title + description).
- No dedicated illustration assets found; icons from `lucide-react` are used as visual placeholders.

## Open Questions

- Do we have an existing empty-state illustration asset to reuse, or should we create a new one?

## Scope Boundaries

- INCLUDE: Devices page layout restructure, toast + modal global handling, empty state behavior.
- EXCLUDE: Changing the pairing flow logic itself beyond moving entry points.
