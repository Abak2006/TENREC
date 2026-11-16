// TENREC - Apply approved actions, record undo snapshots, roll back
use crate::persistence::Database;
use crate::protection::types::*;
use crate::protection::{firewall, guard, process_ctrl};
use crate::security::types::{ActionTarget, ProposedAction};
use crate::util::LockExt;
use chrono::Utc;
use std::sync::{Arc, Mutex};

pub struct ProtectionManager {
    snapshots: Mutex<Vec<RestorationSnapshot>>,
    db: Arc<Database>,
}

fn describe(target: &ActionTarget) -> String {
    match target {
        ActionTarget::BlockRemoteIp { ip } => format!("Outbound traffic to {ip} blocked"),
        ActionTarget::BlockProgram { path } => format!("Internet access blocked for {path}"),
        ActionTarget::BlockInboundPort { port, protocol } => format!("Inbound {protocol} port {port} blocked"),
        ActionTarget::LowerPriority { pid, process_name, .. } => format!("{process_name} (PID {pid}) set to Idle priority"),
        ActionTarget::SuspendProcess { pid, process_name, .. } => format!("{process_name} (PID {pid}) suspended"),
    }
}

/// Re-validates and executes one action. Returns the change needed to undo it.
fn execute(target: &ActionTarget) -> Result<AppliedChange, String> {
    match target {
        ActionTarget::BlockRemoteIp { ip } => {
            let rule_name = firewall::new_rule_name();
            firewall::block_remote_ip(&rule_name, ip)?;
            Ok(AppliedChange::FirewallRule { rule_name })
        }
        ActionTarget::BlockProgram { path } => {
            let rule_name = firewall::new_rule_name();
            firewall::block_program(&rule_name, path)?;
            Ok(AppliedChange::FirewallRule { rule_name })
        }
        ActionTarget::BlockInboundPort { port, protocol } => {
            let rule_name = firewall::new_rule_name();
            firewall::block_inbound_port(&rule_name, *port, protocol)?;
            Ok(AppliedChange::FirewallRule { rule_name })
        }
        ActionTarget::LowerPriority { pid, process_name, creation_time } => {
            guard::check_process(*pid, process_name)?;
            process_ctrl::verify_identity(*pid, *creation_time, process_name)?;
            let previous_class = process_ctrl::get_priority_class(*pid)?;
            process_ctrl::set_priority_class(*pid, process_ctrl::IDLE)?;
            Ok(AppliedChange::Priority { pid: *pid, process_name: process_name.clone(), creation_time: *creation_time, previous_class })
        }
        ActionTarget::SuspendProcess { pid, process_name, creation_time } => {
            guard::check_process(*pid, process_name)?;
            process_ctrl::verify_identity(*pid, *creation_time, process_name)?;
            process_ctrl::suspend_process(*pid)?;
            Ok(AppliedChange::Suspended { pid: *pid, process_name: process_name.clone(), creation_time: *creation_time })
        }
    }
}

/// Undoes one change. Ok(Some(note)) means there was nothing left to undo.
fn undo(change: &AppliedChange) -> Result<Option<String>, String> {
    match change {
        AppliedChange::FirewallRule { rule_name } => firewall::delete_rule(rule_name).map(|_| None),
        AppliedChange::Priority { pid, process_name, creation_time, previous_class } => {
            if process_ctrl::verify_identity(*pid, *creation_time, process_name).is_err() {
                return Ok(Some(format!("{process_name} has exited; nothing to restore.")));
            }
            process_ctrl::set_priority_class(*pid, *previous_class).map(|_| None)
        }
        AppliedChange::Suspended { pid, process_name, creation_time } => {
            if process_ctrl::verify_identity(*pid, *creation_time, process_name).is_err() {
                return Ok(Some(format!("{process_name} has exited; nothing to resume.")));
            }
            process_ctrl::resume_process(*pid).map(|_| None)
        }
    }
}

fn change_kind(c: &AppliedChange) -> &'static str {
    match c {
        AppliedChange::FirewallRule { .. } => "firewall_rule",
        AppliedChange::Priority { .. } => "priority",
        AppliedChange::Suspended { .. } => "suspended",
    }
}

impl ProtectionManager {
    pub fn new(db: Arc<Database>) -> Self {
        let snapshots = match db.load_snapshots() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Could not load restoration snapshots: {e}");
                Vec::new()
            }
        };
        Self { snapshots: Mutex::new(snapshots), db }
    }

    fn persist(&self, snap: &RestorationSnapshot) {
        if let Err(e) = self.db.save_snapshot(snap) {
            log::error!("Could not persist snapshot {}: {e}", snap.id);
        }
    }

    pub fn get_snapshots(&self) -> Vec<RestorationSnapshot> {
        let mut list = self.snapshots.lock_safe().clone();
        list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        list
    }

    pub fn get_active_policies(&self) -> Vec<ActivePolicy> {
        let snaps = self.snapshots.lock_safe();
        let mut out = Vec::new();
        for s in snaps.iter() {
            for (i, c) in s.changes.iter().enumerate() {
                if c.status != ChangeStatus::Restored {
                    out.push(ActivePolicy {
                        id: format!("{}:{i}", s.id),
                        snapshot_id: s.id.clone(),
                        kind: change_kind(&c.change).to_string(),
                        description: c.description.clone(),
                        created_at: s.timestamp.clone(),
                    });
                }
            }
        }
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        out
    }

    pub fn apply(&self, actions: Vec<ProposedAction>) -> ApplyResult {
        let mut outcomes = Vec::new();
        let mut changes = Vec::new();

        for a in actions {
            let result = execute(&a.target);
            let _ = self.db.record_audit_entry(
                "user",
                "apply_action",
                &describe(&a.target),
                result.as_ref().err().map(String::as_str).unwrap_or("applied"),
                result.is_ok(),
            );
            match result {
                Ok(change) => {
                    changes.push(ChangeRecord { change, description: describe(&a.target), status: ChangeStatus::Active, note: None });
                    outcomes.push(ActionOutcome { action_id: a.id, title: a.title, success: true, error: None });
                }
                Err(e) => outcomes.push(ActionOutcome { action_id: a.id, title: a.title, success: false, error: Some(e) }),
            }
        }

        if changes.is_empty() {
            return ApplyResult { snapshot_id: None, outcomes };
        }
        let snap = RestorationSnapshot {
            id: format!("snap_{}", uuid::Uuid::new_v4().simple()),
            timestamp: Utc::now().to_rfc3339(),
            title: format!("{} change(s) applied", changes.len()),
            changes,
            status: SnapshotStatus::Active,
        };
        self.persist(&snap);
        let id = snap.id.clone();
        self.snapshots.lock_safe().push(snap);
        ApplyResult { snapshot_id: Some(id), outcomes }
    }

    fn restore_one(&self, snap: &mut RestorationSnapshot, result: &mut RestoreResult) {
        for c in snap.changes.iter_mut().filter(|c| c.status != ChangeStatus::Restored) {
            let res = undo(&c.change);
            let _ = self.db.record_audit_entry(
                "user",
                "undo_change",
                &c.description,
                res.as_ref().err().map(String::as_str).unwrap_or("restored"),
                res.is_ok(),
            );
            match res {
                Ok(note) => {
                    c.status = ChangeStatus::Restored;
                    c.note = note.clone();
                    result.restored += 1;
                    if let Some(n) = note {
                        result.messages.push(n);
                    }
                }
                Err(e) => {
                    c.status = ChangeStatus::RestoreFailed;
                    c.note = Some(e.clone());
                    result.failed += 1;
                    result.messages.push(format!("{}: {e}", c.description));
                }
            }
        }
        snap.status = if snap.changes.iter().all(|c| c.status == ChangeStatus::Restored) {
            SnapshotStatus::Restored
        } else {
            SnapshotStatus::PartiallyRestored
        };
        self.persist(snap);
    }

    pub fn restore_snapshot(&self, snapshot_id: &str) -> Result<RestoreResult, String> {
        let mut snaps = self.snapshots.lock_safe();
        let snap = snaps.iter_mut().find(|s| s.id == snapshot_id).ok_or("Snapshot not found.")?;
        let mut result = RestoreResult { restored: 0, failed: 0, messages: Vec::new() };
        self.restore_one(snap, &mut result);
        Ok(result)
    }

    /// Undoes every change TENREC has recorded, then (when elevated) removes any
    /// TENREC_RULE_* firewall rules that have no record, e.g. from older versions.
    pub fn restore_all(&self) -> RestoreResult {
        let mut result = RestoreResult { restored: 0, failed: 0, messages: Vec::new() };
        {
            let mut snaps = self.snapshots.lock_safe();
            for snap in snaps.iter_mut().filter(|s| s.status != SnapshotStatus::Restored) {
                self.restore_one(snap, &mut result);
            }
        }
        if process_ctrl::is_elevated() {
            match firewall::remove_all_tenrec_rules() {
                Ok(0) => {}
                Ok(n) => {
                    result.restored += n;
                    result.messages.push(format!("Removed {n} leftover TENREC firewall rule(s)."));
                }
                Err(e) => {
                    result.failed += 1;
                    result.messages.push(e);
                }
            }
        } else {
            result.messages.push("Not running as administrator, so leftover TENREC firewall rules from older versions were not checked.".to_string());
        }
        let _ = self.db.record_audit_entry("user", "restore_all", "all", &format!("{} restored, {} failed", result.restored, result.failed), result.failed == 0);
        result
    }
}
