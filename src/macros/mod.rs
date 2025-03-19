// runar_common/src/macros/mod.rs
//
// Common macros that don't require procedural macro functionality

// This module will contain macros that are used commonly but don't require
// the complexity of procedural macros. These include simple utility macros,
// helper macros, and formatting macros.

// Note: Most complex macros should go in the rust-macros crate instead.

/// Simple macro to create a HashMap with ValueType values
/// 
/// This is a simplified version of the vmap! macro that will be defined in rust-macros.
/// This version only supports the basic creation of a map, without the extraction functionality.
/// 
/// # Examples
/// 
/// ```
/// use runar_common::simple_vmap;
/// use runar_common::types::ValueType;
/// 
/// // Create a map with key-value pairs
/// let map = simple_vmap! {
///     "name" => "John Doe",
///     "age" => 30,
///     "is_admin" => true
/// };
/// 
/// // Create an empty map
/// let empty = simple_vmap! {};
/// ```
#[macro_export]
macro_rules! simple_vmap {
    // Empty map
    {} => {
        {
            let map: std::collections::HashMap<String, $crate::types::ValueType> = std::collections::HashMap::new();
            $crate::types::ValueType::Map(map)
        }
    };

    // Map with entries
    {
        $($key:expr => $value:expr),* $(,)?
    } => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.to_string(), $crate::types::ValueType::from($value));
            )*
            $crate::types::ValueType::Map(map)
        }
    };
}
