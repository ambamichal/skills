> Execution contract: This is a backend-neutral workflow instruction. The Rust engine owns Git commits, push and PR creation. Do not execute publication or branch-changing examples from this document inside an agent stage. Use repository-specific checks and policies; examples are not proof of validation. Delegate using the configured backend when supported; otherwise report the missing capability. Rust command syntax in workflow/README.md takes precedence over historical invocation examples.

# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

**Note**: This template is filled in by the `autodev project prompt plan` command. See `workflow/commands/plan.md` for the execution workflow.

## Summary

[Extract from feature spec: primary requirement + technical approach from research]

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: [e.g., Python 3.11, Swift 5.9, Rust 1.75 or NEEDS CLARIFICATION]
**Primary Dependencies**: [e.g., FastAPI, UIKit, LLVM or NEEDS CLARIFICATION]
**Storage**: [if applicable, e.g., PostgreSQL, CoreData, files or N/A]
**Testing**: [e.g., pytest, XCTest, cargo test or NEEDS CLARIFICATION]
**Target Platform**: [e.g., Linux server, iOS 15+, WASM or NEEDS CLARIFICATION]
**Project Type**: [single/web/mobile - determines source structure]
**Performance Goals**: [domain-specific, e.g., 1000 req/s, 10k lines/sec, 60 fps or NEEDS CLARIFICATION]
**Constraints**: [domain-specific, e.g., <200ms p95, <100MB memory, offline-capable or NEEDS CLARIFICATION]
**Scale/Scope**: [domain-specific, e.g., 10k users, 1M LOC, 50 screens or NEEDS CLARIFICATION]

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

[Gates determined based on constitution file]

## Development Workflow (Constitution v1.3.0)

**MANDATORY**: Every task from this plan MUST follow the Branch → Test → PR → Merge workflow defined in Constitution Principle VI.

### For Each Task Implementation:

1. **Branch Creation**: Create dedicated feature branch before any development
   - Format: `feature/issue-{N}-brief-description`
   - Each GitHub Issue = 1 Task = 1 Branch

2. **Development**: Implement task following spec.md and plan.md
   - Commit with issue reference: `type: description #{N}`
   - Follow constitution principles (multi-tenancy, API-first, etc.)

3. **Testing (100% Required Before PR)**:
   - Backend: `python manage.py check`, `pytest`, `ruff check .`
   - Frontend: `npm run build`, `npm test`, `npm run lint`
   - Manual testing: Verify feature works as specified
   - **NO PR if requirements not 100% met**

4. **Pull Request (Only When Complete)**:
   - Title: `[TN] Brief title`
   - Body must include: Summary, Changes, Testing Evidence, Constitution Compliance
   - Must reference: `Closes #{issue-number}`

5. **Merge & Close**:
   - **Default**: User manually merges and closes issue
   - **Agent Autonomy**: Only when user explicitly instructs "merge PR and close issue"

**Reference**: See `IMPLEMENTATION-PLAN.md` Appendix B for detailed workflow commands.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (autodev project prompt plan command output)
├── research.md          # Phase 0 output (autodev project prompt plan command)
├── data-model.md        # Phase 1 output (autodev project prompt plan command)
├── quickstart.md        # Phase 1 output (autodev project prompt plan command)
├── contracts/           # Phase 1 output (autodev project prompt plan command)
└── tasks.md             # Phase 2 output (autodev project prompt tasks command - NOT created by autodev project prompt plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
