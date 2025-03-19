// runar_common/src/utils/value_converters.rs
//
// Utility functions for converting between ValueType and other types

use serde::de::DeserializeOwned;
use serde_json;

use crate::types::ValueType;

/// Helper function to convert a ValueType to a specified type
/// 
/// This function attempts to deserialize a ValueType into the target type.
/// If deserialization fails, it returns the provided default value.
/// 
/// # Examples
/// 
/// ```
/// use runar_common::types::ValueType;
/// use runar_common::utils::value_to_type;
/// 
/// let value = ValueType::Number(30.0);
/// let age: i32 = value_to_type(&value, 0);
/// assert_eq!(age, 30);
/// ```
pub fn value_to_type<T: DeserializeOwned>(value: &ValueType, default: T) -> T {
    match serde_json::to_value(value) {
        Ok(json_value) => {
            match serde_json::from_value::<T>(json_value) {
                Ok(typed_value) => typed_value,
                Err(_) => default,
            }
        },
        Err(_) => default,
    }
}

/// Extract a value from a ValueType::Map by key
/// 
/// # Examples
/// 
/// ```
/// use runar_common::types::ValueType;
/// use runar_common::utils::extract_value;
/// use std::collections::HashMap;
/// 
/// let mut map = HashMap::new();
/// map.insert("age".to_string(), ValueType::Number(25.0));
/// let value_map = ValueType::Map(map);
/// 
/// let age: i32 = extract_value(&value_map, "age", 0);
/// assert_eq!(age, 25);
/// 
/// // Non-existent key returns default
/// let unknown: i32 = extract_value(&value_map, "unknown", 42);
/// assert_eq!(unknown, 42);
/// ```
pub fn extract_value<T: DeserializeOwned>(
    map: &ValueType, 
    key: &str, 
    default: T
) -> T {
    if let ValueType::Map(map_data) = map {
        if let Some(value) = map_data.get(key) {
            return value_to_type(value, default);
        }
    }
    default
}

/// Extract a value directly from a ValueType
/// 
/// # Examples
/// 
/// ```
/// use runar_common::types::ValueType;
/// use runar_common::utils::extract_direct;
/// 
/// let value = ValueType::String("hello".to_string());
/// 
/// let s: String = extract_direct(&value, String::new());
/// assert_eq!(s, "hello");
/// 
/// // Type mismatch returns default
/// let n: i32 = extract_direct(&value, 42);
/// assert_eq!(n, 42);
/// ```
pub fn extract_direct<T: DeserializeOwned>(value: &ValueType, default: T) -> T {
    value_to_type(value, default)
}
