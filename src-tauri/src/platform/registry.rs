// TENREC - Minimal registry access with exact previous-value capture for undo
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegEnumKeyExW, RegEnumValueW, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY,
    REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_OPTION_NON_VOLATILE, REG_SZ,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hive {
    #[serde(rename = "HKCU")]
    CurrentUser,
    #[serde(rename = "HKLM")]
    LocalMachine,
}

impl Hive {
    fn raw(self) -> HKEY {
        match self {
            Hive::CurrentUser => HKEY_CURRENT_USER,
            Hive::LocalMachine => HKEY_LOCAL_MACHINE,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Hive::CurrentUser => "HKCU",
            Hive::LocalMachine => "HKLM",
        }
    }
}

/// A raw registry value: type code plus bytes, so undo restores it exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegValue {
    pub kind: u32,
    pub data: Vec<u8>,
}

impl RegValue {
    pub fn dword(v: u32) -> Self {
        Self { kind: REG_DWORD, data: v.to_le_bytes().to_vec() }
    }
    pub fn string(s: &str) -> Self {
        Self { kind: REG_SZ, data: wide(s).iter().flat_map(|c| c.to_le_bytes()).collect() }
    }
    pub fn binary(b: &[u8]) -> Self {
        Self { kind: REG_BINARY, data: b.to_vec() }
    }
    pub fn as_dword(&self) -> Option<u32> {
        (self.kind == REG_DWORD && self.data.len() >= 4).then(|| u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]]))
    }
    pub fn as_string(&self) -> Option<String> {
        if self.kind != REG_SZ && self.kind != REG_EXPAND_SZ {
            return None;
        }
        let u16s: Vec<u16> = self.data.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let end = u16s.iter().position(|&c| c == 0).unwrap_or(u16s.len());
        Some(String::from_utf16_lossy(&u16s[..end]))
    }
}

pub fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

fn err_text(code: u32, hive: Hive, key: &str) -> String {
    match code {
        5 => format!("Access denied to {}\\{key}. Administrator rights are needed.", hive.label()),
        _ => format!("Registry error {code} at {}\\{key}", hive.label()),
    }
}

fn open(hive: Hive, key: &str, access: u32) -> Result<Option<Key>, String> {
    let mut h: HKEY = std::ptr::null_mut();
    let rc = unsafe { RegOpenKeyExW(hive.raw(), wide(key).as_ptr(), 0, access | KEY_WOW64_64KEY, &mut h) };
    match rc {
        ERROR_SUCCESS => Ok(Some(Key(h))),
        ERROR_FILE_NOT_FOUND => Ok(None),
        c => Err(err_text(c, hive, key)),
    }
}

/// Reads a value. Ok(None) when the key or value does not exist.
pub fn read(hive: Hive, key: &str, name: &str) -> Result<Option<RegValue>, String> {
    let Some(k) = open(hive, key, KEY_READ)? else { return Ok(None) };
    let wname = wide(name);
    let mut kind = 0u32;
    let mut len = 0u32;
    let rc = unsafe { RegQueryValueExW(k.0, wname.as_ptr(), std::ptr::null(), &mut kind, std::ptr::null_mut(), &mut len) };
    if rc == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    if rc != ERROR_SUCCESS {
        return Err(err_text(rc, hive, key));
    }
    let mut data = vec![0u8; len as usize];
    let rc = unsafe { RegQueryValueExW(k.0, wname.as_ptr(), std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut len) };
    if rc != ERROR_SUCCESS {
        return Err(err_text(rc, hive, key));
    }
    data.truncate(len as usize);
    Ok(Some(RegValue { kind, data }))
}

/// Writes a value, creating the key if needed.
pub fn write(hive: Hive, key: &str, name: &str, value: &RegValue) -> Result<(), String> {
    let mut h: HKEY = std::ptr::null_mut();
    let rc = unsafe {
        RegCreateKeyExW(
            hive.raw(),
            wide(key).as_ptr(),
            0,
            std::ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE | KEY_WOW64_64KEY,
            std::ptr::null(),
            &mut h,
            std::ptr::null_mut(),
        )
    };
    if rc != ERROR_SUCCESS {
        return Err(err_text(rc, hive, key));
    }
    let k = Key(h);
    let rc = unsafe { RegSetValueExW(k.0, wide(name).as_ptr(), 0, value.kind, value.data.as_ptr(), value.data.len() as u32) };
    if rc != ERROR_SUCCESS {
        return Err(err_text(rc, hive, key));
    }
    Ok(())
}

/// Deletes a value. A value that does not exist counts as deleted.
pub fn delete(hive: Hive, key: &str, name: &str) -> Result<(), String> {
    let Some(k) = open(hive, key, KEY_SET_VALUE)? else { return Ok(()) };
    match unsafe { RegDeleteValueW(k.0, wide(name).as_ptr()) } {
        ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
        c => Err(err_text(c, hive, key)),
    }
}

/// Restores a value to `previous`: rewrites it, or deletes it if it did not exist.
pub fn restore(hive: Hive, key: &str, name: &str, previous: &Option<RegValue>) -> Result<(), String> {
    match previous {
        Some(v) => write(hive, key, name, v),
        None => delete(hive, key, name),
    }
}

/// Lists (name, value) pairs of a key. Empty when the key does not exist.
pub fn values(hive: Hive, key: &str) -> Result<Vec<(String, RegValue)>, String> {
    let Some(k) = open(hive, key, KEY_READ)? else { return Ok(Vec::new()) };
    let mut out = Vec::new();
    let mut i = 0u32;
    loop {
        let mut name = vec![0u16; 16384];
        let mut name_len = name.len() as u32;
        let mut kind = 0u32;
        let mut data = vec![0u8; 4096];
        let mut data_len = data.len() as u32;
        let mut rc = unsafe { RegEnumValueW(k.0, i, name.as_mut_ptr(), &mut name_len, std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut data_len) };
        if rc == ERROR_MORE_DATA {
            data = vec![0u8; data_len as usize];
            name_len = name.len() as u32;
            rc = unsafe { RegEnumValueW(k.0, i, name.as_mut_ptr(), &mut name_len, std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut data_len) };
        }
        if rc != ERROR_SUCCESS {
            break;
        }
        data.truncate(data_len as usize);
        out.push((String::from_utf16_lossy(&name[..name_len as usize]), RegValue { kind, data }));
        i += 1;
    }
    Ok(out)
}

/// Lists subkey names. Empty when the key does not exist.
pub fn subkeys(hive: Hive, key: &str) -> Result<Vec<String>, String> {
    let Some(k) = open(hive, key, KEY_READ)? else { return Ok(Vec::new()) };
    let mut out = Vec::new();
    let mut i = 0u32;
    loop {
        let mut name = vec![0u16; 512];
        let mut len = name.len() as u32;
        let rc = unsafe {
            RegEnumKeyExW(k.0, i, name.as_mut_ptr(), &mut len, std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut())
        };
        if rc != ERROR_SUCCESS {
            break;
        }
        out.push(String::from_utf16_lossy(&name[..len as usize]));
        i += 1;
    }
    Ok(out)
}
