# Automated Development Workflow Configuration

This configuration file defines the automated development cycle.
Modify prompts and settings here to customize the workflow without changing the core automation logic.

## Workflow Settings

```yaml
version: "1.1.0"
workflow_name: "Automated Development Cycle"
enabled: true
auto_push: false          # Automatically push changes after successful tests
auto_pr: false           # Automatically create PR (false = ask user confirmation)
auto_merge: false        # Automatically merge PR (false = requires manual merge)
auto_cleanup: false       # Auto checkout main and pull after PR creation
```

## Step 1: Task Discovery (scrum-master-pm agent)

### Purpose
Identify the next sequential issue to work on according to constitution principles.

### Agent Prompt
```
I need to identify the next sequential GitHub issue to work on.

Please:
1. Check the current project status and identify all completed issues
2. Review tasks.md to find the next sequential issue
3. Verify that all prerequisite issues are completed
4. Provide the following information:
   - Next issue number (e.g., #7)
   - Issue title
   - Brief description (1-2 sentences)
   - Branch name to create (format: feature/issue-N-brief-description)
   - Any blockers or dependencies
   - Current phase and progress percentage

Format your response as:
NEXT_ISSUE: #N
ISSUE_TITLE: Title here
BRANCH_NAME: feature/issue-N-description
BLOCKERS: None | List blockers
PHASE: Phase X - Name (N/M issues completed)
```

### Expected Output Format
```
NEXT_ISSUE: #7
ISSUE_TITLE: Create Tenant model with subdomain field
BRANCH_NAME: feature/issue-7-tenant-model
BLOCKERS: None
PHASE: Phase 2 - Foundational Infrastructure (1/24 issues completed)
```

---

## Step 2: Implementation Planning (master-orchestrator agent)

### Purpose
Create detailed implementation plan and delegate tasks to specialized agents.

### Agent Prompt
```
I need to implement the following GitHub issue: #{ISSUE_NUMBER}

Issue Title: {ISSUE_TITLE}

**CRITICAL: YOU ARE AN ORCHESTRATOR, NOT AN IMPLEMENTER**

You MUST NEVER write code yourself. Your ONLY job is to:
1. Analyze the issue
2. Break it into sub-tasks
3. Delegate to specialized agents
4. Coordinate their work
5. Integrate results

**STEP 1: Analysis**
1. Read the issue description and acceptance criteria from GitHub
2. Review relevant spec files in specs/ directory
3. Check the project constitution for compliance requirements
4. Identify the TYPE of work required:
   - Database models/migrations?
   - API endpoints?
   - Python/Django infrastructure (logging, middleware, settings)?
   - Frontend UI?
   - Background tasks?
   - Real-time features?
   - Infrastructure/DevOps?

**STEP 2: Task Breakdown**
Create a detailed implementation plan:
   - Required code changes (files to create/modify)
   - Database migrations needed
   - API endpoints to implement
   - Tests to write
   - Documentation to update

**STEP 3: Agent Delegation (MANDATORY - DO NOT SKIP!)**
You MUST use the Task tool to launch specialized agents. Map tasks to agents:

- **Database models/migrations** -> database-architect
- **API endpoints** -> backend-architect
- **Python/Django infrastructure** (logging, middleware, settings, utils) -> backend-architect
- **Frontend UI** (components, pages, routing) -> frontend-developer
- **Background tasks** (Celery, async) -> async-task-implementer
- **Real-time features** (WebSocket, Channels) -> realtime-integration-specialist
- **Infrastructure** (Docker, CI/CD, monitoring) -> devops-infrastructure
- **Documentation** (README, API docs, comments) -> docs-maintainer

**STEP 4: Execution**
For EACH sub-task:
  a) Launch the appropriate agent using Task tool
  b) Wait for agent to complete
  c) Verify the output
  d) Move to next sub-task

**STEP 5: Integration**
After ALL agents complete:
  a) Run auto-formatting
  b) Verify all files are properly formatted
  c) Report completion summary

**FORBIDDEN ACTIONS:**
- Writing code directly (use agents instead!)
- Creating files yourself (delegate to agents!)
- Implementing logic yourself (delegate to agents!)
- Skipping agent delegation
- Doing work that belongs to specialized agents

Report progress after each agent completes and confirm when ALL agents finish.
```

### Delegation Strategy
- **Database models/migrations** -> database-architect agent
- **API endpoints & business logic** -> backend-architect agent
- **Python/Django infrastructure** (logging, middleware, settings, utilities) -> backend-architect agent
- **Frontend UI** (React components, pages, routing) -> frontend-developer agent
- **Background tasks** (Celery, async jobs) -> async-task-implementer agent
- **Real-time features** (WebSocket, Django Channels) -> realtime-integration-specialist agent
- **Infrastructure** (Docker, CI/CD, deployment) -> devops-infrastructure agent
- **Documentation** (README, API docs, inline comments) -> docs-maintainer agent
- **Testing** (unit tests, integration tests, E2E) -> test-guardian agent

---

## Step 3: Quality Validation (test-guardian agent)

### Purpose
Validate that implementation meets all quality standards before creating PR.

### Agent Prompt
```
I have completed implementation of issue #{ISSUE_NUMBER}: {ISSUE_TITLE}

Please perform comprehensive quality validation:

1. **Run all tests**:
   - Backend: pytest with coverage report
   - Frontend: npm test (if applicable)
   - Integration tests
   - Contract tests (API contracts validation)

2. **Code quality checks**:
   - Backend linting: ruff check .
   - Backend formatting: ruff format --check .
   - Frontend linting: npm run lint (if applicable)
   - TypeScript type checking: npm run type-check (if applicable)

3. **Constitution compliance**:
   - Multi-tenancy: All queries filter by tenant_id
   - API-First: Backend exposes RESTful APIs
   - Independent: Feature can be deployed standalone
   - Spec-driven: Implementation matches spec requirements

4. **Security validation**:
   - No hardcoded secrets
   - Proper input validation
   - SQL injection prevention
   - XSS protection (frontend)

5. **Migration safety** (if database changes):
   - Migrations are reversible
   - No data loss in rollback
   - Migration tested locally

Provide a comprehensive report with:
- PASS or FAIL for each check
- Detailed error messages for failures
- Code coverage percentage
- Recommendations for improvements

ONLY mark as READY FOR PR if ALL checks pass.
```

### Success Criteria
All items must show PASS before proceeding to PR creation.

---

## Step 4: Git Operations & PR Creation

### Git Workflow
```bash
# STEP 1: Auto-format and auto-fix linting (MANDATORY - DO NOT SKIP!)
cd backend && ruff format . && ruff check --fix .
cd ../frontend && npm run format  # Only if frontend changes exist

# STEP 2: Stage all changes
git add .

# STEP 3: Commit with conventional commit format
git commit -m "{COMMIT_TYPE}: {COMMIT_MESSAGE} #{ISSUE_NUMBER}"

# Push to remote branch
git push -u origin {BRANCH_NAME}

# Create pull request with reviewer
gh pr create \
  --title "[T{ISSUE_NUMBER_PADDED}] {ISSUE_TITLE}" \
  --body "{PR_BODY_TEMPLATE}" \
  --reviewer {REVIEWER}
```

### Commit Types
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Adding tests
- `docs`: Documentation changes
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### PR Body Template
```markdown
## Summary
Completes Issue #{ISSUE_NUMBER} (Task T{ISSUE_NUMBER_PADDED}) - {BRIEF_SUMMARY}

## Changes
{LIST_OF_CHANGES}

## Testing
```bash
# Backend validation
cd backend
python manage.py check          # No issues
pytest --cov                    # Coverage: X%
ruff check .                    # No linting errors

# Frontend validation (if applicable)
cd frontend
npm run build                   # Build successful
npm test                        # All tests pass
npm run lint                    # No linting errors
```

## Constitution Compliance
- Spec-Driven Development: Implementation follows spec
- API-First Architecture: Backend exposes RESTful APIs
- Multi-Tenancy Discipline: All queries filter by tenant_id
- Independent User Stories: Feature is independently deployable
- Issue-Driven Development: Follows sequential workflow

## Database Changes
{LIST_MIGRATIONS_IF_ANY}

## API Changes
{LIST_NEW_ENDPOINTS_IF_ANY}

Closes #{ISSUE_NUMBER}
```

---

## Step 5: Cleanup & Next Cycle Preparation

### Post-PR Actions
```bash
# Return to main branch
git checkout main

# Update local main with remote changes
git pull origin main

# Prune deleted remote branches
git fetch --prune
```

---

## Error Handling

### When Tests Fail
```
Test validation failed for issue #{ISSUE_NUMBER}

Failed checks:
{LIST_FAILED_CHECKS}

Actions:
1. Do NOT create PR - implementation is incomplete
2. Fix failing tests
3. Re-run validation
4. Only proceed when all checks pass

Would you like me to:
A) Fix the failing tests automatically
B) Show detailed error logs
C) Abort and return to main branch
```

---

## Customization Guide

### Modifying Agent Prompts
1. Edit the relevant section above
2. Test with a single issue first
3. Iterate on prompt until desired output
4. Document changes in git commit

### Disabling Auto-Features
Set to `false` in Workflow Settings:
- `auto_push: false` - Require manual git push
- `auto_pr: false` - Require manual PR creation
- `auto_merge: false` - Require manual PR merge
- `auto_cleanup: false` - Require manual branch cleanup

---

## Version History

### v1.1.0
- **BREAKING**: Refactored master-orchestrator to be delegation-only (no code implementation)
- Added strict agent delegation requirements with FORBIDDEN ACTIONS
- Clarified backend-architect handles both API endpoints AND Python/Django infrastructure
- Added comprehensive agent responsibility documentation
- Enforced mandatory Task tool usage for all implementations
- Added auto-formatting step to integration phase

### v1.0.0
- Initial automated workflow configuration
- 5-step process: Discovery -> Planning -> Implementation -> Validation -> PR
- Support for all specialized agents
- Configurable auto-push/PR/merge options
