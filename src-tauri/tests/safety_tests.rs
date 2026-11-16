// TENREC - Validation and safety-guard tests (no system changes are made)
use tenrec_lib::protection::{firewall, guard};
use tenrec_lib::util::{is_public_ip_str, is_user_writable_location};

#[test]
fn classifies_public_and_private_addresses() {
    for s in ["10.0.0.1", "172.20.1.1", "192.168.1.5", "127.0.0.1", "169.254.3.4", "100.64.0.1", "0.0.0.0", "::1", "fe80::1", "fd12::1", "::ffff:192.168.0.1"] {
        assert!(!is_public_ip_str(s), "{s} must not be public");
    }
    for s in ["8.8.8.8", "172.32.0.1", "198.51.100.42", "2606:4700::1111", "::ffff:1.1.1.1"] {
        assert!(is_public_ip_str(s), "{s} must be public");
    }
    assert!(!is_public_ip_str("not-an-ip"));
}

#[test]
fn detects_user_writable_locations() {
    assert!(is_user_writable_location(r"C:\Users\a\AppData\Local\Temp\x.exe"));
    assert!(is_user_writable_location(r"C:\Users\a\Downloads\setup.exe"));
    assert!(!is_user_writable_location(r"C:\Program Files\App\app.exe"));
}

#[test]
fn refuses_critical_processes() {
    assert!(guard::check_process(4, "System").is_err());
    assert!(guard::check_process(800, "lsass.exe").is_err());
    assert!(guard::check_process(900, "CSRSS.EXE").is_err());
    assert!(guard::check_process(std::process::id(), "anything.exe").is_err());
    assert!(guard::check_process(12345, "notepad.exe").is_ok());
}

#[test]
fn refuses_private_and_malformed_ips() {
    assert!(guard::check_remote_ip("192.168.1.1").is_err());
    assert!(guard::check_remote_ip("8.8.8.8 dir=in").is_err());
    assert!(guard::check_remote_ip("").is_err());
    assert!(guard::check_remote_ip("1.1.1.1").is_ok());
}

#[test]
fn refuses_missing_or_core_program_paths() {
    assert!(guard::check_program_path("").is_err());
    assert!(guard::check_program_path(r"relative\app.exe").is_err());
    assert!(guard::check_program_path(r"C:\definitely\missing\app.exe").is_err());
    let windir = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    assert!(guard::check_program_path(&format!(r"{windir}\System32\svchost.exe")).is_err());
    assert!(guard::check_program_path(&format!(r"{windir}\System32\cmd.exe")).is_ok());
}

#[test]
fn rule_names_are_prefixed_and_safe() {
    let n = firewall::new_rule_name();
    assert!(n.starts_with(firewall::TENREC_RULE_PREFIX));
    assert!(firewall::check_name(&n).is_ok());
    assert!(firewall::check_name("Core Networking - DNS").is_err());
    assert!(firewall::check_name("TENREC_RULE_x dir=in").is_err());
    assert!(firewall::check_name("TENREC_RULE_").is_err());
}

#[test]
fn invalid_targets_never_reach_netsh() {
    let n = firewall::new_rule_name();
    assert!(firewall::block_remote_ip(&n, "10.0.0.1").is_err());
    assert!(firewall::block_remote_ip(&n, "1.1.1.1 dir=in").is_err());
    assert!(firewall::block_program(&n, r"C:\missing\x.exe").is_err());
    assert!(firewall::block_inbound_port(&n, 0, "TCP").is_err());
    assert!(firewall::block_inbound_port(&n, 80, "ICMP").is_err());
    assert!(firewall::delete_rule("SomeoneElsesRule").is_err());
}
