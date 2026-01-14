# Rust Test Coverage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add test coverage display for Rust backend using cargo-llvm-cov with local HTML reports and CI/CD Codecov integration.

**Architecture:**

- Local: npm script → cargo-llvm-cov → HTML report in target/llvm-cov/html/
- CI: GitHub Actions → cargo-llvm-cov (lcov) → Codecov uploader → PR comment
- Coverage tracking via Codecov with incremental diffs, no minimum threshold enforcement

**Tech Stack:**

- `cargo-llvm-cov`: LLVM-based coverage tool for Rust
- `taiki-e/install-action`: GitHub Action for installing llvm-cov
- `codecov/codecov-action@v4`: Codecov uploader
- `integration_tests` feature flag for existing test suite

---

## Task 1: Add npm script for local coverage

**Files:**

- Modify: `package.json` (root)

**Step 1: Add test:coverage script to package.json**

Add to the `scripts` section:

```json
"test:coverage": "cd src-tauri && cargo llvm-cov --html --workspace --features=integration_tests"
```

**Step 2: Verify the script is valid**

Run: `bun run test:coverage 2>&1 | head -20`
Expected: cargo-llvm-cov may prompt to install necessary components on first run

**Step 3: Commit**

```bash
git add package.json
git commit -m "feat: add test:coverage npm script using cargo-llvm-cov"
```

---

## Task 2: Create GitHub Actions coverage workflow

**Files:**

- Create: `.github/workflows/coverage.yml`

**Step 1: Create the coverage workflow file**

Create `.github/workflows/coverage.yml`:

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
        uses: actions/checkout@v6

      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov

      - name: Setup Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1.15.2

      - name: Generate coverage report
        run: |
          cd src-tauri
          cargo llvm-cov --lcov --output-path lcov.info --workspace --features=integration_tests

      - name: Upload to Codecov
        uses: codecov/codecov-action@v5.5.2
        with:
          files: ./src-tauri/lcov.info
          flags: rust-backend
          name: rust-coverage
          fail_ci_if_error: false
```

**Step 2: Verify YAML syntax**

Run: `yamllint .github/workflows/coverage.yml 2>&1 || echo "yamllint not installed, skipping"`
Expected: No syntax errors (if yamllint available)

**Step 3: Commit**

```bash
git add .github/workflows/coverage.yml
git commit -m "ci: add coverage workflow with cargo-llvm-cov and Codecov"
```

---

## Task 3: Create Codecov configuration

**Files:**

- Create: `codecov.yml`

**Step 1: Create codecov.yml in project root**

Create `codecov.yml`:

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

**Step 2: Verify YAML syntax**

Run: `yamllint codecov.yml 2>&1 || echo "yamllint not installed, skipping"`
Expected: No syntax errors (if yamllint available)

**Step 3: Commit**

```bash
git add codecov.yml
git commit -m "ci: add Codecov configuration with incremental coverage"
```

---

## Task 4: Remove legacy tarpaulin configuration

**Files:**

- Delete: `src-tauri/tarpaulin.toml`
- Modify: `src-tauri/Makefile` (if it references tarpaulin)

**Step 1: Check if Makefile references tarpaulin**

Run: `grep -n "tarpaulin" src-tauri/Makefile || echo "No tarpaulin references found"`
Expected: Output shows any tarpaulin references or confirms none exist

**Step 2: Delete tarpaulin.toml**

Run: `rm src-tauri/tarpaulin.toml`

**Step 3: Remove tarpaulin from Makefile (if present)**

If step 1 found references, remove the tarpaulin target/lines from Makefile.

**Step 4: Verify no other tarpaulin references**

Run: `grep -r "tarpaulin" . --include="*.yml" --include="*.yaml" --include="*.json" --include="*.toml" 2>/dev/null | grep -v target/ | grep -v ".git/" || echo "Clean"`
Expected: No remaining tarpaulin references (excluding build artifacts)

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: remove tarpaulin configuration, migrated to llvm-cov"
```

---

## Task 5: Test local coverage generation

**Files:**

- No file changes (verification task)

**Step 1: Run coverage locally**

Run: `bun run test:coverage`
Expected: Tests compile and run, HTML report generated to `src-tauri/target/llvm-cov/html/index.html`

**Step 2: Verify report exists**

Run: `ls -la src-tauri/target/llvm-cov/html/index.html`
Expected: File exists with non-zero size

**Step 3: Check coverage summary (optional)**

Run: `cd src-tauri && cargo llvm-cov --workspace --features=integration_tests --text`
Expected: Terminal output showing per-file coverage percentages

**Step 4: No commit needed** (verification only)

---

## Task 6: Update documentation (optional)

**Files:**

- Modify: `README.md` or `CONTRIBUTING.md` (if they exist)
- Modify: `CLAUDE.md` (development commands section)

**Step 1: Check if README exists**

Run: `ls README.md 2>/dev/null && echo "Exists" || echo "Not found"`

**Step 2: Add coverage command to development docs**

If README or CONTRIBUTING exists, add to development/testing section:

````markdown
### Test Coverage

Generate local coverage report:

```bash
bun run test:coverage
open src-tauri/target/llvm-cov/html/index.html
```
````

Coverage is automatically uploaded to Codecov on each push/PR.

````

**Step 3: Commit**

```bash
git add README.md CONTRIBUTING.md CLAUDE.md 2>/dev/null || true
git commit -m "docs: add test coverage instructions"
````

---

## Task 7: Verify and push

**Files:**

- No file changes (verification task)

**Step 1: Verify all changes**

Run: `git status`
Expected: Staged changes only, no untracked files

**Step 2: Review commits**

Run: `git log --oneline -7`
Expected: Commits follow conventional commit format

**Step 3: Push to remote**

Run: `git push origin mkdir700/cairo`
Expected: Pushes changes to GitHub, triggers coverage workflow

**Step 4: Monitor CI workflow**

After push, visit GitHub Actions tab to verify the coverage workflow runs successfully.

**Step 5: No commit needed** (verification only)

---

## Success Criteria Verification

After implementation, verify:

- [ ] `bun run test:coverage` generates HTML report locally
- [ ] `.github/workflows/coverage.yml` exists and is valid YAML
- [ ] `codecov.yml` exists with proper ignore patterns
- [ ] `tarpaulin.toml` is deleted
- [ ] CI workflow runs on push/PR
- [ ] Codecov shows coverage report (after first CI run)

## Notes

- First run of `cargo llvm-cov` may take longer as it instruments the code
- Codecov needs to be configured for the repository (visit codecov.io if not already set up)
- Coverage reports are informational only and won't block PR merges
- For local development, the HTML report provides detailed line-by-line coverage visualization
