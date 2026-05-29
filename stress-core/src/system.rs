use sysinfo::System;

/// Point-in-time host metrics used for safety checks and reporting.
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub total_memory: u64,
    pub available_memory: u64,
    pub cpu_count: usize,
}

impl SystemSnapshot {
    pub fn capture() -> Self {
        let mut sys = System::new();
        sys.refresh_memory();
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self {
            total_memory: sys.total_memory(),
            available_memory: sys.available_memory(),
            cpu_count,
        }
    }
}
