# TENREC Testing & Verification Suite

## Test Architecture
1. **Unit Tests (`src-tauri/src/tests/`):**
   - System telemetry parsing (Memory, CPU, Disk, Battery).
   - Network socket parsing and endpoint conversion.
   - Process-network correlation mappings.
   - Baseline statistical calculations (moving averages, deviations).
   - Security detection algorithms (outbound spikes, new destinations, port bursts).
   - Recommendation engine risk classification (SAFE vs RISKY invariants).
   - Approval engine staging and validation.
   - Restoration snapshot creation and serialization.

2. **Safety Invariant Tests:**
   - **Invariant 1**: Detectors cannot invoke protection code directly.
   - **Invariant 2**: Risky actions are never selected by default.
   - **Invariant 3**: No action can be applied without explicit user approval struct.
   - **Invariant 4**: Every applied action records a restoration snapshot and audit entry.
   - **Invariant 5**: Path and command injections are rejected by IPC validators.

3. **Running Tests:**
```powershell
cd src-tauri
cargo test -- --nocapture
```
