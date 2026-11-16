// TENREC - Native desktop application core
pub mod baseline;
pub mod correlation;
pub mod focus;
pub mod hardware;
pub mod ipc;
pub mod network;
pub mod persistence;
pub mod platform;
pub mod processes;
pub mod protection;
pub mod security;
pub mod system;
pub mod telemetry;
pub mod util;

use ipc::commands::*;
use ipc::AppState;
use persistence::Database;
use std::path::Path;
use std::sync::Arc;
use tauri::Manager;

fn open_database() -> (Arc<Database>, Option<String>) {
    match Database::init_default() {
        Ok(db) => (Arc::new(db), None),
        Err(e) => {
            log::error!("Could not open the TENREC database: {e}");
            let db = Database::open(Path::new(":memory:")).expect("in-memory SQLite must open");
            (
                Arc::new(db),
                Some(format!("Could not open the TENREC database ({e}). History and undo records will not be kept after you close TENREC.")),
            )
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (db, storage_error) = open_database();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(if cfg!(debug_assertions) { log::LevelFilter::Info } else { log::LevelFilter::Warn })
                .build(),
        )
        .manage(AppState::new(db, storage_error))
        .setup(|app| {
            let state = app.state::<AppState>();
            let recovered = state.protection.recover_interrupted_focus();
            if recovered > 0 {
                log::warn!("Undid {recovered} focus session(s) that were still active at the last exit");
            }
            let (focus, protection) = (state.focus.clone(), state.protection.clone());
            state.hub.start(state.db.clone(), move |sample| focus.on_sample(sample, &protection));
            log::info!("TENREC started (elevated: {})", protection::process_ctrl::is_elevated());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_system_snapshot,
            get_network_throughput,
            get_active_connections,
            get_listening_ports,
            get_top_destinations,
            get_processes,
            get_applications,
            get_security_incidents,
            set_incident_status,
            get_what_changed,
            get_proposed_actions,
            prepare_process_action,
            prepare_app_block,
            apply_actions,
            get_active_policies,
            get_restoration_snapshots,
            restore_snapshot,
            restore_all,
            get_hardware_diagnostics,
            get_hardware_registry,
            get_audit_log,
            get_database_stats,
            purge_old_telemetry,
            get_focus_status,
            preview_focus,
            start_focus,
            stop_focus,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TENREC");
}
