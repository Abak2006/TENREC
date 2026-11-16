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

use tenrec_lib::controls::privacy;
use tenrec_lib::network::DnsCache;

#[test]
fn privacy_setting_change_is_exactly_undone() {
    // Low-impact HKCU setting; its exact previous value (or absence) is restored.
    const KEY: &str = r"Software\Microsoft\Siuf\Rules";
    let before = registry::read(Hive::CurrentUser, KEY, "NumberOfSIUFInPeriod").unwrap();
    let setting = privacy::list().into_iter().find(|s| s.id == "feedback_requests").unwrap();
    let (pm, db, dir) = manager();
    let snap = pm.record_setting(privacy::set_private("feedback_requests", !setting.is_private).unwrap());
    let flipped = privacy::list().into_iter().find(|s| s.id == "feedback_requests").unwrap();
    assert_eq!(flipped.is_private, !setting.is_private);
    assert_eq!(pm.restore_snapshot(&snap).unwrap().failed, 0);
    assert_eq!(registry::read(Hive::CurrentUser, KEY, "NumberOfSIUFInPeriod").unwrap(), before);
    assert!(privacy::set_private("not_a_setting", true).is_err());
    drop(db);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn every_privacy_setting_is_readable() {
    let list = privacy::list();
    assert!(list.len() >= 10);
    assert!(list.iter().any(|s| s.requires_admin) && list.iter().any(|s| !s.requires_admin));
}

#[test]
fn telemetry_host_matching() {
    assert!(privacy::is_telemetry_host("v10.events.data.microsoft.com"));
    assert!(privacy::is_telemetry_host("watson.microsoft.com"));
    assert!(!privacy::is_telemetry_host("www.microsoft.com"));
    assert!(!privacy::is_telemetry_host("example.com"));
}

#[test]
fn dns_cache_maps_resolved_addresses_back_to_names() {
    use std::net::ToSocketAddrs;
    // The OS resolves the name (this test's own lookup), so it lands in the DNS cache.
    let Ok(addrs) = ("example.com", 80).to_socket_addrs() else {
        eprintln!("offline: skipping DNS cache test");
        return;
    };
    let ip = addrs.map(|a| a.ip().to_string()).next().unwrap();
    let mut cache = DnsCache::new();
    cache.refresh_if_due();
    if cache.is_empty() {
        // Some VPN / secure-DNS clients bypass the Windows DNS client cache entirely.
        // Only fail when Windows itself reports cached entries that TENREC missed.
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-DnsClientCache | Measure-Object).Count"])
            .output()
            .unwrap();
        let os_count: usize = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap_or(0);
        assert_eq!(os_count, 0, "Windows reports {os_count} cached entries but TENREC read none");
        eprintln!("Windows DNS cache is empty on this machine: skipping");
        return;
    }
    assert_eq!(cache.lookup(&ip).as_deref(), Some("example.com"));
}
