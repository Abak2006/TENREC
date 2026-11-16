// TENREC - System Telemetry Subsystem
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod power;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sysinfo::System;

pub use cpu::{CpuCollector, CpuMetrics};
pub use disk::{collect_disk_metrics, DiskMetrics};
pub use gpu::{collect_gpu_metrics, GpuMetrics};
pub use memory::{collect_memory_metrics, MemoryMetrics};
pub use power::{collect_power_metrics, PowerMetrics};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IoMetrics {
    /// Sum of per-process I/O counters (file, device and network I/O), bytes/sec.
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub hostname: String,
    pub os_name: String,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub io: IoMetrics,
    pub power: PowerMetrics,
    pub gpu: GpuMetrics,
}

pub struct SystemManager {
    cpu_collector: CpuCollector,
    sys: System,
    gpu: GpuMetrics,
    hostname: String,
    os_name: String,
    disk: DiskMetrics,
    disk_sampled_at: Option<std::time::Instant>,
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemManager {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        Self {
            cpu_collector: CpuCollector::new(),
            sys,
            gpu: collect_gpu_metrics(),
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::long_os_version().unwrap_or_default(),
            disk: DiskMetrics { drives: Vec::new() },
            disk_sampled_at: None,
        }
    }

    pub fn snapshot(&mut self, io: IoMetrics) -> SystemSnapshot {
        let cpu = self.cpu_collector.collect(&mut self.sys);
        // Volume capacity changes slowly; query it every 30 s instead of every tick.
        let stale = self.disk_sampled_at.map(|t| t.elapsed().as_secs() >= 30).unwrap_or(true);
        if stale {
            self.disk = collect_disk_metrics();
            self.disk_sampled_at = Some(std::time::Instant::now());
        }

        SystemSnapshot {
            timestamp: Utc::now().to_rfc3339(),
            uptime_seconds: System::uptime(),
            hostname: self.hostname.clone(),
            os_name: self.os_name.clone(),
            cpu,
            memory: collect_memory_metrics(),
            disk: self.disk.clone(),
            io,
            power: collect_power_metrics(),
            gpu: self.gpu.clone(),
        }
    }
}
