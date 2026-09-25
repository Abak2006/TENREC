# TENREC Privacy Manifesto

TENREC is strictly **local-first** and privacy-respecting:

1. **Zero External Telemetry**: TENREC never phones home, sends usage statistics, or transmits telemetry to any external servers.
2. **No Payload Inspection**: TENREC inspects only socket metadata (IP, port, protocol, byte counters, TCP states). Application payloads, packet contents, and decrypted TLS streams are never inspected or captured.
3. **No Credential Harvesting**: TENREC does not touch credentials, browser cookies, keystrokes, clipboard data, or private keys.
4. **Local SQLite Persistence**: All baselines, connection records, process logs, and incidents stay exclusively on your local machine in `%LOCALAPPDATA%/tenrec/tenrec.db`.
5. **No Cloud Dependencies**: No user accounts, registration, license servers, or internet connection required for full functionality.
