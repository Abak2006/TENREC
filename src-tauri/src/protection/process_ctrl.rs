// TENREC - Native process control (priority, suspend, resume) and identity checks
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows_sys::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetPriorityClass, GetProcessTimes, OpenProcess, OpenProcessToken, SetPriorityClass,
    IDLE_PRIORITY_CLASS, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_SUSPEND_RESUME,
};

type NtProcFn = unsafe extern "system" fn(HANDLE) -> i32;

/// Owned process handle, closed on drop.
struct ProcHandle(HANDLE);

impl ProcHandle {
    fn open(pid: u32, access: u32) -> Result<Self, String> {
        let h = unsafe { OpenProcess(access, 0, pid) };
        if h.is_null() {
            let err = std::io::Error::last_os_error();
            return Err(match err.raw_os_error() {
                Some(5) => format!("Access denied to PID {pid}. It belongs to another user or is protected; run TENREC as administrator."),
                Some(87) => format!("Process {pid} is no longer running."),
                _ => format!("Could not open PID {pid}: {err}"),
            });
        }
        Ok(Self(h))
    }
}

impl Drop for ProcHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

/// Process creation time (FILETIME ticks). Used to make sure a PID still refers to
/// the same process instance: Windows reuses PIDs.
pub fn creation_time(pid: u32) -> Option<u64> {
    let h = ProcHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION).ok()?;
    let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
    let (mut c, mut e, mut k, mut u) = (zero, zero, zero, zero);
    let ok = unsafe { GetProcessTimes(h.0, &mut c, &mut e, &mut k, &mut u) };
    if ok == 0 {
        return None;
    }
    Some(((c.dwHighDateTime as u64) << 32) | c.dwLowDateTime as u64)
}

/// Errors unless `pid` is still the process instance that was observed.
pub fn verify_identity(pid: u32, expected_creation_time: u64, name: &str) -> Result<(), String> {
    match creation_time(pid) {
        Some(ct) if ct == expected_creation_time => Ok(()),
        Some(_) => Err(format!("PID {pid} now belongs to a different process; {name} has exited. Nothing was changed.")),
        None => Err(format!("{name} (PID {pid}) is no longer running or cannot be accessed.")),
    }
}

pub fn get_priority_class(pid: u32) -> Result<u32, String> {
    let h = ProcHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let class = unsafe { GetPriorityClass(h.0) };
    if class == 0 {
        return Err(format!("Could not read priority of PID {pid}: {}", std::io::Error::last_os_error()));
    }
    Ok(class)
}

pub fn set_priority_class(pid: u32, class: u32) -> Result<(), String> {
    let h = ProcHandle::open(pid, PROCESS_SET_INFORMATION)?;
    if unsafe { SetPriorityClass(h.0, class) } == 0 {
        return Err(format!("Could not change priority of PID {pid}: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

pub const IDLE: u32 = IDLE_PRIORITY_CLASS;

fn nt_proc(name: &[u8]) -> Result<NtProcFn, String> {
    unsafe {
        let ntdll = GetModuleHandleA(c"ntdll.dll".as_ptr() as *const u8);
        if ntdll.is_null() {
            return Err("ntdll.dll is not loaded".to_string());
        }
        let addr = GetProcAddress(ntdll, name.as_ptr()).ok_or_else(|| "ntdll export not found".to_string())?;
        Ok(std::mem::transmute::<unsafe extern "system" fn() -> isize, NtProcFn>(addr))
    }
}

pub fn suspend_process(pid: u32) -> Result<(), String> {
    let f = nt_proc(b"NtSuspendProcess\0")?;
    let h = ProcHandle::open(pid, PROCESS_SUSPEND_RESUME)?;
    let status = unsafe { f(h.0) };
    if status != 0 {
        return Err(format!("NtSuspendProcess failed for PID {pid} (NTSTATUS 0x{:08X})", status as u32));
    }
    Ok(())
}

pub fn resume_process(pid: u32) -> Result<(), String> {
    let f = nt_proc(b"NtResumeProcess\0")?;
    let h = ProcHandle::open(pid, PROCESS_SUSPEND_RESUME)?;
    let status = unsafe { f(h.0) };
    if status != 0 {
        return Err(format!("NtResumeProcess failed for PID {pid} (NTSTATUS 0x{:08X})", status as u32));
    }
    Ok(())
}

/// Whether TENREC itself is running elevated (required for firewall changes).
pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        );
        CloseHandle(token);
        ok != 0 && elevation.TokenIsElevated != 0
    }
}
