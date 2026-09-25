// TENREC - Comprehensive Native Subsystem Tests
use tenrec_lib::baseline::BaselineEngine;
use tenrec_lib::correlation::CorrelationEngine;
use tenrec_lib::network::types::{ConnectionInfo, ListeningPortInfo};
use tenrec_lib::persistence::Database;
use tenrec_lib::processes::types::ProcessItem;
use tenrec_lib::protection::firewall::TENREC_RULE_PREFIX;
use tenrec_lib::protection::ProtectionManager;
use tenrec_lib::security::types::{ActionRisk, IncidentSeverity, ProposedAction};
use tenrec_lib::security::SecurityEngine;
use tenrec_lib::system::SystemManager;
use std::sync::Arc;

#[test]
fn test_system_snapshot_real_telemetry() {
    let sys = SystemManager::new();
    let snapshot = sys.snapshot();

    // Invariant: telemetry must reflect real host values
    assert!(!snapshot.hostname.is_empty(), "Hostname must not be empty");
    assert!(!snapshot.os_name.is_empty(), "OS name must not be empty");
    assert!(snapshot.cpu.core_count > 0, "Host must have at least 1 core");
    assert!(snapshot.cpu.logical_core_count >= snapshot.cpu.core_count);
    assert!(snapshot.memory.total_bytes > 0, "RAM total bytes must be positive");
    assert!(snapshot.memory.used_bytes > 0, "RAM used bytes must be positive");
    assert!(snapshot.memory.usage_percent >= 0.0 && snapshot.memory.usage_percent <= 100.0);
    assert!(!snapshot.disk.drives.is_empty(), "At least one drive volume must be mounted");
}

#[test]
fn test_baseline_engine_learning_and_divergence() {
    let baseline = BaselineEngine::new();
    let app = "chrome.exe";

    // First sample -> status should be 'learning'
    baseline.record_sample(app, 5.0, 500_000_000, 10_000.0);
    assert_eq!(baseline.get_status(app), "learning");

    // Add 10 samples to exit learning
    for _ in 0..10 {
        baseline.record_sample(app, 5.0, 500_000_000, 10_000.0);
    }
    assert_eq!(baseline.get_status(app), "normal");

    // Extreme divergence (CPU 95%, Network 100 MB/s)
    baseline.record_sample(app, 95.0, 500_000_000, 100_000_000.0);
    assert_eq!(baseline.get_status(app), "divergent");
}

#[test]
fn test_security_engine_and_recommendation_classification() {
    let sec = SecurityEngine::new();
    let baseline = BaselineEngine::new();

    let procs = vec![
        ProcessItem {
            pid: 1234,
            ppid: Some(4),
            name: "powershell.exe".to_string(),
            executable_path: "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".to_string(),
            publisher: "Microsoft Corporation".to_string(),
            cpu_percent: 12.5,
            memory_bytes: 45_000_000,
            memory_percent: 0.5,
            net_upload_bps: 20_000,
            net_download_bps: 10_000,
            connection_count: 1,
            start_time: "2026-09-26T04:00:00Z".to_string(),
            is_elevated: false,
            application_name: "Windows PowerShell".to_string(),
        },
    ];

    let conns = vec![
        ConnectionInfo {
            id: "conn_1".to_string(),
            pid: 1234,
            process_name: "powershell.exe".to_string(),
            application_name: "Windows PowerShell".to_string(),
            protocol: "TCP".to_string(),
            local_address: "192.168.1.100".to_string(),
            local_port: 54321,
            remote_address: "198.51.100.42".to_string(), // Public IP
            remote_port: 443,
            state: "ESTABLISHED".to_string(),
            hostname: "198.51.100.42".to_string(),
            bytes_sent: 5000,
            bytes_received: 20000,
            first_seen: "2026-09-26T04:00:00Z".to_string(),
            last_seen: "2026-09-26T04:00:05Z".to_string(),
            is_unusual: true,
            is_new: true,
        },
    ];

    let listening: Vec<ListeningPortInfo> = vec![];

    sec.analyze(&procs, &conns, &listening, &baseline);

    let incidents = sec.get_incidents();
    assert_eq!(incidents.len(), 1, "Powershell outbound to public IP must trigger an incident");
    assert_eq!(incidents[0].severity, IncidentSeverity::High);
    assert!(!incidents[0].evidence.is_empty(), "Evidence list must be populated");

    let actions = sec.get_proposed_actions(Some(&incidents[0].id));
    assert_eq!(actions.len(), 2, "Must propose both SAFE and RISKY actions");

    let safe_action = actions.iter().find(|a| a.risk == ActionRisk::Safe).expect("Must have a SAFE action");
    assert!(safe_action.default_selected, "SAFE action MUST be default_selected: true");
    assert!(safe_action.selected, "SAFE action MUST be selected: true");

    let risky_action = actions.iter().find(|a| a.risk == ActionRisk::Risky).expect("Must have a RISKY action");
    assert!(!risky_action.default_selected, "RISKY action MUST be default_selected: false");
    assert!(!risky_action.selected, "RISKY action MUST be selected: false");
}

#[test]
fn test_protection_rule_prefix_and_snapshot_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("tenrec_test_{}", uuid::Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let db_path = temp_dir.join("test.db");

    let db = Arc::new(Database::open(&db_path).expect("Failed to open test SQLite database"));
    let mgr = ProtectionManager::new(db.clone());

    // Verify rule prefix invariant
    assert_eq!(TENREC_RULE_PREFIX, "TENREC_RULE_");

    let actions = vec![
        ProposedAction {
            id: "act_test_1".to_string(),
            incident_id: None,
            title: "Block test endpoint".to_string(),
            description: "Test description".to_string(),
            risk: ActionRisk::Safe,
            rationale: "Testing protection".to_string(),
            expected_benefit: "Verification".to_string(),
            potential_consequence: "None".to_string(),
            current_state: "Open".to_string(),
            proposed_state: "Blocked".to_string(),
            is_reversible: true,
            default_selected: true,
            selected: true,
            duration_minutes: 0,
            target_type: "firewall_rule".to_string(),
            target_identifier: "203.0.113.10".to_string(),
        }
    ];

    let apply_res = mgr.apply_approved_actions(actions);
    // Even if netsh requires admin in test environment and fails gracefully, snapshot record is created
    assert!(!apply_res.snapshot_id.is_empty());

    let snapshots = mgr.get_snapshots();
    assert_eq!(snapshots.len(), 1);

    // Test rollback
    let restore_res = mgr.restore_snapshot(&apply_res.snapshot_id);
    assert!(restore_res.success);

    // Verify audit log
    let audit = db.get_audit_log(10).expect("Must fetch audit log");
    assert!(!audit.is_empty(), "Audit entries must be recorded");
    assert!(audit.iter().any(|e| e.actor == "USER_EXPLICIT_APPROVAL"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_correlation_engine_tracks_new_destinations() {
    let correlation = CorrelationEngine::new();
    let mut procs = vec![];
    let mut apps = vec![];
    let mut conns = vec![
        ConnectionInfo {
            id: "c1".to_string(),
            pid: 42,
            process_name: "test.exe".to_string(),
            application_name: "TestApp".to_string(),
            protocol: "TCP".to_string(),
            local_address: "127.0.0.1".to_string(),
            local_port: 5000,
            remote_address: "93.184.216.34".to_string(),
            remote_port: 80,
            state: "ESTABLISHED".to_string(),
            hostname: "example.com".to_string(),
            bytes_sent: 100,
            bytes_received: 200,
            first_seen: "2026-09-26T04:00:00Z".to_string(),
            last_seen: "2026-09-26T04:00:01Z".to_string(),
            is_unusual: false,
            is_new: false,
        }
    ];
    let listening = vec![];

    let changes = correlation.correlate(&mut procs, &mut apps, &mut conns, &listening);
    assert!(!changes.is_empty(), "New remote destination must generate a WhatChanged item");
    assert_eq!(changes[0].category, "new_destination");
    assert_eq!(changes[0].entity_name, "93.184.216.34");
}
