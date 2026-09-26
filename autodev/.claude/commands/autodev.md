# Automated Development Cycle

**AUTOMATED WORKFLOW FOR DEVELOPMENT**

This command automates the entire development cycle from task discovery to PR creation.

---

## Overview

You will execute the complete development workflow for the next sequential GitHub issue:

0. **Pre-flight Check** - Sync with latest main branch (if on main)
1. **Task Discovery** - Identify next issue via scrum-master-pm agent
2. **Branch Creation** - Create/switch to feature branch
3. **Implementation** - Execute via master-orchestrator agent
4. **Quality Validation** - Comprehensive testing via test-guardian agent
5. **Git & PR** - Commit, push, and create pull request
6. **Cleanup** - Return to main branch

---

## Configuration

Read workflow settings and agent prompts from:
`.claude/config/autodev-workflow.md`

Extract these settings:
- `auto_push`: Auto push to remote after successful tests
- `auto_pr`: Auto create PR without user confirmation
- `auto_merge`: Auto merge PR (requires approval)
- `auto_cleanup`: Auto checkout main after PR creation

---

## Step-by-Step Execution

### STEP 0: PRE-FLIGHT CHECK

**Objective**: Ensure we have the latest code from main branch

**Action**: Check current branch and sync with remote

```bash
current_branch=$(git branch --show-current)
if [ "$current_branch" == "main" ]; then
  git pull origin main
fi
```

**Decision Point**:
- If on main -> Pull latest and continue
- If on feature branch -> Continue (will sync in Step 2)
- If pull fails -> Report error and retry

---

### STEP 1: TASK DISCOVERY

**Objective**: Find the next sequential issue to implement

**Action**: Use Task tool with `subagent_type: scrum-master-pm`

**Extract from response**:
- `NEXT_ISSUE`: Issue number (e.g., #7)
- `ISSUE_TITLE`: Issue title
- `BRANCH_NAME`: Feature branch name
- `BLOCKERS`: Any blockers preventing work
- `PHASE`: Current development phase

**Decision Point**:
- If `BLOCKERS != None` -> STOP and report blockers to user
- If `NEXT_ISSUE` not found -> Report "All issues completed" or error
- Otherwise -> Continue to Step 2

---

### STEP 2: BRANCH CREATION

**Objective**: Create and switch to feature branch for the issue

**Action**: Execute git commands

```bash
git checkout main
git pull origin main
git checkout -b {BRANCH_NAME}
```

**Decision Point**:
- If branch already exists -> Checkout existing branch
- If git errors -> Report to user and STOP
- Otherwise -> Continue to Step 3

---

### STEP 3: IMPLEMENTATION

**Objective**: Implement the issue using master-orchestrator

**Action**: Use Task tool with `subagent_type: master-orchestrator`

**Monitor Progress**:
- Report each specialized agent invocation
- Show file changes as they happen
- Display any errors immediately

**Decision Point**:
- If implementation fails -> Report errors and STOP
- If implementation succeeds -> Continue to Step 4

---

### STEP 4: QUALITY VALIDATION

**Objective**: Validate implementation meets all quality standards

**Action**: Use Task tool with `subagent_type: test-guardian`

**Validation Checklist**:
- All backend tests pass
- All frontend tests pass
- Linting passes
- Type checking passes
- Constitution compliance verified
- Security checks passed
- Migration safety confirmed

**Decision Point**:
- If ANY check fails -> STOP and ask user for options
- If ALL checks pass -> Continue to Step 5

---

### STEP 5: GIT OPERATIONS & PR CREATION

**Objective**: Commit changes, push to remote, and create PR

#### 5.1: Determine Commit Type & Message

Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`

#### 5.2: Git Add & Commit

```bash
git add .
git commit -m "{commit_message}"
```

#### 5.3: Git Push (if auto_push enabled)

```bash
git push -u origin {BRANCH_NAME}
```

#### 5.4: Create Pull Request (if auto_pr enabled)

```bash
gh pr create \
  --title "[T{ISSUE_NUMBER_PADDED}] {ISSUE_TITLE}" \
  --body "{PR_BODY}" \
  --reviewer {REVIEWER}
```

---

### STEP 6: CLEANUP & NEXT CYCLE PREP

**Objective**: Return to main branch and prepare for next issue

```bash
git checkout main
git pull origin main
git fetch --prune
```

---

## Command Options

Usage: `/autodev [options]`

Options:
- `/autodev` - Run full automated cycle
- `/autodev --dry-run` - Show what would be done without executing
- `/autodev --skip-tests` - Skip validation (NOT RECOMMENDED)
- `/autodev --resume` - Resume from last failed step
- `/autodev --issue N` - Work on specific issue

---

## Execution Instructions

**YOU MUST**:

1. Read the configuration file: `.claude/config/autodev-workflow.md`
2. Execute each step sequentially (0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6)
3. Use Task tool to invoke agents
4. Report progress after each step
5. STOP immediately if any validation fails

**YOU MUST NOT**:

1. Skip any steps
2. Proceed if tests fail
3. Create PR without quality validation
4. Bypass constitution compliance checks

---

## Start Execution

BEGIN automated development cycle now.

Go!
