// TENREC - Integration tests against the real Windows APIs
use std::process::{Child, Command};
use std::sync::Arc;
use tenrec_lib::correlation::CorrelationEngine;
use tenrec_lib::hardware::registry::HardwareRegistry;
use tenrec_lib::hardware::types::{AccessMode, CapabilityState};
use tenrec_lib::network::types::{ConnectionInfo, ListeningPortInfo};
use tenrec_lib::persistence::Database;
use tenrec_lib::processes::types::{ApplicationItem, ProcessItem};
use tenrec_lib::protection::process_ctrl::{creation_time, get_priority_class, IDLE};
use tenrec_lib::protection::types::{ChangeStatus, SnapshotStatus};
use tenrec_lib::protection::ProtectionManager;
use tenrec_lib::security::types::{ActionRisk, ActionTarget, IncidentSeverity, ProposedAction};
use tenrec_lib::security::SecurityEngine;
use tenrec_lib::system::{IoMetrics, SystemManager};
use tenrec_lib::telemetry::{Collectors, TelemetryHub};

/// A harmless long-running child process we are allowed to control.
struct TestChild(Child);
impl TestChild {
    fn spawn() -> Self {
        Self(Command::new("ping").args(["-n", "60", "127.0.0.1"]).spawn().expect("spawn ping"))
    }
    fn pid(&self) -> u32 {
        self.0.id()
    }
}
impl Drop for TestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn temp_db() -> (Arc<Database>, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("tenrec_test_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.db");
    (Arc::new(Database::open(&path).expect("open test db")), dir)
}

fn process(pid: u32, name: &str, path: &str) -> ProcessItem {
    ProcessItem {
        pid,
        ppid: None,
        name: name.to_string(),
        executable_path: path.to_string(),
        company: None,
        application_id: format!("app_{name}"),
        application_name: name.to_string(),
        cpu_percent: 0.0,
        memory_bytes: 1,
        memory_percent: 0.0,
        io_read_bps: 0,
        io_write_bps: 0,
        connection_count: 1,
        start_time: String::new(),
        runs_from_user_writable_location: false,
        restricted: false,
    }
}

fn connection(pid: u32, name: &str, remote: &str, public: bool) -> ConnectionInfo {
    ConnectionInfo {
        id: format!("c_{pid}_{remote}"),
        pid,
        process_name: name.to_string(),
        application_name: name.to_string(),
        protocol: "TCP".to_string(),
        ip_version: 4,
        local_address: "192.168.1.10".to_string(),
        local_port: 50000,
        remote_address: remote.to_string(),
        remote_port: 443,
        state: "ESTABLISHED".to_string(),
        is_public_remote: public,
        is_new: false,
    }
}

#[test]
fn system_snapshot_reports_real_values() {
    let mut sys = SystemManager::new();
    let s = sys.snapshot(IoMetrics::default());
    assert!(!s.hostname.is_empty());
    assert!(s.cpu.logical_core_count >= s.cpu.core_count && s.cpu.core_count > 0);
    assert!(s.memory.total_bytes > 0 && s.memory.used_bytes > 0);
    assert!((0.0..=100.0).contains(&s.memory.usage_percent));
    assert!(!s.disk.drives.is_empty(), "the system drive must be listed");
}

#[test]
fn sampler_collects_processes_sockets_and_names() {
    let hub = TelemetryHub::new();
    let mut c = Collectors::new();
    hub.tick(&mut c);
    let s = hub.tick(&mut c);
    assert_eq!(s.sequence, 2);
    assert!(s.processes.len() > 10, "a Windows host runs many processes");
    let me = s.processes.iter().find(|p| p.pid == std::process::id()).expect("own process listed");
    assert!(!me.executable_path.is_empty());
    for p in &s.processes {
        assert!((0.0..=100.0).contains(&p.cpu_percent), "{} cpu {}", p.name, p.cpu_percent);
    }
    // Sockets must carry real process names, not placeholder PIDs, when the owner is readable.
    let named = s.listening.iter().filter(|l| !l.process_name.starts_with("PID ")).count();
    assert!(named > 0, "at least some listeners resolve to a process name");
    assert!(s.system.is_some() && s.throughput.is_some());
}

#[test]
fn correlation_seeds_silently_then_reports_new_items() {
    let engine = CorrelationEngine::new();
    let apps: Vec<ApplicationItem> = vec![];
    let listening: Vec<ListeningPortInfo> = vec![];

    let mut first = vec![connection(42, "a.exe", "93.184.216.34", true)];
    assert!(engine.observe(&apps, &mut first, &listening).is_empty(), "startup state is not a change");

    let mut second = vec![connection(42, "a.exe", "93.184.216.34", true), connection(42, "a.exe", "151.101.1.69", true)];
    let changes = engine.observe(&apps, &mut second, &listening);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].category, "new_destination");
    assert!(second[1].is_new && !second[0].is_new);

    // Private destinations are not tracked.
    let mut third = vec![connection(42, "a.exe", "10.0.0.5", false)];
    assert!(engine.observe(&apps, &mut third, &listening).is_empty());
}

#[test]
fn script_host_public_connection_raises_incident_with_safe_and_risky_actions() {
    let child = TestChild::spawn();
    let sec = SecurityEngine::new();
    let procs = vec![process(child.pid(), "powershell.exe", r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe")];
    let conns = vec![connection(child.pid(), "powershell.exe", "198.51.100.42", true)];

    sec.analyze(&procs, &conns, &[]);
    sec.analyze(&procs, &conns, &[]);

    let incidents = sec.get_incidents();
    assert_eq!(incidents.len(), 1, "the same condition is one incident, not one per sample");
    assert_eq!(incidents[0].severity, IncidentSeverity::High);
    assert_eq!(incidents[0].observation_count, 2);

    let actions = sec.get_proposed_actions(Some(&incidents[0].id));
    let safe = actions.iter().find(|a| a.risk == ActionRisk::Safe).expect("safe action");
    assert!(safe.default_selected);
    assert_eq!(safe.target, ActionTarget::BlockRemoteIp { ip: "198.51.100.42".into() });
    let risky = actions.iter().find(|a| a.risk == ActionRisk::Risky).expect("risky action");
    assert!(!risky.default_selected);
    assert!(matches!(risky.target, ActionTarget::SuspendProcess { pid, .. } if pid == child.pid()));

    // Private destinations never raise this incident.
    let sec2 = SecurityEngine::new();
    sec2.analyze(&procs, &[connection(child.pid(), "powershell.exe", "192.168.1.1", false)], &[]);
    assert!(sec2.get_incidents().is_empty());
}

#[test]
fn prepare_process_action_refuses_protected_and_missing_processes() {
    let sec = SecurityEngine::new();
    let procs = vec![process(700, "lsass.exe", r"C:\Windows\System32\lsass.exe")];
    assert!(sec.prepare_process_action(&procs, 700, "suspend").is_err());
    assert!(sec.prepare_process_action(&procs, 99999, "priority").is_err());
    let child = TestChild::spawn();
    let procs = vec![process(child.pid(), "PING.EXE", r"C:\Windows\System32\PING.EXE")];
    assert!(sec.prepare_process_action(&procs, child.pid(), "terminate").is_err(), "unknown kinds are rejected");
    let a = sec.prepare_process_action(&procs, child.pid(), "priority").expect("priority action");
    assert!(sec.get_action(&a.id).is_some(), "prepared actions are registered for apply-by-id");
}

#[test]
fn priority_change_is_applied_persisted_and_undone() {
    let child = TestChild::spawn();
    let pid = child.pid();
    let before = get_priority_class(pid).expect("read priority");
    let (db, dir) = temp_db();
    let mgr = ProtectionManager::new(db.clone());

    let action = ProposedAction {
        id: "t1".into(),
        incident_id: None,
        title: "lower".into(),
        description: String::new(),
        risk: ActionRisk::Safe,
        expected_benefit: String::new(),
        potential_consequence: String::new(),
        requires_admin: false,
        default_selected: true,
        target: ActionTarget::LowerPriority { pid, process_name: "PING.EXE".into(), creation_time: creation_time(pid).unwrap() },
    };
    let bad_ip = ProposedAction {
        id: "t2".into(),
        target: ActionTarget::BlockRemoteIp { ip: "192.168.0.1".into() },
        ..action.clone()
    };
    let res = mgr.apply(vec![action, bad_ip]);
    assert!(res.outcomes[0].success, "{:?}", res.outcomes[0].error);
    assert!(!res.outcomes[1].success, "private IPs are refused before reaching netsh");
    assert_eq!(get_priority_class(pid).unwrap(), IDLE);
    let snap_id = res.snapshot_id.expect("snapshot recorded");
    assert_eq!(mgr.get_active_policies().len(), 1);

    // Undo history survives a restart.
    let reopened = ProtectionManager::new(db.clone());
    assert_eq!(reopened.get_snapshots().len(), 1);

    let r = reopened.restore_snapshot(&snap_id).unwrap();
    assert_eq!((r.restored, r.failed), (1, 0));
    assert_eq!(get_priority_class(pid).unwrap(), before, "previous priority is restored");
    let snap = &reopened.get_snapshots()[0];
    assert_eq!(snap.status, SnapshotStatus::Restored);
    assert_eq!(snap.changes[0].status, ChangeStatus::Restored);
    assert!(reopened.get_active_policies().is_empty());

    let audit = db.get_audit_log(10).unwrap();
    assert!(audit.iter().any(|e| e.action_type == "apply_action" && e.success));
    assert!(audit.iter().any(|e| e.action_type == "apply_action" && !e.success));
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stale_pid_is_not_touched() {
    let (db, dir) = temp_db();
    let mgr = ProtectionManager::new(db);
    let child = TestChild::spawn();
    let pid = child.pid();
    let wrong_identity = creation_time(pid).unwrap() + 1;
    let res = mgr.apply(vec![ProposedAction {
        id: "t".into(),
        incident_id: None,
        title: "suspend".into(),
        description: String::new(),
        risk: ActionRisk::Risky,
        expected_benefit: String::new(),
        potential_consequence: String::new(),
        requires_admin: false,
        default_selected: false,
        target: ActionTarget::SuspendProcess { pid, process_name: "PING.EXE".into(), creation_time: wrong_identity },
    }]);
    assert!(!res.outcomes[0].success);
    assert!(res.snapshot_id.is_none());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn capability_list_is_honest() {
    let diag = HardwareRegistry::get_diagnostics();
    assert!(!diag.cpu_name.is_empty() && diag.total_memory_bytes > 0);
    let caps = HardwareRegistry::get_capabilities(&diag);
    assert!(caps.iter().any(|c| c.access_mode == AccessMode::ReadWrite));
    for id in ["per_process_network", "gpu_metrics", "cpu_temperature"] {
        let c = caps.iter().find(|c| c.id == id).unwrap();
        assert_eq!(c.state, CapabilityState::NotImplemented, "{id} must not claim to work");
    }
}
