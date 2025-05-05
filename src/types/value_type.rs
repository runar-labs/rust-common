// runar_common/src/types/value_type.rs
//
// Type-erased value type with Arc preservation

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use log;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::erased_arc::ErasedArc;

/// Container for lazy deserialization of struct data
#[derive(Debug, Clone)]
struct LazyDeserializer {
    /// The original type name from the serialized data
    type_name: String,
    /// The raw serialized bytes
    bytes: Vec<u8>,
}

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

/// Registry for type-specific serialization and deserialization handlers
pub struct TypeRegistry {
    serializers: FxHashMap<String, Box<dyn Fn(&dyn Any) -> Result<Vec<u8>> + Send + Sync>>,
    deserializers:
        FxHashMap<String, Box<dyn Fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>> + Send + Sync>>,
    is_sealed: bool,
}

impl TypeRegistry {
    pub fn new() -> Self {
        TypeRegistry {
            serializers: FxHashMap::default(),
            deserializers: FxHashMap::default(),
            is_sealed: false,
        }
    }

    /// Initialize with default types
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default type handlers
    fn register_defaults(&mut self) {
        // Register primitive types
        self.register::<i32>().unwrap();
        self.register::<i64>().unwrap();
        self.register::<f32>().unwrap();
        self.register::<f64>().unwrap();
        self.register::<bool>().unwrap();
        self.register::<String>().unwrap();

        // Register common container types
        self.register::<Vec<i32>>().unwrap();
        self.register::<Vec<i64>>().unwrap();
        self.register::<Vec<f32>>().unwrap();
        self.register::<Vec<f64>>().unwrap();
        self.register::<Vec<bool>>().unwrap();
        self.register::<Vec<String>>().unwrap();

        // Register common map types
        self.register_map::<String, String>().unwrap();
        self.register_map::<String, i32>().unwrap();
        self.register_map::<String, i64>().unwrap();
        self.register_map::<String, f64>().unwrap();
        self.register_map::<String, bool>().unwrap();
    }

    /// Seal the registry to prevent further modifications
    pub fn seal(&mut self) {
        self.is_sealed = true;
    }

    /// Check if the registry is sealed
    pub fn is_sealed(&self) -> bool {
        self.is_sealed
    }

    /// Register a type for serialization/deserialization
    pub fn register<T: 'static + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync>(
        &mut self,
    ) -> Result<()> {
        if self.is_sealed {
            return Err(anyhow!(
                "Cannot register new types after registry is sealed"
            ));
        }

        // Get the full and simple type names
        let type_name = std::any::type_name::<T>();
        let simple_name = if let Some(last_segment) = type_name.split("::").last() {
            last_segment.to_string()
        } else {
            type_name.to_string()
        };

        // Register serializer using the full type name
        self.serializers.insert(
            type_name.to_string(),
            Box::new(|value: &dyn Any| -> Result<Vec<u8>> {
                if let Some(typed_value) = value.downcast_ref::<T>() {
                    bincode::serialize(typed_value)
                        .map_err(|e| anyhow!("Serialization error: {}", e))
                } else {
                    Err(anyhow!("Type mismatch during serialization"))
                }
            }),
        );

        // Register deserializer using both full and simple type names
        self.deserializers.insert(
            type_name.to_string(),
            Box::new(|bytes: &[u8]| -> Result<Box<dyn Any + Send + Sync>> {
                let value: T = bincode::deserialize(bytes)?;
                Ok(Box::new(value))
            }),
        );

        // Only register the simple name version if it's different and not already registered
        if simple_name != type_name && !self.deserializers.contains_key(&simple_name) {
            self.deserializers.insert(
                simple_name,
                Box::new(|bytes: &[u8]| -> Result<Box<dyn Any + Send + Sync>> {
                    let value: T = bincode::deserialize(bytes)?;
                    Ok(Box::new(value))
                }),
            );
        }

        Ok(())
    }

    /// Register a map type for serialization/deserialization
    pub fn register_map<K, V>(&mut self) -> Result<()>
    where
        K: 'static
            + Serialize
            + for<'de> Deserialize<'de>
            + Clone
            + Send
            + Sync
            + Eq
            + std::hash::Hash,
        V: 'static + Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync,
    {
        if self.is_sealed {
            return Err(anyhow!(
                "Cannot register new types after registry is sealed"
            ));
        }

        // Get the full and simple type names
        let type_name = std::any::type_name::<HashMap<K, V>>();
        let simple_name = if let Some(last_segment) = type_name.split("::").last() {
            last_segment.to_string()
        } else {
            type_name.to_string()
        };

        // Register serializer using the full type name
        self.serializers.insert(
            type_name.to_string(),
            Box::new(|value: &dyn Any| -> Result<Vec<u8>> {
                if let Some(map) = value.downcast_ref::<HashMap<K, V>>() {
                    bincode::serialize(map).map_err(|e| anyhow!("Map serialization error: {}", e))
                } else {
                    Err(anyhow!("Type mismatch during map serialization"))
                }
            }),
        );

        // Register deserializer using both full and simple type names
        self.deserializers.insert(
            type_name.to_string(),
            Box::new(|bytes: &[u8]| -> Result<Box<dyn Any + Send + Sync>> {
                let map: HashMap<K, V> = bincode::deserialize(bytes)?;
                Ok(Box::new(map))
            }),
        );

        // Only register the simple name version if it's different and not already registered
        if simple_name != type_name && !self.deserializers.contains_key(&simple_name) {
            self.deserializers.insert(
                simple_name,
                Box::new(|bytes: &[u8]| -> Result<Box<dyn Any + Send + Sync>> {
                    let map: HashMap<K, V> = bincode::deserialize(bytes)?;
                    Ok(Box::new(map))
                }),
            );
        }

        Ok(())
    }

    /// Register a custom deserializer with a specific type name
    pub fn register_custom_deserializer<T: 'static + Send + Sync>(
        &mut self,
        type_name: &str,
        deserializer: Box<dyn Fn(&[u8]) -> Result<Box<dyn Any + Send + Sync>> + Send + Sync>,
    ) -> Result<()> {
        if self.is_sealed {
            return Err(anyhow!(
                "Cannot register new types after registry is sealed"
            ));
        }

        // Add the custom deserializer
        self.deserializers
            .insert(type_name.to_string(), deserializer);

        Ok(())
    }

    /// Serialize a value using the appropriate registered handler
    pub fn serialize(&self, value: &dyn Any, type_name: &str) -> Result<Vec<u8>> {
        if let Some(serializer) = self.serializers.get(type_name) {
            serializer(value)
                .map_err(|e| anyhow!("Serialization error for type {}: {}", type_name, e))
        } else {
            Err(anyhow!("No serializer registered for type: {}", type_name))
        }
    }

    /// Deserialize bytes to a value using the appropriate registered handler
    pub fn deserialize(&self, type_name: &str, bytes: &[u8]) -> Result<Box<dyn Any + Send + Sync>> {
        if let Some(deserializer) = self.deserializers.get(type_name) {
            deserializer(bytes)
                .map_err(|e| anyhow!("Deserialization error for type {}: {}", type_name, e))
        } else {
            // Handle simple type names
            if type_name.contains("::") {
                if let Some(simple_name) = type_name.split("::").last() {
                    if let Some(deserializer) = self.deserializers.get(simple_name) {
                        return deserializer(bytes).map_err(|e| {
                            anyhow!(
                                "Deserialization error for simplified type {}: {}",
                                simple_name,
                                e
                            )
                        });
                    }
                }
            }

            Err(anyhow!(
                "No deserializer registered for type: {}",
                type_name
            ))
        }
    }

    /// Serialize a value to bytes
    pub fn serialize_value(&self, value: &ArcValueType) -> Result<Vec<u8>> {
        // First byte is the category marker
        let mut result = Vec::new();

        match value.category {
            ValueCategory::Primitive => {
                // Add category marker (1 byte)
                result.push(0x01);

                // Add type name length and type name
                let type_name = value.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // For basic types, use direct serialization
                let any_ref = value.value.as_any()?;
                match type_name {
                    "i32" => self.serialize_primitive::<i32>(any_ref, &mut result)?,
                    "i64" => self.serialize_primitive::<i64>(any_ref, &mut result)?,
                    "f32" => self.serialize_primitive::<f32>(any_ref, &mut result)?,
                    "f64" => self.serialize_primitive::<f64>(any_ref, &mut result)?,
                    "bool" => self.serialize_primitive::<bool>(any_ref, &mut result)?,
                    "String" | "std::string::String" | "alloc::string::String" => {
                        self.serialize_primitive::<String>(any_ref, &mut result)?
                    }
                    _ => {
                        let serialized = self.serialize(any_ref, type_name)?;
                        result.extend_from_slice(&serialized);
                    }
                }
                Ok(result)
            }
            ValueCategory::List => {
                // Add category marker (1 byte)
                result.push(0x02);

                // Add type name length and type name
                let type_name = value.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // For lists, either use fast path or registry
                let any_ref = value.value.as_any()?;
                if type_name.contains("Vec<i32>") {
                    self.serialize_list::<i32>(any_ref, &mut result)?;
                } else if type_name.contains("Vec<i64>") {
                    self.serialize_list::<i64>(any_ref, &mut result)?;
                } else if type_name.contains("Vec<f32>") {
                    self.serialize_list::<f32>(any_ref, &mut result)?;
                } else if type_name.contains("Vec<f64>") {
                    self.serialize_list::<f64>(any_ref, &mut result)?;
                } else if type_name.contains("Vec<bool>") {
                    self.serialize_list::<bool>(any_ref, &mut result)?;
                } else if type_name.contains("Vec<String>") {
                    self.serialize_list::<String>(any_ref, &mut result)?;
                } else {
                    let serialized = self.serialize(any_ref, type_name)?;
                    result.extend_from_slice(&serialized);
                }
                Ok(result)
            }
            ValueCategory::Map => {
                // Add category marker (1 byte)
                result.push(0x03);

                // Add type name length and type name
                let type_name = value.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // For maps, use the registry to serialize
                let any_ref = value.value.as_any()?;
                let serialized = self.serialize(any_ref, type_name)?;
                result.extend_from_slice(&serialized);
                Ok(result)
            }
            ValueCategory::Struct => {
                // Add category marker (1 byte)
                result.push(0x04);

                // Add type name length and type name
                let type_name = value.value.type_name();
                let type_bytes = type_name.as_bytes();
                result.push(type_bytes.len() as u8);
                result.extend_from_slice(type_bytes);

                // For structs, use the registry to serialize
                let any_ref = value.value.as_any()?;
                let serialized = self.serialize(any_ref, type_name)?;
                result.extend_from_slice(&serialized);
                Ok(result)
            }
            ValueCategory::Null => {
                // Just a marker byte for null
                result.push(0x05);
                Ok(result)
            }
            ValueCategory::Bytes => {
                // Add category marker (1 byte)
                result.push(0x06);

                // For bytes, just include the raw data
                if let Ok(bytes) = value.value.as_arc::<Vec<u8>>() {
                    // Add length (4 bytes) followed by the actual bytes
                    let len_bytes = (bytes.len() as u32).to_be_bytes();
                    result.extend_from_slice(&len_bytes);
                    result.extend_from_slice(&bytes);
                    Ok(result)
                } else {
                    Err(anyhow!("Value is not a byte array"))
                }
            }
        }
    }

    /// Deserialize bytes to an ArcValueType
    pub fn deserialize_value(&self, bytes: &[u8]) -> Result<ArcValueType> {
        if bytes.is_empty() {
            return Err(anyhow!("Empty byte array"));
        }

        // First byte is the category marker
        let category = match bytes[0] {
            0x01 => ValueCategory::Primitive,
            0x02 => ValueCategory::List,
            0x03 => ValueCategory::Map,
            0x04 => ValueCategory::Struct,
            0x05 => ValueCategory::Null,
            0x06 => ValueCategory::Bytes,
            _ => return Err(anyhow!("Invalid category marker: {}", bytes[0])),
        };

        // For null, just return a null value
        if category == ValueCategory::Null {
            return Ok(ArcValueType::null());
        }

        // For all other types, we need to extract the type name
        if bytes.len() < 2 {
            return Err(anyhow!("Byte array too short"));
        }

        let type_name_len = bytes[1] as usize;
        if bytes.len() < 2 + type_name_len {
            return Err(anyhow!("Byte array too short for type name"));
        }

        let type_name_bytes = &bytes[2..2 + type_name_len];
        let type_name = String::from_utf8(type_name_bytes.to_vec())
            .map_err(|_| anyhow!("Invalid type name encoding"))?;

        // The actual data starts after the type name
        let data_start = 2 + type_name_len;

        // Regular processing for other types
        match category {
            ValueCategory::Primitive => {
                // Fast path for common primitive types
                match type_name.as_str() {
                    "i32" => {
                        let value: i32 = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    "i64" => {
                        let value: i64 = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    "f32" => {
                        let value: f32 = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    "f64" => {
                        let value: f64 = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    "bool" => {
                        let value: bool = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    "String" | "std::string::String" | "alloc::string::String" => {
                        let value: String = bincode::deserialize(&bytes[data_start..])?;
                        Ok(ArcValueType::new_primitive(value))
                    }
                    _ => {
                        // Use registry for other primitive types
                        let boxed_any = self.deserialize(&type_name, &bytes[data_start..])?;
                        let erased = ErasedArc::from_boxed_any(boxed_any)?;
                        Ok(ArcValueType::new(erased, ValueCategory::Primitive))
                    }
                }
            }
            ValueCategory::List => {
                // Fast path for common list types
                if type_name.contains("Vec<i32>") {
                    let list: Vec<i32> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else if type_name.contains("Vec<i64>") {
                    let list: Vec<i64> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else if type_name.contains("Vec<f32>") {
                    let list: Vec<f32> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else if type_name.contains("Vec<f64>") {
                    let list: Vec<f64> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else if type_name.contains("Vec<bool>") {
                    let list: Vec<bool> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else if type_name.contains("Vec<String>") {
                    let list: Vec<String> = bincode::deserialize(&bytes[data_start..])?;
                    Ok(ArcValueType::new_list(list))
                } else {
                    // Use registry for other list types
                    let boxed_any = self.deserialize(&type_name, &bytes[data_start..])?;
                    let erased = ErasedArc::from_boxed_any(boxed_any)?;
                    Ok(ArcValueType::new(erased, ValueCategory::List))
                }
            }
            ValueCategory::Map => {
                // Fast path for common map types
                if type_name.contains("HashMap<String, String>")
                    || type_name.contains("HashMap<std::string::String, std::string::String>")
                    || type_name.contains("HashMap<alloc::string::String, alloc::string::String>")
                {
                    match bincode::deserialize::<HashMap<String, String>>(&bytes[data_start..]) {
                        Ok(map) => Ok(ArcValueType::new_map(map)),
                        Err(_) => {
                            // Fall back to registry
                            let boxed_any = self.deserialize(&type_name, &bytes[data_start..])?;
                            let erased = ErasedArc::from_boxed_any(boxed_any)?;
                            Ok(ArcValueType::new(erased, ValueCategory::Map))
                        }
                    }
                } else if type_name.contains("HashMap<String, i32>")
                    || type_name.contains("HashMap<std::string::String, i32>")
                {
                    match bincode::deserialize::<HashMap<String, i32>>(&bytes[data_start..]) {
                        Ok(map) => Ok(ArcValueType::new_map(map)),
                        Err(_) => {
                            // Fall back to registry
                            let boxed_any = self.deserialize(&type_name, &bytes[data_start..])?;
                            let erased = ErasedArc::from_boxed_any(boxed_any)?;
                            Ok(ArcValueType::new(erased, ValueCategory::Map))
                        }
                    }
                } else if type_name.contains("HashMap<String, f64>")
                    || type_name.contains("HashMap<std::string::String, f64>")
                {
                    match bincode::deserialize::<HashMap<String, f64>>(&bytes[data_start..]) {
                        Ok(map) => Ok(ArcValueType::new_map(map)),
                        Err(_) => {
                            // Fall back to registry
                            let boxed_any = self.deserialize(&type_name, &bytes[data_start..])?;
                            let erased = ErasedArc::from_boxed_any(boxed_any)?;
                            Ok(ArcValueType::new(erased, ValueCategory::Map))
                        }
                    }
                } else {
                    // Use registry for other map types
                    match self.deserialize(&type_name, &bytes[data_start..]) {
                        Ok(boxed_any) => {
                            let erased = ErasedArc::from_boxed_any(boxed_any)?;
                            Ok(ArcValueType::new(erased, ValueCategory::Map))
                        }
                        Err(_)
                            if type_name.contains("HashMap<") && type_name.contains("String") =>
                        {
                            // Try to deserialize as generic string map
                            match bincode::deserialize::<HashMap<String, String>>(
                                &bytes[data_start..],
                            ) {
                                Ok(map) => Ok(ArcValueType::new_map(map)),
                                Err(e) => Err(e.into()),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
            }
            ValueCategory::Struct => {
                // For structs, we'll use lazy deserialization
                // Store the type name and raw bytes in a LazyDeserializer

                // Create a simple container for the raw bytes and type name
                let lazy_deserializer = LazyDeserializer {
                    type_name: type_name.clone(),
                    bytes: bytes[data_start..].to_vec(),
                };

                // Return as a Bytes category, but we'll handle it specially
                return Ok(ArcValueType::new(
                    ErasedArc::from_value(lazy_deserializer),
                    ValueCategory::Bytes, // Use Bytes to indicate lazy deserialization
                ));
            }
            ValueCategory::Bytes => {
                // For bytes, extract the length and the actual bytes
                if bytes.len() < data_start + 4 {
                    return Err(anyhow!("Byte array too short for length field"));
                }

                let len_bytes = [
                    bytes[data_start],
                    bytes[data_start + 1],
                    bytes[data_start + 2],
                    bytes[data_start + 3],
                ];
                let data_len = u32::from_be_bytes(len_bytes) as usize;

                if bytes.len() < data_start + 4 + data_len {
                    return Err(anyhow!("Byte array too short for data"));
                }

                let data = bytes[data_start + 4..data_start + 4 + data_len].to_vec();
                Ok(ArcValueType::new(
                    ErasedArc::from_value(data),
                    ValueCategory::Bytes,
                ))
            }
            ValueCategory::Null => {
                // This should have been handled above
                unreachable!()
            }
        }
    }

    // Helper methods for serializing common types
    fn serialize_primitive<T: 'static + Serialize>(
        &self,
        value: &dyn Any,
        result: &mut Vec<u8>,
    ) -> Result<()> {
        if let Some(typed_value) = value.downcast_ref::<T>() {
            let serialized = bincode::serialize(typed_value)?;
            result.extend_from_slice(&serialized);
            Ok(())
        } else {
            Err(anyhow!("Failed to downcast primitive value"))
        }
    }

    fn serialize_list<T: 'static + Serialize>(
        &self,
        value: &dyn Any,
        result: &mut Vec<u8>,
    ) -> Result<()> {
        if let Some(typed_value) = value.downcast_ref::<Vec<T>>() {
            let serialized = bincode::serialize(typed_value)?;
            result.extend_from_slice(&serialized);
            Ok(())
        } else {
            Err(anyhow!("Failed to downcast list value"))
        }
    }
}

/// A type-erased value container with Arc preservation
#[derive(Debug, Clone)]
pub struct ArcValueType {
    /// Categorizes the value for dispatch
    pub category: ValueCategory,
    /// The contained type-erased value
    pub value: ErasedArc,
}

impl ArcValueType {
    /// Create a new ArcValueType
    pub fn new(value: ErasedArc, category: ValueCategory) -> Self {
        Self { category, value }
    }

    /// Create a new primitive value
    pub fn new_primitive<T: 'static + fmt::Debug + Send + Sync>(value: T) -> Self {
        let arc = Arc::new(value);
        Self {
            category: ValueCategory::Primitive,
            value: ErasedArc::new(arc),
        }
    }

    /// Create a new struct value
    pub fn from_struct<T: 'static + fmt::Debug + Send + Sync>(value: T) -> Self {
        let arc = Arc::new(value);
        Self {
            category: ValueCategory::Struct,
            value: ErasedArc::new(arc),
        }
    }

    /// Create a new list value
    pub fn new_list<T: 'static + fmt::Debug + Send + Sync>(values: Vec<T>) -> Self {
        let arc = Arc::new(values);
        Self {
            category: ValueCategory::List,
            value: ErasedArc::new(arc),
        }
    }

    /// Create a new list from existing vector
    pub fn from_list<T: 'static + fmt::Debug + Send + Sync>(values: Vec<T>) -> Self {
        Self::new_list(values)
    }

    /// Create a new map value
    pub fn new_map<K, V>(map: HashMap<K, V>) -> Self
    where
        K: 'static + fmt::Debug + Send + Sync,
        V: 'static + fmt::Debug + Send + Sync,
    {
        let arc = Arc::new(map);
        Self {
            category: ValueCategory::Map,
            value: ErasedArc::new(arc),
        }
    }

    /// Create a new map from existing map
    pub fn from_map<K, V>(map: HashMap<K, V>) -> Self
    where
        K: 'static + fmt::Debug + Send + Sync,
        V: 'static + fmt::Debug + Send + Sync,
    {
        Self::new_map(map)
    }

    /// Create a null value
    pub fn null() -> Self {
        Self {
            category: ValueCategory::Null,
            value: ErasedArc::new(Arc::new(())),
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        self.category == ValueCategory::Null
    }

    /// Get value as a reference of the specified type
    pub fn as_type_ref<T: 'static>(&self) -> Result<Arc<T>> {
        self.value.as_arc::<T>()
    }

    /// Get list as a reference of the specified element type
    pub fn as_list_ref<T: 'static>(&self) -> Result<Arc<Vec<T>>> {
        if self.category != ValueCategory::List {
            return Err(anyhow!("Value is not a list"));
        }
        self.value.as_arc::<Vec<T>>()
    }

    /// Get map as a reference of the specified key/value types
    pub fn as_map_ref<K: 'static, V: 'static>(&self) -> Result<Arc<HashMap<K, V>>> {
        if self.category != ValueCategory::Map {
            return Err(anyhow!("Value is not a map"));
        }
        self.value.as_arc::<HashMap<K, V>>()
    }

    /// Get value as the specified type (makes a clone)
    pub fn as_type<T: 'static + Clone>(&self) -> Result<T> {
        let arc = self.as_type_ref::<T>()?;
        Ok((*arc).clone())
    }

    /// Get struct as a reference of the specified type
    pub fn as_struct_ref<T: 'static + Clone + Send + Sync + for<'de> Deserialize<'de>>(
        &self,
    ) -> Result<Arc<T>> {
        // First handle the case where we have a LazyDeserializer
        if self.category == ValueCategory::Bytes {
            // Check if we have a LazyDeserializer
            if let Ok(lazy) = self.value.as_any().and_then(|any| {
                any.downcast_ref::<LazyDeserializer>()
                    .ok_or_else(|| anyhow!("Not a LazyDeserializer"))
            }) {
                // We have a LazyDeserializer, so try to deserialize the bytes to the requested type T
                match bincode::deserialize::<T>(&lazy.bytes) {
                    Ok(value) => {
                        return Ok(Arc::new(value));
                    }
                    Err(e) => {
                        // Failed direct deserialization, log the error and return it
                        log::debug!(
                            "Failed to deserialize as {} from original type {}: {}",
                            std::any::type_name::<T>(),
                            lazy.type_name,
                            e
                        );
                        return Err(anyhow!(
                            "Failed to deserialize to type {} from original type {}: {}",
                            std::any::type_name::<T>(),
                            lazy.type_name,
                            e
                        ));
                    }
                }
            }
            // Standard bytes handling for non-LazyDeserializer bytes
            else if let Ok(bytes) = self.value.as_arc::<Vec<u8>>() {
                // Try direct deserialization
                match bincode::deserialize::<T>(bytes.as_slice()) {
                    Ok(value) => {
                        return Ok(Arc::new(value));
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "Failed to deserialize bytes to type {}: {}",
                            std::any::type_name::<T>(),
                            e
                        ));
                    }
                }
            }

            // If we reach here, we couldn't handle the bytes value
            return Err(anyhow!(
                "Unable to deserialize bytes to requested type {}",
                std::any::type_name::<T>()
            ));
        }

        // Handle struct category
        if self.category == ValueCategory::Struct {
            // Try to directly access as an Arc<T>
            match self.value.as_arc::<T>() {
                Ok(arc_t) => {
                    return Ok(arc_t);
                }
                Err(e) => {
                    // Try to serialize then deserialize as a last resort
                    if let Ok(bytes) = self.value.to_bytes() {
                        // Try direct deserialization
                        match bincode::deserialize::<T>(&bytes) {
                            Ok(value) => {
                                return Ok(Arc::new(value));
                            }
                            Err(deser_err) => {
                                return Err(anyhow!("Type mismatch: cannot convert struct to requested type. Errors: original: {}, deserialize: {}", 
                                                   e, deser_err));
                            }
                        }
                    }

                    return Err(anyhow!("Could not convert struct to requested type: {}", e));
                }
            }
        }

        // If we get here, the category is something else
        Err(anyhow!(
            "Value is not a struct or serialized bytes, category: {:?}",
            self.category
        ))
    }
}

/// Container for serialized value with category information
#[derive(Debug)]
pub struct SerializedValue {
    /// The original value category
    pub category: ValueCategory,
    /// The serialized bytes
    pub data: Vec<u8>,
}

/// Legacy ValueType enum for backward compatibility
/// This will be deprecated in future versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<ValueType>),
    Map(HashMap<String, ValueType>),
}

impl ValueType {
    /// Convert a JSON value to a ValueType
    pub fn from_json(json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => ValueType::Null,
            serde_json::Value::Bool(b) => ValueType::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    ValueType::Number(f)
                } else {
                    ValueType::Null
                }
            }
            serde_json::Value::String(s) => ValueType::String(s),
            serde_json::Value::Array(arr) => {
                let values: Vec<ValueType> =
                    arr.into_iter().map(|v| ValueType::from_json(v)).collect();
                ValueType::Array(values)
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, ValueType::from_json(v));
                }
                ValueType::Map(map)
            }
        }
    }
}
