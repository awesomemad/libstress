# Architecture

## Overview

libstress separates **orchestration** (CLI, profiles, metrics) from **workload crates** that implement specific stress patterns.

```
┌─────────────────────────────────────────────────────────┐
│  libstress (runner)                                      │
│  CLI · profiles · doctor · metrics · dry-run             │
└────────────┬────────────────────────────────────────────┘
             │
    ┌────────┼────────┬──────────┬──────────┬─────────────┐
    ▼        ▼        ▼          ▼          ▼             ▼
 cpu-stress mem    io-stress  net-stress sched-stress compile-stress
                              (extras)   (extras)      + stressgen
```

## Runtime vs compile-time stress

| Path | Mechanism |
|------|-----------|
| **Runtime** | Workloads execute inside the `libstress` process (or spawn bounded threads). CPU, memory, I/O, net, and sched modules use `stress_core::run_timed`. |
| **Compile-time** | `compile-stress` generates a standalone Cargo tree and repeatedly invokes `cargo build` / `cargo check`, stressing rustc, linker, and disk under `target/`. |

Default `libstress run` executes **runtime** modules only. Add `--compile` or use profile `full` / `compiler-torture` for compiler stress.

## Configuration merge order

1. Embedded `libstress.toml` (shipped with `stress-core`)
2. Discovered file (`./libstress.toml` or user config dir)
3. `--config` path (merged on top)
4. `--profile` selection
5. CLI flags on the active subcommand

## Metrics pipeline

When `--live-stats` or `--report` is set, `MetricsCollector` samples `sysinfo` on a background thread. After workloads finish, results are merged into `StressReport` (per-workload timing + peak CPU/RAM samples).

## Extension points

- Add a workspace crate implementing `run(intensity, duration, …) -> Result<()>`
- Register `WorkloadKind` in `stress-core`
- Wire the orchestrator `run_one` match arm
- Document profile workload string in `libstress.toml`
