# TENREC - System Architecture Specification

## 1. System Overview & Philosophy
"Know what your computer is doing. Understand why. Decide what it should do."

TENREC is a local-first Windows system intelligence and control platform. It continuously observes, correlates, and analyzes:
1. **System Health & Resources:** CPU, GPU, RAM, Disks, Battery, Power State, Thermals, Uptime.
2. **Processes & Applications:** Identity, publisher verification, resource consumption, tree ancestry.
3. **Network Behavior:** Active TCP connections, UDP endpoints, listening sockets, interface throughput, remote destinations, protocols.
4. **Security & Anomaly Detection:** Behavioral anomalies, unusual outbound traffic, unexpected listening ports, beaconing indicators, reconnaissance detection, and suspicious DNS queries.
5. **Explainable Recommendations:** Classifying actions into SAFE vs RISKY, estimating impact, stating reversibility, and explaining rationale.
6. **User Approval Layer:** The absolute enforcement barrier ensuring no control action is ever executed autonomously.
7. **Protection & Restoration Engine:** Deterministic Windows Firewall rules, process controls, snapshotting, and one-click "Restore Everything" rollback.

---

## 2. The Absolute Control Pipeline Principle
Detection code **never** invokes protection code. All changes follow this strict pipeline:

```
[System & Network Telemetry]
           │
           ▼
[Correlation Engine (Process ↔ Socket ↔ App ↔ Dest)]
           │
           ▼
[Baseline Engine (Statistical & Moving Averages)]
           │
           ▼
[Security Detectors (Produce Structured Evidence)]
           │
           ▼
[Recommendation Engine (Propose Classified Actions)]
           │
     ┌─────┴──────────────────────┐
     │ Classify: SAFE vs RISKY    │
     │ Provide: Rationale & Impact│
     └─────┬──────────────────────┘
           ▼
[User Approval UI (Interactive Staging)]
     - Safe actions selected by default
     - Risky actions UNSELECTED by default
     - User individually selects/modifies
     - User explicitly clicks "Apply"
           │
           ▼
[Restoration Engine (Snapshot Pre-State & Lifetime)]
           │
           ▼
[Protection Engine (Apply Windows Firewall / Process Action)]
           │
           ▼
[Restoration Engine (Monitor Lifetime / User Rollback / Restore All)]
```

---

## 3. Core Module Hierarchy (Rust Backend `src-tauri/src/`)

```
src-tauri/src/
├── main.rs                 # Tauri Windows entrypoint
├── lib.rs                  # App builder, command registration, plugin wiring
├── core/
│   ├── mod.rs
│   ├── app_state.rs        # Shared concurrent state (RwLock / Arc)
│   └── event_bus.rs        # Typed asynchronous event broadcast
├── system/
│   ├── mod.rs
│   ├── cpu.rs              # Real Windows CPU telemetry (GetSystemTimes / sysinfo)
│   ├── memory.rs           # Real RAM telemetry (GlobalMemoryStatusEx)
│   ├── disk.rs             # Real Disk storage & IO telemetry (GetDiskFreeSpaceExW)
│   ├── power.rs            # Real Battery / AC power telemetry (GetSystemPowerStatus)
│   ├── gpu.rs              # GPU detection & telemetry (DXGI adapter query / graceful fallback)
│   └── provider.rs         # Telemetry collection manager & aggregation
├── processes/
│   ├── mod.rs
│   ├── manager.rs          # Process enumeration, lifecycle, PID tracking
│   ├── identity.rs         # Application identity mapping & publisher verification
│   └── types.rs            # Process & Application data models
├── network/
│   ├── mod.rs
│   ├── sockets.rs          # Real Windows TCP/UDP table capture (GetExtendedTcpTable)
│   ├── listening.rs        # Active listening ports enumeration
│   ├── interfaces.rs       # Real download/upload interface throughput (GetIfTable2)
│   └── destinations.rs     # Remote host/IP aggregation, DNS resolver cache
├── correlation/
│   ├── mod.rs
│   └── engine.rs           # Process ↔ Socket ↔ Application ↔ Destination correlation
├── baseline/
│   ├── mod.rs
│   └── stats.rs            # Statistical moving averages, percentiles, first-seen/last-seen
├── security/
│   ├── mod.rs
│   ├── detectors/          # Modular detection implementations:
│   │   ├── outbound.rs     # Unusual outbound transfer detector
│   │   ├── new_dest.rs     # New destination detector
│   │   ├── ports.rs        # Unexpected listening port detector
│   │   ├── beaconing.rs    # Periodic beaconing pattern detector
│   │   └── recon.rs        # Port scan / connection burst detector
│   └── incident.rs         # Incident models with structured evidence
├── evidence/
│   ├── mod.rs
│   └── model.rs            # Structured, human-explainable evidence objects
├── recommendations/
│   ├── mod.rs
│   └── engine.rs           # Action proposals, SAFE vs RISKY classification
├── approval/
│   ├── mod.rs
│   └── stage.rs            # Staged actions, policy validation, approval verification
├── protection/
│   ├── mod.rs
│   ├── firewall.rs         # Windows Firewall rule enforcement (netsh advfirewall)
│   └── process_ctrl.rs     # Safe process controls (Suspend, Resume, Priority, Terminate)
├── restoration/
│   ├── mod.rs
│   └── engine.rs           # Snapshot storage, rollback execution, Restore All
├── persistence/
│   ├── mod.rs
│   ├── db.rs               # SQLite connection pool & transaction manager
│   ├── schema.rs           # Database table schemas & indexes
│   └── migrations.rs       # Versioned SQLite migrations
└── ipc/
    ├── mod.rs
    └── commands.rs         # Strongly-typed Tauri IPC commands (No arbitrary shell)
```

---

## 4. Local-First SQLite Persistence Schema
All state is persisted in a local SQLite database (`%LOCALAPPDATA%/tenrec/tenrec.db`):
- `system_samples`: Periodic CPU, RAM, Disk, Power, and Network throughput samples.
- `applications`: Known applications, publishers, first seen, last seen, baseline metrics.
- `processes`: Observed processes, executable paths, parent PIDs, lifetime.
- `network_connections`: Connection history, local/remote endpoints, protocols, durations.
- `destinations`: Observed remote IPs/hostnames, aggregate transfer volumes, flags.
- `security_incidents`: Incidents detected, severity, confidence, affected entities.
- `incident_evidence`: Structured key-value evidence pairs supporting each incident.
- `recommendations`: Generated recommendations, safe/risky classification, rationale.
- `approved_actions`: User-approved actions with exact timestamps and signatures.
- `restoration_snapshots`: Pre-action snapshots, rollback parameters, expiration timestamps.
- `active_policies`: Current active firewall and process policies.
- `audit_log`: Tamper-evident log of every control action applied, modified, or restored.

---

## 5. Security & Privacy Guarantees
- **No Payload Inspection**: Only socket metadata (IP, port, protocol, bytes, state).
- **No HTTPS Interception**: Zero MITM proxies or certificate injection.
- **No Keystroke / Credential Capture**: Strictly restricted to system and network metrics.
- **No Telemetry Upload**: 100% local-first, zero third-party cloud calls.
- **Honest Telemetry**: Unsupported metrics explicitly display "Unavailable on this system" rather than synthetic data.
