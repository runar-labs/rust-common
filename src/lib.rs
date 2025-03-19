// runar_common/src/lib.rs
//
// Common traits and utilities for the Runar P2P stack

// Re-export modules
pub mod service_info;
pub mod macros;
pub mod utils;
pub mod types;

// Re-export traits and types at the root level
pub use service_info::ServiceInfo;
