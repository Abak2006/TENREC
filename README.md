# TENREC — Version 0.1

> **"Know what your computer is doing. Understand why. Decide what it should do."**

TENREC is a local-first, native Windows system intelligence and control platform. It provides continuous, transparent visibility into the relationships between system hardware, processes, network sockets, applications, and behavioral anomalies—while strictly enforcing a user-first control model.

---

## 1. Product Vision & Core Philosophy

TENREC is built on the principle that computer monitoring and control should be **transparent**, **correlated**, and **revisable**. Most tools either show disjointed graphs (Task Manager, Resource Monitor) or perform opaque, autonomous actions that destabilize workloads (overzealous antivirus or optimization utilities).

TENREC bridges that divide:
1. **Correlation Over Isolation:** Rather than reporting raw numbers, TENREC connects hardware resource consumption directly to process trees, application identities, socket endpoints, and behavioral baselines:
   $$\text{Hardware (CPU/RAM/Disk/Power)} \longleftrightarrow \text{Process} \longleftrightarrow \text{Application} \longleftrightarrow \text{Socket} \longleftrightarrow \text{Remote Destination} \longleftrightarrow \text{Baseline Deviation}$$
2. **Local-First & Zero Cloud Telemetry:** 100% of telemetry, baselines, and logs remain on your local machine. Nothing is transmitted externally.
3. **Honest Telemetry:** Real Windows API metrics only. No mock telemetry, synthetic metrics, or extrapolated estimates. When a metric is unsupported or restricted, TENREC honestly reports it as unavailable.
4. **The Inviolable Control Pipeline:**
   - **Detection code never modifies system state autonomously.**
   - Detectors only generate structured **evidence**.
   - The recommendation engine proposes actions categorized into **SAFE** (low risk, non-disruptive, selected by default) and **RISKY** (intrusive, unselected by default).
   - The user inspects the rationale, expected benefits, and potential consequences in an interactive review modal before approving.
   - Every change takes an atomic **Restoration Snapshot** before application.
   - Any single action or all applied policies can be reverted with one click (**"Restore Everything"**).

---

## 2. What is Included in Version 0.1

Version 0.1 delivers the foundational end-to-end intelligence and control pipeline:

### A. Real-Time Native Windows Telemetry
- **CPU Subsystem:** Hardware core and logical core counts, kernel vs. user tick delta measurement via Win32 `GetSystemTimes`, per-core utilization breakdown, and high-resolution sampling.
- **Memory Subsystem:** Total, used, available, and percentage RAM utilization directly from Win32 `GlobalMemoryStatusEx`.
- **Disk Subsystem:** Volume enumeration and disk geometry via Win32 `GetDiskFreeSpaceExW`, read/write I/O transfer rates, and capacity indicators.
- **Power & Battery Subsystem:** Real-time AC line connection status, battery discharge level, and power status reporting via Win32 `GetSystemPowerStatus`.
- **Hardware Capability Verification:** Honest capability detection (flagging unavailable vendor-specific sensors like non-elevated NVML/AMD GPU thermal registers).

### B. Network Intelligence & Socket Mapping
- **Extended MIB Sockets:** Real-time enumeration of TCP and UDP connections with PID attribution via Win32 `GetExtendedTcpTable` and `GetExtendedUdpTable`.
- **Throughput Tracker:** Live interface traffic delta rates (bytes in/sec, bytes out/sec) and total bytes counter.
- **Destination Aggregator:** Automatic grouping and tracking of remote destination endpoints, reverse DNS caching, and unusual connection flags.
- **Listening Ports:** Comprehensive detection of local listening sockets, port numbers, bind addresses (`0.0.0.0` vs `127.0.0.1`), and known service identification.

### C. Process & Application Identity
- **Process Enumeration:** Complete process hierarchy capturing PID, Parent PID (PPID), binary paths, memory working sets, and CPU consumption.
- **Publisher & Signature Deduction:** Heuristic and binary path attribution identifying verified operating system components, vendor software, and unsigned binaries.
- **Application Clustering:** Multi-process consolidation (e.g., grouping multi-process browser instances or developer tools under a unified application identity).

### D. Correlation Engine & "What Changed?"
- **Live Correlation:** Dynamic cross-referencing between active sockets and process identities.
- **Timeline Event Stream:** Instant chronological logging of newly observed applications, newly contacted external destinations, and freshly bound listening ports.

### E. Behavioral Baseline Engine
- **Moving Average Profiling:** Statistical tracking of per-application CPU utilization, memory footprints, and network transfer rates.
- **Adaptive State Classification:** Automatic transitions between `learning` ($< 10$ samples), `normal`, and `divergent` ($\ge 3\times$ baseline deviation).

### F. Security Detection & Structured Evidence
- **Heuristic Detectors:** Identification of scripting hosts (`powershell.exe`, `cmd.exe`, `wscript.exe`, etc.) initiating outbound public IP connections, anomalous outbound data volume surges, and unverified services binding to `0.0.0.0`.
- **Structured Evidence:** Clear, transparent key-value pairs (observed value, baseline value, and human-readable explanation) presented for every detected incident.

### G. Protection, Snapshots & Reversibility
- **Scoped Windows Firewall Management:** Programmatic rule deployment targeting specific remote endpoints using the strict invariant prefix `TENREC_RULE_*`, ensuring user rules are never altered.
- **Process Control:** Thread execution suspension via native `NtSuspendProcess` / `NtResumeProcess` and dynamic priority class throttling (`SetPriorityClass` to `IDLE_PRIORITY_CLASS`).
- **Restoration Snapshots:** Full capture of pre-action state before applying approved mitigations.
- **Instant Rollback:** Individual snapshot rollback and global **"Restore Everything"** resetting all firewall rules, process priorities, and thread states.

### H. Persistence & Audit Log
- **Local SQLite Engine:** Bundled SQLite database in `%LOCALAPPDATA%\TENREC\tenrec.db` utilizing Write-Ahead Logging (WAL) for 0-latency concurrent reads.
- **Immutable Audit Trail:** Complete audit logging tracking every user review, approved action, and rollback event.
- **Configurable Retention:** Automated pruning of historical telemetry samples while preserving security incidents and restoration history.

### I. Dark Technical UI Console
- **React 19 + TypeScript + Tailwind:** High-density dark technical console styled with custom scrollbars, metric cards, status pills, and interactive data tables.
- **Canvas-Based Graphs:** Smooth, 60fps canvas-rendered multi-series telemetry graph without heavy external charting libraries.
- **Action Review Modal:** Dedicated approval modal supporting Presets (*Conservative*, *Balanced*, *Advanced*), granular checkbox selection, and risk disclosures.

---

## 3. Technology Stack

| Layer | Technologies |
| :--- | :--- |
| **Desktop Shell** | [Tauri v2](https://v2.tauri.app/) (Rust core + WebView2) |
| **Backend Runtime** | Rust 1.77+ (`stable-x86_64-pc-windows-gnu` / MSYS2 UCRT64 GCC or MSVC) |
| **Windows APIs** | `windows-sys 0.59` (`Win32_Foundation`, `Threading`, `IpHelper`, `WinSock`, `FileSystem`, `Power`) |
| **Local Database** | [rusqlite](https://github.com/rusqlite/rusqlite) 0.33 with bundled SQLite |
| **Frontend Framework** | React 19, TypeScript, Vite |
| **UI Styling** | Custom Dark Technical Design System |

---

## 4. Getting Started

### Prerequisites
1. **Windows 10 / 11 (64-bit)**
2. **Node.js** (LTS v20+ or v24+) & `npm`
3. **Rust Toolchain** (`rustup default stable-x86_64-pc-windows-gnu` or `stable-x86_64-pc-windows-msvc`)
4. **C Compiler / Linker** (MSYS2 UCRT64 GCC or Visual Studio Build Tools)
5. **Microsoft Edge WebView2** (pre-installed on modern Windows)

### Running in Development Mode
```powershell
# Install frontend dependencies
npm install

# Run Vite dev server + Tauri native shell with live reload
npm run tauri dev
```

### Running the Subsystem Tests
```powershell
cd src-tauri
cargo test --test tenrec_tests -- --nocapture
```

### Building the Release Executable
```powershell
# Build frontend assets and native binary
npm run build
cd src-tauri
cargo build --release
```
The compiled standalone binary will be located at:
```
src-tauri/target/release/tenrec.exe
```

---

## 5. Architectural Documents

For deep-dive technical specifications, consult the project documentation:
- [**ARCHITECTURE.md**](ARCHITECTURE.md) — System boundaries, dataflow pipelines, and IPC protocol.
- [**SECURITY.md**](SECURITY.md) — Privilege boundaries, firewall prefix invariants, and audit policies.
- [**PRIVACY.md**](PRIVACY.md) — Zero-telemetry declaration and local data residency guarantees.
- [**WINDOWS.md**](WINDOWS.md) — Comprehensive Win32 API mappings and data structures.
- [**DEVELOPMENT.md**](DEVELOPMENT.md) — Development workflow, debugging, and toolchain setup.
- [**TESTING.md**](TESTING.md) — Test architecture, unit suites, and invariant verification.

---

## 6. Future Scope & Roadmap

The following capabilities are planned for upcoming releases:

### Version 0.2: Deep Kernel Telemetry & Precision Filtering
- **Event Tracing for Windows (ETW):** Kernel trace provider integration (`Microsoft-Windows-Kernel-Process`, `Microsoft-Windows-Kernel-Network`, `Microsoft-Windows-Kernel-Disk`) for microsecond-accurate I/O accounting without polling overhead.
- **Windows Filtering Platform (WFP) Lightweight Driver:** Transition from `netsh` firewall manipulation to a non-intrusive WFP callout filter, enabling byte-accurate traffic throttling and per-connection prompts.
- **Hardware Sensor Integration:** Optional elevation bridge for NVML (NVIDIA) and ADLX (AMD) hardware interfaces to read dedicated GPU core clocks, VRAM thermals, and board power consumption.
- **Self-Hostable Scheduled Polling:** Background tray service mode allowing continuous baseline recording while the primary UI window is closed.

### Version 0.3: Behavioral Machine Learning & Forensic Bundles
- **Local Unsupervised Anomaly Detection:** Lightweight, on-device anomaly classification (e.g. Isolation Forests / local rolling PCA) operating strictly on local SQLite time-series without external network calls.
- **Encrypted Forensic Export Bundles:** Single-click export of an incident’s complete telemetry context, network packet headers, process tree, and audit signatures into an encrypted diagnostic package.
- **Application Reputation Database (Local Cache):** Optional offline hash database (Authenticode verification + local trust store) to minimize baseline training time for common developer tools and system software.

### Version 0.4: Advanced System & Power Policies
- **Intelligent Power Scheme Modulation:** Context-aware switching of Windows power plans (e.g. automatically moving background rendering processes to Efficiency cores or low-power states when running on battery).
- **Service & Task Scheduler Auditing:** Live monitoring of Windows Services and Scheduled Tasks that create background network connections or consume wake timers.
- **Enterprise Group Policy Compatibility:** Verification and non-interference checks to ensure user-approved TENREC policies do not conflict with enterprise MDM or Active Directory policies.

---

## 7. License

Proprietary — TENREC Core Team. All rights reserved.
