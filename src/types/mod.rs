// runar_common/src/types/mod.rs
//
// Type definitions for runar common

// Type modules
mod erased_arc;
mod value_type;
mod value_typed;
mod vmap;

// Re-export the original ValueType
pub use value_type::SerializableStruct;
pub use value_type::ValueType;

// Re-export the macro for the original ValueType
// #[macro_export] places it at the crate root
pub use crate::implement_from_for_valuetype;

// Re-export the new ArcValueType implementation
pub use value_type::ArcValueType;
pub use value_type::ValueCategory;

// Re-export other types
pub use value_typed::value_from_bytes;
pub use value_typed::TypedValue;
pub use vmap::VMap;

// Export implementation details for advanced users
pub mod internal {
    pub use super::value_typed::{
        CustomStruct, MapValue, TypedBytes, Value, ValueBase, ValueConvert,
    };
}
