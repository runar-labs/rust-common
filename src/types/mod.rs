// runar_common/src/types/mod.rs
//
// Type definitions for runar common

// Type modules
mod erased_arc;
pub mod schemas;
mod value_type;
mod vmap;

// Export our types
pub use self::erased_arc::ErasedArc;
pub use self::schemas::{ActionMetadata, EventMetadata, FieldSchema, SchemaDataType, ServiceMetadata};
pub use self::value_type::{ArcValueType, SerializerRegistry, ValueCategory};
pub use vmap::VMap;
// Export the implement_from_for_valuetype macro
#[macro_export]
macro_rules! implement_from_for_valuetype {
    ($t:ty, $variant:ident) => {
        impl From<$t> for $crate::types::ArcValueType {
            fn from(value: $t) -> Self {
                $crate::types::ArcValueType::new_primitive(value)
            }
        }
    };
}
