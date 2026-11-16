// TENREC - Reversible per-process scheduling hints
//
// Every setting is read before it is changed, and only settings that could be read
// are changed, so undo always has an exact value to restore.
use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::SystemInformation::{GetSystemCpuSetInformation, SYSTEM_CPU_SET_INFORMATION};
use windows_sys::Win32::System::Threading::{
    GetPriorityClass, GetProcessDefaultCpuSets, GetProcessInformation, OpenProcess, ProcessMemoryPriority,
    ProcessPowerThrottling, SetPriorityClass, SetProcessDefaultCpuSets, SetProcessInformation, BELOW_NORMAL_PRIORITY_CLASS,
    HIGH_PRIORITY_CLASS, MEMORY_PRIORITY_INFORMATION, PROCESS_POWER_THROTTLING_STATE, PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_SET_LIMITED_INFORMATION,
};

const POWER_THROTTLING_VERSION: u32 = 1;
const POWER_THROTTLING_EXECUTION_SPEED: u32 = 1;
const PROCESS_IO_PRIORITY_CLASS: u32 = 33;
const IO_PRIORITY_LOW: u32 = 1;
const MEMORY_PRIORITY_LOW: u32 = 2;

/// Values captured before a change. `None` = could not be read, so it is not changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuningState {
    pub priority_class: Option<u32>,
    /// (ControlMask, StateMask) of execution-speed throttling ("efficiency mode").
    pub power_throttling: Option<(u32, u32)>,
    pub io_priority: Option<u32>,
    pub memory_priority: Option<u32>,
    /// Empty = no restriction.
    pub cpu_sets: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundTuning {
    pub efficiency_mode: bool,
    pub lower_priority: bool,
    pub low_io: bool,
    pub low_memory: bool,
}

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn open(pid: u32, access: u32) -> Option<Handle> {
    let h = unsafe { OpenProcess(access, 0, pid) };
    (!h.is_null()).then_some(Handle(h))
}

type NtInfoFn = unsafe extern "system" fn(HANDLE, u32, *mut core::ffi::c_void, u32, *mut u32) -> i32;
type NtSetInfoFn = unsafe extern "system" fn(HANDLE, u32, *const core::ffi::c_void, u32) -> i32;

fn ntdll(name: &core::ffi::CStr) -> Option<unsafe extern "system" fn() -> isize> {
    unsafe {
        let m = GetModuleHandleA(c"ntdll.dll".as_ptr() as *const u8);
        if m.is_null() {
            return None;
        }
        GetProcAddress(m, name.as_ptr() as *const u8)
    }
}

fn get_io_priority(h: &Handle) -> Option<u32> {
    let f: NtInfoFn = unsafe { std::mem::transmute(ntdll(c"NtQueryInformationProcess")?) };
    let mut v = 0u32;
    let mut len = 0u32;
    let st = unsafe { f(h.0, PROCESS_IO_PRIORITY_CLASS, &mut v as *mut _ as *mut _, 4, &mut len) };
    (st == 0).then_some(v)
}

fn set_io_priority(h: &Handle, v: u32) -> Result<(), String> {
    let f: NtSetInfoFn = unsafe { std::mem::transmute(ntdll(c"NtSetInformationProcess").ok_or("ntdll export missing")?) };
    let st = unsafe { f(h.0, PROCESS_IO_PRIORITY_CLASS, &v as *const _ as *const _, 4) };
    if st == 0 { Ok(()) } else { Err(format!("Could not set I/O priority (NTSTATUS 0x{:08X})", st as u32)) }
}

fn get_power(h: &Handle) -> Option<(u32, u32)> {
    let mut s = PROCESS_POWER_THROTTLING_STATE { Version: POWER_THROTTLING_VERSION, ControlMask: 0, StateMask: 0 };
    let ok = unsafe { GetProcessInformation(h.0, ProcessPowerThrottling, &mut s as *mut _ as *mut _, std::mem::size_of_val(&s) as u32) };
    (ok != 0).then_some((s.ControlMask, s.StateMask))
}

fn set_power(h: &Handle, control: u32, state: u32) -> Result<(), String> {
    let s = PROCESS_POWER_THROTTLING_STATE { Version: POWER_THROTTLING_VERSION, ControlMask: control, StateMask: state };
    let ok = unsafe { SetProcessInformation(h.0, ProcessPowerThrottling, &s as *const _ as *const _, std::mem::size_of_val(&s) as u32) };
    if ok != 0 { Ok(()) } else { Err(format!("Could not change efficiency mode: {}", std::io::Error::last_os_error())) }
}

fn get_memory(h: &Handle) -> Option<u32> {
    let mut m = MEMORY_PRIORITY_INFORMATION { MemoryPriority: 0 };
    let ok = unsafe { GetProcessInformation(h.0, ProcessMemoryPriority, &mut m as *mut _ as *mut _, 4) };
    (ok != 0).then_some(m.MemoryPriority)
}

fn set_memory(h: &Handle, v: u32) -> Result<(), String> {
    let m = MEMORY_PRIORITY_INFORMATION { MemoryPriority: v };
    let ok = unsafe { SetProcessInformation(h.0, ProcessMemoryPriority, &m as *const _ as *const _, 4) };
    if ok != 0 { Ok(()) } else { Err(format!("Could not change memory priority: {}", std::io::Error::last_os_error())) }
}

fn get_cpu_sets(h: &Handle) -> Option<Vec<u32>> {
    let mut required = 0u32;
    let mut buf = vec![0u32; 256];
    let ok = unsafe { GetProcessDefaultCpuSets(h.0, buf.as_mut_ptr(), buf.len() as u32, &mut required) };
    if ok == 0 {
        return None;
    }
    buf.truncate(required as usize);
    Some(buf)
}

fn set_cpu_sets(h: &Handle, ids: &[u32]) -> Result<(), String> {
    let ok = unsafe { SetProcessDefaultCpuSets(h.0, if ids.is_empty() { std::ptr::null() } else { ids.as_ptr() }, ids.len() as u32) };
    if ok != 0 { Ok(()) } else { Err(format!("Could not change CPU set: {}", std::io::Error::last_os_error())) }
}

const ACCESS: u32 = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_INFORMATION | PROCESS_SET_LIMITED_INFORMATION;

fn open_rw(pid: u32) -> Result<Handle, String> {
    open(pid, ACCESS | PROCESS_QUERY_INFORMATION)
        .or_else(|| open(pid, ACCESS))
        .ok_or_else(|| format!("Access denied to PID {pid}"))
}

pub fn capture(pid: u32) -> Result<TuningState, String> {
    let h = open_rw(pid)?;
    let class = unsafe { GetPriorityClass(h.0) };
    Ok(TuningState {
        priority_class: (class != 0).then_some(class),
        power_throttling: get_power(&h),
        io_priority: get_io_priority(&h),
        memory_priority: get_memory(&h),
        cpu_sets: get_cpu_sets(&h),
    })
}

/// Applies background hints. Returns the state to restore (only fields actually changed).
pub fn apply_background(pid: u32, opts: BackgroundTuning, efficiency_cores: Option<&[u32]>) -> Result<TuningState, String> {
    let before = capture(pid)?;
    let h = open_rw(pid)?;
    let mut changed = TuningState { priority_class: None, power_throttling: None, io_priority: None, memory_priority: None, cpu_sets: None };

    if opts.lower_priority {
        if let Some(prev) = before.priority_class {
            // Never raise anything; only lower Normal/Above-normal processes.
            if (prev == 0x20 || prev == 0x8000) && unsafe { SetPriorityClass(h.0, BELOW_NORMAL_PRIORITY_CLASS) } != 0 {
                changed.priority_class = Some(prev);
            }
        }
    }
    if opts.efficiency_mode {
        if let Some(prev) = before.power_throttling {
            if set_power(&h, POWER_THROTTLING_EXECUTION_SPEED, POWER_THROTTLING_EXECUTION_SPEED).is_ok() {
                changed.power_throttling = Some(prev);
            }
        }
    }
    if opts.low_io {
        if let Some(prev) = before.io_priority {
            if prev > IO_PRIORITY_LOW && set_io_priority(&h, IO_PRIORITY_LOW).is_ok() {
                changed.io_priority = Some(prev);
            }
        }
    }
    if opts.low_memory {
        if let Some(prev) = before.memory_priority {
            if prev > MEMORY_PRIORITY_LOW && set_memory(&h, MEMORY_PRIORITY_LOW).is_ok() {
                changed.memory_priority = Some(prev);
            }
        }
    }
    if let (Some(ids), Some(prev)) = (efficiency_cores, &before.cpu_sets) {
        // Do not override a CPU set someone else already chose.
        if prev.is_empty() && !ids.is_empty() && set_cpu_sets(&h, ids).is_ok() {
            changed.cpu_sets = Some(prev.clone());
        }
    }
    if changed == (TuningState { priority_class: None, power_throttling: None, io_priority: None, memory_priority: None, cpu_sets: None }) {
        return Err("No setting could be changed for this process.".to_string());
    }
    Ok(changed)
}

/// Raises the focus app: High priority and explicitly no efficiency-mode throttling.
pub fn apply_focus(pid: u32) -> Result<TuningState, String> {
    let before = capture(pid)?;
    let h = open_rw(pid)?;
    let mut changed = TuningState { priority_class: None, power_throttling: None, io_priority: None, memory_priority: None, cpu_sets: None };
    if let Some(prev) = before.priority_class {
        if prev != HIGH_PRIORITY_CLASS && unsafe { SetPriorityClass(h.0, HIGH_PRIORITY_CLASS) } != 0 {
            changed.priority_class = Some(prev);
        }
    }
    if let Some(prev) = before.power_throttling {
        if set_power(&h, POWER_THROTTLING_EXECUTION_SPEED, 0).is_ok() {
            changed.power_throttling = Some(prev);
        }
    }
    if changed.priority_class.is_none() && changed.power_throttling.is_none() {
        return Err("Could not raise this process.".to_string());
    }
    Ok(changed)
}

/// Restores every field that `previous` holds.
pub fn restore(pid: u32, previous: &TuningState) -> Result<(), String> {
    let h = open_rw(pid)?;
    let mut errors = Vec::new();
    if let Some(c) = previous.priority_class {
        if unsafe { SetPriorityClass(h.0, c) } == 0 {
            errors.push("priority".to_string());
        }
    }
    if let Some((c, s)) = previous.power_throttling {
        if let Err(e) = set_power(&h, c, s) {
            errors.push(e);
        }
    }
    if let Some(v) = previous.io_priority {
        if let Err(e) = set_io_priority(&h, v) {
            errors.push(e);
        }
    }
    if let Some(v) = previous.memory_priority {
        if let Err(e) = set_memory(&h, v) {
            errors.push(e);
        }
    }
    if let Some(ids) = &previous.cpu_sets {
        if let Err(e) = set_cpu_sets(&h, ids) {
            errors.push(e);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors.join("; ")) }
}

/// CPU set IDs of the least-performant core class on hybrid CPUs (E-cores).
/// None on CPUs where every core has the same efficiency class.
pub fn efficiency_core_set_ids() -> Option<Vec<u32>> {
    let mut len = 0u32;
    unsafe { GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut len, std::ptr::null_mut(), 0) };
    if len == 0 {
        return None;
    }
    let mut buf = vec![0u8; len as usize];
    let ok = unsafe { GetSystemCpuSetInformation(buf.as_mut_ptr() as *mut SYSTEM_CPU_SET_INFORMATION, len, &mut len, std::ptr::null_mut(), 0) };
    if ok == 0 {
        return None;
    }
    let mut sets = Vec::new();
    let mut off = 0usize;
    while off + 8 <= len as usize {
        let info: SYSTEM_CPU_SET_INFORMATION = unsafe { std::ptr::read_unaligned(buf.as_ptr().add(off) as *const _) };
        if info.Size == 0 {
            break;
        }
        let cs = unsafe { info.Anonymous.CpuSet };
        sets.push((cs.Id, cs.EfficiencyClass));
        off += info.Size as usize;
    }
    let min = sets.iter().map(|s| s.1).min()?;
    let max = sets.iter().map(|s| s.1).max()?;
    if min == max {
        return None;
    }
    Some(sets.into_iter().filter(|s| s.1 == min).map(|s| s.0).collect())
}
