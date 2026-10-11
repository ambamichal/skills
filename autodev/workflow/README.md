# AutoDev workflow

Rust executes the workflow; these provider-neutral instructions define the work. Run `autodev init` in a target repository to install this directory without overwriting customization. Set the agent, checks, repository and base branch in `autodev.json`, then commit the configuration and workflow.

## Development cycle

1. `autodev next` selects the next open issue after the highest closed issue, falling back to the lowest open issue.
2. `autodev run` prepares a separate branch/worktree from the configured remote base.
3. Configured stages run in order: discovery, planning, implementation, quality review. Each stage can select its own backend command; otherwise it inherits the main backend. The stage prompt identifies a JSON result file. Write `{"status":"passed","blockers":[]}` only after completing the stage. Use `{"status":"blocked","blockers":["reason"]}` when blocked. Missing/invalid results stop the cycle.
4. Required project commands validate the resulting source tree. Rust commits and optionally pushes/creates a draft PR with reviewers and recorded check evidence.
5. An optional notification command receives the PR URL as its final argument. Configure your notification service independently; no service is assumed.
6. `autodev loop` waits for that exact PR to be manually merged, then selects the next issue. A closed-unmerged PR or failed stage stops the loop. Every stage starts a fresh backend process.

`autodev run --issue N --resume` resumes saved work. Completed agent stages are retained; checks run again before publication. Configuration must match the saved cycle. Failures never erase work. `--until prepare|implement|validate|commit|push|pr` exposes individual cycle boundaries. Push and PR are independently selectable. `workflow.auto_cleanup` or `autodev project cleanup --issue N` removes only a clean, published worktree after verifying remote preservation; the original checkout, branch, logs and state remain.

`autodev loop --max-cycles 5 --poll-interval 30` starts immediately. Use `--wait-for-merge PR_NUMBER` to wait before the first cycle. Zero maximum means unlimited. Ctrl+C stops processing; status and logs are kept under the common Git directory. No automatic merge or permission bypass is provided.

## Specification lifecycle

Commands operate on an explicit feature directory. There are no implicit provider-specific context files. `$ARGUMENTS` in the detailed instruction documents means the `--input` text; `FEATURE_DIR` means the supplied feature path.

```sh
autodev project feature "account login"
autodev project prompt specify specs/001-account-login --input "Users sign in securely"
autodev project prompt clarify specs/001-account-login
autodev project plan specs/001-account-login
autodev project prompt plan specs/001-account-login
autodev project context specs/001-account-login
autodev project prompt tasks specs/001-account-login
autodev project prompt analyze specs/001-account-login
autodev project prompt checklist specs/001-account-login --input security
autodev project prerequisites specs/001-account-login --require-tasks
autodev project issues specs/001-account-login/tasks.md --dry-run
autodev project issues specs/001-account-login/tasks.md
autodev project labels specs/001-account-login/tasks.md
autodev project issue-labels
```

The other instruction operations are `constitution`, `implement`, and `taskstoissues`. Constitution writes `workflow/constitution.md`. Implementation follows task dependencies and checklist gates. Task-to-issues instructions prepare/review task metadata; the `project issues` command performs the actual publication.

Templates preserve the artifact chain: constitution, specification and acceptance scenarios, clarifications, plan, research, data model, contracts, quickstart, dependency-ordered tasks and requirement checklists. Four concise stage instructions cover discovery, planning, implementation and quality. No predefined agent roles or delegation capability are required.

`project context` maintains a marked section in `AGENTS.md` from the plan and preserves text outside that section. `project renumber tasks.md --offset 2` updates task references with a backup. `project issue-titles --offset 2` updates task IDs in remote issue titles. Offsets are explicit migrations, not idempotent synchronization: do not apply one twice. Use `--dry-run` first. Issue creation skips task IDs already present in paginated issues; duplicate IDs stop processing. Task metadata includes phase, priority, parallel eligibility, story, goal and acceptance criteria.

## Execution rules

Rust owns checkout preparation, commit, push and PR creation. Agents must not perform those operations even where historical role examples describe a complete developer workflow. Checks, security/migration reviews and spec compliance must be backed by actual results; prompt text alone is not evidence. Configure backend/frontend coverage, formatting, lint, types, security and migration commands for the target project. For commands in subdirectories use the tool's working-directory option or an explicit shell wrapper.

Dry-run never launches subprocesses, accesses the network or writes files. It cannot determine live issue/PR state. No command skips required validation. The old advertised skip-tests/auto-merge flags contradicted its mandatory review policy and had no executable implementation; they are not part of this workflow.
