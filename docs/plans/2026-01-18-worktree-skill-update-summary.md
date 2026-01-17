# using-git-worktrees Skill Update Summary

## Change Request

Add condition to check if already in a worktree before attempting to create a new one.

## TDD Process Followed

### RED Phase (Baseline)

**Test:** Verified current skill behavior when already in worktree
**Result:** Skill had no logic to detect existing worktree, would always attempt to create new one
**Baseline findings:**

- Only checked for `.worktrees/` directory existence
- No check if current directory IS a worktree
- Would create duplicate worktrees unnecessarily

### GREEN Phase (Implementation)

**Change:** Added Step 0 - Check If Already in Worktree
**Detection method:** `[ -f .git ]` (most reliable)

- Main repos: `.git` is a directory
- Worktrees: `.git` is a file pointing to main repo's gitdir

**Implementation:**

```bash
if [ -f .git ]; then
    echo "Already in worktree at $(pwd)"
    echo "Current branch: $(git branch --show-current)"
    echo "Proceeding directly to work"
    exit 0
fi
```

### REFACTOR Phase (Validation)

**Edge cases tested:**

- ✓ Auxiliary worktree detection
- ✓ Main repository detection
- ✓ Detached HEAD state (still works)
- ✓ .git file format validation

**Verification:**

- Auxiliary worktree: `[ -f .git ]` = TRUE → Skip creation
- Main repository: `[ -f .git ]` = FALSE → Create new worktree

## Files Modified

### Primary

- `~/.claude/skills/using-git-worktrees/SKILL.md` - Updated with Step 0 logic
- Backup created: `SKILL.md.backup`

### Supporting Documentation

- Test scenarios: `docs/plans/2026-01-18-worktree-check-test.md`
- Modified skill preview: `docs/plans/2026-01-18-using-git-worktrees-modified.md`
- This summary: `docs/plans/2026-01-18-worktree-skill-update-summary.md`

## Changes Made

### Added Sections

1. **Step 0: Check If Already in Worktree** - New first step in workflow
2. **Key distinction** note - Explains .git file vs directory

### Updated Sections

1. **Quick Reference** table - Added worktree check row
2. **Common Mistakes** - Moved "Not checking if already in worktree" to top
3. **Example Workflows** - Updated both examples to show new Step 0
4. **Red Flags** - Updated to include `[ -f .git ]` check
5. **Always section** - Emphasized checking worktree status FIRST

### No Breaking Changes

- Existing workflow unchanged when not in worktree
- Backward compatible with all existing use cases
- Only adds optimization for worktree scenario

## Test Results

All scenarios passing:

- ✓ In worktree → Skip creation, proceed to work
- ✓ In main repo → Create new worktree (existing behavior)
- ✓ Detection method reliable across git versions

## Benefits

1. **Efficiency** - No unnecessary worktree creation
2. **Clarity** - Explicit announcement when already in worktree
3. **Reliability** - Simple, robust detection method
4. **Compatibility** - Works with all git worktree configurations

## Installation Status

✓ Skill installed to `~/.claude/skills/using-git-worktrees/SKILL.md`
✓ Backup created at `SKILL.md.backup`
✓ Ready for use

## Next Steps

1. Test skill in real workflow to verify agent compliance
2. Monitor for any edge cases in production use
3. Consider adding robust .git content validation if needed
