// TENREC - Outbound bandwidth limits via Windows Policy-based QoS
//
// Policies go into the ActiveStore, which Windows clears on reboot, so a limit can never
// outlive the session even if TENREC is killed. Requires administrator rights.
// Only outbound (upload) traffic can be limited this way.
use crate::controls::powershell_json;

pub const QOS_PREFIX: &str = "TENREC_QOS_";

fn valid_exe_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !name.is_empty()
        && name.len() <= 128
        && lower.ends_with(".exe")
        && name.chars().all(|c| c.is_ascii_alphanumeric() || " ._-()".contains(c))
}

fn valid_policy_name(name: &str) -> bool {
    name.strip_prefix(QOS_PREFIX).map(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric())).unwrap_or(false)
}

pub fn new_policy_name() -> String {
    format!("{QOS_PREFIX}{}", uuid::Uuid::new_v4().simple())
}

/// Limits upload of every process with this executable file name to `kbps`.
pub fn limit_upload(policy: &str, exe_name: &str, kbps: u32) -> Result<(), String> {
    if !valid_policy_name(policy) {
        return Err("Invalid policy name.".to_string());
    }
    if !valid_exe_name(exe_name) {
        return Err(format!("'{exe_name}' cannot be used in an upload limit."));
    }
    if !(64..=1_000_000).contains(&kbps) {
        return Err("Upload limit must be between 64 kbit/s and 1 Gbit/s.".to_string());
    }
    let bits = kbps as u64 * 1000;
    // Every interpolated value was validated above: no quotes or separators can appear.
    let script = format!(
        "New-NetQosPolicy -Name '{policy}' -AppPathNameMatchCondition '{exe_name}' -ThrottleRateActionBitsPerSecond {bits} -PolicyStore ActiveStore -ErrorAction Stop | Out-Null"
    );
    powershell_json(&script).map(|_| ()).map_err(|e| {
        if e.contains("Access") || e.contains("PermissionDenied") {
            "Upload limits need administrator rights. Restart TENREC with \"Run as administrator\".".to_string()
        } else {
            e
        }
    })
}

pub fn remove(policy: &str) -> Result<(), String> {
    if !valid_policy_name(policy) {
        return Err("Refusing to remove a QoS policy TENREC did not create.".to_string());
    }
    let script = format!("Remove-NetQosPolicy -Name '{policy}' -PolicyStore ActiveStore -Confirm:$false -ErrorAction SilentlyContinue | Out-Null");
    powershell_json(&script).map(|_| ())
}

/// Exposed for integration tests.
pub fn is_valid_exe_name(name: &str) -> bool {
    valid_exe_name(name)
}

pub fn is_valid_policy_name(name: &str) -> bool {
    valid_policy_name(name)
}
