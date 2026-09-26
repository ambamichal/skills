<div align="center">

```text
    _   _   _ _____ ___  ____  _______     __
   / \ | | | |_   _/ _ \|  _ \| ____\ \   / /
  / _ \| | | | | || | | | | | |  _|  \ \ / /
 / ___ \ |_| | | || |_| | |_| | |___  \ V /
/_/   \_\___/  |_| \___/|____/|_____|  \_/

        ISSUE IN. CHECKED COMMIT OUT.
```

**A small Rust CLI for supervised AI development.**

[![Rust](https://img.shields.io/badge/built_with-Rust-dea584?style=flat-square)](Cargo.toml)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](../LICENSE)
[![Stage](https://img.shields.io/badge/stage-experimental_alpha-orange?style=flat-square)](#current-status)

[Quickstart](#quickstart) · [Commands](#commands) · [How it works](#how-it-works) · [Recovery](#when-a-run-fails) · [Testing](#testing)

</div>

---

Give AutoDev one GitHub issue. It prepares a separate worktree, runs your coding agent, checks the resulting changes, and creates a local commit. Add `--publish` to push that commit and open a **draft** PR.

**You choose the issue. Your checks gate the change. You decide when to merge.**

## Current status

`0.2.0-alpha.1` is a supervised, single-cycle CLI. Local tests cover real Git worktrees, commits, local bare-remote pushes, and failure preservation. The agent and GitHub API are controlled fixtures in those tests; this is not a claim of live model-to-GitHub acceptance.

No continuous loop, automatic merge, deployment, or automatic resume. Legacy PowerShell scripts remain reference material, not the supported runner.

## Quickstart

**Prerequisites:** stable Rust, Git, authenticated GitHub CLI (`gh`), and an authenticated coding agent such as Claude Code. Check commands must already be available in the target project's environment.

### 1. Build once

```sh
git clone https://github.com/ambamichal/skills.git
cd skills/autodev
cargo install --locked --path .
```

While [PR #1](https://github.com/ambamichal/skills/pull/1) is still open, use `git clone --branch feat/rust-cli https://github.com/ambamichal/skills.git` to try this implementation.

### 2. Tell AutoDev what a passing change means

Create **`autodev.json` at the root of your target repository**. Replace the GitHub repository, base branch, and checks with your project's values. Commit the file; the original checkout must be clean.

```json
{
  "base_branch": "development",
  "github_repo": "your-account/your-project",
  "agent": ["claude", "-p", "--permission-mode", "acceptEdits"],
  "checks": [
    ["cargo", "fmt", "--check"],
    ["cargo", "test", "--locked"]
  ],
  "timeout_seconds": 3600
}
```

`acceptEdits` permits Claude's file edits; it does not grant unrestricted shell permissions. Review your agent's permission policy. AutoDev itself adds no bypass flag. Other agents work if they accept the task prompt as the final argument.

Commands are **argument arrays**, not shell strings. Arguments containing spaces stay intact. For shell syntax or Windows `.cmd` tools, invoke your shell explicitly, for example `["cmd.exe", "/d", "/c", "npm test"]`. All commands run at the worktree root. Checks must be read-only with respect to source files: use `--check`, not autoformatting commands.

### 3. Inspect, then run

```sh
autodev --repo /path/to/project doctor
autodev --repo /path/to/project run --issue 123 --dry-run
autodev --repo /path/to/project run --issue 123
autodev --repo /path/to/project status
```

Want a draft PR as part of this run?

```sh
autodev --repo /path/to/project run --issue 123 --publish
```

No `--publish`, no orchestrator push. Inspect and publish the retained branch manually if you chose a local-only run; rerunning the same issue intentionally refuses to overwrite it.

## Commands

| Command | What it does |
| --- | --- |
| `doctor` | Validates configuration; checks Git, `gh` authentication, and the agent executable's version. Does not test model authentication or execute project checks. |
| `run --issue N --dry-run` | Reads configuration and prints a plan. No subprocesses, writes, or network calls. Supply the target repository root. |
| `run --issue N` | Implements one open issue, validates unchanged content, and commits locally. |
| `run --issue N --publish` | Also pushes the checked commit and opens a draft PR. Never merges. |
| `status` | Shows the last persisted cycle event, worktree path, and result. |

`origin` fetch and push URLs must match `github_repo`. Standard GitHub HTTPS and `git@github.com:` URLs are supported; GitHub Enterprise URLs and custom SSH aliases are not yet supported.

## How it works

```text
 OPEN ISSUE
     |
     v
 isolated worktree -----> coding agent
                              |
                              v
                     snapshot source tree
                              |
                              v
                       required checks
                              |
                     same source tree?
                              |
                              v
                         local commit
                              |
                   hooks kept checked tree?
                              |
                 +------------+------------+
                 |                         |
              default                  --publish
                 |                         |
                 v                         v
          inspect locally             draft GitHub PR
```

A passing exit code is only part of the contract. AutoDev compares Git trees before and after validation and checks the resulting commit. If a test rewrites source or a commit hook changes checked content, the cycle fails and **does not publish**. The changed files or local commit remain available for inspection.

## When a run fails

AutoDev does not delete failed work or force-push over it.

| Situation | Result / next step |
| --- | --- |
| Agent or check exits nonzero | Cycle fails. Inspect logs and the retained worktree. |
| Check rewrites source | Publication stops. Inspect changes and rerun checks manually. |
| Commit hook changes content | Local commit is retained, but not published. Review and validate it manually. |
| Timeout or Ctrl+C | Agent/check process tree termination is attempted. No subsequent Git/GitHub command starts after cancellation is observed. |
| Push or PR creation fails | Local work remains. A branch may already exist remotely; inspect before retrying manually. |
| Same issue is run again | Existing work is preserved; AutoDev refuses to overwrite it. |
| Stale lock after a forced exit | Verify no cycle is active before removing the lock. Never blindly delete worktrees. |

State lives under the repository's **common Git directory**:

```text
<git-common-dir>/autodev/
├── events.jsonl
├── run.lock
├── logs/issue-123/
│   ├── agent.log
│   └── check-1.log
└── worktrees/issue-123/
```

Use `git rev-parse --path-format=absolute --git-common-dir` to locate it and `git worktree list` to inspect retained checkouts. Logs may contain sensitive agent output; review before sharing.

## Trust boundary

A worktree isolates Git changes; **it is not an operating-system sandbox**. Agents and checks inherit your user permissions and may access files and the network. `--publish` controls AutoDev's own publication steps, not what a privileged agent can do independently. Review executable configuration and use container/OS isolation for untrusted tasks.

Agent and check commands have timeouts. Git/GitHub calls are synchronous: an in-flight operation can delay cancellation. Process-tree cleanup is best effort; detached descendants can survive. A journal phase is progress, not proof a crashed operation completed.

## Testing

From `skills/autodev`:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
```

The suite includes unit checks for configuration, cancellation, state journaling, and locking; subprocess tests for failure paths; and a complete local cycle with **real Git, worktrees, commits, and a bare remote**. Only the GitHub API, remote identity response, and coding agent are simulated in that local cycle. CI runs on Windows, Linux, and macOS.

The live model/GitHub acceptance run remains a separate release gate. Green unit tests do not establish model quality or production readiness.

## Roadmap

- [ ] Complete a disposable-repository acceptance run with a real coding agent and GitHub.
- [ ] Bound Git/GitHub network operations and improve cancellation.
- [ ] Resume a recorded cycle without duplicating work.
- [ ] Follow a specific PR's merge status before starting another issue.

Parallel workers and automatic merges wait until the single-cycle contract is proven.

## Part of Skills

AutoDev lives in [`ambamichal/skills`](https://github.com/ambamichal/skills). Historical `.claude/` prompts and `.specify/` templates are optional references. Rust reads `autodev.json`; it does not interpret the old Markdown configuration or automatically install bundled agents. See [legacy limitations](docs/KNOWN-LIMITATIONS.md).

[MIT license](../LICENSE) · [Third-party notices](../THIRD-PARTY-NOTICES.md) · [Contributing](../CONTRIBUTING.md)
