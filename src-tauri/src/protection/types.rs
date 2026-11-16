// TENREC - Protection & restoration types
use serde::{Deserialize, Serialize};

/// A change TENREC made to the system, with everything needed to undo it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum AppliedChange {
    FirewallRule { rule_name: String },
    Priority { pid: u32, process_name: String, creation_time: u64, previous_class: u32 },
    Suspended { pid: u32, process_name: String, creation_time: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeStatus {
    Active,
    Restored,
    RestoreFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRecord {
    pub change: AppliedChange,
    pub description: String,
    pub status: ChangeStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SnapshotStatus {
    Active,
    Restored,
    PartiallyRestored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorationSnapshot {
    pub id: String,
    pub timestamp: String,
    pub title: String,
    pub changes: Vec<ChangeRecord>,
    pub status: SnapshotStatus,
}

/// A change that is currently in effect (derived from active snapshot records).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePolicy {
    pub id: String,
    pub snapshot_id: String,
    pub kind: String, // "firewall_rule" | "priority" | "suspended"
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionOutcome {
    pub action_id: String,
    pub title: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    /// Present when at least one action succeeded.
    pub snapshot_id: Option<String>,
    pub outcomes: Vec<ActionOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored: usize,
    pub failed: usize,
    pub messages: Vec<String>,
}
