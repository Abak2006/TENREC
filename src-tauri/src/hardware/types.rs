// TENREC - Hardware Discovery & Extensible Capability Registry Types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityState {
    Available,
    RequiresAdmin,
    NotPresent,
    NotImplemented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentCategory {
    Cpu,
    Gpu,
    Memory,
    Storage,
    PowerBattery,
    ThermalFan,
    Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareCapabilityItem {
    pub id: String,
    pub name: String,
    pub category: ComponentCategory,
    pub provider: String,
    pub state: CapabilityState,
    pub access_mode: AccessMode,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareDiagnostics {
    pub manufacturer: String,
    pub model: String,
    pub cpu_name: String,
    pub cpu_cores: usize,
    pub cpu_logical_cores: usize,
    pub gpu_name: String,
    pub gpu_driver_version: Option<String>,
    pub total_memory_bytes: u64,
    pub os_build: String,
    pub is_laptop: bool,
    pub has_battery: bool,
}
