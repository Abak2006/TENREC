// TENREC - Protection Policies, Snapshots, and Rollback Engine
use crate::persistence::Database;
use crate::protection::firewall::WindowsFirewall;
use crate::protection::process_ctrl::ProcessController;
use crate::protection::types::{ActivePolicy, ApplyResult, RestorationSnapshot, RestoreResult};
use crate::security::types::ProposedAction;
use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct ProtectionManager {
    policies: Mutex<Vec<ActivePolicy>>,
    snapshots: Mutex<Vec<RestorationSnapshot>>,
    db: Arc<Database>,
}

impl ProtectionManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            policies: Mutex::new(Vec::new()),
            snapshots: Mutex::new(Vec::new()),
            db,
        }
    }

    pub fn get_active_policies(&self) -> Vec<ActivePolicy> {
        let lock = self.policies.lock().unwrap();
        lock.clone()
    }

    pub fn get_snapshots(&self) -> Vec<RestorationSnapshot> {
        let lock = self.snapshots.lock().unwrap();
        let mut list = lock.clone();
        list.reverse();
        list
    }

    pub fn apply_approved_actions(&self, actions: Vec<ProposedAction>) -> ApplyResult {
        let now = Utc::now().to_rfc3339();
        let snapshot_id = format!("snap_{}_{}", Utc::now().timestamp_millis(), uuid::Uuid::new_v4().simple());

        let mut rules_created = Vec::new();
        let mut suspended_pids = Vec::new();
        let mut summaries = Vec::new();
        let mut applied_count = 0;

        for action in actions {
            match action.target_type.as_str() {
                "firewall_rule" => {
                    let sanitized = action.target_identifier.replace(':', "_").replace('.', "_");
                    let rule_name = format!("TENREC_RULE_BLOCK_{}", sanitized);
                    
                    if let Ok(_) = WindowsFirewall::add_outbound_block_rule(&rule_name, &action.target_identifier) {
                        rules_created.push(rule_name.clone());
                        summaries.push(format!("Blocked outbound endpoint {} via Windows Firewall", action.target_identifier));
                        applied_count += 1;

                        let mut policies = self.policies.lock().unwrap();
                        policies.push(ActivePolicy {
                            id: format!("pol_{}", rule_name),
                            name: format!("Firewall Block: {}", action.target_identifier),
                            policy_type: "FIREWALL_BLOCK".to_string(),
                            application_name: action.title.clone(),
                            destination_endpoint: Some(action.target_identifier.clone()),
                            created_at: now.clone(),
                            expires_at: None,
                            reason: action.rationale.clone(),
                            status: "ACTIVE".to_string(),
                        });
                    }
                }
                "process_priority" => {
                    if let Ok(pid) = action.target_identifier.parse::<u32>() {
                        if let Ok(_) = ProcessController::lower_priority_to_idle(pid) {
                            summaries.push(format!("Set process {} (PID) priority to IDLE", pid));
                            applied_count += 1;

                            let mut policies = self.policies.lock().unwrap();
                            policies.push(ActivePolicy {
                                id: format!("pol_prio_{}", pid),
                                name: format!("Lowered Priority PID {}", pid),
                                policy_type: "PROCESS_PRIORITY".to_string(),
                                application_name: action.title.clone(),
                                destination_endpoint: None,
                                created_at: now.clone(),
                                expires_at: None,
                                reason: action.rationale.clone(),
                                status: "ACTIVE".to_string(),
                            });
                        }
                    }
                }
                "process_suspend" => {
                    if let Ok(pid) = action.target_identifier.parse::<u32>() {
                        if let Ok(_) = ProcessController::suspend_process(pid) {
                            suspended_pids.push(pid);
                            summaries.push(format!("Suspended process execution for PID {}", pid));
                            applied_count += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        let snapshot = RestorationSnapshot {
            id: snapshot_id.clone(),
            timestamp: now.clone(),
            title: format!("User Approved Protection Actions ({} items)", applied_count),
            action_summary: summaries,
            rules_created,
            previous_power_mode: None,
            suspended_pids: if suspended_pids.is_empty() { None } else { Some(suspended_pids) },
            status: "ACTIVE".to_string(),
        };

        // Record in SQLite Audit Log
        let _ = self.db.record_audit_entry(
            "USER_EXPLICIT_APPROVAL",
            "APPLY_PROTECTION",
            &snapshot_id,
            &format!("Applied {} user-approved mitigation action(s)", applied_count),
            true,
        );

        let mut snapshots = self.snapshots.lock().unwrap();
        snapshots.push(snapshot);

        ApplyResult {
            success: applied_count > 0,
            applied_count,
            snapshot_id,
        }
    }

    pub fn restore_snapshot(&self, snapshot_id: &str) -> RestoreResult {
        let mut snapshots = self.snapshots.lock().unwrap();
        let target_snap = match snapshots.iter_mut().find(|s| s.id == snapshot_id) {
            Some(s) => s,
            None => {
                return RestoreResult {
                    success: false,
                    reverted_count: 0,
                    message: "Snapshot not found".to_string(),
                }
            }
        };

        if target_snap.status == "RESTORED" {
            return RestoreResult {
                success: true,
                reverted_count: 0,
                message: "Snapshot was already restored".to_string(),
            };
        }

        let mut reverted = 0;

        // Delete firewall rules created in this snapshot
        for rule in &target_snap.rules_created {
            if let Ok(_) = WindowsFirewall::delete_rule(rule) {
                reverted += 1;
            }
        }

        // Resume any suspended processes
        if let Some(ref pids) = target_snap.suspended_pids {
            for &pid in pids {
                if let Ok(_) = ProcessController::resume_process(pid) {
                    reverted += 1;
                }
            }
        }

        target_snap.status = "RESTORED".to_string();

        // Update matching policies
        let mut policies = self.policies.lock().unwrap();
        for pol in policies.iter_mut() {
            if target_snap.rules_created.iter().any(|r| pol.id.contains(r)) {
                pol.status = "REVERTED".to_string();
            }
        }

        // Record in SQLite Audit Log
        let _ = self.db.record_audit_entry(
            "USER_RESTORE",
            "ROLLBACK_SNAPSHOT",
            snapshot_id,
            &format!("Reverted snapshot {} ({} items restored)", snapshot_id, reverted),
            true,
        );

        RestoreResult {
            success: true,
            reverted_count: reverted,
            message: format!("Successfully rolled back snapshot {}", snapshot_id),
        }
    }

    pub fn restore_all(&self) -> RestoreResult {
        let mut snapshots = self.snapshots.lock().unwrap();
        let mut total_reverted = 0;

        for snap in snapshots.iter_mut() {
            if snap.status == "ACTIVE" {
                for rule in &snap.rules_created {
                    if let Ok(_) = WindowsFirewall::delete_rule(rule) {
                        total_reverted += 1;
                    }
                }
                if let Some(ref pids) = snap.suspended_pids {
                    for &pid in pids {
                        if let Ok(_) = ProcessController::resume_process(pid) {
                            total_reverted += 1;
                        }
                    }
                }
                snap.status = "RESTORED".to_string();
            }
        }

        let mut policies = self.policies.lock().unwrap();
        for pol in policies.iter_mut() {
            pol.status = "REVERTED".to_string();
        }

        // Record in SQLite Audit Log
        let _ = self.db.record_audit_entry(
            "USER_RESTORE_ALL",
            "RESTORE_EVERYTHING",
            "GLOBAL",
            &format!("Completely restored all {} active items to baseline", total_reverted),
            true,
        );

        RestoreResult {
            success: true,
            reverted_count: total_reverted,
            message: "All active protection policies and actions reverted to default state.".to_string(),
        }
    }
}
