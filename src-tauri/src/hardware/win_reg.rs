// TENREC - Native Windows Registry Telemetry Helper (Zero Subprocesses)
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::sync::OnceLock;
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};

/// Reads a string value from HKEY_LOCAL_MACHINE registry key without spawning any processes
pub fn get_hklm_string(subkey: &str, value_name: &str) -> Option<String> {
    unsafe {
        let subkey_wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let value_wide: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();

        let mut hkey: HKEY = std::ptr::null_mut();
        let status = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            subkey_wide.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        );

        if status != 0 || hkey.is_null() {
            return None;
        }

        let mut data_type: u32 = 0;
        let mut data_size: u32 = 0;

        // First query size
        let status = RegQueryValueExW(
            hkey,
            value_wide.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            std::ptr::null_mut(),
            &mut data_size,
        );

        if status != 0 || data_size == 0 {
            RegCloseKey(hkey);
            return None;
        }

        let mut buffer = vec![0u8; data_size as usize];
        let status = RegQueryValueExW(
            hkey,
            value_wide.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            buffer.as_mut_ptr(),
            &mut data_size,
        );

        RegCloseKey(hkey);

        if status != 0 {
            return None;
        }

        // Parse UTF-16 string (REG_SZ)
        if data_type == REG_SZ && data_size >= 2 {
            let u16_slice = std::slice::from_raw_parts(
                buffer.as_ptr() as *const u16,
                (data_size as usize) / 2,
            );
            let trimmed = match u16_slice.iter().position(|&c| c == 0) {
                Some(pos) => &u16_slice[..pos],
                None => u16_slice,
            };
            let os_str = OsString::from_wide(trimmed);
            return os_str.into_string().ok();
        }

        None
    }
}

// Static Cache for system hardware identity (cached once, zero repeated I/O)
static SYSTEM_IDENTITY_CACHE: OnceLock<(String, String)> = OnceLock::new();
static GPU_DETAILS_CACHE: OnceLock<(String, Option<String>)> = OnceLock::new();

/// Cached query for System Manufacturer and Model via native Win32 Registry
pub fn get_cached_system_manufacturer_model() -> (String, String) {
    SYSTEM_IDENTITY_CACHE
        .get_or_init(|| {
            let mfg = get_hklm_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemManufacturer")
                .or_else(|| get_hklm_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BaseBoardManufacturer"))
                .unwrap_or_else(|| "Unknown manufacturer".to_string());

            let model = get_hklm_string(r"HARDWARE\DESCRIPTION\System\BIOS", "SystemProductName")
                .or_else(|| get_hklm_string(r"HARDWARE\DESCRIPTION\System\BIOS", "BaseBoardProduct"))
                .unwrap_or_else(|| "Unknown model".to_string());

            (mfg.trim().to_string(), model.trim().to_string())
        })
        .clone()
}

/// Cached query for Primary Display Adapter and Driver Version via native Win32 Registry
pub fn get_cached_gpu_details() -> (String, Option<String>) {
    GPU_DETAILS_CACHE
        .get_or_init(|| {
            // Check Class 0000 first (dedicated or primary GPU)
            let gpu0 = get_hklm_string(
                r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000",
                "DriverDesc",
            );
            let ver0 = get_hklm_string(
                r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000",
                "DriverVersion",
            );

            if let Some(desc) = gpu0 {
                if !desc.trim().is_empty() {
                    return (desc.trim().to_string(), ver0);
                }
            }

            // Check Class 0001 (secondary/integrated GPU)
            let gpu1 = get_hklm_string(
                r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0001",
                "DriverDesc",
            );
            let ver1 = get_hklm_string(
                r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0001",
                "DriverVersion",
            );

            if let Some(desc) = gpu1 {
                if !desc.trim().is_empty() {
                    return (desc.trim().to_string(), ver1);
                }
            }

            ("Unknown display adapter".to_string(), None)
        })
        .clone()
}
