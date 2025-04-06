// runar_common/src/macros/vmap_macros.rs
//
// This file contains macros for working with ValueType maps.

/// Create a HashMap with ValueType values
/// 
/// This macro allows for easy creation of parameter maps for service requests.
/// 
/// # Examples
/// 
/// ```
/// use runar_common::vmap;
/// use runar_common::types::ValueType;
/// 
/// let map = vmap! {
///     "name" => "John Doe",
///     "age" => 30,
///     "is_admin" => true
/// };
/// 
/// // Create an empty map
/// let empty = vmap! {};
/// ```
///
/// ```ignore
/// // Extract a value from a map with default
/// let payload = ValueType::Map(std::collections::HashMap::new());
/// let data = vmap!(payload, "data" => String::new());
///
/// // Extract a direct value with default
/// let response = ValueType::String("test".to_string());
/// let value = vmap!(response, => "default");
/// ```
#[macro_export]
macro_rules! vmap {
    // Empty map
    {} => {
        {
            use std::collections::HashMap;
            use $crate::types::ValueType;
            let map: HashMap<String, ValueType> = HashMap::new();
            ValueType::Map(map)
        }
    };
    
    // Map with key-value pairs
    { $($key:expr => $value:expr),* $(,)? } => {
        {
            use std::collections::HashMap;
            use $crate::types::ValueType;
            let mut map = HashMap::new();
            $(
                map.insert($key.to_string(), ValueType::from($value));
            )*
            ValueType::Map(map)
        }
    };
    
    // Extract a value from a map with default
    ($map:expr, $key:expr => $default:expr) => {
        {
            match &$map {
                $crate::types::ValueType::Map(map_data) => {
                    match map_data.get($key) {
                        Some(value_type) => match value_type {
                            $crate::types::ValueType::String(s) => {
                                // Use type_name_of_val to detect default type
                                let default_type = std::any::type_name_of_val(&$default);
                                if default_type.ends_with("&str") || default_type.ends_with("String") {
                                    s.clone()
                                } else {
                                    $default
                                }
                            },
                            $crate::types::ValueType::Number(n) => {
                                // Use type_name_of_val to detect default type
                                let default_type = std::any::type_name_of_val(&$default);
                                if default_type.ends_with("f64") {
                                    *n
                                } else if default_type.ends_with("i32") {
                                    *n as i32
                                } else if default_type.ends_with("u32") {
                                    *n as u32
                                } else if default_type.ends_with("i64") {
                                    *n as i64
                                } else if default_type.ends_with("String") || default_type.ends_with("&str") {
                                    n.to_string()
                                } else {
                                    $default
                                }
                            },
                            $crate::types::ValueType::Bool(b) => {
                                // Use type_name_of_val to detect default type
                                let default_type = std::any::type_name_of_val(&$default);
                                if default_type.ends_with("bool") {
                                    *b
                                } else if default_type.ends_with("String") || default_type.ends_with("&str") {
                                    b.to_string()
                                } else {
                                    $default
                                }
                            },
                            _ => $default,
                        },
                        None => $default,
                    }
                },
                _ => $default,
            }
        }
    };
    
    // Extract a direct value with default
    ($value:expr, => $default:expr) => {
        match &$value {
            $crate::types::ValueType::String(s) => s.clone(),
            $crate::types::ValueType::Number(n) => {
                // Use type_name_of_val to detect default type
                let default_type = std::any::type_name_of_val(&$default);
                if default_type.ends_with("&str") || default_type.ends_with("String") {
                    n.to_string()
                } else if default_type.ends_with("f64") {
                    *n
                } else if default_type.ends_with("i32") {
                    *n as i32
                } else if default_type.ends_with("u32") {
                    *n as u32
                } else if default_type.ends_with("i64") {
                    *n as i64
                } else {
                    $default
                }
            },
            $crate::types::ValueType::Bool(b) => {
                // Use type_name_of_val to detect default type
                let default_type = std::any::type_name_of_val(&$default);
                if default_type.ends_with("bool") {
                    *b
                } else if default_type.ends_with("String") || default_type.ends_with("&str") {
                    b.to_string()
                } else {
                    $default
                }
            },
            _ => $default,
        }
    };
    
    // Simple key extraction without default (for test compatibility)
    ($map:expr, $key:expr) => {
        {
            match &$map {
                $crate::types::ValueType::Map(map_data) => {
                    match map_data.get($key) {
                        Some(value_type) => value_type.clone(),
                        None => $crate::types::ValueType::Null,
                    }
                },
                _ => $crate::types::ValueType::Null,
            }
        }
    };
}

/// Extract values from ValueType with defaults
/// 
/// This macro allows extracting values from a ValueType with default values
/// if the key is not found or the value has the wrong type.
/// 
/// # Examples
/// 
/// ```ignore
/// use runar_common::vmap_extract;
/// use runar_common::types::ValueType;
/// use runar_common::vjson;
/// 
/// // Create a test value
/// let value = ValueType::Map({
///     let mut map = std::collections::HashMap::new();
///     map.insert("name".to_string(), ValueType::String("John Doe".to_string()));
///     map.insert("age".to_string(), ValueType::Number(25.0));
///     map
/// });
/// 
/// // Extract a string from a ValueType
/// let name = vmap_extract!(value, "name", "Anonymous");
/// assert_eq!(name, "John Doe");
/// 
/// // Extract a number from a ValueType
/// let age = vmap_extract!(value, "age", 0);
/// assert_eq!(age, 25);
/// ```
#[macro_export]
macro_rules! vmap_extract {
    // Extract a value from a ValueType with default
    ($value:expr, $key:expr, $default:expr) => {
        {
            match &$value {
                $crate::types::ValueType::Map(map) => {
                    match map.get($key) {
                        Some(value_type) => match value_type {
                            $crate::types::ValueType::String(s) => {
                                match &$default {
                                    _ if false => unreachable!(), // This is a hack to make typechecking work
                                    _ | &str => s.clone(),
                                    _ | String => s.clone(),
                                    _ | i32 => s.parse::<i32>().unwrap_or($default),
                                    _ | i64 => s.parse::<i64>().unwrap_or($default),
                                    _ | f64 => s.parse::<f64>().unwrap_or($default),
                                    _ | u32 => s.parse::<u32>().unwrap_or($default),
                                    _ | bool => s.parse::<bool>().unwrap_or($default),
                                    _ => $default
                                }
                            },
                            $crate::types::ValueType::Number(n) => {
                                match &$default {
                                    _ if false => unreachable!(), // This is a hack to make typechecking work
                                    _ | f64 => *n,
                                    _ | i32 => *n as i32,
                                    _ | u32 => *n as u32,
                                    _ | i64 => *n as i64,
                                    _ | String => n.to_string(),
                                    _ | &str => n.to_string(),
                                    _ => $default
                                }
                            },
                            $crate::types::ValueType::Bool(b) => {
                                match &$default {
                                    _ if false => unreachable!(), // This is a hack to make typechecking work
                                    _ | bool => *b,
                                    _ | i32 => if *b { 1 } else { 0 },
                                    _ | i64 => if *b { 1 } else { 0 },
                                    _ | u32 => if *b { 1 } else { 0 },
                                    _ | f64 => if *b { 1.0 } else { 0.0 },
                                    _ | String => b.to_string(),
                                    _ | &str => b.to_string(),
                                    _ => $default
                                }
                            },
                            _ => $default,
                        },
                        None => $default,
                    }
                },
                _ => $default,
            }
        }
    };
    
    // Extract a direct value with default
    ($value:expr, => $default:expr) => {
        $crate::vmap!($value, => $default)
    };
}

/// Extract a string value from a ValueType with default (alias for backward compatibility)
/// 
/// # Examples
/// 
/// ```no_run
/// use runar_common::vmap_extract_string;
/// use runar_common::types::ValueType;
/// use runar_common::vjson;
/// 
/// let data = vjson!({ "name": "John Doe" });
/// let name = vmap_extract_string!(data, "name", "Anonymous");
/// assert_eq!(name, "John Doe");
/// ```
#[macro_export]
macro_rules! vmap_extract_string {
    ($value:expr, $key:expr, $default:expr) => {
        {
            match &$value {
                $crate::types::ValueType::Map(map) => {
                    match map.get($key) {
                        Some(value_type) => match value_type {
                            $crate::types::ValueType::String(s) => s.clone(),
                            $crate::types::ValueType::Number(n) => n.to_string(),
                            $crate::types::ValueType::Bool(b) => b.to_string(),
                            _ => $default.to_string(),
                        },
                        None => $default.to_string(),
                    }
                },
                _ => $default.to_string(),
            }
        }
    };
    
    ($value:expr, => $default:expr) => {
        match &$value {
            $crate::types::ValueType::String(s) => s.clone(),
            $crate::types::ValueType::Number(n) => n.to_string(),
            $crate::types::ValueType::Bool(b) => b.to_string(),
            _ => $default.to_string(),
        }
    };
}

/// Extract an i32 value from a ValueType with default (alias for backward compatibility)
/// 
/// # Examples
/// 
/// ```no_run
/// use runar_common::vmap_i32;
/// use runar_common::types::ValueType;
/// use runar_common::vjson;
/// 
/// let data = vjson!({ "age": 25 });
/// let age = vmap_i32!(data, "age", 0);
/// assert_eq!(age, 25);
/// ```
#[macro_export]
macro_rules! vmap_i32 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as i32,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<i32>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as i32,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<i32>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as i32,
                $crate::types::ValueType::String(s) => s.parse::<i32>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract an f64 value from a ValueType with default (alias for backward compatibility)
/// 
/// # Examples
/// 
/// ```no_run
/// use runar_common::vmap_extract_f64;
/// use runar_common::types::ValueType;
/// use runar_common::vjson;
/// 
/// let data = vjson!({ "score": 95.5 });
/// let score = vmap_extract_f64!(data, "score", 0.0);
/// assert_eq!(score, 95.5);
/// ```
#[macro_export]
macro_rules! vmap_extract_f64 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<f64>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<f64>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n,
                $crate::types::ValueType::String(s) => s.parse::<f64>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1.0 } else { 0.0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract a bool value from a ValueType with default (alias for backward compatibility)
/// 
/// # Examples
/// 
/// ```no_run
/// use runar_common::vmap_bool;
/// use runar_common::types::ValueType;
/// use runar_common::vjson;
/// 
/// let data = vjson!({ "is_admin": true });
/// let is_admin = vmap_bool!(data, "is_admin", false);
/// assert_eq!(is_admin, true);
/// ```
#[macro_export]
macro_rules! vmap_bool {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Bool(b) => result = *b,
                            $crate::types::ValueType::Number(n) => result = *n != 0.0,
                            $crate::types::ValueType::String(s) => {
                                let lower = s.to_lowercase();
                                result = lower == "true" || lower == "yes" || lower == "1";
                            },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Bool(b) => result = *b,
                                    $crate::types::ValueType::Number(n) => result = *n != 0.0,
                                    $crate::types::ValueType::String(s) => {
                                        let lower = s.to_lowercase();
                                        result = lower == "true" || lower == "yes" || lower == "1";
                                    },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Bool(b) => *b,
                $crate::types::ValueType::Number(n) => *n != 0.0,
                $crate::types::ValueType::String(s) => {
                    let lower = s.to_lowercase();
                    lower == "true" || lower == "yes" || lower == "1"
                },
                _ => $default,
            };
            result
        }
    };
}

// Add the missing extraction macros for various types

/// Extract a value from a ValueType using a dot-separated path
/// 
/// # Examples
/// 
/// ```ignore
/// let nested_map = vmap! {
///     "user" => vmap! {
///         "profile" => vmap! {
///             "name" => "John Doe"
///         }
///     }
/// };
/// 
/// let name = get_nested_value(&nested_map, "user.profile.name");
/// assert_eq!(name, Some(&ValueType::String("John Doe".to_string())));
/// ```
#[macro_export]
macro_rules! get_nested_value {
    ($value:expr, $key:expr) => {{
        let parts: Vec<&str> = $key.split('.').collect();
        let mut current = $value;
        let mut result = None;
        
        for (i, part) in parts.iter().enumerate() {
            if let $crate::types::ValueType::Map(map) = current {
                if let Some(value) = map.get(*part) {
                    if i == parts.len() - 1 {
                        result = Some(value);
                    } else {
                        current = value;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        result
    }};
}

/// Extract a string value from a ValueType with default (alias for backward compatibility)
#[macro_export]
macro_rules! vmap_str {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default.to_string();
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::String(s) => result = s.clone(),
                            $crate::types::ValueType::Number(n) => result = n.to_string(),
                            $crate::types::ValueType::Bool(b) => result = b.to_string(),
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::String(s) => result = s.clone(),
                                    $crate::types::ValueType::Number(n) => result = n.to_string(),
                                    $crate::types::ValueType::Bool(b) => result = b.to_string(),
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::String(s) => s.clone(),
                $crate::types::ValueType::Number(n) => n.to_string(),
                $crate::types::ValueType::Bool(b) => b.to_string(),
                _ => $default.to_string(),
            };
            result
        }
    };
}

/// Extract an i8 value from a ValueType with default
#[macro_export]
macro_rules! vmap_i8 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as i8,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<i8>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as i8,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<i8>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as i8,
                $crate::types::ValueType::String(s) => s.parse::<i8>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract an i16 value from a ValueType with default
#[macro_export]
macro_rules! vmap_i16 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as i16,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<i16>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as i16,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<i16>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as i16,
                $crate::types::ValueType::String(s) => s.parse::<i16>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract an i64 value from a ValueType with default
#[macro_export]
macro_rules! vmap_i64 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as i64,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<i64>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as i64,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<i64>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as i64,
                $crate::types::ValueType::String(s) => s.parse::<i64>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract a u8 value from a ValueType with default
#[macro_export]
macro_rules! vmap_u8 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as u8,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<u8>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as u8,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<u8>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as u8,
                $crate::types::ValueType::String(s) => s.parse::<u8>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract a u32 value from a ValueType with default
#[macro_export]
macro_rules! vmap_u32 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as u32,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<u32>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as u32,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<u32>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as u32,
                $crate::types::ValueType::String(s) => s.parse::<u32>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract a u64 value from a ValueType with default
#[macro_export]
macro_rules! vmap_u64 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as u64,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<u64>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as u64,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<u64>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1 } else { 0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as u64,
                $crate::types::ValueType::String(s) => s.parse::<u64>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1 } else { 0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract an f32 value from a ValueType with default
#[macro_export]
macro_rules! vmap_f32 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n as f32,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<f32>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n as f32,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<f32>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n as f32,
                $crate::types::ValueType::String(s) => s.parse::<f32>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1.0 } else { 0.0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract an f64 value from a ValueType with default (alias for backward compatibility)
#[macro_export]
macro_rules! vmap_f64 {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default;
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        match v {
                            $crate::types::ValueType::Number(n) => result = *n,
                            $crate::types::ValueType::String(s) => {
                                if let Ok(num) = s.parse::<f64>() {
                                    result = num;
                                }
                            },
                            $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                            _ => {}
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                match next_value {
                                    $crate::types::ValueType::Number(n) => result = *n,
                                    $crate::types::ValueType::String(s) => {
                                        if let Ok(num) = s.parse::<f64>() {
                                            result = num;
                                        }
                                    },
                                    $crate::types::ValueType::Bool(b) => result = if *b { 1.0 } else { 0.0 },
                                    _ => {}
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Number(n) => *n,
                $crate::types::ValueType::String(s) => s.parse::<f64>().unwrap_or($default),
                $crate::types::ValueType::Bool(b) => if *b { 1.0 } else { 0.0 },
                _ => $default,
            };
            result
        }
    };
}

/// Extract a Vec value from a ValueType with default
#[macro_export]
macro_rules! vmap_vec {
    ($value:expr, $key:expr, $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let key_parts: Vec<&str> = $key.split('.').collect();
            let mut current_value = &value;
            let mut result = $default.clone();
            
            // First handle the case of a simple key (no dots)
            if key_parts.len() == 1 {
                if let $crate::types::ValueType::Map(map) = current_value {
                    if let Some(v) = map.get($key) {
                        if let $crate::types::ValueType::Array(a) = v {
                            let mut vec_result = Vec::new();
                            for item in a {
                                match item {
                                    $crate::types::ValueType::String(s) => vec_result.push(s.clone()),
                                    $crate::types::ValueType::Number(n) => vec_result.push(n.to_string()),
                                    $crate::types::ValueType::Bool(b) => vec_result.push(b.to_string()),
                                    _ => {}
                                }
                            }
                            result = vec_result;
                        }
                    }
                }
            } else {
                // Handle nested keys using traversal
                for (i, part) in key_parts.iter().enumerate() {
                    if let $crate::types::ValueType::Map(map) = current_value {
                        if let Some(next_value) = map.get(*part) {
                            if i == key_parts.len() - 1 {
                                // We've reached the final key, extract value
                                if let $crate::types::ValueType::Array(a) = next_value {
                                    let mut vec_result = Vec::new();
                                    for item in a {
                                        match item {
                                            $crate::types::ValueType::String(s) => vec_result.push(s.clone()),
                                            $crate::types::ValueType::Number(n) => vec_result.push(n.to_string()),
                                            $crate::types::ValueType::Bool(b) => vec_result.push(b.to_string()),
                                            _ => {}
                                        }
                                    }
                                    result = vec_result;
                                }
                            } else {
                                // Continue traversing
                                current_value = next_value;
                            }
                        } else {
                            // Key not found at this level, exit loop
                            break;
                        }
                    } else {
                        // Not a map, can't continue traversing, exit loop
                        break;
                    }
                }
            }
            
            result
        }
    };
    
    ($value:expr, => $default:expr) => {
        {
            let value = $value.clone(); // Create a binding that lives for the entire block
            let result = match &value {
                $crate::types::ValueType::Array(a) => {
                    let mut vec_result = Vec::new();
                    for item in a {
                        match item {
                            $crate::types::ValueType::String(s) => vec_result.push(s.clone()),
                            $crate::types::ValueType::Number(n) => vec_result.push(n.to_string()),
                            $crate::types::ValueType::Bool(b) => vec_result.push(b.to_string()),
                            _ => {}
                        }
                    }
                    vec_result
                },
                _ => $default.clone(),
            };
            result
        }
    };
} 