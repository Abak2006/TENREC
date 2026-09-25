// TENREC - Security Incident & Evidence Data Structures
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceItem {
    pub key: String,
    pub label: String,
    pub observed_value: String,
    pub baseline_value: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIncident {
    pub id: String,
    pub title: String,
    pub severity: IncidentSeverity,
    pub confidence_percent: u8,
    pub first_observed: String,
    pub last_observed: String,
    pub affected_application: String,
    pub affected_pid: Option<u32>,
    pub destinations: Vec<String>,
    pub summary: String,
    pub why_detected: String,
    pub baseline_comparison: String,
    pub evidence: Vec<EvidenceItem>,
    pub proposed_action_ids: Vec<String>,
    pub status: String, // "ACTIVE" | "REVIEWED" | "DISMISSED"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionRisk {
    Safe,
    Risky,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedAction {
    pub id: String,
    pub incident_id: Option<String>,
    pub title: String,
    pub description: String,
    pub risk: ActionRisk,
    pub rationale: String,
    pub expected_benefit: String,
    pub potential_consequence: String,
    pub current_state: String,
    pub proposed_state: String,
    pub is_reversible: bool,
    pub default_selected: bool,
    pub selected: bool,
    pub duration_minutes: u32,
    pub target_type: String, // "firewall_rule" | "power_mode" | "process_priority" | "process_suspend" | "network_throttle"
    pub target_identifier: String,
}
