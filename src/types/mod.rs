// runar_common/src/types/mod.rs
//
// Type definitions for runar common

mod value_type;
mod value_typed;
mod vmap;

// Re-export types
pub use value_type::*;
pub use vmap::*;

// Export the public TypedValue API as the primary interface
pub use value_typed::{value_from_bytes, TypeInfo, TypedValue};

// Export implementation details for advanced users
pub mod internal {
    pub use super::value_typed::{
        CustomStruct, MapValue, TypedBytes, Value, ValueBase, ValueConvert,
    };
}
