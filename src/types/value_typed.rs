// runar_common/src/types/value_typed.rs
//
// Type-preserving ValueType system for Runar

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};
use bincode::{config};

/// Type information for serialization/deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeInfo {
    Primitive(PrimitiveType),
    List(Box<TypeInfo>),
    Map(Box<TypeInfo>, Box<TypeInfo>), // Key, Value types
    Struct(String),                    // Struct type name
    Null,
    Raw,                               // Raw bytes
}

/// Primitive type identifiers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimitiveType {
    String,
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    Bytes,
}

/// A typed container for raw bytes with type information for lazy deserialization
#[derive(Debug)]
pub struct TypedBytes {
    /// Raw serialized data
    pub bytes: Vec<u8>,
    /// Type information for deserialization
    pub type_info: TypeInfo,
    /// Cached deserialized value (Option to allow for lazy deserialization)
    pub deserialized: Option<Box<dyn Any + Send + Sync>>,
}

// Manual clone implementation since we can't derive Clone for Box<dyn Any>
impl Clone for TypedBytes {
    fn clone(&self) -> Self {
        TypedBytes {
            bytes: self.bytes.clone(),
            type_info: self.type_info.clone(),
            deserialized: None, // Don't clone the cached value, it will be recomputed if needed
        }
    }
}

impl TypedBytes {
    /// Create a new TypedBytes container
    pub fn new(bytes: Vec<u8>, type_info: TypeInfo) -> Self {
        TypedBytes {
            bytes,
            type_info,
            deserialized: None,
        }
    }

    /// Attempt to deserialize the bytes into the specified type
    pub fn deserialize<T: 'static + Clone + Send + Sync + for<'a> Deserialize<'a>>(&self) -> Result<T> {
        // If already deserialized, return the cached value
        if let Some(deserialized) = &self.deserialized {
            if let Some(value) = deserialized.downcast_ref::<T>() {
                return Ok(value.clone());
            }
        }

        // Otherwise, deserialize
        let value: T = bincode::deserialize(&self.bytes)?;
        
        // In a real implementation, we'd cache the value here, but we'll skip it 
        // to avoid borrowing issues for now
        
        Ok(value)
    }
}

/// Common interface for all value types
pub trait ValueBase: Debug + Send + Sync {
    /// Serialize this value to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;
    
    /// Get type information for this value
    fn type_info(&self) -> TypeInfo;
    
    /// Get this value as a dynamic Any trait object
    fn as_any(&self) -> &dyn Any;
    
    /// Clone this value into a new Box
    fn clone_box(&self) -> Box<dyn ValueBase + Send + Sync>;
}

/// Interface for value type conversion
pub trait ValueConvert {
    /// Convert to a specific type
    fn as_type<U: 'static + Clone + Send + Sync>(&self) -> Result<U>;
    
    /// Convert to a map
    fn as_map<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash, 
             V: 'static + Clone + Send + Sync>(&self) -> Result<HashMap<K, V>>;
    
    /// Convert to a list
    fn as_list<U: 'static + Clone + Send + Sync>(&self) -> Result<Vec<U>>;
    
    /// Type-safe conversion using Rust's type system
    fn try_into<U: 'static>(&self) -> Result<U>
        where U: TryFrom<Box<dyn Any>>;
}

/// For struct type preservation
pub trait CustomStruct: Debug + Any + Send + Sync {
    /// Serialize this struct to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;
    
    /// Get the type name of this struct
    fn type_name(&self) -> &'static str;
    
    /// Clone this struct into a new Box
    fn clone_box(&self) -> Box<dyn CustomStruct + Send + Sync>;
    
    /// Get this struct as a dynamic Any trait object
    fn as_any(&self) -> &dyn Any;
}

// Helper for cloning trait objects
impl Clone for Box<dyn CustomStruct + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// Blanket implementation of CustomStruct for any type that implements the required traits
impl<T: 'static + Debug + Clone + Send + Sync + Serialize> CustomStruct for T {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| anyhow!("Serialization error: {}", e))
    }
    
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
    
    fn clone_box(&self) -> Box<dyn CustomStruct + Send + Sync> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Primary Value type that preserves type information
#[derive(Debug)]
pub enum Value<T> {
    /// Basic typed value
    Value(T),
    /// Homogeneous list of values (stored directly as Vec<T>)
    List(Vec<T>),
    /// Custom struct with type preservation
    Struct(Box<dyn CustomStruct + Send + Sync>),
    /// Null/None value
    Null,
    /// Raw bytes with type information for lazy deserialization
    Bytes(TypedBytes),
}

// Manual clone implementation to handle Box<dyn CustomStruct>
impl<T: Clone> Clone for Value<T> {
    fn clone(&self) -> Self {
        match self {
            Value::Value(value) => Value::Value(value.clone()),
            Value::List(list) => Value::List(list.clone()),
            Value::Struct(s) => Value::Struct(s.clone()),
            Value::Null => Value::Null,
            Value::Bytes(bytes) => Value::Bytes(bytes.clone()),
        }
    }
}

impl<T: 'static + Clone + Send + Sync + Debug> Value<T> {
    /// Primary constructor for creating a Value from a basic type
    pub fn new(value: T) -> Self {
        Value::Value(value)
    }
    
    /// Create a new list of values
    pub fn new_list(values: Vec<T>) -> Self {
        Value::List(values)
    }
    
    /// Create a null value
    pub fn null() -> Value<()> {
        Value::Null
    }
}

impl<T: 'static + Clone + Send + Sync + Serialize + Debug> ValueBase for Value<T> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Value::Value(value) => {
                let mut buffer = Vec::new();
                
                // Add type marker
                buffer.push(0x01); // Marker for ValueType<T> with primitive T
                
                // Serialize the value
                let serialized = bincode::serialize(value)?;
                buffer.extend_from_slice(&serialized);
                
                Ok(buffer)
            },
            Value::List(values) => {
                let mut buffer = Vec::new();
                
                // Add type marker
                buffer.push(0x02); // Marker for ValueType<Vec<T>> for lists
                
                // Serialize the list elements directly
                let serialized = bincode::serialize(values)?;
                buffer.extend_from_slice(&serialized);
                
                Ok(buffer)
            },
            Value::Struct(custom_struct) => {
                let mut buffer = Vec::new();
                
                // Add type marker
                buffer.push(0x04); // Marker for ValueType<T> with Struct T
                
                // Get the type name and bytes from the custom struct
                let type_name = custom_struct.type_name().to_string();
                let struct_bytes = custom_struct.to_bytes()?;
                
                // Encode the type name length and bytes
                let serialized_name = bincode::serialize(&type_name)?;
                buffer.extend_from_slice(&serialized_name);
                
                // Append the struct's serialized bytes
                buffer.extend_from_slice(&struct_bytes);
                
                Ok(buffer)
            },
            Value::Null => {
                // Just a marker for Null
                Ok(vec![0x05])
            },
            Value::Bytes(typed_bytes) => {
                // Just pass through the already serialized bytes
                let mut buffer = Vec::new();
                buffer.push(0x06); // Marker for Raw Bytes
                buffer.extend_from_slice(&typed_bytes.bytes);
                Ok(buffer)
            },
        }
    }
    
    fn type_info(&self) -> TypeInfo {
        match self {
            Value::Value(_) => {
                // Determine the primitive type based on T
                let primitive_type = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
                    PrimitiveType::String
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
                    PrimitiveType::Int32
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i64>() {
                    PrimitiveType::Int64
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
                    PrimitiveType::Float32
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>() {
                    PrimitiveType::Float64
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
                    PrimitiveType::Bool
                } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Vec<u8>>() {
                    PrimitiveType::Bytes
                } else {
                    // Default to using bincode for custom types
                    return TypeInfo::Struct(std::any::type_name::<T>().to_string());
                };
                
                TypeInfo::Primitive(primitive_type)
            },
            Value::List(_) => {
                let element_type = Self::element_type_info();
                TypeInfo::List(Box::new(element_type))
            },
            Value::Struct(custom_struct) => {
                TypeInfo::Struct(custom_struct.type_name().to_string())
            },
            Value::Null => TypeInfo::Null,
            Value::Bytes(typed_bytes) => typed_bytes.type_info.clone(),
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        match self {
            Value::Value(value) => value as &dyn Any,
            _ => self as &dyn Any,
        }
    }
    
    fn clone_box(&self) -> Box<dyn ValueBase + Send + Sync> {
        Box::new(self.clone())
    }
}

impl<T: 'static + Clone + Send + Sync + Debug> Value<T> {
    // Helper to determine element type info for lists
    fn element_type_info() -> TypeInfo {
        // Determine the primitive type based on T
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
            TypeInfo::Primitive(PrimitiveType::String)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
            TypeInfo::Primitive(PrimitiveType::Int32)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i64>() {
            TypeInfo::Primitive(PrimitiveType::Int64)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
            TypeInfo::Primitive(PrimitiveType::Float32)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>() {
            TypeInfo::Primitive(PrimitiveType::Float64)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
            TypeInfo::Primitive(PrimitiveType::Bool)
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Vec<u8>>() {
            TypeInfo::Primitive(PrimitiveType::Bytes)
        } else {
            // Default to using struct for custom types
            TypeInfo::Struct(std::any::type_name::<T>().to_string())
        }
    }
}

impl<T: 'static + Clone + Send + Sync + for<'a> Deserialize<'a> + Debug> ValueConvert for Value<T> {
    fn as_type<U: 'static + Clone + Send + Sync>(&self) -> Result<U> {
        match self {
            Value::Value(value) => {
                // Check if T and U are the same type
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
                    // Safe to transmute since we verified the types match
                    let ptr = value as *const T as *const U;
                    let ref_u = unsafe { &*ptr };
                    Ok(ref_u.clone())
                } else {
                    Err(anyhow!("Type mismatch: cannot convert {:?} to requested type", std::any::type_name::<T>()))
                }
            },
            Value::Bytes(typed_bytes) => {
                // For types that implement Deserialize, we can attempt deserialization
                if std::any::TypeId::of::<U>() == std::any::TypeId::of::<T>() ||
                   std::any::TypeId::of::<U>() == std::any::TypeId::of::<Vec<T>>() {
                    // We need to ensure U implements Deserialize for this code path
                    match bincode::deserialize::<T>(&typed_bytes.bytes) {
                        Ok(value) => {
                            // Now convert T to U if possible
                            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
                                let ptr = &value as *const T as *const U;
                                let ref_u = unsafe { &*ptr };
                                Ok(ref_u.clone())
                            } else {
                                Err(anyhow!("Type mismatch: cannot convert deserialized value to requested type"))
                            }
                        },
                        Err(e) => Err(anyhow!("Deserialization error: {}", e))
                    }
                } else {
                    Err(anyhow!("Type mismatch for deserialization"))
                }
            },
            _ => Err(anyhow!("Cannot convert {:?} to requested type", self)),
        }
    }
    
    fn as_map<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash, 
             V: 'static + Clone + Send + Sync>(&self) -> Result<HashMap<K, V>> {
        Err(anyhow!("Value<T> does not directly store maps, use MapValue<K, V> instead"))
    }
    
    fn as_list<U: 'static + Clone + Send + Sync>(&self) -> Result<Vec<U>> {
        match self {
            Value::List(values) => {
                // Check if U and T are the same type
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
                    // Create a new Vec<U> and copy all elements from values
                    // This is safer than using raw pointers and transmute
                    let mut result = Vec::with_capacity(values.len());
                    for value in values {
                        // Safe cast since we verified the types match
                        let ptr = value as *const T as *const U;
                        let ref_u = unsafe { &*ptr };
                        result.push(ref_u.clone());
                    }
                    Ok(result)
                } else {
                    Err(anyhow!("Type mismatch: cannot convert list of {:?} to list of requested type", std::any::type_name::<T>()))
                }
            },
            Value::Bytes(typed_bytes) => {
                // For bytes, we need to check if it contains a list of T
                if let TypeInfo::List(_) = &typed_bytes.type_info {
                    // First deserialize as Vec<T>
                    match bincode::deserialize::<Vec<T>>(&typed_bytes.bytes) {
                        Ok(list_t) => {
                            // Then convert Vec<T> to Vec<U> if types match
                            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>() {
                                let mut result = Vec::with_capacity(list_t.len());
                                for value in &list_t {
                                    let ptr = value as *const T as *const U;
                                    let ref_u = unsafe { &*ptr };
                                    result.push(ref_u.clone());
                                }
                                Ok(result)
                            } else {
                                Err(anyhow!("Type mismatch: cannot convert list of {:?} to list of requested type", std::any::type_name::<T>()))
                            }
                        },
                        Err(e) => Err(anyhow!("Failed to deserialize TypedBytes as Vec<T>: {}", e))
                    }
                } else {
                    Err(anyhow!("TypedBytes does not contain a list"))
                }
            },
            _ => Err(anyhow!("Not a list: {:?}", self)),
        }
    }
    
    fn try_into<U: 'static>(&self) -> Result<U>
        where U: TryFrom<Box<dyn Any>> {
        match self {
            Value::Value(value) => {
                let boxed: Box<dyn Any> = Box::new(value.clone());
                U::try_from(boxed).map_err(|_| anyhow!("Type conversion failed"))
            },
            _ => Err(anyhow!("Cannot convert {:?} using try_into", self)),
        }
    }
}

/// Map value type that preserves key and value type information
#[derive(Debug)]
pub struct MapValue<K, V> {
    /// The actual map entries
    pub entries: HashMap<K, V>,
    /// Optional serialized form for lazy deserialization
    pub serialized: Option<TypedBytes>,
    /// Type markers (needed for type inference)
    _key_marker: PhantomData<K>,
    _value_marker: PhantomData<V>,
}

// Manual clone implementation for MapValue to handle TypedBytes
impl<K: Clone, V: Clone> Clone for MapValue<K, V> {
    fn clone(&self) -> Self {
        MapValue {
            entries: self.entries.clone(),
            serialized: self.serialized.clone(),
            _key_marker: PhantomData,
            _value_marker: PhantomData,
        }
    }
}

impl<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash + Debug, 
     V: 'static + Clone + Send + Sync + Debug> MapValue<K, V> {
    /// Primary constructor for creating a MapValue from a HashMap
    pub fn new(entries: HashMap<K, V>) -> Self {
        MapValue {
            entries,
            serialized: None,
            _key_marker: PhantomData,
            _value_marker: PhantomData,
        }
    }
    
    /// Create from serialized bytes (for lazy deserialization)
    pub fn from_bytes(bytes: Vec<u8>, type_info: TypeInfo) -> Self {
        MapValue {
            entries: HashMap::new(),
            serialized: Some(TypedBytes::new(bytes, type_info)),
            _key_marker: PhantomData,
            _value_marker: PhantomData,
        }
    }
}

impl<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash + Serialize + Debug, 
     V: 'static + Clone + Send + Sync + Serialize + Debug> ValueBase for MapValue<K, V> {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        // If we already have a serialized form, use that
        if let Some(typed_bytes) = &self.serialized {
            let mut buffer = Vec::new();
            buffer.push(0x03); // Marker for MapValueType<K, V>
            buffer.extend_from_slice(&typed_bytes.bytes);
            return Ok(buffer);
        }
        
        // Otherwise, serialize the map
        let mut buffer = Vec::new();
        
        // Add type marker
        buffer.push(0x03); // Marker for MapValueType<K, V>
        
        // Serialize the map
        let serialized = bincode::serialize(&self.entries)?;
        buffer.extend_from_slice(&serialized);
        
        Ok(buffer)
    }
    
    fn type_info(&self) -> TypeInfo {
        // If we already have type info, use that
        if let Some(typed_bytes) = &self.serialized {
            return typed_bytes.type_info.clone();
        }
        
        // Otherwise, compute the type info
        let key_type = Self::key_type_info();
        let value_type = Self::value_type_info();
        
        TypeInfo::Map(Box::new(key_type), Box::new(value_type))
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn clone_box(&self) -> Box<dyn ValueBase + Send + Sync> {
        Box::new(self.clone())
    }
}

impl<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash + Debug, 
     V: 'static + Clone + Send + Sync + Debug> MapValue<K, V> {
    // Helper to determine key type info
    fn key_type_info() -> TypeInfo {
        // Determine the primitive type based on K
        if std::any::TypeId::of::<K>() == std::any::TypeId::of::<String>() {
            TypeInfo::Primitive(PrimitiveType::String)
        } else if std::any::TypeId::of::<K>() == std::any::TypeId::of::<i32>() {
            TypeInfo::Primitive(PrimitiveType::Int32)
        } else if std::any::TypeId::of::<K>() == std::any::TypeId::of::<i64>() {
            TypeInfo::Primitive(PrimitiveType::Int64)
        } else if std::any::TypeId::of::<K>() == std::any::TypeId::of::<bool>() {
            TypeInfo::Primitive(PrimitiveType::Bool)
        } else {
            // Default to using struct for custom types
            TypeInfo::Struct(std::any::type_name::<K>().to_string())
        }
    }
    
    // Helper to determine value type info
    fn value_type_info() -> TypeInfo {
        // Determine the primitive type based on V
        if std::any::TypeId::of::<V>() == std::any::TypeId::of::<String>() {
            TypeInfo::Primitive(PrimitiveType::String)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<i32>() {
            TypeInfo::Primitive(PrimitiveType::Int32)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<i64>() {
            TypeInfo::Primitive(PrimitiveType::Int64)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<f32>() {
            TypeInfo::Primitive(PrimitiveType::Float32)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<f64>() {
            TypeInfo::Primitive(PrimitiveType::Float64)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<bool>() {
            TypeInfo::Primitive(PrimitiveType::Bool)
        } else if std::any::TypeId::of::<V>() == std::any::TypeId::of::<Vec<u8>>() {
            TypeInfo::Primitive(PrimitiveType::Bytes)
        } else {
            // Default to using struct for custom types
            TypeInfo::Struct(std::any::type_name::<V>().to_string())
        }
    }
}

impl<K: 'static + Clone + Send + Sync + Eq + std::hash::Hash + for<'a> Deserialize<'a> + Debug, 
     V: 'static + Clone + Send + Sync + for<'a> Deserialize<'a> + Debug> ValueConvert for MapValue<K, V> {
    fn as_type<U: 'static + Clone + Send + Sync>(&self) -> Result<U> {
        Err(anyhow!("MapValue<K, V> does not directly convert to basic types"))
    }
    
    fn as_map<KU: 'static + Clone + Send + Sync + Eq + std::hash::Hash, 
              VU: 'static + Clone + Send + Sync>(&self) -> Result<HashMap<KU, VU>> {
        // Check if K and KU, V and VU are the same types
        if std::any::TypeId::of::<K>() == std::any::TypeId::of::<KU>() &&
           std::any::TypeId::of::<V>() == std::any::TypeId::of::<VU>() {
            // If we have serialized data but no entries, deserialize first
            if self.entries.is_empty() && self.serialized.is_some() {
                let typed_bytes = self.serialized.as_ref().unwrap();
                let map: HashMap<K, V> = bincode::deserialize(&typed_bytes.bytes)?;
                
                // Create a new map and copy the entries
                let mut result = HashMap::with_capacity(map.len());
                for (k, v) in &map {
                    let key_ptr = k as *const K as *const KU;
                    let val_ptr = v as *const V as *const VU;
                    let key = unsafe { &*key_ptr }.clone();
                    let val = unsafe { &*val_ptr }.clone();
                    result.insert(key, val);
                }
                
                return Ok(result);
            }
            
            // Create a new map and copy the entries
            let mut result = HashMap::with_capacity(self.entries.len());
            for (k, v) in &self.entries {
                let key_ptr = k as *const K as *const KU;
                let val_ptr = v as *const V as *const VU;
                let key = unsafe { &*key_ptr }.clone();
                let val = unsafe { &*val_ptr }.clone();
                result.insert(key, val);
            }
            
            Ok(result)
        } else {
            Err(anyhow!("Type mismatch: cannot convert map of {:?} -> {:?} to map of requested types", 
                std::any::type_name::<K>(), std::any::type_name::<V>()))
        }
    }
    
    fn as_list<U: 'static + Clone + Send + Sync>(&self) -> Result<Vec<U>> {
        Err(anyhow!("MapValue<K, V> does not directly convert to lists"))
    }
    
    fn try_into<U: 'static>(&self) -> Result<U>
        where U: TryFrom<Box<dyn Any>> {
        let boxed: Box<dyn Any> = Box::new(self.entries.clone());
        U::try_from(boxed).map_err(|_| anyhow!("Type conversion failed"))
    }
}

/// Creates a Value from raw bytes with type information
pub fn value_from_bytes(data: &[u8]) -> Result<Box<dyn ValueBase>> {
    if data.is_empty() {
        return Err(anyhow!("Empty data"));
    }
    
    // Extract type information but don't deserialize payload yet
    let type_marker = data[0];
    // We'll implement this parser in a separate function
    let (type_info, offset) = parse_type_info(&data[1..])?;
    
    // Store bytes and type info for lazy deserialization
    let typed_bytes = TypedBytes::new(
        data[offset+1..].to_vec(), // +1 to skip the marker
        type_info.clone()
    );
    
    // Create appropriate Value with TypedBytes
    match type_marker {
        0x01 => Ok(Box::new(Value::<()>::Bytes(typed_bytes))),
        0x02 => Ok(Box::new(Value::<()>::Bytes(typed_bytes))),
        0x03 => {
            // Create MapValue with serialized data for lazy deserialization
            Ok(Box::new(MapValue::<(), ()>::from_bytes(typed_bytes.bytes, typed_bytes.type_info)))
        },
        0x04 => Ok(Box::new(Value::<()>::Bytes(typed_bytes))),
        0x05 => Ok(Box::new(Value::<()>::Null)),
        0x06 => Ok(Box::new(Value::<Vec<u8>>::new(data[offset+1..].to_vec()))),
        _ => Err(anyhow!("Unknown type marker: {}", type_marker)),
    }
}

/// Helper function to parse type information from serialized bytes
fn parse_type_info(data: &[u8]) -> Result<(TypeInfo, usize)> {
    if data.is_empty() {
        return Err(anyhow!("Empty data for type info"));
    }
    
    // First byte indicates the type info format
    let type_info_marker = data[0];
    let mut offset = 1; // Skip the marker byte
    
    match type_info_marker {
        // Primitive type
        0x01 => {
            if data.len() < 2 {
                return Err(anyhow!("Invalid primitive type data"));
            }
            
            let primitive_type = match data[1] {
                0x01 => PrimitiveType::String,
                0x02 => PrimitiveType::Int32,
                0x03 => PrimitiveType::Int64,
                0x04 => PrimitiveType::Float32,
                0x05 => PrimitiveType::Float64,
                0x06 => PrimitiveType::Bool,
                0x07 => PrimitiveType::Bytes,
                _ => return Err(anyhow!("Unknown primitive type marker: {}", data[1])),
            };
            
            offset += 1; // Skip the primitive type marker
            Ok((TypeInfo::Primitive(primitive_type), offset))
        },
        
        // List type
        0x02 => {
            // Recursively parse the element type
            let (element_type, element_offset) = parse_type_info(&data[offset..])?;
            offset += element_offset;
            
            Ok((TypeInfo::List(Box::new(element_type)), offset))
        },
        
        // Map type
        0x03 => {
            // Recursively parse the key and value types
            let (key_type, key_offset) = parse_type_info(&data[offset..])?;
            offset += key_offset;
            
            let (value_type, value_offset) = parse_type_info(&data[offset..])?;
            offset += value_offset;
            
            Ok((TypeInfo::Map(Box::new(key_type), Box::new(value_type)), offset))
        },
        
        // Struct type
        0x04 => {
            // Read the length of the type name
            if data.len() < offset + 4 {
                return Err(anyhow!("Invalid struct type data"));
            }
            
            let name_len = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
            offset += 4;
            
            // Read the type name
            if data.len() < offset + name_len {
                return Err(anyhow!("Invalid struct type name data"));
            }
            
            let name = String::from_utf8(data[offset..offset+name_len].to_vec())
                .map_err(|_| anyhow!("Invalid struct type name"))?;
            
            offset += name_len;
            
            Ok((TypeInfo::Struct(name), offset))
        },
        
        // Null type
        0x05 => Ok((TypeInfo::Null, offset)),
        
        // Raw type
        0x06 => Ok((TypeInfo::Raw, offset)),
        
        _ => Err(anyhow!("Unknown type info marker: {}", type_info_marker)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_value_construction() {
        // Test primitive values
        let s = Value::<String>::new("Hello".to_string());
        let i = Value::<i32>::new(42);
        let f = Value::<f64>::new(3.14159);
        let b = Value::<bool>::new(true);
        
        // Test null
        let null = Value::<()>::null();
        
        // Basic assertions to check the types
        if let Value::Value(val) = &s {
            assert_eq!(val, "Hello");
        } else {
            panic!("Expected Value::Value variant");
        }
        
        if let Value::Value(val) = &i {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Value::Value variant");
        }
        
        if let Value::Null = &null {
            // This is expected
        } else {
            panic!("Expected Value::Null variant");
        }
    }
    
    #[test]
    fn test_list_construction() {
        // Test list of strings
        let str_list = Value::<String>::new_list(vec!["one".to_string(), "two".to_string()]);
        
        // Test list of integers
        let int_list = Value::<i32>::new_list(vec![1, 2, 3]);
        
        // Basic assertions
        if let Value::List(values) = &str_list {
            assert_eq!(values.len(), 2);
            assert_eq!(&values[0], "one");
        } else {
            panic!("Expected Value::List variant");
        }
        
        if let Value::List(values) = &int_list {
            assert_eq!(values.len(), 3);
            assert_eq!(values[1], 2);
        } else {
            panic!("Expected Value::List variant");
        }
    }
    
    #[test]
    fn test_map_construction() {
        // Create string to int map
        let mut map = HashMap::new();
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);
        
        let map_val = MapValue::<String, i32>::new(map.clone());
        
        // Test map entries
        assert_eq!(map_val.entries.len(), 2);
        assert_eq!(map_val.entries.get("one"), Some(&1));
        assert_eq!(map_val.entries.get("two"), Some(&2));
    }
    
    #[test]
    fn test_as_type_conversion() {
        // Create a value
        let i = Value::<i32>::new(42);
        
        // Try to convert to the correct type
        let result: Result<i32> = i.as_type();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        // Try to convert to the wrong type (should fail)
        let result: Result<String> = i.as_type();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_as_list_conversion() {
        // Create a list of integers
        let int_list = Value::<i32>::new_list(vec![1, 2, 3]);
        
        // Try to convert to the correct type
        let result: Result<Vec<i32>> = int_list.as_list();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
        
        // Try to convert to the wrong type (should fail)
        let result: Result<Vec<String>> = int_list.as_list();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_as_map_conversion() {
        // Create string to int map
        let mut map = HashMap::new();
        map.insert("one".to_string(), 1);
        map.insert("two".to_string(), 2);
        
        let map_val = MapValue::<String, i32>::new(map.clone());
        
        // Try to convert to the correct type
        let result: Result<HashMap<String, i32>> = map_val.as_map();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get("one"), Some(&1));
        
        // Try to convert to the wrong type (should fail)
        let result: Result<HashMap<i32, String>> = map_val.as_map();
        assert!(result.is_err());
    }
} 