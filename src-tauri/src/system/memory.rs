// TENREC - Native Windows Memory (RAM) Telemetry
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

pub fn collect_memory_metrics() -> MemoryMetrics {
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;

        if GlobalMemoryStatusEx(&mut status as *mut _) != 0 {
            let total = status.ullTotalPhys;
            let available = status.ullAvailPhys;
            let used = total.saturating_sub(available);
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                status.dwMemoryLoad as f64
            };

            MemoryMetrics {
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent: percent.clamp(0.0, 100.0),
            }
        } else {
            // Fallback to sysinfo if API call fails
            let mut sys = sysinfo::System::new();
            sys.refresh_memory();
            let total = sys.total_memory();
            let used = sys.used_memory();
            let available = sys.available_memory();
            let percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

            MemoryMetrics {
                total_bytes: total,
                used_bytes: used,
                available_bytes: available,
                usage_percent: percent,
            }
        }
    }
}
