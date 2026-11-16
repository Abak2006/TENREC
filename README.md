# TENREC

**See what your Windows PC is doing, and why.** TENREC is a local-first desktop app that links processes, network connections and resource use, flags suspicious behaviour with plain-language explanations, and lets you block or pause things with changes you can always undo.

Version 0.2 · Windows 10 (19041+) / 11 · Tauri 2 + Rust + React

## What it does

- **Processes and applications**: every process with CPU (as a share of all cores), memory, I/O rate, open connections, and the company name from the executable's version info. Processes are grouped into applications by executable.
- **Network**: live TCP connections and TCP/UDP listening ports, IPv4 and IPv6, each mapped to its owning process. It also shows which ports are reachable from other machines, which public addresses each app has contacted, and total upload/download rates.
- **What changed**: new applications, new public destinations and new listening ports since TENREC started. What was already running at startup is treated as the baseline, not as a change.
- **Security rules**: four explainable heuristics, each listing its known false positives:
  - script hosts (PowerShell, cmd, mshta, rundll32, certutil…) connecting to public IPs
  - programs running from Downloads or Temp that go online
  - programs running from Downloads or Temp that accept connections
  - listeners on common backdoor ports
- **Reversible protection**:
  - block an IP, block a program's internet access, or block an inbound port (Windows Firewall)
  - lower a process's priority, or suspend it
  - every change goes through a review dialog, is recorded in a local SQLite database, and can be undone, including after a restart
  - **Undo everything** also removes leftover `TENREC_RULE_*` firewall rules

## What it does not do (yet)

These are limits of the current version, and the app says so wherever they apply:

- **No per-process bandwidth.** Windows needs ETW for that.
- **No hostname lookups.** TENREC sends no traffic of its own.
- **No GPU utilisation, temperatures or fan data.**
- **No malware detection.** The security rules are heuristics, and normal software can trigger them.
- **Company names are not checked against signatures.** They are what the file claims.
- **Partial data without admin rights.** Windows hides CPU and I/O for service and system processes unless TENREC runs as administrator. Those show as "n/a".
- **Firewall actions need administrator rights.** Process priority and suspend work as a standard user for your own processes.

## Run it

Download the installer from the [Releases](../../releases) page (when published). The installers are not code-signed yet, so Windows SmartScreen will warn on first run.

For firewall actions, right-click TENREC and choose **Run as administrator**.

## Develop

Prerequisites:

- Node.js 24+
- Rust 1.77+ with the MSVC toolchain
- Microsoft Edge WebView2 (included in Windows 11)

```bash
npm install
npm run tauri dev        # app with hot reload (Vite on :5173)
npm run build            # type check + frontend production build
npm run lint             # oxlint
cd src-tauri && cargo test && cargo clippy --all-targets
npm run tauri build      # MSI + NSIS installers in src-tauri/target/release/bundle
```

The tests run against the real Windows APIs. One test temporarily lowers the priority of a child `ping` process it starts itself, then restores it. No test changes firewall rules.

## Data and privacy

Everything stays in `%LOCALAPPDATA%\TENREC\tenrec.db`:

- one system sample every 30 s, kept for 7 days
- undo records
- an audit log of every action

TENREC makes no network requests. See [PRIVACY.md](PRIVACY.md) and [SECURITY.md](SECURITY.md).

## Docs

- [ARCHITECTURE.md](ARCHITECTURE.md): how sampling, detection and the action pipeline fit together
- [SECURITY.md](SECURITY.md): trust boundaries and the safety rules for privileged actions
- [WINDOWS.md](WINDOWS.md): Windows APIs used
- [DEVELOPMENT.md](DEVELOPMENT.md) and [TESTING.md](TESTING.md)

## License

Proprietary. All rights reserved.
