# CI integration

## GitHub Actions (this repo)

The workflow [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs:

- `cargo fmt --check`
- `cargo clippy` (warnings denied)
- `cargo test --workspace`
- `cargo build --release -p libstress`
- `libstress doctor`
- `libstress run --profile ci-quick --dry-run`

## Using libstress in your project

```yaml
- name: Install libstress
  run: cargo install --path . --locked  # or download release binary

- name: Preflight
  run: libstress doctor

- name: Stress (quick)
  run: libstress run --profile ci-quick --report json
```

For compile stress in CI, ensure adequate disk space and cache `target/` if desired. Prefer `ci-full` with `compile_mode = "check"` for faster feedback.

## Bench mode

```bash
libstress bench --profile ci-quick --rounds 5
```

Runs the profile repeatedly and prints success count (useful for flake detection).

## Artifacts

Upload JSON reports:

```yaml
- run: libstress run --profile ci-quick --report json > stress-report.json
- uses: actions/upload-artifact@v4
  with:
    name: libstress-report
    path: stress-report.json
```
