# TENREC

**See what your Windows PC is doing, and why.** TENREC is a local-first desktop app that links processes, network connections and resource use, flags suspicious behaviour with plain-language explanations, and lets you block or pause things with changes you can always undo.

Version 0.3 · Windows 10 (19041+) / 11 · Tauri 2 + Rust + React

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

### Control

- **Focus mode**: pick one app and give it priority.
  - The focus app gets High priority, and its efficiency mode is turned off.
  - Other apps you run get Windows efficiency mode and lower CPU, disk and memory priority. On hybrid CPUs they can be kept on the efficiency cores.
  - Optionally, the focus app is set to use the high-performance GPU.
  - Optionally, the Windows power mode is switched for the session.
  - With admin rights, chosen apps can be blocked from the network or have their upload capped.
  - Apps you exclude are left alone, along with core Windows processes. Processes that start during the session are handled too.
  - Ending focus undoes everything. If TENREC closes unexpectedly, the next launch undoes it.
- **Startup**:
  - Turn startup apps on or off from the Run keys and Startup folders. This uses the same switch as Task Manager.
  - Turn third-party scheduled tasks on or off. Windows' own `\Microsoft\` tasks are never shown or changed.
- **Privacy**:
  - Twelve settings, each writing the same value the Settings app or a documented Group Policy writes: advertising ID, tailored experiences, Start suggestions and tips, silent app installs, language list, app launch tracking, online speech, feedback requests, web results in Start, activity history, optional diagnostic data.
  - A list of connections to Microsoft diagnostic endpoints, matched using the local Windows DNS cache.
- **Maintenance**:
  - Switch the power mode and power plan.
  - Clean up your temp files, Windows temp files, Windows Update downloads, crash dumps and the Recycle Bin. A scan comes first, and each category needs confirmation.

Every change in this section except file cleanup is recorded and can be undone from the Protection page.

## What it does not do (yet)

These are limits of the current version, and the app says so wherever they apply:

- **No per-process bandwidth.** Windows needs ETW for that.
- **No DNS queries of its own.** Hostnames come only from the Windows DNS cache. Some VPN or secure-DNS clients (Cloudflare WARP, for example) bypass that cache; then no hostnames are shown.
- **No GPU utilisation, temperatures or fan data.**
- **No malware detection.** The security rules are heuristics, and normal software can trigger them.
- **Company names are not checked against signatures.** They are what the file claims.
- **Partial data without admin rights.** Windows hides CPU and I/O for service and system processes unless TENREC runs as administrator. Those show as "n/a".
- **Some actions need administrator rights:**
  - firewall actions
  - Focus network options
  - all-users startup entries
  - machine-wide privacy policies
  - Windows temp and Update cleanup

  Everything else works as a standard user.
- **Upload limits only.** Windows can only limit an app's *outgoing* bandwidth this way. It cannot reserve download speed for the focus app.
- **Deleted files cannot be restored.** Cleanup is the one irreversible action, which is why it always asks first.

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
