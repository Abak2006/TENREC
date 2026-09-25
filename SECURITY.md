# TENREC Security Policy & Control Model

## 1. Absolute Product Principle
TENREC is built around the fundamental invariant that **software must never alter the operating system or user processes autonomously**.

1. **Detection Is Passive**: Detectors merely generate structured observations and evidence. They possess no reference or ability to invoke the protection engine.
2. **Explicit User Consent**: Proposed actions are partitioned strictly into **SAFE** and **RISKY**. Safe actions are checked by default; risky actions are left unchecked.
3. **No Batch Autonomous Approval**: The user must explicitly inspect each action, see its rationale and consequence, and confirm execution.
4. **Reversibility Guarantee**: No change is applied without an atomic snapshot of current state and a pre-calculated restoration procedure.

---

## 2. Attack Surface & Self-Defense
TENREC monitors security-sensitive system layers and therefore enforces strict self-protection:

### A. IPC / Command Hardening
- **No Arbitrary Execution**: TENREC exposes zero generic shell execution or eval commands (`execute_command` or similar do NOT exist).
- **Strongly Typed IPC**: Every Tauri command accepts strictly validated structs (e.g. `Pid`, `RuleId`, `SnapshotId`).
- **Input Validation**:
  - Remote IP addresses must parse into valid `std::net::IpAddr`.
  - Process IDs are checked against active OS process tables before any action.
  - Executable paths are normalized and validated to prevent path traversal or injection.
  - Firewall rule names use reserved prefixes (`TENREC_RULE_...`) and strict alphanumeric sanitization.

### B. Privilege Boundary
- Standard telemetry runs in user-mode with standard privileges.
- Privileged operations (such as Windows Firewall rule modifications or process suspension) explicitly declare their elevated requirement.
- When elevation is unavailable, the UI cleanly disables the control with an honest explanation ("Requires Administrator privileges").

### C. Audit Trail
Every control action (creation, application, expiration, rollback) writes an immutable record to the local SQLite `audit_log` table with:
- Timestamp (UTC)
- Action type & target identifier
- Initiator (User explicit approval)
- Previous system state snapshot
- New system state
- Execution result status
