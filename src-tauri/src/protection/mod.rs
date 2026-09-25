// TENREC - Protection & Restoration Module
pub mod firewall;
pub mod process_ctrl;
pub mod snapshots;
pub mod types;

pub use firewall::WindowsFirewall;
pub use process_ctrl::ProcessController;
pub use snapshots::ProtectionManager;
pub use types::{ActivePolicy, ApplyResult, RestorationSnapshot, RestoreResult};
