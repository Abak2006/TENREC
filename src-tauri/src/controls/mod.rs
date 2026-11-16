// TENREC - Windows controls (startup, tasks, privacy, power, cleanup)
pub mod cleanup;
pub mod privacy;
pub mod startup;
pub mod tasks;

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Runs a fixed PowerShell script (never built from external input) and parses its
/// JSON output. Empty output is returned as JSON null.
pub fn powershell_json(script: &str) -> Result<serde_json::Value, String> {
    let out = Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
        .map_err(|e| format!("Could not start PowerShell: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let text = stdout.trim();
    if !out.status.success() {
        return Err(format!("PowerShell failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    if text.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(text).map_err(|e| format!("Could not read PowerShell output: {e}"))
}
