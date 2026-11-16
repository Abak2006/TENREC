// TENREC - Protection & Restoration Module
pub mod firewall;
pub mod guard;
pub mod process_ctrl;
pub mod snapshots;
pub mod types;

pub use snapshots::ProtectionManager;
pub use types::{ActionOutcome, ActivePolicy, ApplyResult, RestorationSnapshot, RestoreResult};
