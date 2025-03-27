// runar_common/src/utils/value_converters.rs
//
// Utility functions for converting between ValueType and other types

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;

use crate::types::ValueType;

/// Helper function to convert a ValueType to a specified type
/// 
/// This function attempts to deserialize a ValueType into the target type.
/// If deserialization fails, it returns the provided default value.
/// 
/// # Examples
/// 
/// ```ignore
/// use runar_common::types::ValueType;
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
/// ```ignore
/// use runar_common::types::ValueType;
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
/// ```ignore
/// use runar_common::types::ValueType;
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

/// Helper function to convert any serializable value to ValueType
/// 
/// # Examples
/// 
/// ```ignore
/// use runar_common::types::{ValueType, SerializableStruct};
/// use runar_common::utils::to_value_type;
/// use std::collections::HashMap;
/// 
/// #[derive(serde::Serialize)]
/// struct User {
///     name: String,
///     age: i32,
/// }
/// 
/// let user = User { 
///     name: "Alice".to_string(), 
///     age: 30 
/// };
/// 
/// let value = to_value_type(user);
/// assert!(matches!(value, ValueType::Json(_)));
/// ```
pub fn to_value_type<T: serde::Serialize>(value: T) -> crate::types::ValueType {
    // Convert the value to a JSON Value first
    let json_value = match serde_json::to_value(&value) {
        Ok(v) => v,
        Err(_) => return crate::types::ValueType::Null,
    };
    
    // Then convert to ValueType
    match json_value {
        serde_json::Value::Null => crate::types::ValueType::Null,
        serde_json::Value::Bool(b) => crate::types::ValueType::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                crate::types::ValueType::Number(f)
            } else {
                crate::types::ValueType::Null
            }
        },
        serde_json::Value::String(s) => crate::types::ValueType::String(s),
        serde_json::Value::Array(arr) => {
            let values: Vec<crate::types::ValueType> = arr.into_iter()
                .map(|v| {
                    // For each element, convert from JSON to ValueType
                    crate::types::ValueType::from_json(v)
                })
                .collect();
            crate::types::ValueType::Array(values)
        },
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                // For each value in the object, convert from JSON to ValueType
                map.insert(k, crate::types::ValueType::from_json(v));
            }
            crate::types::ValueType::Map(map)
        },
    }
}
