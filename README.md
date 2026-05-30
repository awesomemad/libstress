# libstress

[![CI](https://github.com/omnis/libstress/actions/workflows/ci.yml/badge.svg)](https://github.com/omnis/libstress/actions/workflows/ci.yml)
[![License: GPL 3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)

**libstress** is a safe-but-heavy stress testing framework for Rust developers and CI pipelines. It saturates CPU, RAM, disk I/O, the Rust compiler, and optional extras (loopback networking, thread scheduler) without privileged or destructive operations.

## Features

- **Modular workspace** — enable only what you need via Cargo features
- **Profiles** — `libstress.toml` presets (`ci-quick`, `compiler-torture`, `full`, …)
- **Preflight** — `libstress doctor` checks toolchain, RAM headroom, temp dir
- **Reports** — JSON or text summaries with optional live CPU/RAM sampling
- **Automation-ready** — GitHub Actions workflow, bench mode, dry-run planning
- **Safety caps** — memory limited to 70% of system RAM by default

## Quick start

```bash
# Build (includes net + sched extras by default)
cargo build --release

# Preflight
./target/release/libstress doctor

# List profiles
./target/release/libstress profiles

# CI-friendly 15s CPU + memory run
./target/release/libstress run --profile ci-quick

# Full runtime + compiler stress with JSON report
./target/release/libstress run --profile full --report json --live-stats

# Plan without executing
./target/release/libstress run --profile dev --dry-run
```

## Workspace crates

| Crate | Description |
|-------|-------------|
| `stress-core` | Config, profiles, metrics, doctor, safety |
| `cpu-stress` | Hashing, matrix multiply, branch chaos, Rayon |
| `mem-stress` | Allocation churn, fragmentation, bandwidth |
| `io-stress` | Parallel writes, hashing, artifact explosion |
| `net-stress` | Loopback TCP echo stress (**extras**) |
| `sched-stress` | Thread spawn/join churn, capped (**extras**) |
| `compile-stress` | `stressgen` + repeated `cargo` builds |
| `stress-macros` | Proc-macros for generated compile stress |
| `libstress` | CLI binary (`runner`) |

## CLI reference

| Command | Purpose |
|---------|---------|
| `run` | Execute workloads (flags or `--profile`) |
| `cpu` / `mem` / `io` / `compile` | Single module |
| `net` / `sched` | Extra modules (default build) |
| `extras net` | Alias for network stress |
| `doctor` | Environment preflight |
| `profiles` | List built-in profile names |
| `bench` | Repeat a profile N times |
| `info` | Host CPU/RAM snapshot |

### Global flags

| Flag | Description |
|------|-------------|
| `--config PATH` | Load `libstress.toml` |
| `--profile NAME` | Apply named profile |
| `--report json\|text` | Emit summary after run |
| `--live-stats` | Sample CPU/RAM during run |
| `--dry-run` | Print plan only |

### Common flags

| Flag | Description |
|------|-------------|
| `--intensity 1-10` | Scales workload size |
| `--duration 30s` | Humantime duration |
| `--threads N` | Workers (`0` = auto) |

## Profiles (`libstress.toml`)

Copy [libstress.toml](libstress.toml) to your project root or `~/.config/libstress/libstress.toml`.

| Profile | Typical use |
|---------|-------------|
| `ci-quick` | 15s CPU + memory for PR CI |
| `ci-full` | 2 min multi-module + `cargo check` stress |
| `dev` | 60s CPU + memory + I/O local dev |
| `compiler-torture` | 30 min incremental compile loop |
| `full` | 5 min runtime + compile |
| `overnight` | 8h sustained (use with care) |
| `net-loopback` | Loopback TCP saturation |

## Cargo features (`libstress` binary)

```bash
# Minimal (no net/sched)
cargo build -p libstress --no-default-features

# GPU adapter probe
cargo build -p libstress --features gpu

# Everything
cargo build -p libstress --features full
```

## Compile stress & `stressgen`

```bash
cargo run -p compile-stress --bin stressgen -- --modules 200 --intensity 7
cd target/stressgen-out && cargo build --release
```

`libstress compile` automates generation under `%TEMP%/libstress-compile` and loops `cargo build` / `check` / incremental modes.

## Safety

- Memory: **≤ 70%** of total RAM unless `--memory-cap-override`
- Network: **127.0.0.1 only** (loopback)
- Threads: sched stress capped at **256** workers
- Disk: uses configurable temp directories only

See [docs/SAFETY.md](docs/SAFETY.md).

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Configuration & profiles](docs/CONFIGURATION.md)
- [CI integration](docs/CI.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

Licensed under **MIT OR Apache-2.0** at your option.
