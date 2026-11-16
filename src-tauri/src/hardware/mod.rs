// TENREC - Hardware Extensibility Subsystem
pub mod registry;
pub mod types;
pub mod win_reg;

pub use registry::HardwareRegistry;
pub use types::{
    AccessMode, CapabilityState, ComponentCategory, HardwareCapabilityItem, HardwareDiagnostics,
};
