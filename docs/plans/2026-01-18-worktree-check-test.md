# Worktree Check - Test Scenarios

## Purpose

Test whether agents detect when already in a worktree and skip creation, versus current behavior (always tries to create new worktree).

## RED Phase - Baseline (Current Behavior)

**Testing WITHOUT the worktree check feature**

### Test Method

Dispatch subagent with a task that would trigger `using-git-worktrees`, while already inside a worktree. Document whether agent:

1. Detects they're already in a worktree
2. Skips worktree creation
3. Proceeds directly to work

### Baseline Expectation

Agent will NOT detect they're in a worktree and will attempt to create a new worktree anyway.

---

## GREEN Phase - With Feature

**Testing WITH the worktree check feature**

### Expected Behavior

Agent should:

1. Check if currently in a worktree first
2. If yes: Skip creation, announce already in worktree, proceed to work
3. If no: Follow existing worktree creation flow

---

## Test Scenarios

### Scenario 1: Already in Worktree

**Setup:** Agent is inside `.worktrees/feature-auth/`

**Task:** "Implement login validation"

**Baseline (current):**

- Agent checks for `.worktrees/` directory
- Finds it exists
- Attempts to create new worktree anyway (maybe `.worktrees/feature-auth-2/`)
- Or asks user where to create

**With Feature:**

- Agent detects `git worktree list` shows current path is a worktree
- Announces: "Already in worktree at .worktrees/feature-auth, proceeding directly to work"
- Skips creation, starts implementation

### Scenario 2: Not in Worktree (Main Branch)

**Setup:** Agent is in main repo directory on `main` branch

**Task:** "Implement login validation"

**Expected (both baseline and with feature):**

- Follows normal worktree creation flow
- Should create new worktree

### Scenario 3: Worktree with Non-standard Name

**Setup:** Agent is in `~/workspaces/myproject-feature/` (a manually created worktree)

**Task:** "Add dark mode support"

**Baseline:** Creates another worktree

**With Feature:** Detects it's a worktree, proceeds directly

---

## Detection Method

How to detect if currently in a worktree:

```bash
# Method 1: Check if .git file exists (worktree indicator)
[ -f .git ] && echo "In worktree"

# Method 2: Use git worktree list
git worktree list --porcelain | grep -q "worktree $(pwd)" && echo "In worktree"

# Method 3: Compare git dir with main
[ "$(git rev-parse --git-dir)" != "$(git rev-parse --show-toplevel)/.git" ] && echo "In worktree"
```

**Recommended:** Method 2 (most reliable across git versions)
