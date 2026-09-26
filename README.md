# Skills

Open-source tools and workflows for coding agents.

| Project | Description | Status |
| --- | --- | --- |
| [AutoDev](autodev/) | Rust CLI for supervised GitHub issue-to-PR development in isolated worktrees | Experimental alpha |

## AutoDev

```sh
git clone https://github.com/ambamichal/skills.git
cd skills/autodev
cargo install --locked --path .
```

Start with the [AutoDev README](autodev/README.md) for configuration, commands, testing, and limitations. The Rust CLI is the current implementation. Historical prompts and PowerShell assets are retained inside `autodev/` as reference material.

## Contributing and license

See [CONTRIBUTING.md](CONTRIBUTING.md). MIT licensed; third-party-derived files retain their [upstream notices](THIRD-PARTY-NOTICES.md).
