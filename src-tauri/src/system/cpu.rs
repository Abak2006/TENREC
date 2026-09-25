// TENREC - Native Windows CPU Telemetry
use serde::{Deserialize, Serialize};
use sysinfo::System;
use windows_sys::Win32::System::Threading::GetSystemTimes;
use windows_sys::Win32::Foundation::FILETIME;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    pub usage_percent: f64,
    pub frequency_mhz: Option<u64>,
    pub core_count: usize,
    pub logical_core_count: usize,
    pub per_core_usage: Vec<f64>,
    pub temperature_c: Option<f64>,
}

pub struct CpuCollector {
    prev_idle: u64,
    prev_kernel: u64,
    prev_user: u64,
    has_prev: bool,
}

impl CpuCollector {
    pub fn new() -> Self {
        Self {
            prev_idle: 0,
            prev_kernel: 0,
            prev_user: 0,
            has_prev: false,
        }
    }

    fn filetime_to_u64(ft: &FILETIME) -> u64 {
        ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
    }

    pub fn collect(&mut self, sys: &mut System) -> CpuMetrics {
        // Refresh sysinfo CPU state for per-core breakdown
        sys.refresh_cpu_all();

        let mut idle_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

        let win_calc = unsafe {
            GetSystemTimes(
                &mut idle_time as *mut _,
                &mut kernel_time as *mut _,
                &mut user_time as *mut _,
            )
        };

        let mut calculated_usage = sys.global_cpu_usage() as f64;

        if win_calc != 0 {
            let idle = Self::filetime_to_u64(&idle_time);
            let kernel = Self::filetime_to_u64(&kernel_time);
            let user = Self::filetime_to_u64(&user_time);

            if self.has_prev {
                let delta_idle = idle.saturating_sub(self.prev_idle);
                let delta_kernel = kernel.saturating_sub(self.prev_kernel);
                let delta_user = user.saturating_sub(self.prev_user);
                let total = delta_kernel + delta_user;

                if total > 0 && total >= delta_idle {
                    calculated_usage = ((total - delta_idle) as f64 / total as f64) * 100.0;
                }
            }

            self.prev_idle = idle;
            self.prev_kernel = kernel;
            self.prev_user = user;
            self.has_prev = true;
        }

        let per_core_usage: Vec<f64> = sys.cpus().iter().map(|c| c.cpu_usage() as f64).collect();
        let core_count = sys.physical_core_count().unwrap_or(sys.cpus().len());
        let logical_core_count = sys.cpus().len();

        let frequency_mhz = sys.cpus().first().map(|c| c.frequency());

        CpuMetrics {
            usage_percent: calculated_usage.clamp(0.0, 100.0),
            frequency_mhz,
            core_count,
            logical_core_count,
            per_core_usage,
            temperature_c: None, // Honestly reported as None unless hardware vendor sensor exposed
        }
    }
}
