<!--
CONSTITUTION TEMPLATE
=====================
This is a template for creating your project's constitution.
Customize the principles below to match your project's needs.

Instructions:
1. Copy this file to .specify/memory/constitution.md in your project
2. Replace [PROJECT NAME] with your actual project name
3. Customize principles based on your project requirements
4. Remove principles that don't apply
5. Add project-specific principles as needed
-->

# [PROJECT NAME] Constitution

## Core Principles

### I. Spec-Driven Development

Every feature MUST begin with a complete specification before any implementation work starts.

**Rules:**
- All features require a `spec.md` file in `/specs/[###-feature-name]/` directory
- Specifications MUST define user scenarios with acceptance criteria in Given-When-Then format
- Specifications MUST include functional requirements (FR-###) and success criteria (SC-###)
- Implementation work MUST NOT begin until the specification is reviewed and approved
- Any unclear requirements MUST be marked with [NEEDS CLARIFICATION: ...] and resolved before proceeding

**Rationale:**
Spec-driven development ensures shared understanding between stakeholders and developers, reduces rework, and provides clear acceptance criteria.

### II. API-First Architecture

Every feature MUST expose its functionality through well-defined APIs with comprehensive contracts.

**Rules:**
- All backend functionality MUST be accessible via RESTful API endpoints
- API contracts MUST be defined in `/specs/[###-feature-name]/contracts/` before implementation
- Contract tests MUST be written and MUST fail before API implementation
- All APIs MUST follow OpenAPI 3.1 specification
- API responses MUST use consistent error format and HTTP status codes

**Rationale:**
API-first design enables multiple frontends to consume the same backend services consistently.

### III. [CUSTOMIZE: Add Your Critical Principle]

[Description of your project's most critical architectural or security principle]

**Rules:**
- [Rule 1]
- [Rule 2]
- [Rule 3]

**Rationale:**
[Why this principle is non-negotiable for your project]

### IV. Independent User Stories

Each user story MUST be independently implementable, testable, and deployable as a standalone increment of value.

**Rules:**
- User stories MUST be prioritized (P1, P2, P3...) in order of business value
- Each user story MUST include "Independent Test" section describing how it can be validated standalone
- Tasks MUST be organized by user story in `tasks.md` to enable independent delivery
- P1 (highest priority) user story MUST constitute a viable MVP that delivers measurable value
- User stories MAY integrate with each other but MUST remain independently functional
- Each user story completion MUST represent a potential release/demo checkpoint

**Rationale:**
Independent user stories enable incremental delivery, reduce risk, allow parallel development, and provide frequent validation points with stakeholders.

### V. MVP-First Mindset

**Priority: Speed to market over feature completeness. Deliver core value quickly, then iterate based on real user feedback.**

**Rules:**
- **MVP is always simplest version** that delivers core value to customers
- **Defer all non-essential features** until MVP is validated with users
- **Choose simple solutions over complex ones**
- **Remove technology unless proven necessary**
- **Specifications MUST limit scope**: Maximum 3 [NEEDS CLARIFICATION] markers; make informed guesses otherwise
- **Done is better than perfect**: Functional MVP > feature-complete product months later
- **Validate assumptions quickly**: Launch MVP, learn from users, iterate

**MVP Decision Framework:**
1. **Is this blocking core business value?** → MVP
2. **Does this enable monetization?** → MVP
3. **Is this nice-to-have efficiency?** → Phase 2
4. **Is this advanced feature?** → Phase 3

**Rationale:**
Speed and market validation are more critical than advanced features for early-stage products.

### VI. Issue-Driven Development Process

All development work MUST be tracked through GitHub issues and executed in strict sequential order with mandatory branch, testing, and pull request workflow.

**Branch & Development Rules:**
- Every code change MUST be associated with a GitHub issue
- Issues MUST be executed strictly in sequential order by issue number
- Each issue MUST have its own dedicated feature branch created before any development work begins
- Branch names MUST reference the issue number (format: `feature/issue-{number}-brief-description`)
- All commits MUST reference the associated issue number in the message
- Developers MUST NOT skip issues or work on multiple issues simultaneously

**Testing & Validation Rules:**
- BEFORE creating a pull request, the solution MUST be tested to ensure it meets 100% of the requirements
- All tests MUST pass
- Code MUST pass linting checks
- Manual testing MUST verify the feature works as specified
- If solution does NOT meet 100% of requirements, continue development - DO NOT create PR

**Pull Request Rules:**
- Pull requests MUST ONLY be created when the solution is 100% complete and tested
- PR title format: `[TN] Brief descriptive title`
- PR body MUST include: Summary, Testing evidence, Constitution compliance checklist, Reference to closing issue
- Pull requests MUST target the `main` branch

**Issue Closing Rules:**
- **Default**: User manually closes issues after PR merge and verification
- **Agent autonomy**: If user explicitly instructs agent to "merge PR and close issue", agent executes merge and close
- Agent MUST NOT close issues without explicit user instruction

**Issue-Task Numbering Convention:**
- Tasks in tasks.md MUST be numbered to match their corresponding GitHub Issue
- **Mapping formula**: Issue #N = Task TN (e.g., Issue #3 = Task T003, Issue #42 = Task T042)

**Rationale:**
Issue-driven development provides clear traceability, prevents work fragmentation, ensures systematic progress through the backlog, and maintains a single source of truth for what is being worked on.

## Security & Compliance Requirements

**Data Protection:**
- All personal data MUST be handled in compliance with applicable data protection laws
- Personal data retention policies MUST be documented and enforced
- Users MUST be able to request data export and deletion

**Authentication & Authorization:**
- All API endpoints MUST require authentication (except public endpoints explicitly documented)
- Role-based access control (RBAC) MUST enforce permissions
- JWT tokens MUST use rotating refresh tokens with maximum 24-hour access token lifetime
- Password requirements MUST enforce minimum complexity

**Infrastructure Security:**
- All communication MUST use TLS 1.3+
- Security headers MUST be enforced (CSP, HSTS, X-Frame-Options, etc.)
- Rate limiting MUST be implemented on all public endpoints
- All dependencies MUST be scanned for vulnerabilities in CI/CD pipeline
- Production secrets MUST use environment variables or secure vaults

## Quality & Testing Standards

**Testing Requirements:**
- Contract tests MUST be written for all new API endpoints
- Integration tests MUST be written for multi-step user journeys
- Tests MUST be written BEFORE implementation (TDD) when explicitly requested in spec
- All tests MUST pass before merging to main branch
- Test coverage targets: 80%+ for critical business logic

**Code Quality:**
- All code MUST pass linting and formatting checks
- Complex logic MUST include inline comments explaining business rules
- Database migrations MUST be reviewed for safety
- Breaking API changes MUST follow semantic versioning and deprecation policy

**Performance Standards:**
- API response time: p95 < 500ms for read operations, < 1000ms for write operations
- Large datasets MUST use pagination (max 100 items per page)

## Governance

### Amendment Process

This constitution may be amended when:
- New architectural patterns are adopted
- Technology stack changes materially
- New non-negotiable principles are identified
- Existing principles are found to be counterproductive and need refinement

**Amendment Procedure:**
1. Proposed amendment documented with rationale and impact analysis
2. Review by technical lead and product owner
3. Update constitution using `/speckit.constitution` command
4. Increment version according to semantic versioning
5. Update all dependent templates for consistency
6. Announce changes to all team members

### Versioning Policy

Constitution follows semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes (principle removed, incompatible governance change)
- **MINOR**: New principle added or existing principle materially expanded
- **PATCH**: Clarifications, typo fixes, wording improvements

### Compliance & Review

**All development work MUST:**
- Verify alignment with constitution principles during spec review
- Check constitution compliance gate in `plan.md` before implementation
- Justify any deviations in "Complexity Tracking" section of plan
- Use constitution as arbiter for architectural disagreements

**Constitution violations:**
- MUST be raised during code review
- MAY be approved with explicit justification and documentation
- MUST be tracked as technical debt if approved temporarily
- REQUIRE amendment if pattern becomes widespread

**Periodic review:**
- Constitution SHOULD be reviewed quarterly for relevance
- Metrics on principle violations SHOULD be tracked
- Amendments SHOULD be batched to avoid constant version churn

**Version**: 1.0.0 | **Ratified**: [DATE] | **Last Amended**: [DATE]
