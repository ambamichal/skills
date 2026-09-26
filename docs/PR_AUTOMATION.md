> Historical documentation: the source preview has known bugs and unverified behavior. Read the repository README and docs/KNOWN-LIMITATIONS.md before use.

# Pull Request: Automated Development Workflow

## Title
feat: Automated Development Workflow for Sequential Issue Processing

## Summary

Implements comprehensive automation system to eliminate manual overhead in the development workflow. Reduces time from ~15-25 minutes to ~2 minutes per issue.

## Problem Statement

Current workflow requires 6 manual steps per issue:
1. ✋ Call scrum-master-pm agent to check next task
2. ✋ Manually create/switch branch
3. ✋ Call master-orchestrator agent to plan and execute
4. ✋ Call test-guardian agent to validate
5. ✋ Manually run git add, commit, push, create PR
6. ✋ Manually checkout main and pull

With many issues remaining, this manual overhead is unsustainable.

## Solution

Single command automation: `/autodev`

### Components

1. **Slash Command** (`.claude/commands/autodev.md`)
   - Main workflow orchestrator
   - Executes 6-step automated cycle
   - Handles decision points and error recovery

2. **Configuration File** (`.claude/config/autodev-workflow.md`)
   - Customizable agent prompts
   - Workflow settings (auto-push, auto-PR, auto-merge)
   - PR templates
   - Error handling strategies

3. **PowerShell Helper** (`.specify/scripts/powershell/Invoke-AutoDev.ps1`)
   - Standalone script for CI/CD integration
   - Advanced git operations
   - Component testing

4. **Documentation** (`docs/AUTOMATION-README.md`)
   - Complete usage guide
   - Troubleshooting
   - Customization instructions

### Automated Workflow Steps

1. **Task Discovery** - scrum-master-pm agent finds next sequential issue
2. **Branch Creation** - Auto create/checkout feature branch
3. **Implementation** - master-orchestrator delegates to specialized agents
4. **Quality Validation** - test-guardian validates all quality gates
5. **Git & PR** - Auto commit, push, create pull request
6. **Cleanup** - Return to main branch, ready for next cycle

### Quality Gates

Before PR creation, ALL checks must pass:
- ✅ All backend tests
- ✅ All frontend tests
- ✅ Linting
- ✅ Type checking
- ✅ Constitution compliance
- ✅ Security validation
- ✅ Migration safety

## Usage

### Basic Usage
```bash
/autodev                 # Full automated cycle
/autodev --dry-run      # Preview without executing
/autodev --issue N      # Work on specific issue
/autodev --resume       # Resume from last failure
```

### Configuration
Edit `.claude/config/autodev-workflow.md`:

```yaml
auto_push: true      # Auto push after tests pass
auto_pr: true        # Auto create PR
auto_merge: false    # Require manual merge (recommended)
auto_cleanup: true   # Auto return to main
```

### PowerShell (Optional)
```powershell
.\.specify\scripts\powershell\Invoke-AutoDev.ps1 -Action FullCycle
```

## Benefits

### Time Savings
- **Before**: 15-25 min overhead per issue
- **After**: 2 min overhead per issue

### Quality Improvements
- Consistent validation before every PR
- No skipped quality checks
- Automated constitution compliance verification

### Developer Experience
- Focus on implementation, not process
- Reduced context switching
- Clear progress tracking

## Constitution Compliance

- ✅ **Spec-Driven Development**: Follows existing spec workflow
- ✅ **API-First Architecture**: No architectural changes
- ✅ **Independent User Stories**: Each issue processed independently
- ✅ **MVP-First Mindset**: Uses AI agents for orchestration
- ✅ **Issue-Driven Development**: Enforces sequential order

## Breaking Changes

None - this is purely additive functionality.

## Files Changed

- **Added**: `.claude/commands/autodev.md` (Main slash command)
- **Added**: `.claude/config/autodev-workflow.md` (Configuration)
- **Added**: `.specify/scripts/powershell/Invoke-AutoDev.ps1` (Helper script)
- **Added**: `docs/AUTOMATION-README.md` (Documentation)

## Checklist

- [x] Configuration file created with customizable prompts
- [x] Main /autodev slash command implemented
- [x] PowerShell helper script created
- [x] Comprehensive documentation written
- [x] Error handling implemented
- [x] Dry-run mode tested
- [x] Quality gates defined
- [x] Constitution compliance verified

---

**Ready to save hours of manual work!** 🚀
