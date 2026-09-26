# Workflow parity map

Reference: the project's workflow source at commit `4dd7c19`. Rust replaces execution; the stages and specification process remain part of the product. Historical bugs and unimplemented flags are distinguished from capabilities below.

| Capability | Current implementation | Verification |
| --- | --- | --- |
| Next sequential issue / explicit issue | `next`, `run [--issue N]`; paginated issues, highest-closed rule with lowest-open fallback | Selection unit test; loop integration |
| Dependency/blocker discovery and progress reporting | `stages/discovery.md`, discovery stage result gate | Zero-exit blocked-result regression; semantic work requires real-agent acceptance |
| Planning and implementation | `stages/planning.md` and `stages/implementation.md`; execution strategy belongs to the backend | Stage order/resume regression |
| Specification lifecycle | Nine `project prompt` operations: specify, clarify, plan, tasks, analyze, checklist, constitution, implement, taskstoissues | Installed assets and project prompt gate integration |
| Artifact templates | Constitution, spec, plan, tasks, checklist and agent context templates | Bundled assets; scaffolding test |
| Feature branch/spec scaffold | `project feature NAME [--number N]`; numbering inspects specs and local/remote branches | Real Git regression |
| Plan setup / artifact discovery | `project plan`, `project prerequisites [--require-tasks|--paths-only]` | Existing plan preservation and artifact checks |
| Plan-derived agent context | `project context`; managed section in `AGENTS.md`, user content retained | Context preservation integration |
| Task issue generation | `project issues TASKS`; phase/priority/story/parallel/goal/acceptance metadata; existing IDs skipped | Parser unit and mocked GitHub integration |
| Label creation and assignment | `project labels TASKS`, `project issue-labels`; metadata plus backend/frontend/API/UI/database/DevOps inference | Inference and mocked GitHub regression |
| Task/issue numbering migration | `project renumber --offset`, `project issue-titles --offset`; task backup | Boundary/offset and backup regression |
| Project-specific quality checks | Configured commands plus quality stage review; backend/frontend/coverage/lint/types/security/migration checks supplied by the project | Check failure/tree stability tests; no fabricated pass claims |
| Branch creation, conventional commit | `run --until prepare|commit`; dedicated worktree from remote base | Real Git cycle tests |
| Independent push / PR control | `--until push|pr`, `--publish`, `workflow.auto_push/auto_pr` | Publication policy and cycle tests |
| Reviewer assignment and PR summary | `workflow.reviewers`; task title, changed files, executed-check evidence | Fixture argument assertions |
| Cleanup / next-cycle readiness | Original checkout remains intact; optional removal of clean published worktree; next cycle fetches the latest base | Real Git preservation and cleanup regression |
| Repeated fresh agent cycles | `loop`, poll interval, maximum cycles, immediate start or initial PR wait | Open/merged/closed PR integration |
| Wait for manual merge | Monitor the exact PR and configured base; never infer merge from arbitrary branch movement | Exact PR lookup and closed-without-merge regression |
| Logs / progress / failure preservation | Per-stage/check logs, journal, configuration/state checkpoints, `status` | Journal, timeout, cancellation and resume tests |
| Notifications | GitHub's normal PR notifications; optional `workflow.notify` command receives PR URL | Configured hook fixture; actual delivery external |
| Bounded retries | Base fetch and non-force push retry with backoff; exhausted failures are preserved for explicit resume | Existing failure/recovery checks |
| Dry-run | Local-only plan; no subprocess, network or write | Dry-run integration |

## Corrected historical behavior

The old loop watched any base SHA change, could miss merges during agent execution, accepted some nonzero agent exits, and left workers running after timeout. The Rust engine tracks a specific PR, stops on failures and attempts process-tree termination. The original checkout is preserved instead of being forcibly switched back to a hardcoded branch. A visible Windows terminal is replaced by portable subprocesses and persistent logs.

The original helper had pull/push retry loops, while the prompt workflow advertised `resume`, `skip-tests` and approval-dependent `auto_merge` without an executable state machine. Rust now provides checkpointed resume; required checks remain mandatory and merge remains a human action. GitHub Mobile notification delivery was external to the old workflow and remains external.

Instruction parity is not proof that a model follows the instructions. A full acceptance run must still demonstrate discovery, implementation, quality review, issue-to-PR execution, manual merge and the next cycle with a real backend. Automated tests use controlled backend/GitHub fixtures and real local Git where stated.
