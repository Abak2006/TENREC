// TENREC - Executable version resource reader (CompanyName / FileDescription)
//
// These strings come from the file's own version resource. They are NOT a signature
// check: any binary can claim any company name. The UI labels them accordingly.
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

#[derive(Debug, Clone, Default)]
pub struct VersionInfo {
    pub company: Option<String>,
    pub description: Option<String>,
    pub product: Option<String>,
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

unsafe fn query_string(block: &[u8], sub_block: &str) -> Option<String> {
    let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut len: u32 = 0;
    let key = wide(sub_block);
    if VerQueryValueW(block.as_ptr() as *const _, key.as_ptr(), &mut ptr, &mut len) == 0 || ptr.is_null() || len == 0 {
        return None;
    }
    let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    let value = String::from_utf16_lossy(&slice[..end]).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

pub fn read_version_info(path: &str) -> VersionInfo {
    if path.is_empty() {
        return VersionInfo::default();
    }
    let wpath = wide(path);
    unsafe {
        let mut handle: u32 = 0;
        let size = GetFileVersionInfoSizeW(wpath.as_ptr(), &mut handle);
        if size == 0 {
            return VersionInfo::default();
        }
        let mut block = vec![0u8; size as usize];
        if GetFileVersionInfoW(wpath.as_ptr(), 0, size, block.as_mut_ptr() as *mut _) == 0 {
            return VersionInfo::default();
        }

        // Find the first language/codepage pair; fall back to US English + Unicode.
        let mut translations: Vec<(u16, u16)> = Vec::new();
        let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut len: u32 = 0;
        let key = wide("\\VarFileInfo\\Translation");
        if VerQueryValueW(block.as_ptr() as *const _, key.as_ptr(), &mut ptr, &mut len) != 0 && !ptr.is_null() {
            let pairs = std::slice::from_raw_parts(ptr as *const u16, (len / 2) as usize);
            for chunk in pairs.chunks_exact(2) {
                translations.push((chunk[0], chunk[1]));
            }
        }
        translations.push((0x0409, 0x04B0));
        translations.push((0x0409, 0x04E4));

        for (lang, cp) in translations {
            let prefix = format!("\\StringFileInfo\\{:04x}{:04x}\\", lang, cp);
            let company = query_string(&block, &format!("{prefix}CompanyName"));
            let description = query_string(&block, &format!("{prefix}FileDescription"));
            let product = query_string(&block, &format!("{prefix}ProductName"));
            if company.is_some() || description.is_some() || product.is_some() {
                return VersionInfo { company, description, product };
            }
        }
    }
    VersionInfo::default()
}
