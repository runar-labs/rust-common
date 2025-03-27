// runar_common/src/lib.rs
//
// Common traits and utilities for the Runar P2P stack

// Export modules
pub mod service_info;
pub mod macros;
pub mod utils;
pub mod types;

// Re-export traits and types at the root level
pub use service_info::ServiceInfo;

// Re-export the macros so they can be used with `use runar_common::vmap;` syntax
// These macros are already automatically available at the crate root namespace with #[macro_export]
// This is just to make imports cleaner and more intuitive

// NOTE: vmap_opt has been removed in favor of using None/Some(vmap!{}) 