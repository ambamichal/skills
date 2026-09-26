# AutoDev

A Rust CLI for supervised issue-to-PR development with a coding agent.

**0.2.0-alpha.1 — experimental.** One explicit issue, one isolated Git worktree, required checks, and an optional draft PR. No automatic merge or continuous loop. Failed work is preserved.

## Install

Install stable Rust, Git, GitHub CLI, and your coding agent. Authenticate GitHub CLI and the agent separately.

```sh
git clone https://github.com/ambamichal/skills.git
cd skills/autodev
cargo install --locked --path .
```

## Configure

Copy `autodev.example.json` to `autodev.json` in your target repository. Set `github_repo` to its GitHub owner/name, `base_branch` to its integration branch, and `checks` to actual project checks. Commit the configuration: the starting worktree must be clean.

```json
{
  "base_branch": "development",
  "github_repo": "your-account/your-project",
  "agent": ["claude", "-p"],
  "checks": [["cargo", "test", "--locked"]],
  "timeout_seconds": 3600
}
```

Commands are argument arrays, not shell strings. For shell syntax or Windows `.cmd` tools, invoke a trusted shell explicitly, e.g. `["cmd.exe", "/d", "/c", "npm test"]`. Commands execute at the worktree root. At least one check is mandatory. Configuration is trusted executable policy; review it before running.

The agent receives the task prompt as its final argument. No model or credentials are bundled. No permission-bypass flag is added. Configure your agent's permissions separately.

## Usage

```sh
autodev --repo /path/to/project doctor
autodev --repo /path/to/project run --issue 123 --dry-run
autodev --repo /path/to/project run --issue 123
autodev --repo /path/to/project status
```

- `doctor`: checks Git, GitHub authentication, the agent executable's `--version`, and configuration. It does not run checks or verify model authentication.
- `--dry-run`: reads configuration and prints the plan. No processes, network requests, or writes. Supply the root containing `autodev.json`.
- `run`: fetches the configured base, creates `autodev/issue-123` in a separate worktree, runs the agent and checks, then commits locally. Branch/HEAD changes by the agent or checks are rejected. The original checkout stays on its branch.
- `status`: displays the last recorded cycle event.

To authorize pushing and creating a **draft** PR at the start of a cycle:

```sh
autodev --repo /path/to/project run --issue 123 --publish
```

Publication happens only after local checks pass. AutoDev never merges, deploys, closes an issue, or deletes work. Review the diff and CI. Without `--publish`, publish the preserved branch manually after inspection; rerunning the same issue refuses to overwrite existing work.

## Failures and state

State is stored in the common Git directory under `autodev/`: `events.jsonl`, `logs/issue-N/`, and `worktrees/issue-N/`. Logs may contain sensitive output; inspect before sharing. A repository-wide lock prevents overlapping cycles.

Ctrl+C signals cancellation. Agent and check process trees are terminated on cancellation or timeout. Git/GitHub calls are synchronous and can delay cancellation until they return. Forced termination can leave a stale lock; verify its PID is no longer running before manually removing it. A recorded phase is progress, not proof a crashed operation finished.

There is no automatic resume. Inspect the journal, logs, and `git worktree list` after failure. Preserve needed work before manually removing worktrees or branches. Failed PR creation can leave a pushed branch: create the PR manually after review.

**A worktree is not a security sandbox.** Agents and checks have your user permissions, filesystem access, and network access. Process-tree termination is best effort; detached descendants may survive. Use OS/container isolation for untrusted tasks.

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Controlled subprocess tests cover dry-run, orchestration, failure, timeout, and work preservation. They do not exercise a paid model or live GitHub publication. CI targets Windows, Linux, and macOS.

Next: a disposable-repository live acceptance run, then persisted resume and PR-specific merge tracking. Parallel workers and automatic merging are deferred.

## Historical assets

The `.claude/` prompts and `.specify/` templates remain as reference material. Rust reads `autodev.json`; it does not interpret the old Markdown configuration or automatically load bundled agents. Legacy PowerShell runners are unsupported and have [known bugs](docs/KNOWN-LIMITATIONS.md).

## License

[MIT](../LICENSE). Spec Kit-derived files retain their [upstream notice](../THIRD-PARTY-NOTICES.md). This is an independent community project; Claude Code is a separate product.
