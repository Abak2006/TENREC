// TENREC - Native Windows Power & Battery Telemetry
use serde::{Deserialize, Serialize};
use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerMetrics {
    pub ac_connected: bool,
    pub has_battery: bool,
    pub battery_percent: Option<u8>,
    pub battery_status: String,
    pub power_mode: String,
}

pub fn collect_power_metrics() -> PowerMetrics {
    unsafe {
        let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        if GetSystemPowerStatus(&mut status as *mut _) != 0 {
            let ac_connected = status.ACLineStatus == 1;
            let has_battery = status.BatteryFlag != 128 && status.BatteryFlag != 255;
            let battery_percent = if has_battery && status.BatteryLifePercent <= 100 {
                Some(status.BatteryLifePercent)
            } else {
                None
            };

            let battery_status = if !has_battery {
                "No Battery Present".to_string()
            } else if (status.BatteryFlag & 8) != 0 {
                "Charging".to_string()
            } else if (status.BatteryFlag & 4) != 0 {
                "Critical".to_string()
            } else if (status.BatteryFlag & 2) != 0 {
                "Low".to_string()
            } else {
                "Normal Discharging".to_string()
            };

            let power_mode = if ac_connected {
                "Balanced (AC Plugged)".to_string()
            } else {
                "Power Efficiency (DC Battery)".to_string()
            };

            PowerMetrics {
                ac_connected,
                has_battery,
                battery_percent,
                battery_status,
                power_mode,
            }
        } else {
            PowerMetrics {
                ac_connected: true,
                has_battery: false,
                battery_percent: None,
                battery_status: "Telemetry Unavailable".to_string(),
                power_mode: "Standard Windows Profile".to_string(),
            }
        }
    }
}
