use std::collections::HashMap;
use crate::types::ValueType;
use anyhow::Result;

/// VMap wrapper for easier ValueType manipulation
#[derive(Debug, Clone)]
pub struct VMap(pub HashMap<String, ValueType>);

impl VMap {
    /// Create a new empty VMap
    pub fn new() -> Self {
        VMap(HashMap::new())
    }

    /// Create a VMap from an existing HashMap
    pub fn from_hashmap(map: HashMap<String, ValueType>) -> Self {
        VMap(map)
    }

    /// Create a VMap from a ValueType (if it's a Map, otherwise return empty VMap)
    pub fn from_value_type(value: ValueType) -> Self {
        match value {
            ValueType::Map(map) => VMap(map),
            _ => VMap::new(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&ValueType> {
        self.0.get(key)
    }

    /// Insert a value
    pub fn insert<K: Into<String>, V: Into<ValueType>>(&mut self, key: K, value: V) {
        self.0.insert(key.into(), value.into());
    }

    /// Extract a string value with error handling
    pub fn get_string(&self, key: &str) -> Result<String> {
        match self.0.get(key) {
            Some(ValueType::String(s)) => Ok(s.clone()),
            Some(other) => Err(anyhow::anyhow!("Expected String for key '{}', got {:?}", key, other)),
            None => Err(anyhow::anyhow!("Key '{}' not found", key)),
        }
    }

    /// Extract a number as i32 with error handling
    pub fn get_number_as_int(&self, key: &str) -> Result<i32> {
        match self.0.get(key) {
            Some(ValueType::Number(n)) => Ok(*n as i32),
            Some(other) => Err(anyhow::anyhow!("Expected Number for key '{}', got {:?}", key, other)),
            None => Err(anyhow::anyhow!("Key '{}' not found", key)),
        }
    }

    /// Extract a number as f64 with error handling
    pub fn get_number_as_float(&self, key: &str) -> Result<f64> {
        match self.0.get(key) {
            Some(ValueType::Number(n)) => Ok(*n),
            Some(other) => Err(anyhow::anyhow!("Expected Number for key '{}', got {:?}", key, other)),
            None => Err(anyhow::anyhow!("Key '{}' not found", key)),
        }
    }

    /// Extract a boolean value with error handling
    pub fn get_bool(&self, key: &str) -> Result<bool> {
        match self.0.get(key) {
            Some(ValueType::Bool(b)) => Ok(*b),
            Some(other) => Err(anyhow::anyhow!("Expected Bool for key '{}', got {:?}", key, other)),
            None => Err(anyhow::anyhow!("Key '{}' not found", key)),
        }
    }

    /// Convert to inner HashMap
    pub fn into_inner(self) -> HashMap<String, ValueType> {
        self.0
    }

    /// Get a reference to the inner HashMap
    pub fn as_hashmap(&self) -> &HashMap<String, ValueType> {
        &self.0
    }
}

impl From<HashMap<String, ValueType>> for VMap {
    fn from(map: HashMap<String, ValueType>) -> Self {
        VMap(map)
    }
}

impl From<VMap> for ValueType {
    fn from(vmap: VMap) -> Self {
        ValueType::Map(vmap.0)
    }
}

impl From<VMap> for HashMap<String, ValueType> {
    fn from(vmap: VMap) -> Self {
        vmap.0
    }
} 