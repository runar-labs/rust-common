// runar_common/src/types/mod.rs
//
// Type definitions for runar common

// Type modules
mod erased_arc;
mod value_type;
mod value_typed;
mod vmap;

// Export our types
pub use self::erased_arc::ErasedArc;
pub use self::value_type::{ArcValueType, TypeRegistry, ValueCategory, ValueType};
pub use self::value_typed::{value_from_bytes, TypedValue};
pub use vmap::VMap;

// Export the implement_from_for_valuetype macro
#[macro_export]
macro_rules! implement_from_for_valuetype {
    ($t:ty, $variant:ident) => {
        impl From<$t> for $crate::types::ValueType {
            fn from(value: $t) -> Self {
                $crate::types::ValueType::$variant(value)
            }
        }
    };
}

// Export implementation details for advanced users
pub mod internal {
    pub use super::value_typed::{
        CustomStruct, MapValue, TypedBytes, Value, ValueBase, ValueConvert,
    };
}
