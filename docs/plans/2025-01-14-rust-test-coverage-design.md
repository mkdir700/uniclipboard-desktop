# Rust Test Coverage Display Design

**Date:** 2025-01-14
**Status:** Design Approved
**Author:** Claude Code

## Overview

Add test coverage support for the Rust backend using `cargo-llvm-cov`, with local HTML reports and CI/CD integration via Codecov for incremental coverage tracking.

## Architecture

### Three Components

1. **Local Development**
   - npm script `bun run test:coverage` generates HTML reports
   - Output: `target/llvm-cov/html/index.html`
   - Interactive browser-based coverage exploration

2. **CI/CD Workflow**
   - GitHub Actions job runs on push/PR
   - Generates lcov format report
   - Uploads to Codecov via codecov-action

3. **Codecov Configuration**
   - PR comments with incremental coverage diff
   - No minimum threshold enforcement
   - Historical tracking and trend visualization

### Dependency Flow

`npm script` → `cargo llvm-cov` → `lcov report` → `Codecov API` → PR comment

## Implementation

### Local Development

**package.json:**

```json
{
  "scripts": {
    "test:coverage": "cd src-tauri && cargo llvm-cov --html --workspace --features=integration_tests"
  }
}
```

**Usage:**

```bash
bun run test:coverage
open target/llvm-cov/html/index.html  # macOS
```

**Optional VS Code Integration:**

- Install `rust-analyzer` coverage extension
- View inline coverage in editor (green=covered, red=uncovered)

### CI/CD Workflow

**File:** `.github/workflows/coverage.yml`

```yaml
name: Coverage

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  coverage:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Setup Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Generate coverage report
        run: |
          cd src-tauri
          cargo llvm-cov --lcov --output-path lcov.info --workspace --features=integration_tests

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: ./src-tauri/lcov.info
          flags: rust-backend
          name: rust-coverage
          fail_ci_if_error: false
```

### Codecov Configuration

**File:** `codecov.yml` (project root)

```yaml
coverage:
  status:
    # Project-wide coverage (informational only)
    project:
      default:
        target: auto
        threshold: 0%
        base: auto
    # Patch coverage (new code changes)
    patch:
      default:
        target: auto
        threshold: 0%
        informational: true # Don't fail PR, just show diff

# Exclude generated code, tests, and migrations
ignore:
  - 'src-tauri/migrations/**'
  - '**/tests/**'
  - '**/*_test.rs'
  - '**/integration_test.rs'
  - '**/generated/**'

# PR comment settings
comment:
  require_base: false
  require_changes: false
  layout: 'reach,diff,flags,files,footer'
  behavior: default
```

### Workspace Compatibility

The project uses a workspace structure with multiple crates:

```
src-tauri/crates/
├── uc-app/
├── uc-core/
├── uc-infra/
├── uc-platform/
└── uc-tauri/
```

The `--workspace` flag ensures coverage is aggregated across all crates.

### Feature Flags

Use `--features=integration_tests` to match existing test configuration (previously in tarpaulin.toml).

## Migration from Tarpaulin

**Action Items:**

1. Delete `src-tauri/tarpaulin.toml`
2. Remove any tarpaulin references from Makefile or scripts
3. Update CI if tarpaulin was previously configured

**Why llvm-cov over tarpaulin:**

- More accurate for async code and complex type systems
- Better macro expansion support
- Official Rust toolchain integration
- Community trend direction

## Error Handling

- **CI failures:** Coverage job failure does not block PR merge (not a required check)
- **Local errors:** Clear error messages for missing tests or compilation failures
- **Codecov upload:** Uses `fail_ci_if_error: false` to avoid blocking on upload issues

## Success Criteria

- [ ] Local `bun run test:coverage` generates HTML report
- [ ] CI runs coverage job on push/PR
- [ ] Codecov shows coverage report in PR comments
- [ ] Incremental coverage diff displays for new code
- [ ] No PR merge blocking based on coverage
- [ ] tarpaulin.toml removed

## Future Enhancements

- Add coverage badge to README
- Set minimum coverage thresholds once test suite matures
- Add `--text` output for terminal-friendly coverage summary
- Integrate with `cargo-nextest` for faster test runs
