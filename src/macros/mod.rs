// runar_common/src/macros/mod.rs
//
// Common macros that don't require procedural macro functionality

// This module will contain macros that are used commonly but don't require
// the complexity of procedural macros. These include simple utility macros,
// helper macros, and formatting macros.

// Note: Most complex macros should go in the rust-macros crate instead.

/// Create a ValueType::Map and for map creation and value extraction
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::vmap;
/// use runar_common::types::ValueType;
/// 
/// // Create a map
/// let map = vmap! {
///     "name" => "John Doe",
///     "age" => 30,
///     "is_admin" => true
/// };
/// 
/// // Extract values as ValueType objects
/// let name = vmap!(map, "name");  // Returns ValueType::String
/// let age = vmap!(map, "age");    // Returns ValueType::Number
/// ```
#[macro_export]
macro_rules! vmap {
    // Empty map creation
    {} => {
        {
            let map: std::collections::HashMap<String, $crate::types::ValueType> = std::collections::HashMap::new();
            $crate::types::ValueType::Map(map)
        }
    };

    // Map creation with entries
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

    // Simple key lookup with default ValueType::Null
    ($map:expr, $key:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut result = $crate::types::ValueType::Null;
            
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        if let Some(value) = m.get(*part) {
                            result = value.clone();
                        }
                        break;
                    } else {
                        // Continue traversing
                        if let Some($crate::types::ValueType::Map(_)) = m.get(*part) {
                            current_map = m.get(*part).unwrap().clone();
                        } else {
                            // Not a map or key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            result
        }
    };

    // Value extraction with default
    ($map:expr, $key:expr => $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_value = $crate::types::ValueType::from($default);
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut result = default_value.clone();
            
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        if let Some(value) = m.get(*part) {
                            result = value.clone();
                        }
                        break;
                    } else {
                        // Continue traversing
                        if let Some($crate::types::ValueType::Map(_)) = m.get(*part) {
                            current_map = m.get(*part).unwrap().clone();
                        } else {
                            // Not a map or key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            result
        }
    };
}

/// Extract a String value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_str};
/// 
/// let map = vmap! { 
///     "name" => "John Doe",
///     "profile" => vmap! {
///         "email" => "john@example.com"
///     } 
/// };
/// 
/// // Simple lookup
/// let name: String = vmap_str!(map, "name", "default");  // Returns "John Doe"
/// 
/// // Nested lookup with dot notation
/// let email: String = vmap_str!(map, "profile.email", ""); // Returns "john@example.com"
/// ```
#[macro_export]
macro_rules! vmap_str {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_str = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a string
            match final_value {
                Some($crate::types::ValueType::String(s)) => s,
                Some($crate::types::ValueType::Number(n)) => n.to_string(),
                Some($crate::types::ValueType::Bool(b)) => b.to_string(),
                _ => default_str.to_string()
            }
        }
    };
}

/// Extract an i32 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_i32};
/// 
/// let map = vmap! { 
///     "age" => 30,
///     "stats" => vmap! {
///         "score" => 42
///     }
/// };
/// 
/// // Simple lookup
/// let age: i32 = vmap_i32!(map, "age", 0);  // Returns 30
/// 
/// // Nested lookup with dot notation
/// let score: i32 = vmap_i32!(map, "stats.score", 0); // Returns 42
/// ```
#[macro_export]
macro_rules! vmap_i32 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an i32
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as i32,
                Some($crate::types::ValueType::String(s)) => s.parse::<i32>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract an f64 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_f64};
/// 
/// let map = vmap! { 
///     "score" => 98.5,
///     "stats" => vmap! {
///         "average" => 87.3
///     }
/// };
/// 
/// // Simple lookup
/// let score: f64 = vmap_f64!(map, "score", 0.0);  // Returns 98.5
/// 
/// // Nested lookup with dot notation
/// let avg: f64 = vmap_f64!(map, "stats.average", 0.0); // Returns 87.3
/// ```
#[macro_export]
macro_rules! vmap_f64 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_float = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an f64
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n,
                Some($crate::types::ValueType::String(s)) => s.parse::<f64>().unwrap_or(default_float),
                _ => default_float
            }
        }
    };
}

/// Extract a bool value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_bool};
/// 
/// let map = vmap! { 
///     "is_admin" => true,
///     "permissions" => vmap! {
///         "can_edit" => true
///     }
/// };
/// 
/// // Simple lookup
/// let is_admin: bool = vmap_bool!(map, "is_admin", false);  // Returns true
/// 
/// // Nested lookup with dot notation
/// let can_edit: bool = vmap_bool!(map, "permissions.can_edit", false); // Returns true
/// ```
#[macro_export]
macro_rules! vmap_bool {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_bool = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a bool
            match final_value {
                Some($crate::types::ValueType::Bool(b)) => b,
                Some($crate::types::ValueType::String(s)) => {
                    if s == "true" { true } 
                    else if s == "false" { false }
                    else { default_bool }
                },
                _ => default_bool
            }
        }
    };
}

/// Create a ValueType from JSON
#[macro_export]
macro_rules! vjson {
    ($($json:tt)+) => {
        $crate::types::ValueType::from(serde_json::json!($($json)+))
    };
}

/// Extract an i8 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_i8};
/// 
/// let map = vmap! { 
///     "small_num" => 42,
///     "nested" => vmap! {
///         "value" => 8
///     }
/// };
/// 
/// // Simple lookup
/// let val: i8 = vmap_i8!(map, "small_num", 0);  // Returns 42
/// 
/// // Nested lookup with dot notation
/// let nested_val: i8 = vmap_i8!(map, "nested.value", 0); // Returns 8
/// ```
#[macro_export]
macro_rules! vmap_i8 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an i8
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as i8,
                Some($crate::types::ValueType::String(s)) => s.parse::<i8>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract an i16 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_i16};
/// 
/// let map = vmap! { 
///     "medium_num" => 1000,
///     "nested" => vmap! {
///         "value" => 500
///     }
/// };
/// 
/// // Simple lookup
/// let val: i16 = vmap_i16!(map, "medium_num", 0);  // Returns 1000
/// 
/// // Nested lookup with dot notation
/// let nested_val: i16 = vmap_i16!(map, "nested.value", 0); // Returns 500
/// ```
#[macro_export]
macro_rules! vmap_i16 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an i16
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as i16,
                Some($crate::types::ValueType::String(s)) => s.parse::<i16>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract an i64 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_i64};
/// 
/// let map = vmap! { 
///     "large_num" => 9000000000_i64,
///     "nested" => vmap! {
///         "value" => 8000000000_i64
///     }
/// };
/// 
/// // Simple lookup
/// let val: i64 = vmap_i64!(map, "large_num", 0);  // Returns 9000000000
/// 
/// // Nested lookup with dot notation
/// let nested_val: i64 = vmap_i64!(map, "nested.value", 0); // Returns 8000000000
/// ```
#[macro_export]
macro_rules! vmap_i64 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an i64
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as i64,
                Some($crate::types::ValueType::String(s)) => s.parse::<i64>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract a u8 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_u8};
/// 
/// let map = vmap! { 
///     "byte" => 255,
///     "nested" => vmap! {
///         "value" => 128
///     }
/// };
/// 
/// // Simple lookup
/// let val: u8 = vmap_u8!(map, "byte", 0);  // Returns 255
/// 
/// // Nested lookup with dot notation
/// let nested_val: u8 = vmap_u8!(map, "nested.value", 0); // Returns 128
/// ```
#[macro_export]
macro_rules! vmap_u8 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a u8
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as u8,
                Some($crate::types::ValueType::String(s)) => s.parse::<u8>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract a u32 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_u32};
/// 
/// let map = vmap! { 
///     "unsigned" => 4294967295_u32,
///     "nested" => vmap! {
///         "value" => 123456789_u32
///     }
/// };
/// 
/// // Simple lookup
/// let val: u32 = vmap_u32!(map, "unsigned", 0);  // Returns 4294967295
/// 
/// // Nested lookup with dot notation
/// let nested_val: u32 = vmap_u32!(map, "nested.value", 0); // Returns 123456789
/// ```
#[macro_export]
macro_rules! vmap_u32 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a u32
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as u32,
                Some($crate::types::ValueType::String(s)) => s.parse::<u32>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract a u64 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_u64};
/// 
/// let map = vmap! { 
///     "big_unsigned" => 18446744073709551615_u64,
///     "nested" => vmap! {
///         "value" => 9223372036854775808_u64
///     }
/// };
/// 
/// // Simple lookup
/// let val: u64 = vmap_u64!(map, "big_unsigned", 0);  // Returns 18446744073709551615
/// 
/// // Nested lookup with dot notation
/// let nested_val: u64 = vmap_u64!(map, "nested.value", 0); // Returns 9223372036854775808
/// ```
#[macro_export]
macro_rules! vmap_u64 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_int = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a u64
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as u64,
                Some($crate::types::ValueType::String(s)) => s.parse::<u64>().unwrap_or(default_int),
                _ => default_int
            }
        }
    };
}

/// Extract an f32 value from a ValueType::Map
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_f32};
/// 
/// let map = vmap! { 
///     "float32" => 3.14159,
///     "nested" => vmap! {
///         "value" => 2.71828
///     }
/// };
/// 
/// // Simple lookup
/// let val: f32 = vmap_f32!(map, "float32", 0.0);  // Returns 3.14159 as f32
/// 
/// // Nested lookup with dot notation
/// let nested_val: f32 = vmap_f32!(map, "nested.value", 0.0); // Returns 2.71828 as f32
/// ```
#[macro_export]
macro_rules! vmap_f32 {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_float = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to an f32
            match final_value {
                Some($crate::types::ValueType::Number(n)) => n as f32,
                Some($crate::types::ValueType::String(s)) => s.parse::<f32>().unwrap_or(default_float),
                _ => default_float
            }
        }
    };
}

/// Extract a DateTime value from a ValueType::Map using the chrono crate
/// 
/// This macro requires the `chrono` feature to be enabled.
/// It parses dates in ISO 8601 format (e.g., "2023-05-01T14:30:00Z").
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_datetime};
/// use chrono::{DateTime, Utc};
/// 
/// let map = vmap! { 
///     "created_at" => "2023-05-01T14:30:00Z",
///     "user" => vmap! {
///         "last_login" => "2023-06-15T09:45:00Z"
///     }
/// };
/// 
/// // Simple lookup
/// let created: DateTime<Utc> = vmap_datetime!(map, "created_at", Utc::now());
/// 
/// // Nested lookup with dot notation
/// let last_login: DateTime<Utc> = vmap_datetime!(map, "user.last_login", Utc::now());
/// ```
#[macro_export]
macro_rules! vmap_datetime {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_date = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a DateTime
            match final_value {
                Some($crate::types::ValueType::String(s)) => {
                    match chrono::DateTime::parse_from_rfc3339(&s) {
                        Ok(dt) => dt.with_timezone(&chrono::Utc),
                        Err(_) => default_date
                    }
                },
                _ => default_date
            }
        }
    };
}

/// Extract a Date value from a ValueType::Map using the chrono crate
/// 
/// This macro requires the `chrono` feature to be enabled.
/// It parses dates in ISO 8601 format (e.g., "2023-05-01").
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_date};
/// use chrono::{NaiveDate, Utc};
/// 
/// let map = vmap! { 
///     "birth_date" => "1990-01-15",
///     "user" => vmap! {
///         "registration_date" => "2023-06-15"
///     }
/// };
/// 
/// // Simple lookup with default being today's date
/// let birth_date: NaiveDate = vmap_date!(map, "birth_date", Utc::now().date_naive());
/// 
/// // Nested lookup with dot notation
/// let reg_date: NaiveDate = vmap_date!(map, "user.registration_date", Utc::now().date_naive());
/// ```
#[macro_export]
macro_rules! vmap_date {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_date = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a Date
            match final_value {
                Some($crate::types::ValueType::String(s)) => {
                    match chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                        Ok(date) => date,
                        Err(_) => default_date
                    }
                },
                _ => default_date
            }
        }
    };
}

/// Extract a Vec<T> value from a ValueType::Map
/// 
/// This macro extracts an array from a map and converts it to a Vec<T>.
/// 
/// # Examples
/// 
/// ```rust
/// use runar_common::{vmap, vmap_vec};
/// 
/// let map = vmap! { 
///     "tags" => vec!["rust", "programming", "macros"],
///     "user" => vmap! {
///         "interests" => vec!["coding", "reading"]
///     }
/// };
/// 
/// // Simple lookup
/// let tags: Vec<String> = vmap_vec!(map, "tags", Vec::<String>::new());
/// assert_eq!(tags, vec!["rust", "programming", "macros"]);
/// 
/// // Nested lookup with dot notation
/// let interests: Vec<String> = vmap_vec!(map, "user.interests", Vec::<String>::new());
/// assert_eq!(interests, vec!["coding", "reading"]);
/// ```
#[macro_export]
macro_rules! vmap_vec {
    ($map:expr, $key:expr, $default:expr) => {
        {
            let map = $map.clone();
            let key = $key;
            let default_vec = $default;
            
            // Split the key by dots to handle nested lookups
            let key_parts: Vec<&str> = key.split('.').collect();
            let mut current_map = map;
            let mut final_value: Option<$crate::types::ValueType> = None;
            
            // Traverse the nested maps
            for (i, part) in key_parts.iter().enumerate() {
                if let $crate::types::ValueType::Map(m) = &current_map {
                    if i == key_parts.len() - 1 {
                        // Last part - get the value
                        final_value = m.get(*part).cloned();
                        break;
                    } else {
                        // Continue traversing
                        if let Some(next_map) = m.get(*part) {
                            if let $crate::types::ValueType::Map(_) = next_map {
                                current_map = next_map.clone();
                            } else {
                                // Not a map
                                break;
                            }
                        } else {
                            // Key not found
                            break;
                        }
                    }
                } else {
                    // Not a map
                    break;
                }
            }
            
            // Convert the final value to a Vec<T>
            match final_value {
                Some($crate::types::ValueType::Array(arr)) => {
                    let mut result = Vec::with_capacity(arr.len());
                    for item in arr {
                        match item {
                            $crate::types::ValueType::String(s) => result.push(s),
                            $crate::types::ValueType::Number(n) => result.push(n.to_string()),
                            $crate::types::ValueType::Bool(b) => result.push(b.to_string()),
                            _ => {}
                        }
                    }
                    result
                },
                _ => default_vec
            }
        }
    };
}

// NOTE: The vmap_opt macro has been removed. 
// Use None instead of vmap_opt! {} 
// Use Some(vmap! { key => value }) instead of vmap_opt! { key => value }
