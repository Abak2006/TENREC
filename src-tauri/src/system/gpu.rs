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
            status_message: "Adapter name only. TENREC does not collect GPU utilisation or temperature yet.".to_string(),
        }
    } else {
        GpuMetrics {
            name: "Unknown display adapter".to_string(),
            is_available: false,
            memory_total_bytes: None,
            memory_used_bytes: None,
            utilization_percent: None,
            temperature_c: None,
            status_message: "Display adapter could not be identified from the registry.".to_string(),
        }
    }
}

fn query_primary_display_adapter() -> Option<String> {
    let (name, _) = crate::hardware::win_reg::get_cached_gpu_details();
    if !name.is_empty() && name != "Unknown display adapter" {
        Some(name)
    } else {
        None
    }
}
