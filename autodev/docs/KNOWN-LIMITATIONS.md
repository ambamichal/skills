> These limitations describe the historical PowerShell scripts and prompts. For the Rust CLI, see the [AutoDev README](../README.md). A live model-to-GitHub acceptance run is still pending.

# Known limitations

This source preview has not passed an end-to-end issue-to-PR acceptance test. It has no automated regression suite or CI checks. Do not treat historical documentation or generated PR templates as evidence that checks passed.

## Continuous runner: release blockers

- The base branch is hardcoded to `main`.
- `-DryRun` still permits checkout, pull, fetch, and log writes.
- Claude is launched with `--dangerously-skip-permissions`.
- A nonzero process exit code is interpreted as successful completion.
- Branch SHA changes are used instead of the current PR's merge status. Updating the baseline after a cycle can miss a merge.
- Timeout does not terminate the worker; durable resume and duplicate-run protection are absent.

## Helper script: release blockers

- `[CmdletBinding()]` and an explicit `Verbose` parameter collide.
- Splitting command strings on spaces breaks quoted arguments.
- Captured native-command output can interfere with boolean test results.
- PR descriptions contain prewritten success claims instead of recorded validation evidence.

## Workflow prompts

- Assumptions about project layout, branch names, tests, and reviewers require customization.
- Agent delegation and command options are host-dependent instructions, not enforced runtime guarantees.
- There is no reliable persisted resume state.
- The broad `git add .` examples require manual review to avoid including unrelated files.

## Next milestone

Implement and test one supervised issue-to-PR cycle before enabling unattended execution: correct base branch, isolated worktree, fail-closed validation, preservation of failed work, explicit PR tracking, cancellation, and bounded retries. Regression tests should cover each failure above.
