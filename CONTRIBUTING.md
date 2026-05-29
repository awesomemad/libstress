# Contributing

Thanks for improving libstress!

## Development setup

```bash
git clone https://github.com/omnis/libstress.git
cd libstress
cargo build --workspace
```

## Checks before PR

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p libstress
cargo run -p libstress -- doctor
```

## Adding a workload module

1. Create `your-stress/` crate with `pub fn run(...) -> anyhow::Result<()>`
2. Add to workspace `Cargo.toml` members
3. Extend `WorkloadKind` and profile parsing in `stress-core`
4. Wire `runner/src/orchestrator.rs` `run_one`
5. Document in `libstress.toml` and `docs/CONFIGURATION.md`

## Code style

- Match existing module layout and `tracing` for logs
- Keep safety caps conservative; document overrides
- Prefer focused diffs over large refactors

## License

By contributing, you agree that your contributions are licensed under GNU Public License 3.0, same as the project.
