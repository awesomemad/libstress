# Configuration

## File locations

libstress loads configuration from (first match wins for discovery, then merges):

1. Embedded defaults (built into `stress-core`)
2. `./libstress.toml` in the current working directory
3. `%USERPROFILE%\.config\libstress.toml` (Windows) or `~/.config/libstress/libstress.toml` (Unix)
4. Explicit `--config /path/to/libstress.toml`

## Profile schema

```toml
[profile.my-profile]
duration_secs = 120
intensity = 7
threads = 0                    # 0 = auto-detect CPUs
workloads = ["cpu", "mem", "io", "compile", "net", "sched"]
memory = "2GB"                 # optional
memory_cap_override = false
io_dir = "libstress-io"
compile_modules = 100
compile_mode = "incremental"   # build-release | check | incremental
compile_dir = "libstress-compile"
ecs = false                    # Bevy ECS in generated compile stress
net_port = 19456
```

## Workload names

| String | Module |
|--------|--------|
| `cpu` | cpu-stress |
| `mem`, `memory` | mem-stress |
| `io`, `disk` | io-stress |
| `compile`, `compiler` | compile-stress |
| `net`, `network` | net-stress |
| `sched`, `threads` | sched-stress |
| `gpu` | GPU probe (requires `gpu` feature) |

Empty `workloads` defaults to `cpu`, `mem`, `io`.

## Examples

```bash
# Use a profile
libstress run --profile ci-quick

# Override duration on top of profile
libstress run --profile dev --duration 10m

# Custom config file
libstress run --config ./ci/libstress.toml --profile ci-full
```
