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

## `tests/focus_tests.rs`

These tests only ever touch child processes they start themselves.

- Background tuning is applied, then restored field by field.
- A focus session raises the focus app, lowers a background app, ignores processes it has already handled, and restores both when it ends. Only one session can run at a time.
- Excluded apps are left untouched.
- A session left running by a simulated crash is undone by `recover_interrupted_focus` on the next start.

## `tests/controls_tests.rs`

These tests use their own `TENREC_TEST_*` entries and remove them afterwards.

- A test-only startup Run entry is disabled through StartupApproved, then the undo is checked.
- A test-only scheduled task is created, disabled, re-enabled through undo, and deleted. Windows tasks are refused.
- One low-impact privacy value is flipped and restored exactly.
- Telemetry hostname matching.
- DNS cache lookup. When Windows' own cache is empty, the test checks that against `Get-DnsClientCache` instead of failing.

## `tests/phase4_tests.rs`

- Localised `powercfg` output is parsed, and real power state is read.
- QoS input validation rejects injection attempts and out-of-range rates.
- Focus network options are refused without admin rights, and nothing is left applied.
- Cleanup on a throwaway folder deletes only old files, removes empty folders, and leaves a junction target outside the folder intact.

## Not covered by automated tests

These need administrator rights or change system-wide state, so check them manually from an elevated session:

- firewall rule creation and deletion
- QoS upload limits
- changing the power mode or power plan
- HKLM privacy policies
- all-users startup entries
- Windows temp and Update cleanup

Steps for firewall rules:

1. Block an IP.
2. Run `netsh advfirewall firewall show rule name=all | findstr TENREC_RULE`.
3. Undo the block.
4. Run the same command again and confirm the rule is gone.
