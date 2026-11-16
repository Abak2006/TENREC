# Testing

```bash
cd src-tauri
cargo test
```

## `tests/safety_tests.rs`

Pure validation. Makes no system changes.

- Public vs private IPv4/IPv6 classification, including CGNAT, link-local, ULA and v4-mapped addresses.
- Protected process names and PIDs are refused.
- Program-block paths: missing, relative and core-Windows binaries are refused.
- Firewall rule names: only `TENREC_RULE_[A-Za-z0-9_]+` is accepted, so a name like `TENREC_RULE_x dir=in` is rejected.
- Invalid IPs, ports and protocols never reach `netsh`.

## `tests/tenrec_tests.rs`

Runs against the real system.

- A system snapshot contains real values.
- The sampler produces processes, CPU within 0–100, socket owners resolved to names, and system/throughput data.
- Correlation treats startup state as the baseline and reports only later changes. Private destinations are ignored.
- A script host connecting to a public IP produces one incident, with a pre-selected low-risk block and a non-pre-selected suspend. A private destination produces nothing.
- Preparing an action for `lsass.exe`, a missing PID or an unknown action kind is refused.
- **End-to-end undo:** lowers the priority of a `ping` child process, persists the snapshot, reopens the manager from the database, undoes the change, and checks that the original priority class is back and that both successful and failed attempts are in the audit log.
- A stale process identity (wrong creation time) is never touched.
- The capability list marks unimplemented features as not implemented.

## Not covered by automated tests

Firewall rule creation and deletion need administrator rights and change system state. Check them manually from an elevated session:

1. Block an IP.
2. Run `netsh advfirewall firewall show rule name=all | findstr TENREC_RULE`.
3. Undo the block.
4. Run the same command again and confirm the rule is gone.
