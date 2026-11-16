// TENREC - Tauri IPC commands
//
// Read commands return the latest background sample. Commands that change the system
// accept only action IDs created by the backend, never raw targets from the webview.
use crate::correlation::WhatChangedItem;
use crate::hardware::{HardwareCapabilityItem, HardwareDiagnostics, HardwareRegistry};
use crate::ipc::state::AppState;
use crate::network::{ConnectionInfo, ListeningPortInfo, NetworkThroughput, TopDestination};
use crate::persistence::{AuditEntry, DatabaseStats};
use crate::processes::{ApplicationItem, ProcessItem};
use crate::protection::process_ctrl::is_elevated;
use crate::protection::{ActionOutcome, ActivePolicy, ApplyResult, RestorationSnapshot, RestoreResult};
use crate::security::{IncidentStatus, ProposedAction, SecurityIncident};
use crate::system::SystemSnapshot;
use crate::telemetry::SAMPLE_INTERVAL;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub version: String,
    pub elevated: bool,
    pub sample_sequence: u64,
    pub sample_interval_ms: u64,
    pub last_error: Option<String>,
    pub storage_error: Option<String>,
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> AppStatus {
    let s = state.hub.latest();
    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        elevated: is_elevated(),
        sample_sequence: s.sequence,
        sample_interval_ms: SAMPLE_INTERVAL.as_millis() as u64,
        last_error: s.last_error.clone(),
        storage_error: state.storage_error.clone(),
    }
}

#[tauri::command]
pub fn get_system_snapshot(state: State<'_, AppState>) -> Option<SystemSnapshot> {
    state.hub.latest().system.clone()
}

#[tauri::command]
pub fn get_network_throughput(state: State<'_, AppState>) -> Option<NetworkThroughput> {
    state.hub.latest().throughput.clone()
}

#[tauri::command]
pub fn get_active_connections(state: State<'_, AppState>) -> Vec<ConnectionInfo> {
    state.hub.latest().connections.clone()
}

#[tauri::command]
pub fn get_listening_ports(state: State<'_, AppState>) -> Vec<ListeningPortInfo> {
    state.hub.latest().listening.clone()
}

#[tauri::command]
pub fn get_top_destinations(state: State<'_, AppState>) -> Vec<TopDestination> {
    state.hub.latest().destinations.clone()
}

#[tauri::command]
pub fn get_processes(state: State<'_, AppState>) -> Vec<ProcessItem> {
    state.hub.latest().processes.clone()
}

#[tauri::command]
pub fn get_applications(state: State<'_, AppState>) -> Vec<ApplicationItem> {
    state.hub.latest().applications.clone()
}

#[tauri::command]
pub fn get_security_incidents(state: State<'_, AppState>) -> Vec<SecurityIncident> {
    state.hub.security.get_incidents()
}

#[tauri::command]
pub fn set_incident_status(state: State<'_, AppState>, incident_id: String, status: IncidentStatus) -> Result<(), String> {
    if state.hub.security.set_incident_status(&incident_id, status) {
        Ok(())
    } else {
        Err("Incident not found.".to_string())
    }
}

#[tauri::command]
pub fn get_what_changed(state: State<'_, AppState>) -> Vec<WhatChangedItem> {
    state.hub.correlation.get_what_changed()
}

#[tauri::command]
pub fn get_proposed_actions(state: State<'_, AppState>, incident_id: Option<String>) -> Vec<ProposedAction> {
    state.hub.security.get_proposed_actions(incident_id.as_deref())
}

#[tauri::command(async)]
pub fn prepare_process_action(state: State<'_, AppState>, pid: u32, kind: String) -> Result<ProposedAction, String> {
    let sample = state.hub.latest();
    state.hub.security.prepare_process_action(&sample.processes, pid, &kind)
}

#[tauri::command(async)]
pub fn prepare_app_block(state: State<'_, AppState>, app_id: String) -> Result<ProposedAction, String> {
    let sample = state.hub.latest();
    let app = sample
        .applications
        .iter()
        .find(|a| a.id == app_id)
        .ok_or("That application is no longer running.")?;
    state.hub.security.prepare_program_block(app)
}

#[tauri::command(async)]
pub fn apply_actions(state: State<'_, AppState>, action_ids: Vec<String>) -> Result<ApplyResult, String> {
    if action_ids.is_empty() {
        return Err("No actions selected.".to_string());
    }
    if action_ids.len() > 50 {
        return Err("Too many actions in one request.".to_string());
    }
    let mut actions = Vec::new();
    let mut missing = Vec::new();
    for id in action_ids {
        match state.hub.security.get_action(&id) {
            Some(a) => actions.push(a),
            None => missing.push(ActionOutcome {
                action_id: id,
                title: "Unknown action".to_string(),
                success: false,
                error: Some("This action has expired. Close the dialog and try again.".to_string()),
            }),
        }
    }
    let mut result = state.protection.apply(actions);
    result.outcomes.extend(missing);
    Ok(result)
}

#[tauri::command]
pub fn get_active_policies(state: State<'_, AppState>) -> Vec<ActivePolicy> {
    state.protection.get_active_policies()
}

#[tauri::command]
pub fn get_restoration_snapshots(state: State<'_, AppState>) -> Vec<RestorationSnapshot> {
    state.protection.get_snapshots()
}

#[tauri::command(async)]
pub fn restore_snapshot(state: State<'_, AppState>, snapshot_id: String) -> Result<RestoreResult, String> {
    state.protection.restore_snapshot(&snapshot_id)
}

#[tauri::command(async)]
pub fn restore_all(state: State<'_, AppState>) -> RestoreResult {
    state.protection.restore_all()
}

static DIAGNOSTICS: OnceLock<HardwareDiagnostics> = OnceLock::new();

fn diagnostics() -> &'static HardwareDiagnostics {
    DIAGNOSTICS.get_or_init(HardwareRegistry::get_diagnostics)
}

#[tauri::command(async)]
pub fn get_hardware_diagnostics() -> HardwareDiagnostics {
    diagnostics().clone()
}

#[tauri::command(async)]
pub fn get_hardware_registry() -> Vec<HardwareCapabilityItem> {
    HardwareRegistry::get_capabilities(diagnostics())
}

#[tauri::command]
pub fn get_audit_log(state: State<'_, AppState>) -> Result<Vec<AuditEntry>, String> {
    state.db.get_audit_log(100).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    state.db.get_database_stats().map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn purge_old_telemetry(state: State<'_, AppState>, retention_hours: u32) -> Result<usize, String> {
    if retention_hours == 0 || retention_hours > 24 * 365 {
        return Err("Retention must be between 1 hour and 1 year.".to_string());
    }
    state.db.purge_old_telemetry(retention_hours).map_err(|e| e.to_string())
}
