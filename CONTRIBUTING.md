# Contributing

Start with the root README and the README of the project you are changing. For AutoDev, read the README's trust boundary and roadmap, then run these checks from `autodev/`:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
cargo package --locked
```

Keep pull requests focused and explain the problem, changed behavior, and checks actually run. Add a small regression check for executable logic changes. Use disposable repositories for automation tests; do not test against production branches. Keep the runtime and repository layout independent of agent vendors; backend examples belong in the README. When changing the root license, keep `autodev/LICENSE` identical so the Cargo package includes its license.

Do not include credentials, private issues, local machine paths, or agent transcripts. Preserve third-party notices. Contributions are provided under this repository's MIT license.

The first priority is a reliable single-issue workflow. Parallel workers and automatic merging are deferred until that workflow is verified.
