// TENREC - Native Desktop Application Core Entry
pub mod baseline;
pub mod correlation;
pub mod ipc;
pub mod network;
pub mod persistence;
pub mod processes;
pub mod protection;
pub mod security;
pub mod system;

use ipc::commands::*;
use ipc::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            log::info!("TENREC native desktop shell initialized successfully.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_snapshot,
            get_network_throughput,
            get_active_connections,
            get_listening_ports,
            get_top_destinations,
            get_processes,
            get_applications,
            get_security_incidents,
            get_what_changed,
            get_proposed_actions,
            get_active_policies,
            get_restoration_snapshots,
            get_hardware_capabilities,
            apply_approved_actions,
            restore_snapshot,
            restore_all,
            get_audit_log,
            get_database_stats,
            purge_old_telemetry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TENREC tauri application");
}
