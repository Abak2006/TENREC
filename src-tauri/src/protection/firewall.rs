// TENREC - Scoped Windows Firewall Management
// Invariant: ALL created rules MUST have the prefix TENREC_RULE_
use std::process::Command;

pub const TENREC_RULE_PREFIX: &str = "TENREC_RULE_";

pub struct WindowsFirewall;

impl WindowsFirewall {
    /// Add an outbound block rule for a specific remote IP or IP/CIDR
    pub fn add_outbound_block_rule(rule_name: &str, remote_ip: &str) -> Result<(), String> {
        if !rule_name.starts_with(TENREC_RULE_PREFIX) {
            return Err(format!("Rule name must begin with '{}'", TENREC_RULE_PREFIX));
        }

        // netsh advfirewall firewall add rule name="<rule_name>" dir=out action=block remoteip=<ip>
        let output = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={}", rule_name),
                "dir=out",
                "action=block",
                &format!("remoteip={}", remote_ip),
            ])
            .output()
            .map_err(|e| format!("Failed to invoke netsh: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Firewall command failed: {}{}", stdout, stderr));
        }

        Ok(())
    }

    /// Add an outbound block rule for a specific program executable
    pub fn add_program_block_rule(rule_name: &str, program_path: &str) -> Result<(), String> {
        if !rule_name.starts_with(TENREC_RULE_PREFIX) {
            return Err(format!("Rule name must begin with '{}'", TENREC_RULE_PREFIX));
        }

        let output = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name={}", rule_name),
                "dir=out",
                "action=block",
                &format!("program={}", program_path),
            ])
            .output()
            .map_err(|e| format!("Failed to invoke netsh: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("Firewall command failed: {}{}", stdout, stderr));
        }

        Ok(())
    }

    /// Delete a rule by its exact name
    pub fn delete_rule(rule_name: &str) -> Result<(), String> {
        if !rule_name.starts_with(TENREC_RULE_PREFIX) {
            return Err("Safety violation: Refusing to delete non-TENREC firewall rule.".to_string());
        }

        let output = Command::new("netsh")
            .args(&[
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                &format!("name={}", rule_name),
            ])
            .output()
            .map_err(|e| format!("Failed to delete rule via netsh: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // If the rule was already absent, netsh returns "No rules match the specified criteria" which is fine
            if !stdout.contains("No rules match") && !stderr.contains("No rules match") {
                return Err(format!("Failed to delete firewall rule: {}{}", stdout, stderr));
            }
        }

        Ok(())
    }
}
