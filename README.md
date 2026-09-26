# Skills

Open-source tools and workflows for coding agents.

## Featured: AutoDev

```text
    _   _   _ _____ ___  ____  _______     __
   / \ | | | |_   _/ _ \|  _ \| ____\ \   / /
  / _ \| | | | | || | | | | | |  _|  \ \ / /
 / ___ \ |_| | | || |_| | |_| | |___  \ V /
/_/   \_\___/  |_| \___/|____/|_____|  \_/
```

**One issue. An isolated worktree. Required checks. A draft PR when you ask.**

[Explore AutoDev →](autodev/README.md)

| Project | Implementation | Status |
| --- | --- | --- |
| [AutoDev](autodev/) | Rust CLI for supervised issue-to-PR development | Experimental alpha |

```sh
git clone https://github.com/ambamichal/skills.git
cd skills/autodev
cargo install --locked --path .
```

While PR #1 is open, clone with `--branch feat/rust-cli` to try the Rust implementation.

Start with the [AutoDev quickstart](autodev/README.md#quickstart). Configuration, recovery, testing, and limitations are documented alongside the tool. Historical workflow assets stay inside the AutoDev directory.

[Contribute](CONTRIBUTING.md) · [MIT license](LICENSE) · [Third-party notices](THIRD-PARTY-NOTICES.md)
