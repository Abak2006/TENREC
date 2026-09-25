// TENREC - Tauri Native IPC Commands
use crate::correlation::WhatChangedItem;
use crate::ipc::state::AppState;
use crate::network::types::{ConnectionInfo, ListeningPortInfo, NetworkThroughput, TopDestination};
use crate::persistence::{AuditEntry, DatabaseStats};
use crate::processes::types::{ApplicationItem, ProcessItem};
use crate::protection::{ActivePolicy, ApplyResult, RestorationSnapshot, RestoreResult};
use crate::security::{ProposedAction, SecurityIncident};
use crate::system::SystemSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareCapability {
    pub name: String,
    pub available: bool,
    pub details: String,
    pub category: String, // "system" | "vendor_extension"
}

#[tauri::command]
pub fn get_system_snapshot(state: State<'_, AppState>) -> Result<SystemSnapshot, String> {
    let sys = state.system.lock().unwrap();
    let snapshot = sys.snapshot();

    // Persist periodically to SQLite
    let _ = state.db.insert_system_sample(
        &snapshot.timestamp,
        snapshot.cpu.usage_percent as f32,
        snapshot.memory.used_bytes,
        snapshot.memory.total_bytes,
        snapshot.disk.read_bytes_sec as f64,
        snapshot.disk.write_bytes_sec as f64,
        0.0,
        0.0,
        snapshot.power.battery_percent.map(|p| p as f32),
        snapshot.power.ac_connected,
    );

    Ok(snapshot)
}

#[tauri::command]
pub fn get_network_throughput(state: State<'_, AppState>) -> Result<NetworkThroughput, String> {
    let net = state.network.lock().unwrap();
    Ok(net.get_throughput())
}

#[tauri::command]
pub fn get_active_connections(state: State<'_, AppState>) -> Result<Vec<ConnectionInfo>, String> {
    let net = state.network.lock().unwrap();
    Ok(net.get_connections())
}

#[tauri::command]
pub fn get_listening_ports(state: State<'_, AppState>) -> Result<Vec<ListeningPortInfo>, String> {
    let net = state.network.lock().unwrap();
    Ok(net.get_listening_ports())
}

#[tauri::command]
pub fn get_top_destinations(state: State<'_, AppState>) -> Result<Vec<TopDestination>, String> {
    let net = state.network.lock().unwrap();
    Ok(net.get_top_destinations())
}

#[tauri::command]
pub fn get_processes(state: State<'_, AppState>) -> Result<Vec<ProcessItem>, String> {
    let net_mgr = state.network.lock().unwrap();
    let conns = net_mgr.get_connections();
    let listening = net_mgr.get_listening_ports();
    drop(net_mgr);

    let mut socket_counts = HashMap::new();
    for c in &conns {
        *socket_counts.entry(c.pid).or_insert(0) += 1;
    }

    let mut proc_mgr = state.processes.lock().unwrap();
    let mut procs = proc_mgr.get_processes(&socket_counts);
    let mut apps = proc_mgr.get_applications(&procs);
    drop(proc_mgr);

    let mut conns_clone = conns.clone();
    let _ = state.correlation.correlate(&mut procs, &mut apps, &mut conns_clone, &listening);

    // Update baselines
    for p in procs.iter() {
        state.baseline.record_sample(&p.application_name, p.cpu_percent as f32, p.memory_bytes, p.net_upload_bps as f64);
    }

    // Run security detection
    state.security.analyze(&procs, &conns_clone, &listening, &state.baseline);

    Ok(procs)
}

#[tauri::command]
pub fn get_applications(state: State<'_, AppState>) -> Result<Vec<ApplicationItem>, String> {
    let net_mgr = state.network.lock().unwrap();
    let conns = net_mgr.get_connections();
    drop(net_mgr);

    let mut socket_counts = HashMap::new();
    for c in &conns {
        *socket_counts.entry(c.pid).or_insert(0) += 1;
    }

    let mut proc_mgr = state.processes.lock().unwrap();
    let procs = proc_mgr.get_processes(&socket_counts);
    let mut apps = proc_mgr.get_applications(&procs);
    drop(proc_mgr);

    // Populate baseline status
    for app in apps.iter_mut() {
        app.baseline_status = state.baseline.get_status(&app.name);
    }

    Ok(apps)
}

#[tauri::command]
pub fn get_security_incidents(state: State<'_, AppState>) -> Result<Vec<SecurityIncident>, String> {
    Ok(state.security.get_incidents())
}

#[tauri::command]
pub fn get_what_changed(state: State<'_, AppState>) -> Result<Vec<WhatChangedItem>, String> {
    Ok(state.correlation.get_what_changed())
}

#[tauri::command]
pub fn get_proposed_actions(state: State<'_, AppState>, incident_id: Option<String>) -> Result<Vec<ProposedAction>, String> {
    Ok(state.security.get_proposed_actions(incident_id.as_deref()))
}

#[tauri::command]
pub fn get_active_policies(state: State<'_, AppState>) -> Result<Vec<ActivePolicy>, String> {
    Ok(state.protection.get_active_policies())
}

#[tauri::command]
pub fn get_restoration_snapshots(state: State<'_, AppState>) -> Result<Vec<RestorationSnapshot>, String> {
    Ok(state.protection.get_snapshots())
}

#[tauri::command]
pub fn get_hardware_capabilities() -> Result<Vec<HardwareCapability>, String> {
    Ok(vec![
        HardwareCapability {
            name: "Kernel ETW Telemetry".to_string(),
            available: true,
            details: "Native Windows Event Tracing for Windows kernel provider interface active".to_string(),
            category: "system".to_string(),
        },
        HardwareCapability {
            name: "Extended MIB Socket Table".to_string(),
            available: true,
            details: "GetExtendedTcpTable and GetExtendedUdpTable with strict PID ownership resolution".to_string(),
            category: "system".to_string(),
        },
        HardwareCapability {
            name: "Storage Volume Query".to_string(),
            available: true,
            details: "GetDiskFreeSpaceExW direct cluster geometry and real-time read/write counters".to_string(),
            category: "system".to_string(),
        },
        HardwareCapability {
            name: "Windows Power Subsystem".to_string(),
            available: true,
            details: "GetSystemPowerStatus real-time AC line and battery capacity meter".to_string(),
            category: "system".to_string(),
        },
        HardwareCapability {
            name: "Process Priority & Suspend API".to_string(),
            available: true,
            details: "Direct SetPriorityClass and ntdll NtSuspendProcess thread freeze controls".to_string(),
            category: "system".to_string(),
        },
        HardwareCapability {
            name: "Discrete GPU Hardware Sensors".to_string(),
            available: false,
            details: "Hardware NVML/AMD GPU vendor extension sensors unavailable in non-elevated user session".to_string(),
            category: "vendor_extension".to_string(),
        },
    ])
}

#[tauri::command]
pub fn apply_approved_actions(state: State<'_, AppState>, actions: Vec<ProposedAction>) -> Result<ApplyResult, String> {
    Ok(state.protection.apply_approved_actions(actions))
}

#[tauri::command]
pub fn restore_snapshot(state: State<'_, AppState>, snapshot_id: String) -> Result<RestoreResult, String> {
    Ok(state.protection.restore_snapshot(&snapshot_id))
}

#[tauri::command]
pub fn restore_all(state: State<'_, AppState>) -> Result<RestoreResult, String> {
    Ok(state.protection.restore_all())
}

#[tauri::command]
pub fn get_audit_log(state: State<'_, AppState>) -> Result<Vec<AuditEntry>, String> {
    state.db.get_audit_log(50).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    state.db.get_database_stats().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn purge_old_telemetry(state: State<'_, AppState>, retention_hours: u32) -> Result<usize, String> {
    state.db.purge_old_telemetry(retention_hours).map_err(|e| e.to_string())
}
