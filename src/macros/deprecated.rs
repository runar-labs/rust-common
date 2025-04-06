// runar_common/src/macros/deprecated.rs
//
// Deprecated macros that are maintained for backward compatibility

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
/// 
/// # Deprecation Notice
/// 
/// This macro is deprecated in favor of the more powerful `vmap!` macro.
/// Please use `vmap!` for all new code.
#[deprecated(since = "0.1.0", note = "please use the `vmap!` macro instead")]
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