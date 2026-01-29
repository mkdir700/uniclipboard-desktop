# Findings

## Context

- Task: remove all splash screen logic and assets from UniClipboard.

## Discovery

- Splash assets/scripts: `scripts/generate-splash.ts`, `src/splashscreen/template.html`, `src/splashscreen/animations.css`, `public/splashscreen.html`, `public/splashscreen.css`.
- Build hooks: `package.json` scripts `generate:splash`, `predev`, `prebuild`.
- Frontend theme sync for splash: `src/contexts/SettingContext.tsx` writes `uc-theme` and `uc-theme-color` to localStorage for splash.
- Frontend handshake: `src/main.tsx` invokes `frontend_ready` command to coordinate startup barrier.
- Backend splash window and startup barrier: `src-tauri/src/main.rs` creates `splashscreen` window and injects theme; `src-tauri/crates/uc-tauri/src/commands/startup.rs` closes splashscreen and shows main window.
- Docs: `docs/plans/2026-01-18-splashscreen-generator-design.md`, `docs/plans/2026-01-18-splashscreen-generator-implementation.md`.
