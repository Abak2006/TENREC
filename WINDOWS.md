# Windows APIs used

| Area | API | Notes |
|---|---|---|
| CPU total | `GetSystemTimes` | Delta of idle vs kernel+user time |
| CPU per core, per process | sysinfo (PDH / `GetProcessTimes`) | Per-process value divided by logical cores |
| Process I/O | sysinfo (`GetProcessIoCounters`) | Includes file, device and network I/O |
| Process identity | `GetProcessTimes` (creation time) | Guards against PID reuse |
| Executable metadata | `GetFileVersionInfoW`, `VerQueryValueW` | CompanyName / FileDescription, not signature-verified |
| Memory | `GlobalMemoryStatusEx` | |
| Drives | `GetLogicalDrives`, `GetDriveTypeW`, `GetDiskFreeSpaceExW`, `GetVolumeInformationW` | Fixed and removable drives only (network drives can block) |
| Power | `GetSystemPowerStatus` | |
| Sockets | `GetExtendedTcpTable`, `GetExtendedUdpTable` | `AF_INET` and `AF_INET6`, owner PID tables |
| Interface throughput | sysinfo (`GetIfTable2`) | Sum over all adapters |
| Hardware identity | Registry `HKLM\HARDWARE\DESCRIPTION\System\BIOS`, display class key | |
| Elevation | `OpenProcessToken`, `GetTokenInformation(TokenElevation)` | |
| Priority | `GetPriorityClass`, `SetPriorityClass` | Previous class stored for undo |
| Suspend / resume | `NtSuspendProcess`, `NtResumeProcess` (ntdll) | Undocumented but long-stable |
| Firewall | `netsh advfirewall firewall add/delete rule` | Rules named `TENREC_RULE_*` only |
| Firewall cleanup | PowerShell `Get-NetFirewallRule -DisplayName 'TENREC_RULE_*' \| Remove-NetFirewallRule` | Fixed script; admin only |

Without administrator rights, `OpenProcess` is denied for most service and system processes. TENREC then cannot read their path, CPU or I/O, and marks them as restricted ("n/a") instead of showing 0.
