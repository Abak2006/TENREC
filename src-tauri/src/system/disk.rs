// TENREC - Native Windows Disk Telemetry
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetLogicalDrives, GetVolumeInformationW,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskDrive {
    pub mount_point: String,
    pub volume_name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
    pub file_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskMetrics {
    pub drives: Vec<DiskDrive>,
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
}

pub fn collect_disk_metrics() -> DiskMetrics {
    let mut drives = Vec::new();

    unsafe {
        let drive_mask = GetLogicalDrives();
        for i in 0..26 {
            if (drive_mask & (1 << i)) != 0 {
                let letter = (b'A' + i) as char;
                let root_str = format!("{}:\\\0", letter);
                let wide_root: Vec<u16> = root_str.encode_utf16().collect();

                let mut free_bytes_caller: u64 = 0;
                let mut total_bytes: u64 = 0;
                let mut total_free_bytes: u64 = 0;

                let success = GetDiskFreeSpaceExW(
                    wide_root.as_ptr(),
                    &mut free_bytes_caller as *mut _ as *mut _,
                    &mut total_bytes as *mut _ as *mut _,
                    &mut total_free_bytes as *mut _ as *mut _,
                );

                if success != 0 && total_bytes > 0 {
                    let used_bytes = total_bytes.saturating_sub(total_free_bytes);
                    let usage_percent = (used_bytes as f64 / total_bytes as f64) * 100.0;

                    // Query Volume Name & File System
                    let mut volume_buf = [0u16; 260];
                    let mut fs_buf = [0u16; 260];

                    let vol_res = GetVolumeInformationW(
                        wide_root.as_ptr(),
                        volume_buf.as_mut_ptr(),
                        volume_buf.len() as u32,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        fs_buf.as_mut_ptr(),
                        fs_buf.len() as u32,
                    );

                    let volume_name = if vol_res != 0 {
                        let len = volume_buf.iter().position(|&c| c == 0).unwrap_or(0);
                        OsString::from_wide(&volume_buf[..len]).to_string_lossy().to_string()
                    } else {
                        String::new()
                    };

                    let file_system = if vol_res != 0 {
                        let len = fs_buf.iter().position(|&c| c == 0).unwrap_or(0);
                        OsString::from_wide(&fs_buf[..len]).to_string_lossy().to_string()
                    } else {
                        "NTFS".to_string()
                    };

                    drives.push(DiskDrive {
                        mount_point: format!("{}:\\", letter),
                        volume_name,
                        total_bytes,
                        free_bytes: total_free_bytes,
                        used_bytes,
                        usage_percent: usage_percent.clamp(0.0, 100.0),
                        file_system,
                    });
                }
            }
        }
    }

    DiskMetrics {
        drives,
        read_bytes_sec: 0,
        write_bytes_sec: 0,
    }
}
