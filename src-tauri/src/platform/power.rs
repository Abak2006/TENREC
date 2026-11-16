// TENREC - Power plans (powercfg) and the Windows 11 power mode slider
use serde::{Deserialize, Serialize};
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows_sys::core::GUID;
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Power mode overlay GUIDs used by the Settings > Power mode slider.
pub const MODE_BEST_EFFICIENCY: &str = "961cc777-2547-4f9d-8174-7d86181b8a7a";
pub const MODE_BALANCED: &str = "00000000-0000-0000-0000-000000000000";
pub const MODE_BEST_PERFORMANCE: &str = "ded574b5-45a0-4f42-8737-46345c09c238";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerPlan {
    pub guid: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerState {
    pub plans: Vec<PowerPlan>,
    /// Current power mode overlay ("best_efficiency" | "balanced" | "best_performance"),
    /// None when this Windows build does not expose it.
    pub mode: Option<String>,
}

pub fn is_guid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && [8, 4, 4, 4, 12].iter().zip(&parts).all(|(n, p)| p.len() == *n && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn powercfg(args: &[&str]) -> Result<String, String> {
    let out = Command::new("powercfg.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(args)
        .output()
        .map_err(|e| format!("Could not start powercfg: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() {
        Ok(text)
    } else {
        Err(format!("powercfg failed: {}{}", text.trim(), String::from_utf8_lossy(&out.stderr).trim()))
    }
}

/// Parses `powercfg /list`. Labels are localised, so only the GUID, the
/// parenthesised name and the trailing `*` are used.
pub fn parse_plan_list(text: &str) -> Vec<PowerPlan> {
    let mut plans: Vec<PowerPlan> = Vec::new();
    for line in text.lines() {
        let Some(guid) = line.split_whitespace().find(|w| is_guid(w)) else { continue };
        let name = match (line.find('('), line.rfind(')')) {
            (Some(a), Some(b)) if b > a => line[a + 1..b].to_string(),
            _ => guid.to_string(),
        };
        let active = line.trim_end().ends_with('*');
        if let Some(existing) = plans.iter_mut().find(|p| p.guid.eq_ignore_ascii_case(guid)) {
            existing.active |= active;
        } else {
            plans.push(PowerPlan { guid: guid.to_ascii_lowercase(), name, active });
        }
    }
    plans
}

pub fn active_plan() -> Result<String, String> {
    let text = powercfg(&["/getactivescheme"])?;
    text.split_whitespace().find(|w| is_guid(w)).map(|g| g.to_ascii_lowercase()).ok_or("Could not read the active power plan.".to_string())
}

pub fn set_plan(guid: &str) -> Result<(), String> {
    if !is_guid(guid) {
        return Err("Invalid power plan id.".to_string());
    }
    powercfg(&["/setactive", guid]).map(|_| ())
}

fn parse_guid(s: &str) -> Option<GUID> {
    if !is_guid(s) {
        return None;
    }
    let hex: String = s.chars().filter(|c| *c != '-').collect();
    let v = u128::from_str_radix(&hex, 16).ok()?;
    Some(GUID::from_u128(v))
}

fn guid_string(g: &GUID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        g.data1, g.data2, g.data3, g.data4[0], g.data4[1], g.data4[2], g.data4[3], g.data4[4], g.data4[5], g.data4[6], g.data4[7]
    )
}

type GetOverlayFn = unsafe extern "system" fn(*mut GUID) -> u32;
type SetOverlayFn = unsafe extern "system" fn(*const GUID) -> u32;

fn powrprof(name: &core::ffi::CStr) -> Option<unsafe extern "system" fn() -> isize> {
    unsafe {
        let lib = LoadLibraryA(c"powrprof.dll".as_ptr() as *const u8);
        if lib.is_null() {
            return None;
        }
        GetProcAddress(lib, name.as_ptr() as *const u8)
    }
}

/// Current power mode overlay GUID (Windows 10 1709+).
pub fn mode_guid() -> Option<String> {
    let f: GetOverlayFn = unsafe { std::mem::transmute(powrprof(c"PowerGetEffectiveOverlayScheme")?) };
    let mut g = GUID::from_u128(0);
    (unsafe { f(&mut g) } == 0).then(|| guid_string(&g))
}

pub fn set_mode_guid(guid: &str) -> Result<(), String> {
    let g = parse_guid(guid).ok_or("Invalid power mode id.")?;
    let f: SetOverlayFn = unsafe {
        std::mem::transmute(powrprof(c"PowerSetActiveOverlayScheme").ok_or("This Windows version does not expose the power mode setting.")?)
    };
    match unsafe { f(&g) } {
        0 => Ok(()),
        rc => Err(format!("Windows refused to change the power mode (error {rc}). Some power plans, such as Ultimate Performance, do not support power modes.")),
    }
}

pub fn mode_name(guid: &str) -> Option<&'static str> {
    match guid {
        MODE_BEST_EFFICIENCY => Some("best_efficiency"),
        MODE_BALANCED => Some("balanced"),
        MODE_BEST_PERFORMANCE => Some("best_performance"),
        _ => None,
    }
}

pub fn mode_from_name(name: &str) -> Option<&'static str> {
    match name {
        "best_efficiency" => Some(MODE_BEST_EFFICIENCY),
        "balanced" => Some(MODE_BALANCED),
        "best_performance" => Some(MODE_BEST_PERFORMANCE),
        _ => None,
    }
}

pub fn state() -> Result<PowerState, String> {
    let plans = parse_plan_list(&powercfg(&["/list"])?);
    Ok(PowerState { plans, mode: mode_guid().and_then(|g| mode_name(&g).map(str::to_string)) })
}
