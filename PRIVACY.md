# Privacy

- **No network activity of its own.** No telemetry, update checks, accounts, or DNS/hostname lookups.
- **Metadata only.** TENREC reads socket tables (addresses, ports, owning PID, TCP state). It never captures packets or inspects payloads.
- **What is stored**, in `%LOCALAPPDATA%\TENREC\tenrec.db` (your user profile only):
  - one system-wide sample every 30 seconds (CPU, memory, I/O, network rates, battery), kept for 7 days
  - undo records for changes you approved
  - an audit log of actions
- **What is not stored.** Process lists, connection lists, destinations and incidents are kept in memory only, and are gone when TENREC closes.
- **Deleting data.** Settings → Stored data deletes samples. Deleting the folder above removes everything.
