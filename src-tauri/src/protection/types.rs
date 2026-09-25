// TENREC - Protection & Restoration Types
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePolicy {
    pub id: String,
    pub name: String,
    pub policy_type: String, // "FIREWALL_BLOCK" | "FIREWALL_RESTRICT" | "POWER_MODE" | "PROCESS_PRIORITY"
    pub application_name: String,
    pub destination_endpoint: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub reason: String,
    pub status: String, // "ACTIVE" | "EXPIRED" | "REVERTED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorationSnapshot {
    pub id: String,
    pub timestamp: String,
    pub title: String,
    pub action_summary: Vec<String>,
    pub rules_created: Vec<String>,
    pub previous_power_mode: Option<String>,
    pub suspended_pids: Option<Vec<u32>>,
    pub status: String, // "ACTIVE" | "RESTORED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
    pub success: bool,
    pub applied_count: usize,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub reverted_count: usize,
    pub message: String,
}
