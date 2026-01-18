---
name: executing-plans
description: Execute an implementation plan with verification and review checkpoints
---
```

# Executing Plans

## Overview

Load plan, review critically, decompose into sequential tasks, execute tasks one by one, and report progress in review checkpoints.

**Core principle:**
Sequential execution with verification and review checkpoints.

**Announce at start:**
"I'm using the executing-plans skill to implement this plan."

---

## Execution Model

### Your Role

You act as:

* Planner - Review and decompose the plan
* Executor - Implement each task step by step
* Verifier - Run tests and verify changes
* Reviewer - Coordinate code reviews at checkpoints

---

## The Process

---

### Step 1: Load and Review Plan

1. Read the full plan
2. Review critically:

   * Architecture consistency
   * Dependency order
   * Missing steps
   * Risk areas
3. If concerns exist:

   * STOP
   * Raise questions to human partner
4. If no blocking issues:

   * Decompose plan into sequential tasks
   * Create TodoWrite
   * Proceed to execution

---

### Step 2: Execute Tasks Sequentially

For each task:

1. Mark task as in_progress
2. Follow plan steps exactly
3. Implement the required changes
4. Mark task as completed

Proceed to next task only after current task is complete.

---

### Step 3: Review Checkpoint

After all tasks are complete:

1. Verify the implementation:

   * Consistency
   * Architecture alignment
   * Interface compatibility
2. Resolve conflicts if needed
3. Produce summary:

   * What was implemented
   * Verification results
   * Open risks

---

### Step 4: Verification (MANDATORY)

Before code review, you MUST verify work is actually complete:

1. Identify verification commands:
   - Test command (e.g., `cargo test`, `npm test`)
   - Build command (e.g., `cargo build`, `npm run build`)
   - Lint/typecheck (if applicable)

2. Run EACH verification command:
   ```bash
   # Run the FULL command, not partial checks
   <verification_command>
   ```

3. READ and VERIFY output:
   - Exit code is 0 (success)
   - No failures in tests
   - No critical errors in build
   - Count test results (e.g., "34/34 passing")

4. Create verification summary:
   - What commands were run
   - Actual results (with counts/exit codes)
   - Any failures or errors found

5. ONLY if all verifications pass, proceed to Step 5

**CRITICAL RULES:**
- NO completion claims without FRESH verification evidence
- Do NOT skip verification even if "simple"
- Do NOT proceed with failing tests

**Red Flags - STOP:**
- Using "should pass", "probably works", "seems correct"
- About to proceed without running verification commands

---

### Step 5: Code Review (MANDATORY)

1. Get git SHAs for the changes:
   ```bash
   BASE_SHA=$(git rev-parse HEAD~<num_tasks>)
   HEAD_SHA=$(git rev-parse HEAD)
   ```

2. Use requesting-code-review skill:
   - WHAT_WAS_IMPLEMENTED: Summary of tasks completed
   - PLAN_OR_REQUIREMENTS: Reference to plan file being executed
   - BASE_SHA: Commit before work started
   - HEAD_SHA: Current commit after changes
   - DESCRIPTION: Brief summary of changes

3. Act on feedback:
   - Fix Critical issues before proceeding
   - Fix Important issues before continuing
   - Note Minor issues for final review

4. Only after review is complete, announce:

"Work complete. Ready for feedback."

WAIT for human review before continuing.

---

### Step 6: Complete Development

After ALL tasks are completed and verified:

Announce:

"I'm using the finishing-a-development-branch skill to complete this work."

**REQUIRED SUB-SKILL:**
Use superpowers:finishing-a-development-branch

Follow that skill strictly for:

* Final verification
* Test execution
* Merge readiness
* Output formatting

---

## When to Stop and Ask for Help

STOP IMMEDIATELY if:

* You hit a blocker
* Required dependency is missing
* Test or verification repeatedly fails
* Instructions are ambiguous
* Plan conflicts with architecture constraints

DO NOT guess.
Request clarification.

---

## When to Revisit Earlier Steps

Return to Step 1 when:

* Plan is updated
* Architecture direction changes
* Scope is redefined

---

## Enforcement Rules (Critical)

If ANY of the following happens, execution is INVALID:

* Tasks are skipped
* Verification steps are ignored
* Code review checkpoint is skipped

---

## Remember

* Execute tasks sequentially
* Always verify before claiming completion
* Always pause at review checkpoints
* Stop on blockers
* Do not improvise beyond plan

