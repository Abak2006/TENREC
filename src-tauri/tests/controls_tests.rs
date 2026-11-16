// TENREC - Startup and scheduled task controls.
// These tests only create and modify their own TENREC_TEST_* entries and remove them.
use std::collections::HashSet;
use std::process::Command;
use std::sync::Arc;
use tenrec_lib::controls::{startup, tasks};
use tenrec_lib::persistence::Database;
use tenrec_lib::platform::registry::{self, Hive, RegValue};
use tenrec_lib::protection::ProtectionManager;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

fn manager() -> (ProtectionManager, Arc<Database>, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("tenrec_ctl_{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = Arc::new(Database::open(&dir.join("t.db")).unwrap());
    (ProtectionManager::new(db.clone()), db, dir)
}

#[test]
fn parses_run_commands() {
    assert_eq!(startup::executable_from_command(r#""C:\Program Files\App\app.exe" --min"#).as_deref(), Some(r"C:\Program Files\App\app.exe"));
    assert_eq!(startup::executable_from_command(r"C:\Tools\tool.exe /silent").as_deref(), Some(r"C:\Tools\tool.exe"));
    let windir = std::env::var("windir").unwrap();
    assert_eq!(startup::executable_from_command(r"%windir%\system32\cmd.exe /c x"), Some(format!(r"{windir}\system32\cmd.exe")));
    assert_eq!(startup::executable_from_command("   "), None);
}

#[test]
fn startup_item_toggle_is_recorded_and_undone() {
    let name = format!("TENREC_TEST_{}", uuid::Uuid::new_v4().simple());
    registry::write(Hive::CurrentUser, RUN, &name, &RegValue::string(r#""C:\Windows\System32\cmd.exe" /c exit"#)).unwrap();

    let item = startup::list(&HashSet::new()).into_iter().find(|i| i.name == name).expect("test entry listed");
    assert!(item.enabled, "no StartupApproved value means enabled");
    assert!(!item.requires_admin);

    let (pm, db, dir) = manager();
    let change = startup::set_enabled(&item.id, false).unwrap();
    let snap = pm.record_setting(change);
    let after = startup::list(&HashSet::new()).into_iter().find(|i| i.name == name).unwrap();
    assert!(!after.enabled);
    assert!(startup::set_enabled(&item.id, false).is_err(), "already disabled");

    let r = pm.restore_snapshot(&snap).unwrap();
    assert_eq!(r.failed, 0);
    assert!(registry::read(Hive::CurrentUser, APPROVED_RUN, &name).unwrap().is_none(), "approved value removed again");
    assert!(startup::list(&HashSet::new()).into_iter().find(|i| i.name == name).unwrap().enabled);

    registry::delete(Hive::CurrentUser, RUN, &name).unwrap();
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn scheduled_task_toggle_is_recorded_and_undone() {
    let name = format!("TENREC_TEST_{}", uuid::Uuid::new_v4().simple());
    let path = format!(r"\{name}");
    let created = Command::new("schtasks")
        .args(["/Create", "/TN", &path, "/TR", "cmd.exe /c exit", "/SC", "ONCE", "/ST", "23:59", "/F"])
        .output()
        .unwrap();
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));

    let listed = tasks::list().unwrap().into_iter().find(|t| t.path == path).expect("task listed");
    assert!(listed.enabled);

    let (pm, db, dir) = manager();
    let snap = pm.record_setting(tasks::set_enabled(&path, false).unwrap());
    assert!(!tasks::list().unwrap().into_iter().find(|t| t.path == path).unwrap().enabled);
    assert!(tasks::set_enabled(r"\Microsoft\Windows\Defrag\ScheduledDefrag", false).is_err(), "Windows tasks are not accepted");

    assert_eq!(pm.restore_snapshot(&snap).unwrap().failed, 0);
    assert!(tasks::list().unwrap().into_iter().find(|t| t.path == path).unwrap().enabled);

    let _ = Command::new("schtasks").args(["/Delete", "/TN", &path, "/F"]).output();
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}
