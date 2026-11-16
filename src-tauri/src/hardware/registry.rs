// TENREC - Hardware Registry & Provider Discovery
use crate::hardware::types::{
    AccessMode, CapabilityState, ComponentCategory, HardwareCapabilityItem, HardwareDiagnostics,
};
use crate::hardware::win_reg::{get_cached_gpu_details, get_cached_system_manufacturer_model};
use sysinfo::System;

pub struct HardwareRegistry;

impl HardwareRegistry {
    /// Discover system hardware diagnostics from native system information
    pub fn get_diagnostics() -> HardwareDiagnostics {
        let mut sys = System::new_all();
        sys.refresh_all();

        let (mfg, model) = Self::query_system_manufacturer_model();
        let cpu_name = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cpu_cores = sys.physical_core_count().unwrap_or(sys.cpus().len());
        let cpu_logical_cores = sys.cpus().len();

        let (gpu_name, gpu_driver) = Self::query_gpu_details();
        let total_mem = sys.total_memory();
        let os_build = System::os_version().unwrap_or_default();

        let has_battery = Self::check_battery_present();
        let is_laptop = has_battery || model.to_lowercase().contains("laptop") || model.to_lowercase().contains("notebook");

        HardwareDiagnostics {
            manufacturer: mfg,
            model,
            cpu_name,
            cpu_cores,
            cpu_logical_cores,
            gpu_name,
            gpu_driver_version: gpu_driver,
            total_memory_bytes: total_mem,
            os_build,
            is_laptop,
            has_battery,
        }
    }

    /// What TENREC actually reads or controls on this machine. Every "Available" entry
    /// corresponds to code that runs; everything else is listed as not implemented.
    pub fn get_capabilities(diag: &HardwareDiagnostics) -> Vec<HardwareCapabilityItem> {
        let elevated = crate::protection::process_ctrl::is_elevated();
        let item = |id: &str, name: &str, category, provider: &str, state, access_mode, description: String| HardwareCapabilityItem {
            id: id.to_string(),
            name: name.to_string(),
            category,
            provider: provider.to_string(),
            state,
            access_mode,
            description,
        };
        use AccessMode::*;
        use CapabilityState::*;
        use ComponentCategory::*;
        vec![
            item("cpu_usage", "CPU usage", Cpu, "GetSystemTimes, sysinfo", Available, ReadOnly,
                 "Total and per-core utilisation, sampled every 2 seconds.".into()),
            item("memory_usage", "Memory usage", Memory, "GlobalMemoryStatusEx", Available, ReadOnly,
                 "Physical memory used and available.".into()),
            item("volumes", "Volume capacity", Storage, "GetDiskFreeSpaceExW", Available, ReadOnly,
                 "Size and free space of fixed and removable drives.".into()),
            item("process_io", "Process I/O rates", Storage, "GetProcessIoCounters (via sysinfo)", Available, ReadOnly,
                 "Read/write bytes per second per process. Includes file, device and network I/O; Windows does not separate disk-only I/O here.".into()),
            item("battery", "Battery and AC status", PowerBattery, "GetSystemPowerStatus",
                 if diag.has_battery { Available } else { NotPresent }, ReadOnly,
                 if diag.has_battery { "Charge level and charging state.".into() } else { "No battery detected.".into() }),
            item("sockets", "Connections and listening ports", Platform, "GetExtendedTcpTable / GetExtendedUdpTable", Available, ReadOnly,
                 "IPv4 and IPv6 sockets with owning process. Hostnames are not resolved and per-connection byte counts are not collected.".into()),
            item("gpu_name", "Display adapter name", Gpu, "Registry (display class key)", Available, ReadOnly,
                 format!("Detected: {}", diag.gpu_name)),
            item("firewall", "Firewall rules (block IP, program, inbound port)", Platform, "netsh advfirewall",
                 if elevated { Available } else { RequiresAdmin }, ReadWrite,
                 if elevated { "Only rules named TENREC_RULE_* are ever created or removed.".into() }
                 else { "Restart TENREC with \"Run as administrator\" to apply firewall actions.".into() }),
            item("priority", "Process priority", Platform, "SetPriorityClass", Available, ReadWrite,
                 "Lower a process to Idle priority and restore its previous class. Processes of other users need administrator rights.".into()),
            item("suspend", "Suspend / resume process", Platform, "NtSuspendProcess / NtResumeProcess", Available, ReadWrite,
                 "Pause and resume a process. Core Windows processes are refused.".into()),
            item("per_process_network", "Per-process network bandwidth", Platform, "ETW (planned)", NotImplemented, ReadOnly,
                 "Needs Event Tracing for Windows; not collected yet.".into()),
            item("gpu_metrics", "GPU utilisation and temperature", Gpu, "Vendor APIs (planned)", NotImplemented, ReadOnly,
                 "Not collected yet.".into()),
            item("cpu_temperature", "CPU temperature and fans", ThermalFan, "-", NotImplemented, ReadOnly,
                 "Windows has no standard API for this without a kernel driver; not collected.".into()),
        ]
    }

    fn query_system_manufacturer_model() -> (String, String) {
        get_cached_system_manufacturer_model()
    }

    fn query_gpu_details() -> (String, Option<String>) {
        get_cached_gpu_details()
    }

    fn check_battery_present() -> bool {
        use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        unsafe {
            let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut status) != 0 {
                // BatteryFlag & 128 means "No system battery"
                return (status.BatteryFlag & 128) == 0;
            }
        }
        false
    }
}
