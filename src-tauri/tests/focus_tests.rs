// TENREC - Focus mode and process tuning tests.
// Only child processes started by the test are ever tuned: the samples handed to the
// focus manager are filtered down to those PIDs.
use std::process::{Child, Command};
use std::sync::Arc;
use tenrec_lib::focus::{FocusManager, FocusOptions};
use tenrec_lib::persistence::Database;
use tenrec_lib::platform::tuning::{self, BackgroundTuning};
use tenrec_lib::protection::types::{SnapshotKind, SnapshotStatus};
use tenrec_lib::protection::ProtectionManager;
use tenrec_lib::telemetry::{Collectors, Sample, TelemetryHub};

struct TestChild(Child);
impl TestChild {
    fn spawn(program: &str, args: &[&str]) -> Self {
        Self(Command::new(program).args(args).spawn().expect("spawn child"))
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
    let dir = std::env::temp_dir().join(format!("tenrec_focus_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    (Arc::new(Database::open(&dir.join("t.db")).unwrap()), dir)
}

/// A real sample restricted to the given PIDs and their applications.
fn sample_of(pids: &[u32]) -> Sample {
    let hub = TelemetryHub::new();
    let mut c = Collectors::new();
    hub.tick(&mut c);
    let full = hub.tick(&mut c);
    let processes: Vec<_> = full.processes.iter().filter(|p| pids.contains(&p.pid)).cloned().collect();
    let app_ids: Vec<&str> = processes.iter().map(|p| p.application_id.as_str()).collect();
    let applications = full.applications.iter().filter(|a| app_ids.contains(&a.id.as_str())).cloned().collect();
    let s = Sample { processes, applications, ..Default::default() };
    assert_eq!(s.processes.len(), pids.len(), "test children must appear in the sample");
    s
}

#[test]
fn background_tuning_is_fully_restored() {
    let child = TestChild::spawn("ping", &["-n", "60", "127.0.0.1"]);
    let before = tuning::capture(child.pid()).unwrap();
    let opts = BackgroundTuning { efficiency_mode: true, lower_priority: true, low_io: true, low_memory: true };
    let ecores = tuning::efficiency_core_set_ids();
    let previous = tuning::apply_background(child.pid(), opts, ecores.as_deref()).expect("apply");
    let during = tuning::capture(child.pid()).unwrap();
    assert_ne!(during, before, "something must have changed");
    if previous.priority_class.is_some() {
        assert_eq!(during.priority_class, Some(0x4000), "below normal");
    }
    tuning::restore(child.pid(), &previous).unwrap();
    let after = tuning::capture(child.pid()).unwrap();
    assert_eq!(after.priority_class, before.priority_class);
    assert_eq!(after.io_priority, before.io_priority);
    assert_eq!(after.memory_priority, before.memory_priority);
    assert_eq!(after.cpu_sets, before.cpu_sets);
}

#[test]
fn focus_session_tunes_only_eligible_processes_and_undoes_on_stop() {
    let focus_child = TestChild::spawn("ping", &["-n", "60", "127.0.0.1"]);
    let bg_child = TestChild::spawn("pathping", &["-n", "-q", "1", "-p", "60000", "127.0.0.1"]);
    let sample = sample_of(&[focus_child.pid(), bg_child.pid()]);
    let focus_app = sample.applications.iter().find(|a| a.pids.contains(&focus_child.pid())).unwrap().clone();
    let fg_before = tuning::capture(focus_child.pid()).unwrap();
    let bg_before = tuning::capture(bg_child.pid()).unwrap();

    let (db, dir) = temp_db();
    let protection = ProtectionManager::new(db.clone());
    let focus = FocusManager::new();
    let options = FocusOptions { efficiency_mode: true, lower_priority: true, low_io: true, ..Default::default() };

    let preview = focus.preview(&sample, &focus_app, &options);
    assert_eq!((preview.focus_process_count, preview.background_process_count), (1, 1));

    let status = focus.start(&sample, &focus_app, options.clone(), &protection).unwrap();
    assert!(status.active);
    assert_eq!(status.tuned_processes, 2, "focus app raised and background app lowered");
    assert!(focus.start(&sample, &focus_app, options, &protection).is_err(), "only one session at a time");
    assert_eq!(tuning::capture(focus_child.pid()).unwrap().priority_class, Some(0x80), "focus app at High");
    assert_eq!(tuning::capture(bg_child.pid()).unwrap().priority_class, Some(0x4000), "background at Below normal");

    // A second sample does not re-tune the same processes.
    focus.on_sample(&sample, &protection);
    assert_eq!(focus.status().tuned_processes, 2);

    let r = focus.stop(&protection).unwrap();
    assert_eq!(r.failed, 0, "{:?}", r.messages);
    assert_eq!(tuning::capture(focus_child.pid()).unwrap().priority_class, fg_before.priority_class);
    assert_eq!(tuning::capture(bg_child.pid()).unwrap().priority_class, bg_before.priority_class);
    assert!(!focus.status().active);
    assert!(focus.stop(&protection).is_err());
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn excluded_apps_are_left_alone() {
    let focus_child = TestChild::spawn("ping", &["-n", "60", "127.0.0.1"]);
    let bg_child = TestChild::spawn("pathping", &["-n", "-q", "1", "-p", "60000", "127.0.0.1"]);
    let sample = sample_of(&[focus_child.pid(), bg_child.pid()]);
    let focus_app = sample.applications.iter().find(|a| a.pids.contains(&focus_child.pid())).unwrap().clone();
    let bg_app_id = sample.processes.iter().find(|p| p.pid == bg_child.pid()).unwrap().application_id.clone();
    let bg_before = tuning::capture(bg_child.pid()).unwrap();

    let (db, dir) = temp_db();
    let protection = ProtectionManager::new(db.clone());
    let focus = FocusManager::new();
    let options = FocusOptions { lower_priority: true, exclusions: vec![bg_app_id], ..Default::default() };
    focus.start(&sample, &focus_app, options, &protection).unwrap();
    assert_eq!(tuning::capture(bg_child.pid()).unwrap(), bg_before);
    focus.stop(&protection).unwrap();
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn interrupted_session_is_undone_on_next_launch() {
    let focus_child = TestChild::spawn("ping", &["-n", "60", "127.0.0.1"]);
    let sample = sample_of(&[focus_child.pid()]);
    let focus_app = sample.applications[0].clone();
    let before = tuning::capture(focus_child.pid()).unwrap();

    let (db, dir) = temp_db();
    {
        let protection = ProtectionManager::new(db.clone());
        let focus = FocusManager::new();
        focus.start(&sample, &focus_app, FocusOptions::default(), &protection).unwrap();
        assert_eq!(tuning::capture(focus_child.pid()).unwrap().priority_class, Some(0x80));
        // Simulated crash: the manager is dropped without stop().
    }
    let protection = ProtectionManager::new(db.clone());
    assert_eq!(protection.recover_interrupted_focus(), 1);
    assert_eq!(tuning::capture(focus_child.pid()).unwrap().priority_class, before.priority_class);
    let snap = &protection.get_snapshots()[0];
    assert_eq!(snap.kind, SnapshotKind::Focus);
    assert_eq!(snap.status, SnapshotStatus::Restored);
    assert_eq!(protection.recover_interrupted_focus(), 0, "nothing left to recover");
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}
