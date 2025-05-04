// runar_common/src/types/value_type.rs
//
// ValueType definition for Runar system

use anyhow::{anyhow, Result};
use base64;
use serde::{Deserialize, Serialize};
use serde_bytes;
use serde_json::Value;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use super::erased_arc::ErasedArc;

/// Categorizes the value for efficient dispatch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueCategory {
    Primitive,
    List,
    Map,
    Struct,
    Null,
    /// Serialized bytes (lazy deserialization)
    Bytes,
}

/// A type-erased value container with Arc preservation
///
/// ArcValueType is a replacement for the older ValueType that guarantees
/// Arc identity preservation when retrieving references.
pub struct ArcValueType {
    /// Type category for more efficient dispatching
    category: ValueCategory,
    /// The type-erased Arc pointer
    value: ErasedArc,
}

impl ArcValueType {
    /// Create a new ArcValueType from a primitive value
    pub fn from_value<T: 'static + Clone + Send + Sync>(value: T) -> Self {
        Self {
            category: ValueCategory::Primitive,
            value: ErasedArc::new(value),
        }
    }

    /// Create a ArcValueType containing a list
    pub fn from_list<T: 'static + Clone + Send + Sync>(values: Vec<T>) -> Self {
        Self {
            category: ValueCategory::List,
            value: ErasedArc::new(values),
        }
    }

    /// Create a ArcValueType containing a map
    pub fn from_map<K, V>(map: HashMap<K, V>) -> Self
    where
        K: 'static + Clone + Send + Sync + Eq + std::hash::Hash,
        V: 'static + Clone + Send + Sync,
    {
        Self {
            category: ValueCategory::Map,
            value: ErasedArc::new(map),
        }
    }

    /// Create a ArcValueType containing a struct
    pub fn from_struct<T: 'static + Clone + Send + Sync>(value: T) -> Self {
        Self {
            category: ValueCategory::Struct,
            value: ErasedArc::new(value),
        }
    }

    /// Create a null ArcValueType
    pub fn null() -> Self {
        Self {
            category: ValueCategory::Null,
            value: ErasedArc::new(()),
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        self.category == ValueCategory::Null
    }

    /// Get the value category
    pub fn category(&self) -> ValueCategory {
        self.category
    }

    /// Get a reference to a primitive type
    pub fn as_type_ref<T: 'static>(&self) -> Result<Arc<T>> {
        match self.category {
            ValueCategory::Primitive => self.value.as_arc::<T>(),
            ValueCategory::Bytes => {
                // We have serialized bytes, try to deserialize
                let serialized = self.value.as_arc::<SerializedValue>()?;
                if serialized.category != ValueCategory::Primitive {
                    return Err(anyhow!("Serialized value is not a primitive type"));
                }

                // Parse the serialized data
                let bytes = &serialized.data;
                if bytes.len() < 3 {
                    return Err(anyhow!("Invalid serialized primitive data"));
                }

                // Skip the category marker
                let pos = 1;

                // Get the type name
                let type_name_len = bytes[pos] as usize;
                if pos + 1 + type_name_len >= bytes.len() {
                    return Err(anyhow!("Invalid serialized primitive data"));
                }
                let type_name = std::str::from_utf8(&bytes[pos + 1..pos + 1 + type_name_len])?;

                // Skip the type name
                let pos = pos + 1 + type_name_len;

                // Get the primitive type marker
                let primitive_type = bytes[pos];
                let pos = pos + 1;

                // Deserialize based on the primitive type
                match primitive_type {
                    0x01 => {
                        // String
                        if pos + 4 > bytes.len() {
                            return Err(anyhow!("Invalid serialized string data"));
                        }
                        let data_len = u32::from_le_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                        ]) as usize;
                        if pos + 4 + data_len > bytes.len() {
                            return Err(anyhow!("Invalid serialized string data"));
                        }
                        let string: String =
                            bincode::deserialize(&bytes[pos + 4..pos + 4 + data_len])?;

                        // Check if the requested type is String
                        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
                            // This is unsafe but necessary for type conversion
                            // We've verified the types match
                            let value = Arc::new(string);
                            let ptr = Arc::into_raw(value) as *const T;
                            let arc = unsafe { Arc::from_raw(ptr) };
                            Ok(arc)
                        } else {
                            Err(anyhow!(
                                "Type mismatch: expected {}, found String",
                                std::any::type_name::<T>()
                            ))
                        }
                    }
                    0x02 => {
                        // i32
                        if pos + 4 > bytes.len() {
                            return Err(anyhow!("Invalid serialized i32 data"));
                        }
                        let value = i32::from_le_bytes([
                            bytes[pos],
                            bytes[pos + 1],
                            bytes[pos + 2],
                            bytes[pos + 3],
                        ]);

                        // Check if the requested type is i32
                        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
                            // This is unsafe but necessary for type conversion
                            // We've verified the types match
                            let value = Arc::new(value);
                            let ptr = Arc::into_raw(value) as *const T;
                            let arc = unsafe { Arc::from_raw(ptr) };
                            Ok(arc)
                        } else {
                            Err(anyhow!(
                                "Type mismatch: expected {}, found i32",
                                std::any::type_name::<T>()
                            ))
                        }
                    }
                    // Add handling for other primitive types as needed
                    // ...
                    _ => Err(anyhow!(
                        "Unsupported primitive type marker: {}",
                        primitive_type
                    )),
                }
            }
            _ => Err(anyhow!(
                "Value is not a primitive type. Use the appropriate method for category: {:?}",
                self.category
            )),
        }
    }

    /// Get a reference to a list
    pub fn as_list_ref<T: 'static>(&self) -> Result<Arc<Vec<T>>> {
        match self.category {
            ValueCategory::List => self.value.as_arc::<Vec<T>>(),
            ValueCategory::Bytes => {
                // We have serialized bytes, try to deserialize
                let serialized = self.value.as_arc::<SerializedValue>()?;
                if serialized.category != ValueCategory::List {
                    return Err(anyhow!("Serialized value is not a list"));
                }

                // Parse the serialized data
                let bytes = &serialized.data;
                if bytes.len() < 3 {
                    return Err(anyhow!("Invalid serialized list data"));
                }

                // Skip the category marker
                let pos = 1;

                // Get the type name
                let type_name_len = bytes[pos] as usize;
                if pos + 1 + type_name_len >= bytes.len() {
                    return Err(anyhow!("Invalid serialized list data"));
                }
                let type_name = std::str::from_utf8(&bytes[pos + 1..pos + 1 + type_name_len])?;

                // Skip the type name
                let pos = pos + 1 + type_name_len;

                // Deserialize the list
                if pos + 4 > bytes.len() {
                    return Err(anyhow!("Invalid serialized list data"));
                }
                let data_len = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                if pos + 4 + data_len > bytes.len() {
                    return Err(anyhow!("Invalid serialized list data"));
                }

                // Determine the list element type based on the type name
                if type_name.contains("String")
                    && std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>()
                {
                    let list: Vec<String> =
                        bincode::deserialize(&bytes[pos + 4..pos + 4 + data_len])?;
                    // This is unsafe but necessary for type conversion
                    // We've verified the types match
                    let value = Arc::new(list);
                    let ptr = Arc::into_raw(value) as *const Vec<T>;
                    let arc = unsafe { Arc::from_raw(ptr) };
                    Ok(arc)
                } else if type_name.contains("i32")
                    && std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>()
                {
                    let list: Vec<i32> = bincode::deserialize(&bytes[pos + 4..pos + 4 + data_len])?;
                    // This is unsafe but necessary for type conversion
                    // We've verified the types match
                    let value = Arc::new(list);
                    let ptr = Arc::into_raw(value) as *const Vec<T>;
                    let arc = unsafe { Arc::from_raw(ptr) };
                    Ok(arc)
                } else {
                    Err(anyhow!("Unsupported list element type or type mismatch"))
                }
            }
            _ => Err(anyhow!(
                "Value is not a list. Use the appropriate method for category: {:?}",
                self.category
            )),
        }
    }

    /// Get a reference to a map
    pub fn as_map_ref<K, V>(&self) -> Result<Arc<HashMap<K, V>>>
    where
        K: 'static + Clone + Send + Sync + Eq + std::hash::Hash,
        V: 'static + Clone + Send + Sync,
    {
        match self.category {
            ValueCategory::Map => self.value.as_arc::<HashMap<K, V>>(),
            ValueCategory::Bytes => {
                // We have serialized bytes, try to deserialize
                let serialized = self.value.as_arc::<SerializedValue>()?;
                if serialized.category != ValueCategory::Map {
                    return Err(anyhow!("Serialized value is not a map"));
                }

                // Parse the serialized data
                let bytes = &serialized.data;
                if bytes.len() < 3 {
                    return Err(anyhow!("Invalid serialized map data"));
                }

                // Skip the category marker
                let pos = 1;

                // Get the type name
                let type_name_len = bytes[pos] as usize;
                if pos + 1 + type_name_len >= bytes.len() {
                    return Err(anyhow!("Invalid serialized map data"));
                }
                let type_name = std::str::from_utf8(&bytes[pos + 1..pos + 1 + type_name_len])?;

                // Skip the type name
                let pos = pos + 1 + type_name_len;

                // Deserialize the map
                if pos + 4 > bytes.len() {
                    return Err(anyhow!("Invalid serialized map data"));
                }
                let data_len = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                if pos + 4 + data_len > bytes.len() {
                    return Err(anyhow!("Invalid serialized map data"));
                }

                // Determine the map types based on the type name
                if type_name.contains("String")
                    && type_name.contains("HashMap")
                    && std::any::TypeId::of::<K>() == std::any::TypeId::of::<String>()
                    && std::any::TypeId::of::<V>() == std::any::TypeId::of::<String>()
                {
                    let map: HashMap<String, String> =
                        bincode::deserialize(&bytes[pos + 4..pos + 4 + data_len])?;
                    // This is unsafe but necessary for type conversion
                    // We've verified the types match
                    let value = Arc::new(map);
                    let ptr = Arc::into_raw(value) as *const HashMap<K, V>;
                    let arc = unsafe { Arc::from_raw(ptr) };
                    Ok(arc)
                } else {
                    Err(anyhow!("Unsupported map type or type mismatch"))
                }
            }
            _ => Err(anyhow!(
                "Value is not a map. Use the appropriate method for category: {:?}",
                self.category
            )),
        }
    }

    /// Get a reference to a struct
    pub fn as_struct_ref<T: 'static>(&self) -> Result<Arc<T>> {
        match self.category {
            ValueCategory::Struct => self.value.as_arc::<T>(),
            ValueCategory::Bytes => {
                // We have serialized bytes, try to deserialize
                let serialized = self.value.as_arc::<SerializedValue>()?;
                if serialized.category != ValueCategory::Struct {
                    return Err(anyhow!("Serialized value is not a struct"));
                }

                // Parse the serialized data
                let bytes = &serialized.data;
                if bytes.len() < 3 {
                    return Err(anyhow!("Invalid serialized struct data"));
                }

                // Skip the category marker
                let pos = 1;

                // Get the type name
                let type_name_len = bytes[pos] as usize;
                if pos + 1 + type_name_len >= bytes.len() {
                    return Err(anyhow!("Invalid serialized struct data"));
                }
                let type_name = std::str::from_utf8(&bytes[pos + 1..pos + 1 + type_name_len])?;

                // Currently struct serialization is not implemented
                return Err(anyhow!(
                    "Struct deserialization not yet implemented for type: {}",
                    type_name
                ));
            }
            _ => Err(anyhow!(
                "Value is not a struct. Use the appropriate method for category: {:?}",
                self.category
            )),
        }
    }

    /// Access a value by converting to a specific type (with cloning)
    pub fn as_type<T: 'static + Clone>(&self) -> Result<T> {
        let arc = match self.category {
            ValueCategory::Primitive => self.value.as_arc::<T>()?,
            ValueCategory::Struct => self.value.as_arc::<T>()?,
            _ => {
                return Err(anyhow!(
                    "Cannot convert {:?} to this type directly",
                    self.category
                ))
            }
        };

        Ok((*arc).clone())
    }

    /// Convert to a list with cloning
    pub fn as_list<T: 'static + Clone>(&self) -> Result<Vec<T>> {
        match self.category {
            ValueCategory::List => {
                let arc = self.value.as_arc::<Vec<T>>()?;
                Ok((*arc).clone())
            }
            _ => Err(anyhow!("Value is not a list")),
        }
    }

    /// Convert to a map with cloning
    pub fn as_map<K, V>(&self) -> Result<HashMap<K, V>>
    where
        K: 'static + Clone + Send + Sync + Eq + std::hash::Hash,
        V: 'static + Clone + Send + Sync,
    {
        match self.category {
            ValueCategory::Map => {
                let arc = self.value.as_arc::<HashMap<K, V>>()?;
                Ok((*arc).clone())
            }
            _ => Err(anyhow!("Value is not a map")),
        }
    }

    /// Get the type name of the contained value
    pub fn type_name(&self) -> &'static str {
        self.value.type_name()
    }

    /// Serialize the value to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // First byte is the category marker
        let mut result = Vec::new();

        match self.category {
            ValueCategory::Primitive => {
                // Add category marker (1 byte)
                result.push(0x01);

                // Add type name length and type name
                let type_name = self.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // Handle different primitive types
                if let Ok(s) = self.value.as_arc::<String>() {
                    result.push(0x01); // String type marker
                    let bytes = bincode::serialize(&*s)?;
                    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    result.extend_from_slice(&bytes);
                } else if let Ok(i) = self.value.as_arc::<i32>() {
                    result.push(0x02); // i32 type marker
                    result.extend_from_slice(&i.to_le_bytes());
                } else if let Ok(i) = self.value.as_arc::<i64>() {
                    result.push(0x03); // i64 type marker
                    result.extend_from_slice(&i.to_le_bytes());
                } else if let Ok(f) = self.value.as_arc::<f64>() {
                    result.push(0x04); // f64 type marker
                    result.extend_from_slice(&f.to_le_bytes());
                } else if let Ok(b) = self.value.as_arc::<bool>() {
                    result.push(0x05); // bool type marker
                    result.push(if *b { 1 } else { 0 });
                } else {
                    // Try generic serialization for other types
                    result.push(0xFF); // Generic serialization marker
                                       // Serialize using bincode
                                       // Note: this requires the type to implement Serialize
                                       // and will result in a runtime error if it doesn't
                    let serialized = bincode::serialize(&self.value.type_name())?;
                    result.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
                    result.extend_from_slice(&serialized);
                }
            }
            ValueCategory::List => {
                // Add category marker (1 byte)
                result.push(0x02);

                // Add type name length and type name
                let type_name = self.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // Serialize the list using bincode
                // Note: this requires the element type to implement Serialize
                if type_name.contains("String") {
                    if let Ok(list) = self.value.as_arc::<Vec<String>>() {
                        let bytes = bincode::serialize(&*list)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize list of strings"));
                    }
                } else if type_name.contains("i32") {
                    if let Ok(list) = self.value.as_arc::<Vec<i32>>() {
                        let bytes = bincode::serialize(&*list)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize list of i32"));
                    }
                } else if type_name.contains("i64") {
                    if let Ok(list) = self.value.as_arc::<Vec<i64>>() {
                        let bytes = bincode::serialize(&*list)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize list of i64"));
                    }
                } else if type_name.contains("f64") {
                    if let Ok(list) = self.value.as_arc::<Vec<f64>>() {
                        let bytes = bincode::serialize(&*list)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize list of f64"));
                    }
                } else if type_name.contains("bool") {
                    if let Ok(list) = self.value.as_arc::<Vec<bool>>() {
                        let bytes = bincode::serialize(&*list)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize list of bool"));
                    }
                } else {
                    // This is a limitation - we only support certain primitive types for now
                    return Err(anyhow!("Unsupported list element type: {}", type_name));
                }
            }
            ValueCategory::Map => {
                // Add category marker (1 byte)
                result.push(0x03);

                // Add type name length and type name
                let type_name = self.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // Serialize the map using bincode
                // For now, only support string->string maps
                if type_name.contains("String") && type_name.contains("HashMap") {
                    if let Ok(map) = self.value.as_arc::<HashMap<String, String>>() {
                        let bytes = bincode::serialize(&*map)?;
                        result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                        result.extend_from_slice(&bytes);
                    } else {
                        return Err(anyhow!("Failed to serialize string->string map"));
                    }
                } else {
                    // This is a limitation - we only support certain map types for now
                    return Err(anyhow!("Unsupported map type: {}", type_name));
                }
            }
            ValueCategory::Struct => {
                // Add category marker (1 byte)
                result.push(0x04);

                // Add type name length and type name
                let type_name = self.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // For structs, we need specific handling based on the struct type
                // This is just a placeholder - in a real implementation, you'd need
                // to implement serialization for your specific struct types
                return Err(anyhow!(
                    "Struct serialization not yet implemented for type: {}",
                    type_name
                ));
            }
            ValueCategory::Null => {
                // Add category marker (1 byte)
                result.push(0x05);
                // No additional data needed for null
            }
            ValueCategory::Bytes => {
                // We're already serialized, just return the bytes
                // This shouldn't happen in normal usage, but just in case
                return Err(anyhow!("Cannot serialize already serialized bytes"));
            }
        }

        Ok(result)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(anyhow!("Empty bytes"));
        }

        // First byte is the category marker
        let category = match bytes[0] {
            0x01 => ValueCategory::Primitive,
            0x02 => ValueCategory::List,
            0x03 => ValueCategory::Map,
            0x04 => ValueCategory::Struct,
            0x05 => ValueCategory::Null,
            _ => return Err(anyhow!("Invalid category marker: {}", bytes[0])),
        };

        match category {
            ValueCategory::Null => {
                // Just return a null value
                Ok(Self::null())
            }
            _ => {
                // For all other types, store the bytes and deserialize lazily
                // This is more efficient than deserializing immediately
                Ok(Self {
                    category: ValueCategory::Bytes,
                    value: ErasedArc::new(SerializedValue {
                        category,
                        data: bytes.to_vec(),
                    }),
                })
            }
        }
    }
}

impl Clone for ArcValueType {
    fn clone(&self) -> Self {
        Self {
            category: self.category,
            value: self.value.clone(),
        }
    }
}

impl fmt::Debug for ArcValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArcValueType")
            .field("category", &self.category)
            .field("type", &self.value.type_name())
            .finish()
    }
}

// TODO: Implement serialization support
// This will be added in the next implementation phase.

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

// Manual implementation of PartialEq for ValueType
// This avoids the issue with comparing the Struct variant's Arc<dyn ...>
impl PartialEq for ValueType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueType::Json(a), ValueType::Json(b)) => a == b,
            (ValueType::Map(a), ValueType::Map(b)) => a == b,
            (ValueType::Array(a), ValueType::Array(b)) => a == b,
            (ValueType::String(a), ValueType::String(b)) => a == b,
            (ValueType::Number(a), ValueType::Number(b)) => a == b,
            (ValueType::Bool(a), ValueType::Bool(b)) => a == b,
            (ValueType::Null, ValueType::Null) => true,
            (ValueType::Bytes(a), ValueType::Bytes(b)) => a == b,
            // Treat Struct variants as unequal for now
            (ValueType::Struct(_), ValueType::Struct(_)) => false,
            // All other combinations are unequal
            _ => false,
        }
    }
}

// We want this to be clonable (especially for use in HashMaps)
impl Eq for ValueType {}

/// Trait for structs that can be serialized into ValueType
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

// We need a manual implementation of Clone since we can't derive it
impl Clone for Box<dyn SerializableStruct + Send + Sync + 'static> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Wrapper to make a SerializableStruct cloneable
pub struct StructArc(pub Box<dyn SerializableStruct + Send + Sync + 'static>);

impl Clone for StructArc {
    fn clone(&self) -> Self {
        StructArc(self.0.clone())
    }
}

// Blanket implementation for any type that can be serialized by serde
impl<T> SerializableStruct for T
where
    T: std::fmt::Debug + serde::Serialize + Clone + Send + Sync + 'static,
{
    fn to_map(&self) -> Result<HashMap<String, ValueType>> {
        // Convert to JSON first
        let json = serde_json::to_value(self)?;

        // Then extract a map
        match json {
            Value::Object(map) => {
                let mut value_map = HashMap::new();
                for (key, value) in map {
                    value_map.insert(key, ValueType::from_json(value));
                }
                Ok(value_map)
            }
            _ => Err(anyhow!("Cannot convert to map: not a JSON object")),
        }
    }

    fn to_json_value(&self) -> Result<Value> {
        Ok(serde_json::to_value(self)?)
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
                    // For NaN or infinity, represent as null
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
                    Ok(json) => json,
                    Err(_) => Value::Null, // Default to null on error
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

// Remove the generic Vec<T> implementation and replace with specific implementations
impl From<Vec<ValueType>> for ValueType {
    fn from(v: Vec<ValueType>) -> Self {
        ValueType::Array(v)
    }
}

impl From<Vec<String>> for ValueType {
    fn from(v: Vec<String>) -> Self {
        ValueType::Array(v.into_iter().map(ValueType::from).collect())
    }
}

impl From<Vec<&str>> for ValueType {
    fn from(v: Vec<&str>) -> Self {
        ValueType::Array(v.into_iter().map(ValueType::from).collect())
    }
}

impl From<Vec<f64>> for ValueType {
    fn from(v: Vec<f64>) -> Self {
        ValueType::Array(v.into_iter().map(ValueType::from).collect())
    }
}

impl From<Vec<i32>> for ValueType {
    fn from(v: Vec<i32>) -> Self {
        ValueType::Array(v.into_iter().map(ValueType::from).collect())
    }
}

impl From<Vec<bool>> for ValueType {
    fn from(v: Vec<bool>) -> Self {
        ValueType::Array(v.into_iter().map(ValueType::from).collect())
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

// Remove the generic From<T> implementation and replace with a macro
// that can be used to implement From for specific struct types

/// Macro to implement From<YourStruct> for ValueType
///
/// Use this macro to implement From for your custom struct types that implement SerializableStruct.
/// Example:
/// ```ignore
/// use runar_common::implement_from_for_valuetype;
/// use runar_common::types::SerializableStruct;
/// use std::collections::HashMap;
/// use runar_common::types::ValueType;
/// use anyhow::Result;
///
/// #[derive(Debug)]
/// struct MyStruct {}
///
/// impl SerializableStruct for MyStruct {
///     fn to_map(&self) -> Result<HashMap<String, ValueType>> {
///         Ok(HashMap::new())
///     }
///     
///     fn to_json_value(&self) -> Result<serde_json::Value> {
///         Ok(serde_json::json!({}))
///     }
///     
///     fn type_name(&self) -> &'static str {
///         "MyStruct"
///     }
///     
///     fn clone_box(&self) -> Box<dyn SerializableStruct + Send + Sync + 'static> {
///         Box::new(MyStruct {})
///     }
/// }
///
/// implement_from_for_valuetype!(MyStruct);
/// ```
#[macro_export]
macro_rules! implement_from_for_valuetype {
    ($struct_type:ty) => {
        impl From<$struct_type> for $crate::types::ValueType {
            fn from(s: $struct_type) -> Self {
                $crate::types::ValueType::Struct(std::sync::Arc::new(s))
            }
        }
    };
}

/// A helper struct to store serialized data
#[derive(Clone, Debug)]
struct SerializedValue {
    category: ValueCategory,
    data: Vec<u8>,
}
