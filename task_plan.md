# Task Plan: Remove splash screen logic

## Goal

Remove all splash screen logic and assets from the app (Tauri window, startup barrier/handshake, generator scripts, templates, and docs) so startup is splash-free.

## Current Phase

Phase 4

## Phases

### Phase 1: Requirements & Discovery

- [x] Understand user intent
- [x] Identify constraints and requirements
- [x] Document findings in findings.md
- **Status:** complete

### Phase 2: Planning & Structure

- [x] Define technical approach
- [x] Identify all splash-related files and code paths
- [x] Document decisions with rationale
- **Status:** complete

### Phase 3: Implementation

- [x] Remove splash assets and generator scripts
- [x] Remove Tauri splash window creation and related startup logic
- [x] Remove frontend handshake and localStorage sync for splash
- [x] Update docs to reflect removal
- **Status:** complete

### Phase 4: Testing & Verification

- [ ] Verify build/dev startup still works without splash
- [ ] Document test results in progress.md
- **Status:** in_progress

### Phase 5: Delivery

- [ ] Review modified files
- [ ] Deliver summary and next steps
- **Status:** pending

## Key Questions

1. Where is splashscreen created and closed in Tauri startup flow?
2. What build scripts or assets generate/use splashscreen.html?
3. What frontend logic exists only for splashscreen handshakes?

## Decisions Made

| Decision                                                            | Rationale                                                             |
| ------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Remove splash end-to-end (assets, scripts, window logic, handshake) | User requested full removal; avoids dead code and hidden dependencies |
| Keep startup barrier to show main window after backend ready        | Main window starts hidden; barrier still needed without splash        |

## Errors Encountered

| Error                                                              | Attempt | Resolution                                                                                         |
| ------------------------------------------------------------------ | ------- | -------------------------------------------------------------------------------------------------- |
| `python3: can't open file '/scripts/session-catchup.py'`           | 1       | Reran with explicit path `/home/wuy6/.codex/skills/planning-with-files/scripts/session-catchup.py` |
| `fatal: pathspec 'dist/splashscreen.html' did not match any files` | 1       | Will re-run `git add` without non-existent dist files                                              |

## Notes

- Follow project rules: no unwrap/expect, no silent async failures, keep architecture boundaries.
