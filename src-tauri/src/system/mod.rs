// TENREC - System Telemetry Subsystem
pub mod cpu;
pub mod memory;
pub mod disk;
pub mod power;
pub mod gpu;

use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::sync::Mutex;
use chrono::Utc;

pub use cpu::{CpuCollector, CpuMetrics};
pub use memory::{collect_memory_metrics, MemoryMetrics};
pub use disk::{collect_disk_metrics, DiskMetrics};
pub use power::{collect_power_metrics, PowerMetrics};
pub use gpu::{collect_gpu_metrics, GpuMetrics};

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
    pub power: PowerMetrics,
    pub gpu: GpuMetrics,
}

pub struct SystemManager {
    cpu_collector: Mutex<CpuCollector>,
    sysinfo_cache: Mutex<System>,
    cached_gpu: GpuMetrics,
}

impl SystemManager {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cached_gpu = collect_gpu_metrics();

        Self {
            cpu_collector: Mutex::new(CpuCollector::new()),
            sysinfo_cache: Mutex::new(sys),
            cached_gpu,
        }
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        let mut sys = self.sysinfo_cache.lock().unwrap();
        let mut cpu_col = self.cpu_collector.lock().unwrap();

        let cpu = cpu_col.collect(&mut sys);
        let memory = collect_memory_metrics();
        let disk = collect_disk_metrics();
        let power = collect_power_metrics();

        let uptime_seconds = System::uptime();
        let hostname = System::host_name().unwrap_or_else(|| "Windows PC".to_string());
        let os_name = System::long_os_version().unwrap_or_else(|| "Windows 11".to_string());

        SystemSnapshot {
            timestamp: Utc::now().to_rfc3339(),
            uptime_seconds,
            hostname,
            os_name,
            cpu,
            memory,
            disk,
            power,
            gpu: self.cached_gpu.clone(),
        }
    }
}
