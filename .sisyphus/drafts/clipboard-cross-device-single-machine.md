# Draft: Cross-device Clipboard Transfer (single-device development)

## Original Request

- User wants to design + implement cross-device clipboard content transfer.
- Current constraint: only one physical device available for development/testing.

## Context (repo-level, inferred)

- Repo appears to be UniClipboard (Rust + Tauri), aiming for privacy-first E2E-encrypted clipboard sync.
- Architecture guideline in this repo: `uc-app -> uc-core <- uc-infra / uc-platform` (hexagonal boundaries).

## Key Challenges

- Validate a multi-device protocol with only one machine.
- Avoid coupling sync engine to OS clipboard / network stack (needs ports + mocks).
- Decide transport topology early (LAN-only vs relay/server vs hybrid).
- Support rich clipboard types (text/image/file) vs MVP scope.

## Candidate Approach (for discussion)

- Treat a "device" as a logical identity + local state, not a physical machine.
- Run 2 logical peers on one machine (separate data dirs / profiles + distinct ports) and connect over `localhost`.
- Keep sync engine in `uc-core` with ports for Clipboard / Transport / Storage so:
  - unit tests can simulate clipboard events deterministically
  - integration/e2e can spin up two peers as processes for real protocol verification

## Open Questions

- Can we run two app instances on the same machine for initial validation?
- MVP connectivity: LAN direct, relay server, or both?
- MVP content types: text only vs include images/files?
- UX goal: automatic sync vs manual send/pull?

## Scope Boundaries (not set yet)

- INCLUDE: TBD
- EXCLUDE: TBD
