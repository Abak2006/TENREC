# TENREC

> "Know what your computer is doing. Understand why. Decide what it should do."

TENREC is a clean-slate, local-first Windows system intelligence and control platform.

---

## What Differentiates TENREC?
TENREC does not merely display isolated numbers. It dynamically correlates:
$$\text{System State} \longleftrightarrow \text{Process} \longleftrightarrow \text{Application} \longleftrightarrow \text{Socket} \longleftrightarrow \text{Destination} \longleftrightarrow \text{Baseline} \longleftrightarrow \text{Security Context}$$

### The Control Principle
**TENREC never modifies your system autonomously.**
1. Detectors produce **evidence**, not direct actions.
2. The recommendation engine categorizes changes into **SAFE** (low-impact, reversible) and **RISKY** (interruption potential).
3. Safe actions are selected by default; risky actions are unselected by default.
4. The user reviews, edits, and explicitly approves every change.
5. Every applied change creates a restoration snapshot, enabling instant **"Restore Everything"** rollback.

---

## Architecture
- **Desktop Shell:** Tauri v2 + React 19 + TypeScript + Vite
- **Native Backend:** Rust with direct Windows API integration (`GetSystemTimes`, `GlobalMemoryStatusEx`, `GetDiskFreeSpaceExW`, `GetSystemPowerStatus`, `GetExtendedTcpTable`, `GetIfTable2`)
- **Persistence:** Local SQLite database with migrations (`rusqlite`)
- **Telemetry Integrity:** 100% real data. No mock numbers in production. Unsupported hardware metrics are honestly reported as unavailable.
