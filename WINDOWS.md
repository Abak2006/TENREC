# TENREC Windows Platform Engineering Guide

## 1. Supported Versions
- Primary Target: **Windows 11 (64-bit)**
- Compatible: **Windows 10 (Build 19041+)**

## 2. Real Windows APIs Used
TENREC relies exclusively on real, stable Windows APIs:

- **CPU Telemetry:**
  - `GetSystemTimes` for precise kernel and user time ticks.
  - Performance counters via sysinfo.
- **Memory Telemetry:**
  - `GlobalMemoryStatusEx` (`MEMORYSTATUSEX`) for total physical RAM, available RAM, and memory load percentage.
- **Storage / Disk Telemetry:**
  - `GetDiskFreeSpaceExW` for real byte-accurate volume capacity and free space.
  - `GetLogicalDriveStringsW` for enumerating active logical drives.
- **Power & Battery:**
  - `GetSystemPowerStatus` (`SYSTEM_POWER_STATUS`) for AC line status, battery flag, and battery life percentage.
- **Network Telemetry:**
  - `GetExtendedTcpTable` (`MIB_TCPTABLE_OWNER_PID`) for mapping active TCP connections to owning PIDs.
  - `GetExtendedUdpTable` (`MIB_UDPTABLE_OWNER_PID`) for mapping active UDP endpoints to owning PIDs.
  - `GetIfTable2` / `MIB_IF_ROW2` for network interface throughput (InOctets, OutOctets, Discards, Errors).
- **Process Management:**
  - `CreateToolhelp32Snapshot`, `Process32FirstW`, `Process32NextW` for process enumeration and parent PID tracking.
  - `OpenProcess`, `GetProcessMemoryInfo` for process working set and memory.
  - `QueryFullProcessImageNameW` for executable binary paths.
- **Protection & Firewall:**
  - Windows Advanced Firewall (`netsh advfirewall firewall`) with strictly prefixed rules (`TENREC_RULE_*`).
  - Native process thread control APIs (`OpenProcess`, `NtSuspendProcess`, `NtResumeProcess`) for safe process control with elevation checks.
