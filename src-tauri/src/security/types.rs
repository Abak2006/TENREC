// TENREC - Security incidents, evidence and proposed actions
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceItem {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum IncidentStatus {
    Active,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIncident {
    pub id: String,
    /// Stable identifier of the rule that produced this incident.
    pub detector: String,
    pub title: String,
    pub severity: IncidentSeverity,
    pub first_observed: String,
    pub last_observed: String,
    /// Number of samples in which the condition was observed.
    pub observation_count: u64,
    /// Whether the condition was present in the most recent sample.
    pub currently_observed: bool,
    pub affected_application: String,
    pub affected_pid: Option<u32>,
    pub summary: String,
    /// Plain explanation of the rule, including its known false positives.
    pub why_detected: String,
    pub evidence: Vec<EvidenceItem>,
    pub proposed_action_ids: Vec<String>,
    pub status: IncidentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActionRisk {
    Safe,
    Risky,
}

/// What an action does. Built only by the backend from observed state, never
/// accepted from the UI, and re-validated immediately before it is applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ActionTarget {
    /// Outbound block to one public IP address.
    BlockRemoteIp { ip: String },
    /// Outbound block for every connection made by one executable.
    BlockProgram { path: String },
    /// Inbound block on a local port.
    BlockInboundPort { port: u16, protocol: String },
    /// Set priority class to IDLE. `creation_time` pins the exact process instance.
    LowerPriority { pid: u32, process_name: String, creation_time: u64 },
    /// Suspend every thread. `creation_time` pins the exact process instance.
    SuspendProcess { pid: u32, process_name: String, creation_time: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedAction {
    pub id: String,
    pub incident_id: Option<String>,
    pub title: String,
    pub description: String,
    pub risk: ActionRisk,
    pub expected_benefit: String,
    pub potential_consequence: String,
    pub requires_admin: bool,
    pub default_selected: bool,
    pub target: ActionTarget,
}
