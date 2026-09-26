# AutoDev

Experimental, spec-driven development workflows for Claude Code: take a GitHub issue through planning, implementation, validation, and a pull request.

**Status: source preview (v0.1.0-alpha.1).** This is a collection of prompts, agents, templates, and legacy PowerShell scripts, not a production-ready autonomous service. The continuous runner has known correctness and safety bugs. Do not run it unattended, including with `-DryRun`. See [known limitations](docs/KNOWN-LIMITATIONS.md).

## Included

- `/autodev`: issue-to-PR workflow instructions for Claude Code.
- Ten specialized agent definitions for planning, implementation, and validation.
- Spec Kit-derived commands, templates, and PowerShell utilities.
- `Start-AutoDevLoop.ps1`: historical continuous runner, retained for development.
- `Invoke-AutoDev.ps1`: historical helper, currently affected by known bugs.

## Explore the workflow

Requires Git, authenticated GitHub CLI, and Claude Code. PowerShell scripts target Windows; cross-platform execution has not been verified.

1. Clone this repository: `git clone https://github.com/ambamichal/autodev.git`.
2. Review `.claude/commands/autodev.md` and `.claude/config/autodev-workflow.md` before using them.
3. In a disposable project, back up existing configuration, then copy the `.claude/` and `.specify/` directories without overwriting project-specific rules.
4. Copy `.specify/memory/constitution-template.md` to `.specify/memory/constitution.md` and customize it.
5. Adapt the workflow to the project's base branch, toolchain, test commands, specs, and reviewers. The bundled configuration assumes a Django/React-style project and the legacy scripts assume `main`.
6. Set `auto_push`, `auto_pr`, and `auto_cleanup` to `false` for initial supervised evaluation. Keep `auto_merge: false`.
7. In Claude Code, request `/autodev --issue N`. Review the plan and each proposed action. Inspect the diff and actual test results before publishing changes.

Slash-command options are instructions interpreted by the coding agent, not a validated CLI interface. Resume, dry-run, nested agent delegation, and permission behavior are not release guarantees. Do not supply credentials in issue text or commit agent logs.

## Development direction

The next milestone is a verified single-issue cycle with configurable base branch, isolated working directory, mandatory validation, and a reviewed PR. Continuous operation comes after regression tests for failure handling, cancellation, and merge detection.

Read [known limitations](docs/KNOWN-LIMITATIONS.md) and [contributing](CONTRIBUTING.md). Older documents are retained as historical design references; this README describes the release status.

## License and attribution

MIT; see [LICENSE](LICENSE). Bundled Spec Kit-derived materials retain the upstream MIT notice in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Claude Code is a separate product and is not included. This is an independent community project.
