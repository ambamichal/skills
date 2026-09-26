> Historical documentation: the source preview has known bugs and unverified behavior. Read the repository README and docs/KNOWN-LIMITATIONS.md before use.

# Automated Development Workflow

**Automated end-to-end development cycle for your projects**

Version: 1.0.0
Last Updated: 2025-11-23

---

## 🎯 Overview

This automation eliminates manual overhead in the development workflow, automating the entire cycle from task discovery to PR creation.

### Manual vs Automated Workflow

**Before (Manual - 6 steps)**:
1. ✋ Call scrum-master-pm agent to check next task
2. ✋ Manually create/switch branch based on agent response
3. ✋ Call master-orchestrator agent to plan and execute
4. ✋ Call test-guardian agent to validate
5. ✋ Manually run git add, commit, push, create PR
6. ✋ Manually checkout main and pull

**After (Automated - 1 command)**:
```bash
/autodev
```

Everything happens automatically! ✨

---

## 📦 Components

### 1. Configuration File
**Location**: `.claude/config/autodev-workflow.md`

Contains:
- Workflow settings (auto-push, auto-PR, auto-merge)
- Agent prompts for each step
- PR templates
- Error handling strategies
- Customization guide

### 2. Slash Command
**Location**: `.claude/commands/autodev.md`

Main automation orchestrator that:
- Executes 6-step workflow
- Invokes specialized agents
- Handles git operations
- Creates pull requests
- Reports progress

**Usage**:
```bash
/autodev                    # Full automated cycle
/autodev --dry-run         # Preview without executing
/autodev --skip-tests      # Skip validation (not recommended)
/autodev --issue N         # Work on specific issue
/autodev --resume          # Resume from last failure
```

### 3. Helper Script (Optional)
**Location**: `.specify/scripts/powershell/Invoke-AutoDev.ps1`

PowerShell script for:
- Standalone workflow execution
- Advanced git operations
- Custom CI/CD integration
- Testing automation components

**Usage**:
```powershell
# Find next issue
.\Invoke-AutoDev.ps1 -Action CheckNext

# Create branch
.\Invoke-AutoDev.ps1 -Action CreateBranch -IssueNumber 7

# Run all tests
.\Invoke-AutoDev.ps1 -Action RunTests

# Create PR
.\Invoke-AutoDev.ps1 -Action CreatePR -IssueNumber 7

# Full cycle
.\Invoke-AutoDev.ps1 -Action FullCycle -AutoPush -AutoPR
```

### 4. Continuous Development Loop
**Location**: `.specify/scripts/powershell/Start-AutoDevLoop.ps1`

**Fully automated infinite loop** that:
- Monitors PRs for merge status
- Automatically runs `/autodev` after PR merge
- Handles Claude Code rate limits (waits and retries)
- Runs indefinitely until stopped or max cycles reached

**Usage**:
```powershell
# Start continuous loop
.\Start-AutoDevLoop.ps1

# Limit number of cycles
.\Start-AutoDevLoop.ps1 -MaxCycles 10

# Custom poll interval (check PR every 30 seconds)
.\Start-AutoDevLoop.ps1 -PollInterval 30

# Dry run (preview without executing)
.\Start-AutoDevLoop.ps1 -DryRun
```

---

## 🚀 Quick Start

### First Time Setup

1. **Verify dependencies**:
   ```bash
   # Check GitHub CLI
   gh --version

   # Check Git
   git --version
   ```

2. **Review configuration**:
   ```bash
   # Open and review settings
   cat .claude/config/autodev-workflow.md
   ```

3. **Test with dry-run**:
   ```bash
   /autodev --dry-run
   ```

4. **Run first automated cycle**:
   ```bash
   /autodev
   ```

### Configuration Options

Edit `.claude/config/autodev-workflow.md`:

```yaml
version: "1.0.0"
workflow_name: "Automated Development Cycle"
enabled: true
auto_push: true          # ✅ Auto push after tests pass
auto_pr: true            # ✅ Auto create PR
auto_merge: false        # ⚠️ Require manual merge
auto_cleanup: true       # ✅ Auto return to main
```

---

## 📋 Workflow Steps

### Step 1: Task Discovery
**Agent**: scrum-master-pm
**Purpose**: Find next sequential issue

### Step 2: Branch Creation
**Purpose**: Set up isolated development environment

### Step 3: Implementation
**Agent**: master-orchestrator
**Purpose**: Execute issue implementation

### Step 4: Quality Validation
**Agent**: test-guardian
**Purpose**: Ensure code quality before PR

### Step 5: Git Operations & PR
**Purpose**: Commit changes and create pull request

### Step 6: Cleanup
**Purpose**: Prepare for next issue

---

## 🔧 Troubleshooting

### Common Issues

#### 1. "Next issue not found"
```bash
# Check open issues
gh issue list --state open --limit 50

# Manually specify issue
/autodev --issue 8
```

#### 2. "Tests failing"
```bash
# View detailed test output
cd backend && pytest -v

# Auto-fix linting
ruff check . --fix
```

#### 3. "Git push fails"
```bash
# Check git status
git status

# Pull latest changes
git pull origin main
```

---

## 📊 Performance Metrics

### Time Savings

**Manual workflow** (per issue):
- Task discovery: 2-3 min
- Branch creation: 1 min
- Implementation: varies
- Testing: 5-10 min
- Git operations: 3-5 min
- PR creation: 2-3 min
**Total overhead**: ~15-25 min per issue

**Automated workflow** (per issue):
- Automation overhead: ~2 min
**Total overhead**: ~2 min per issue

**Savings**: 13-23 min per issue

---

## 🔒 Security & Best Practices

1. **Always review PRs before merging**
2. **Keep auto_merge disabled initially**
3. **Test in small batches**
4. **Monitor for drift**
5. **Update prompts iteratively**

---

## 📚 Additional Resources

### SpecKit Commands

These work alongside automation:

- `/speckit.plan` - Create implementation plan
- `/speckit.tasks` - Generate task breakdown
- `/speckit.implement` - Manual implementation mode
- `/speckit.analyze` - Cross-artifact consistency check

---

## 🗺️ Roadmap

### v1.0.0 (Current)
- ✅ Basic automation (task → PR)
- ✅ Configurable prompts
- ✅ Quality validation
- ✅ Error handling

### v1.1.0 (Planned)
- ⏳ CI/CD integration
- ⏳ Auto-merge with approval
- ⏳ Metrics dashboard
- ⏳ Parallel task execution

---

**Happy Automating! 🚀**
