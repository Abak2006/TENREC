# Architecture

```
┌──────────── React UI (src/) ────────────┐        ┌──────────────── Rust core (src-tauri/src/) ─────────────────┐
│ lib/store.ts  polls every 2 s (live)    │  IPC   │ ipc/commands.rs   thin commands, read latest sample          │
│               and 15 s (protection)     │ ─────► │ telemetry.rs      sampler thread, one tick every 2 s:        │
│ pages/*       one file per page         │        │   processes → sockets → apps → correlation → baseline →      │
│ components/action-review.tsx            │        │   security rules → system metrics → throughput → publish     │
│   asks the backend to build actions,    │        │ security/         rules + action registry (IDs only)         │
│   sends back only action IDs            │        │ protection/       guard → firewall / process_ctrl → snapshots│
└─────────────────────────────────────────┘        │ persistence/      SQLite: samples, undo records, audit log   │
                                                   └──────────────────────────────────────────────────────────────┘
```

## Sampling

`TelemetryHub` (telemetry.rs) owns one background thread. Every 2 seconds it:

1. Refreshes the process table once (sysinfo). CPU is normalised by logical core count, and I/O rates come from per-process I/O counters.
2. Reads TCP/UDP tables for IPv4 and IPv6 (`GetExtendedTcpTable` / `GetExtendedUdpTable`) and attributes them to processes from step 1.
3. Groups processes into applications by executable path. Display names and company names come from the file's version resource.
4. Runs correlation (what changed), the per-app baseline (exponential moving average of CPU and I/O), and the security rules.
5. Reads CPU/memory/power, plus drives every 30 s and interface throughput, then publishes an immutable `Arc<Sample>`.

IPC read commands clone from the latest sample, so they are cheap and consistent with each other, and they never block the UI thread. Panics inside a tick are caught and reported through `get_status().lastError`. A sample is written to SQLite every 30 s, and samples older than 7 days are purged at startup.

## Detection to action

```
rule fires → SecurityIncident + ProposedAction{id, target}   (registry in SecurityEngine)
user opens review → UI fetches actions by incident or asks backend to prepare one (pid/app id)
user approves     → apply_actions([ids])  → registry lookup → guard re-validates → execute
                  → ChangeRecord with undo data → RestorationSnapshot persisted → audit entry
undo              → per change: firewall delete / restore previous priority / resume
```

`ActionTarget` is an enum built only by the backend: `BlockRemoteIp`, `BlockProgram`, `BlockInboundPort`, `LowerPriority` and `SuspendProcess`. The webview never supplies an IP, path, port or PID that gets executed. It can only reference an action ID the backend created.

Process actions carry the process creation time. Before applying or undoing, TENREC checks that the PID still belongs to that same process instance, because Windows reuses PIDs.
