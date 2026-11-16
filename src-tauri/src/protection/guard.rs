// TENREC - Safety checks applied before any action is proposed or executed
use crate::util::is_public_ip_str;
use std::path::Path;

/// Processes whose suspension or deprioritisation can hang or crash Windows,
/// break the desktop, or disable security software.
const PROTECTED_PROCESSES: &[&str] = &[
    "system", "registry", "idle", "smss.exe", "csrss.exe", "wininit.exe", "winlogon.exe",
    "services.exe", "lsass.exe", "lsaiso.exe", "svchost.exe", "dwm.exe", "explorer.exe",
    "fontdrvhost.exe", "memory compression", "secure system", "msmpeng.exe", "nissrv.exe",
    "securityhealthservice.exe", "audiodg.exe", "conhost.exe", "sihost.exe", "ctfmon.exe",
    "taskhostw.exe", "runtimebroker.exe", "searchhost.exe", "startmenuexperiencehost.exe",
    "shellexperiencehost.exe", "tenrec.exe",
];

/// Executables whose outbound traffic Windows itself depends on.
const PROTECTED_PROGRAMS: &[&str] = &[
    "svchost.exe", "lsass.exe", "services.exe", "system", "msmpeng.exe", "tenrec.exe",
];

pub fn check_process(pid: u32, name: &str) -> Result<(), String> {
    if pid <= 4 || pid == std::process::id() {
        return Err(format!("{name} (PID {pid}) is a core system process and cannot be controlled."));
    }
    let lower = name.to_ascii_lowercase();
    if PROTECTED_PROCESSES.contains(&lower.as_str()) {
        return Err(format!("{name} is a protected Windows process. Suspending or deprioritising it can freeze or crash Windows."));
    }
    Ok(())
}

pub fn check_program_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Windows did not report an executable path for this application, so it cannot be blocked by program.".to_string());
    }
    let p = Path::new(path);
    if !p.is_absolute() || !p.is_file() {
        return Err(format!("{path} is not an existing executable file."));
    }
    // netsh arguments are quoted; a quote in the path would break that quoting.
    if path.contains('"') {
        return Err("Executable path contains an invalid character.".to_string());
    }
    let file = p.file_name().and_then(|f| f.to_str()).unwrap_or("").to_ascii_lowercase();
    if PROTECTED_PROGRAMS.contains(&file.as_str()) {
        return Err(format!("Blocking {file} would break core Windows networking (DNS, updates, sign-in)."));
    }
    Ok(())
}

pub fn check_remote_ip(ip: &str) -> Result<(), String> {
    if !is_public_ip_str(ip) {
        return Err(format!("{ip} is not a public IP address. TENREC only blocks public destinations, so it cannot cut off your router, DNS server or local network by mistake."));
    }
    Ok(())
}

pub fn check_port(port: u16, protocol: &str) -> Result<(), String> {
    if port == 0 {
        return Err("Port 0 is not a valid port.".to_string());
    }
    if protocol != "TCP" && protocol != "UDP" {
        return Err(format!("Unsupported protocol '{protocol}'."));
    }
    Ok(())
}
