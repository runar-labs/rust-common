// runar_common/src/types/value_type.rs
//
// ValueType definition for Runar system

use anyhow::{anyhow, Result};
use base64;
use serde::{Deserialize, Serialize};
use serde_bytes;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// ValueType represents a dynamically typed value that can be passed between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    /// JSON value (for compatibility with external services)
    Json(Value),
    /// HashMap of string keys to ValueType values
    Map(HashMap<String, ValueType>),
    /// Vector of ValueType values
    Array(Vec<ValueType>),
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Bool(bool),
    /// Null/None value
    Null,
    /// Binary data
    #[serde(with = "serde_bytes")]
    Bytes(Vec<u8>),
    /// Raw struct data (serialized on demand)
    #[serde(skip)]
    Struct(Arc<dyn SerializableStruct + Send + Sync + 'static>),
}

/// Trait for types that can be stored in a ValueType::Struct
pub trait SerializableStruct: std::fmt::Debug {
    /// Convert to a HashMap representation
    fn to_map(&self) -> Result<HashMap<String, ValueType>>;

    /// Convert to JSON Value
    fn to_json_value(&self) -> Result<Value>;

    /// Get the type name (for debugging)
    fn type_name(&self) -> &'static str;

    /// Clone the struct (required since we can't directly clone a dyn trait)
    fn clone_box(&self) -> Box<dyn SerializableStruct + Send + Sync + 'static>;
}

// We need to create our own Clone impl for Box<dyn SerializableStruct>
// to avoid orphan rule violations
impl Clone for Box<dyn SerializableStruct + Send + Sync + 'static> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Define our own wrapper to avoid the orphan rule violation for Arc cloning
pub struct StructArc(pub Box<dyn SerializableStruct + Send + Sync + 'static>);

impl Clone for StructArc {
    fn clone(&self) -> Self {
        StructArc(self.0.clone())
    }
}

/// Implementation for any type that implements Serialize and Debug
impl<T> SerializableStruct for T
where
    T: std::fmt::Debug + serde::Serialize + Clone + Send + Sync + 'static,
{
    fn to_map(&self) -> Result<HashMap<String, ValueType>> {
        // Convert to JSON first
        let json = serde_json::to_value(self)?;

        // Then convert JSON to map
        match json {
            Value::Object(map) => {
                let mut value_map = HashMap::new();
                for (key, value) in map {
                    value_map.insert(key, ValueType::from_json(value));
                }
                Ok(value_map)
            }
            _ => Err(anyhow!("Expected a JSON object, got: {:?}", json)),
        }
    }

    fn to_json_value(&self) -> Result<Value> {
        serde_json::to_value(self).map_err(|e| anyhow!("Serialization error: {}", e))
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn clone_box(&self) -> Box<dyn SerializableStruct + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

impl ValueType {
    /// Create a ValueType from a JSON Value
    pub fn from_json(value: Value) -> Self {
        ValueType::Json(value)
    }

    /// Convert this ValueType to a JSON Value
    pub fn to_json(&self) -> Value {
        match self {
            ValueType::Json(value) => value.clone(),
            ValueType::Map(map) => {
                let mut json_map = serde_json::Map::new();
                for (key, value) in map {
                    json_map.insert(key.clone(), value.to_json());
                }
                Value::Object(json_map)
            }
            ValueType::Array(array) => {
                let json_array = array.iter().map(|v| v.to_json()).collect();
                Value::Array(json_array)
            }
            ValueType::String(s) => Value::String(s.clone()),
            ValueType::Number(n) => {
                if let Some(f) = serde_json::Number::from_f64(*n) {
                    Value::Number(f)
                } else {
                    Value::Null
                }
            }
            ValueType::Bool(b) => Value::Bool(*b),
            ValueType::Null => Value::Null,
            ValueType::Bytes(b) => {
                // Base64 encode binary data for JSON representation
                let base64 = base64::encode(b);
                Value::String(base64)
            }
            ValueType::Struct(s) => {
                // Serialize the struct to JSON on demand
                match s.to_json_value() {
                    Ok(v) => v,
                    Err(_) => Value::Null,
                }
            }
        }
    }

    /// Convert this ValueType to a HashMap if possible
    pub fn to_map(&self) -> Result<HashMap<String, ValueType>> {
        match self {
            ValueType::Map(map) => Ok(map.clone()),
            ValueType::Json(Value::Object(obj)) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), ValueType::from_json(v.clone()));
                }
                Ok(map)
            }
            ValueType::Struct(s) => s.to_map(),
            _ => Err(anyhow!("Cannot convert {:?} to HashMap", self)),
        }
    }

    /// Get a reference to a map if this ValueType is a Map
    pub fn as_map(&self) -> Option<&HashMap<String, ValueType>> {
        match self {
            ValueType::Map(map) => Some(map),
            _ => None,
        }
    }

    /// Get a mutable reference to a map if this ValueType is a Map
    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, ValueType>> {
        match self {
            ValueType::Map(map) => Some(map),
            _ => None,
        }
    }
    
    /// Get a reference to an array if this ValueType is an Array
    pub fn as_array(&self) -> Option<&Vec<ValueType>> {
        match self {
            ValueType::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    /// Get a reference to a string if this ValueType is a String
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ValueType::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get a number if this ValueType is a Number
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ValueType::Number(n) => Some(*n),
            _ => None,
        }
    }
    
    /// Get a boolean if this ValueType is a Bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ValueType::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Get bytes if this ValueType is Bytes
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            ValueType::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

// Implement From for common types to convert to ValueType
impl From<String> for ValueType {
    fn from(s: String) -> Self {
        ValueType::String(s)
    }
}

impl From<&str> for ValueType {
    fn from(s: &str) -> Self {
        ValueType::String(s.to_string())
    }
}

impl From<f64> for ValueType {
    fn from(n: f64) -> Self {
        ValueType::Number(n)
    }
}

impl From<i32> for ValueType {
    fn from(n: i32) -> Self {
        ValueType::Number(n as f64)
    }
}

impl From<i64> for ValueType {
    fn from(n: i64) -> Self {
        ValueType::Number(n as f64)
    }
}

impl From<u32> for ValueType {
    fn from(n: u32) -> Self {
        ValueType::Number(n as f64)
    }
}

impl From<u64> for ValueType {
    fn from(n: u64) -> Self {
        ValueType::Number(n as f64)
    }
}

impl From<bool> for ValueType {
    fn from(b: bool) -> Self {
        ValueType::Bool(b)
    }
}

impl From<Vec<u8>> for ValueType {
    fn from(b: Vec<u8>) -> Self {
        ValueType::Bytes(b)
    }
}

impl<T: Into<ValueType>> From<Vec<T>> for ValueType {
    fn from(v: Vec<T>) -> Self {
        ValueType::Array(v.into_iter().map(|x| x.into()).collect())
    }
}

impl<T: Into<ValueType>> From<HashMap<String, T>> for ValueType {
    fn from(m: HashMap<String, T>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in m {
            map.insert(k, v.into());
        }
        ValueType::Map(map)
    }
}

impl From<Value> for ValueType {
    fn from(v: Value) -> Self {
        ValueType::Json(v)
    }
}

impl<T: SerializableStruct + Send + Sync + 'static> From<T> for ValueType {
    fn from(s: T) -> Self {
        ValueType::Struct(Arc::new(s))
    }
}
