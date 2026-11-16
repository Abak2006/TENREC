// TENREC - Scoped Windows Firewall management via netsh
//
// Invariants:
// - Every rule TENREC creates is named TENREC_RULE_<random id>; TENREC never edits or
//   deletes a rule without that prefix.
// - Every value placed on the netsh command line has been validated by `guard`
//   (public IP, existing absolute path without quotes, numeric port) and the rule
//   name is generated here, so no caller-controlled text reaches netsh unchecked.
use crate::protection::guard;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const TENREC_RULE_PREFIX: &str = "TENREC_RULE_";

pub fn new_rule_name() -> String {
    format!("{}{}", TENREC_RULE_PREFIX, uuid::Uuid::new_v4().simple())
}

fn run(args: &str) -> Result<String, String> {
    let out = Command::new("netsh")
        .creation_flags(CREATE_NO_WINDOW)
        // raw_arg keeps netsh's own `key="value with spaces"` quoting intact.
        .raw_arg(args)
        .output()
        .map_err(|e| format!("Could not start netsh: {e}"))?;
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)).trim().to_string();
    if out.status.success() {
        Ok(text)
    } else if text.to_ascii_lowercase().contains("elevation") || text.contains("Run as administrator") {
        Err("Windows Firewall changes need administrator rights. Restart TENREC with \"Run as administrator\" to apply this.".to_string())
    } else {
        Err(format!("netsh failed: {text}"))
    }
}

/// Accepts only names TENREC generates: the prefix followed by [A-Za-z0-9_].
pub fn check_name(rule_name: &str) -> Result<(), String> {
    let suffix = rule_name.strip_prefix(TENREC_RULE_PREFIX).ok_or("Refusing to touch a firewall rule TENREC did not create.")?;
    if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid TENREC rule name.".to_string());
    }
    Ok(())
}

pub fn block_remote_ip(rule_name: &str, ip: &str) -> Result<(), String> {
    check_name(rule_name)?;
    guard::check_remote_ip(ip)?;
    run(&format!("advfirewall firewall add rule name={rule_name} dir=out action=block remoteip={ip}")).map(|_| ())
}

pub fn block_program(rule_name: &str, path: &str) -> Result<(), String> {
    check_name(rule_name)?;
    guard::check_program_path(path)?;
    run(&format!("advfirewall firewall add rule name={rule_name} dir=out action=block program=\"{path}\"")).map(|_| ())
}

pub fn block_inbound_port(rule_name: &str, port: u16, protocol: &str) -> Result<(), String> {
    check_name(rule_name)?;
    guard::check_port(port, protocol)?;
    run(&format!("advfirewall firewall add rule name={rule_name} dir=in action=block protocol={protocol} localport={port}")).map(|_| ())
}

/// Deletes a TENREC rule. A rule that no longer exists counts as deleted.
pub fn delete_rule(rule_name: &str) -> Result<(), String> {
    check_name(rule_name)?;
    match run(&format!("advfirewall firewall delete rule name={rule_name}")) {
        Ok(_) => Ok(()),
        Err(e) if e.contains("No rules match") => Ok(()),
        Err(e) => Err(e),
    }
}

/// Removes every rule whose display name starts with TENREC_RULE_, including rules
/// left behind by older versions that did not persist their undo history.
/// Returns how many rules were removed.
pub fn remove_all_tenrec_rules() -> Result<usize, String> {
    // Fixed script, no external input. Uses the NetSecurity module because netsh
    // cannot match rules by prefix and its text output is localised.
    let script = "$r = @(Get-NetFirewallRule -DisplayName 'TENREC_RULE_*' -ErrorAction SilentlyContinue); \
                  $r | Remove-NetFirewallRule -ErrorAction Stop; $r.Count";
    let out = Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
        .map_err(|e| format!("Could not start PowerShell: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !out.status.success() || !stderr.is_empty() {
        if stderr.contains("Access is denied") || stderr.contains("PermissionDenied") {
            return Err("Removing firewall rules needs administrator rights. Restart TENREC with \"Run as administrator\".".to_string());
        }
        return Err(format!("Could not clean up TENREC firewall rules: {stderr}"));
    }
    Ok(stdout.lines().last().and_then(|l| l.trim().parse().ok()).unwrap_or(0))
}
