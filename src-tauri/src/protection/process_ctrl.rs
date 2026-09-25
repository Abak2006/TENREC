// TENREC - Native Windows Process Control (Priority, Suspend, Resume)
use std::ffi::CString;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    PROCESS_SET_INFORMATION, PROCESS_SUSPEND_RESUME,
};

type PfnNtSuspendProcess = unsafe extern "system" fn(HANDLE) -> i32;
type PfnNtResumeProcess = unsafe extern "system" fn(HANDLE) -> i32;

pub struct ProcessController;

impl ProcessController {
    /// Set process priority to IDLE
    pub fn lower_priority_to_idle(pid: u32) -> Result<(), String> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
            if handle.is_null() {
                return Err(format!("Could not open process {} for priority change", pid));
            }

            let success = SetPriorityClass(handle, IDLE_PRIORITY_CLASS);
            CloseHandle(handle);

            if success == 0 {
                return Err(format!("Failed to set IDLE priority for PID {}", pid));
            }
        }
        Ok(())
    }

    /// Restore process priority to NORMAL
    pub fn restore_normal_priority(pid: u32) -> Result<(), String> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, 0, pid);
            if handle.is_null() {
                return Err(format!("Could not open process {} for priority change", pid));
            }

            let success = SetPriorityClass(handle, NORMAL_PRIORITY_CLASS);
            CloseHandle(handle);

            if success == 0 {
                return Err(format!("Failed to restore NORMAL priority for PID {}", pid));
            }
        }
        Ok(())
    }

    /// Suspend all threads of a process cleanly using NtSuspendProcess
    pub fn suspend_process(pid: u32) -> Result<(), String> {
        unsafe {
            let ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
            if ntdll.is_null() {
                return Err("Failed to locate ntdll.dll".to_string());
            }

            let fn_name = CString::new("NtSuspendProcess").unwrap();
            let proc_addr = GetProcAddress(ntdll, fn_name.as_ptr() as *const u8);
            if proc_addr.is_none() {
                return Err("NtSuspendProcess symbol not found".to_string());
            }

            let nt_suspend: PfnNtSuspendProcess = std::mem::transmute(proc_addr);

            let handle = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
            if handle.is_null() {
                return Err(format!("Cannot open PID {} for suspension (Access Denied / Protected)", pid));
            }

            let status = nt_suspend(handle);
            CloseHandle(handle);

            if status != 0 {
                return Err(format!("NtSuspendProcess returned error NTSTATUS: 0x{:X}", status));
            }
        }
        Ok(())
    }

    /// Resume a previously suspended process using NtResumeProcess
    pub fn resume_process(pid: u32) -> Result<(), String> {
        unsafe {
            let ntdll = GetModuleHandleA(b"ntdll.dll\0".as_ptr());
            if ntdll.is_null() {
                return Err("Failed to locate ntdll.dll".to_string());
            }

            let fn_name = CString::new("NtResumeProcess").unwrap();
            let proc_addr = GetProcAddress(ntdll, fn_name.as_ptr() as *const u8);
            if proc_addr.is_none() {
                return Err("NtResumeProcess symbol not found".to_string());
            }

            let nt_resume: PfnNtResumeProcess = std::mem::transmute(proc_addr);

            let handle = OpenProcess(PROCESS_SUSPEND_RESUME, 0, pid);
            if handle.is_null() {
                return Err(format!("Cannot open PID {} for resumption", pid));
            }

            let status = nt_resume(handle);
            CloseHandle(handle);

            if status != 0 {
                return Err(format!("NtResumeProcess returned error NTSTATUS: 0x{:X}", status));
            }
        }
        Ok(())
    }
}
