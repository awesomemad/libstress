# Safety model

libstress is designed to be **heavy but not destructive**.

## Memory

- Default cap: **70% of total system RAM** for allocation targets
- Override: `--memory-cap-override` (user acknowledges risk)
- Preflight: doctor warns if available RAM &lt; 512 MiB headroom

## Disk

- I/O and compile stress write only under:
  - `%TEMP%/<io_dir>` (default `libstress-io`)
  - `%TEMP%/<compile_dir>` (default `libstress-compile`)
- No raw block device access

## Network

- `net-stress` binds and connects to **127.0.0.1** only
- Default port `19456` (configurable via profile `net_port`)

## Threads

- `sched-stress` refuses more than **256** concurrent spawns per cycle

## What libstress does NOT do

- Fork bombs or unbounded process creation
- Disk fill outside configured directories
- Kernel/module manipulation or privileged syscalls
- WAN/network egress stress

## Recommendations

- Run on dedicated CI runners or dev machines, not production servers
- Use `ci-quick` in PR pipelines; reserve `overnight` for manual soak
- Run `libstress doctor` before first heavy session
