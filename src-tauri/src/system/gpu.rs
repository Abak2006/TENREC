// TENREC - Native Windows GPU Telemetry
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetrics {
    pub name: String,
    pub is_available: bool,
    pub memory_total_bytes: Option<u64>,
    pub memory_used_bytes: Option<u64>,
    pub utilization_percent: Option<f64>,
    pub temperature_c: Option<f64>,
    pub status_message: String,
}

pub fn collect_gpu_metrics() -> GpuMetrics {
    // Probe Windows Display Adapters via Windows Registry or WMI
    // We check HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000
    // for DriverDesc
    let adapter_name = query_primary_display_adapter();

    if let Some(name) = adapter_name {
        GpuMetrics {
            name,
            is_available: true,
            memory_total_bytes: None,
            memory_used_bytes: None,
            utilization_percent: None,
            temperature_c: None,
            status_message: "Display Adapter active. Advanced per-frame telemetry requires vendor hardware extension.".to_string(),
        }
    } else {
        GpuMetrics {
            name: "Generic Display Adapter".to_string(),
            is_available: false,
            memory_total_bytes: None,
            memory_used_bytes: None,
            utilization_percent: None,
            temperature_c: None,
            status_message: "GPU temperature / metrics unavailable on this system.".to_string(),
        }
    }
}

fn query_primary_display_adapter() -> Option<String> {
    // Check registry for active display adapter description
    use std::process::Command;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-CimInstance Win32_VideoController | Select-Object -First 1).Name"])
        .output()
        .ok()?;

    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}
