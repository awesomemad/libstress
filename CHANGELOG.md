# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-05-29

### Added

- `libstress.toml` profiles: `ci-quick`, `ci-full`, `dev`, `full`, `compiler-torture`, `overnight`, `net-loopback`
- Commands: `doctor`, `profiles`, `bench`, `net`, `sched`, `extras`
- Global flags: `--config`, `--profile`, `--report`, `--dry-run`, `--live-stats`
- Crates: `net-stress` (loopback TCP), `sched-stress` (thread churn)
- Metrics collector with JSON/text reports
- GitHub Actions CI workflow
- Documentation under `docs/`

### Changed

- CLI merges config file + profile + flags
- `libstress run` supports `--net` and `--sched` flags

## [0.1.0] - 2026-05-29

### Added

- Initial workspace: cpu, mem, io, compile stress, stressgen, runner CLI
- Safety RAM cap (70%), optional GPU feature
