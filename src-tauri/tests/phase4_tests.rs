// TENREC - Power, upload-limit and cleanup tests.
// Cleanup is only exercised on a throwaway folder; real temp folders are only scanned.
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tenrec_lib::controls::cleanup;
use tenrec_lib::focus::{FocusManager, FocusOptions};
use tenrec_lib::persistence::Database;
use tenrec_lib::platform::{power, qos};
use tenrec_lib::protection::process_ctrl::is_elevated;
use tenrec_lib::protection::types::SnapshotStatus;
use tenrec_lib::protection::ProtectionManager;
use tenrec_lib::telemetry::{Collectors, Sample, TelemetryHub};

#[test]
fn parses_localised_powercfg_output() {
    let text = "Vorhandene Energieschemas (* Aktiv)\n-----\n\
        Energieschema-GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Ausbalanciert)\n\
        Energieschema-GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (Höchstleistung) *\n\
        Energieschema-GUID: 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c  (Höchstleistung)\n";
    let plans = power::parse_plan_list(text);
    assert_eq!(plans.len(), 2, "duplicates are merged");
    assert_eq!(plans[1].name, "Höchstleistung");
    assert!(plans[1].active && !plans[0].active);
    assert!(power::is_guid("381b4222-f694-41f0-9685-ff5bb260df2e"));
    assert!(!power::is_guid("381b4222-f694-41f0-9685-ff5bb260df2e & x"));
    assert!(power::set_plan("not-a-guid").is_err());
}

#[test]
fn reads_real_power_state() {
    let s = power::state().expect("powercfg works");
    assert!(!s.plans.is_empty());
    assert_eq!(s.plans.iter().filter(|p| p.active).count(), 1);
    assert_eq!(power::active_plan().unwrap(), s.plans.iter().find(|p| p.active).unwrap().guid);
}

#[test]
fn upload_limit_inputs_are_validated() {
    assert!(qos::is_valid_exe_name("OneDrive.exe"));
    assert!(qos::is_valid_exe_name("Steam Client (x64).exe"));
    assert!(!qos::is_valid_exe_name("a.exe' ; Remove-Item C:\\ ; '"));
    assert!(!qos::is_valid_exe_name("notes.txt"));
    assert!(qos::is_valid_policy_name(&qos::new_policy_name()));
    assert!(!qos::is_valid_policy_name("SomeoneElse"));
    let n = qos::new_policy_name();
    assert!(qos::limit_upload(&n, "x.exe' -Foo '", 1000).is_err());
    assert!(qos::limit_upload(&n, "x.exe", 1).is_err(), "out-of-range rate");
    assert!(qos::remove("SomeoneElse").is_err());
}

struct TestChild(Child);
impl Drop for TestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn network_focus_options_need_admin_and_leave_nothing_behind() {
    if is_elevated() {
        eprintln!("running elevated: skipping the non-admin refusal test");
        return;
    }
    let child = TestChild(Command::new("ping").args(["-n", "60", "127.0.0.1"]).spawn().unwrap());
    let hub = TelemetryHub::new();
    let mut c = Collectors::new();
    hub.tick(&mut c);
    let full = hub.tick(&mut c);
    let processes: Vec<_> = full.processes.iter().filter(|p| p.pid == child.0.id()).cloned().collect();
    let applications: Vec<_> = full.applications.iter().filter(|a| a.pids.contains(&child.0.id())).cloned().collect();
    let sample = Sample { processes, applications, ..Default::default() };
    let app = sample.applications[0].clone();

    let dir = std::env::temp_dir().join(format!("tenrec_p4_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = Arc::new(Database::open(&dir.join("t.db")).unwrap());
    let pm = ProtectionManager::new(db.clone());
    let focus = FocusManager::new();
    let opts = FocusOptions { block_apps: vec!["app_other".into()], ..Default::default() };
    let err = focus.start(&sample, &app, opts, &pm).unwrap_err();
    assert!(err.contains("administrator"), "{err}");
    assert!(!focus.status().active);
    assert!(pm.get_snapshots().iter().all(|s| s.status == SnapshotStatus::Restored), "nothing left active");
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn cleanup_deletes_only_old_files_inside_the_folder_and_skips_junctions() {
    let base = std::env::temp_dir().join(format!("tenrec_clean_{}", uuid::Uuid::new_v4().simple()));
    let root = base.join("root");
    let outside = base.join("outside");
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(root.join("a.tmp"), vec![0u8; 1000]).unwrap();
    std::fs::write(root.join("sub").join("b.tmp"), vec![0u8; 500]).unwrap();
    std::fs::write(outside.join("keep.txt"), b"must survive").unwrap();
    // A junction inside root pointing outside must not be followed.
    let junction = root.join("link");
    let ok = Command::new("cmd").args(["/c", "mklink", "/J"]).arg(&junction).arg(&outside).output().unwrap().status.success();
    assert!(ok, "mklink /J failed");

    // Files just written are younger than one hour, so nothing is deleted.
    let (freed, deleted, _, err) = cleanup::clean_folder(&root, Duration::from_secs(3600));
    assert_eq!((freed, deleted, err), (0, 0, None));

    // With no age limit the two files go, the empty subfolder goes, the junction target stays.
    let (freed, deleted, skipped, err) = cleanup::clean_folder(&root, Duration::ZERO);
    assert_eq!((freed, deleted, skipped, err), (1500, 2, 0, None));
    assert!(!root.join("sub").exists());
    assert!(outside.join("keep.txt").exists(), "junction target untouched");

    let _ = std::fs::remove_dir(&junction);
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn cleanup_scan_reports_every_category() {
    let scan = cleanup::scan();
    assert_eq!(scan.len(), 5);
    assert!(scan.iter().any(|c| c.requires_admin));
}
