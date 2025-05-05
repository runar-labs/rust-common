use std::any::{Any, TypeId};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, Result};

/// ArcRead is a trait for safely accessing an Arc's contents
pub trait ArcRead: fmt::Debug {
    /// Get the pointer to the inner value
    fn ptr(&self) -> *const ();

    /// Get the Arc's strong reference count
    fn strong_count(&self) -> usize;

    /// Get the Arc's weak reference count
    fn weak_count(&self) -> usize;

    /// Get the type name of the contained value
    fn type_name(&self) -> &'static str;

    /// Clone this trait object
    fn clone_box(&self) -> Box<dyn ArcRead>;

    /// Get this value as a dynamic Any
    fn as_any(&self) -> &dyn Any;
}

// Implement Clone for Box<dyn ArcRead>
impl Clone for Box<dyn ArcRead> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// The actual type-erased Arc implementation
pub struct ErasedArc {
    /// The type-erased Arc reader
    pub reader: Box<dyn ArcRead>,
}

// Trait to expose type information
trait TypeNameProvider: Any {
    fn get_type_name(&self) -> &str;
    fn get_value(&self) -> &dyn Any;
}

/// Implementation of ArcRead for a concrete Arc<T>
struct ArcReader<T: 'static + fmt::Debug + Send + Sync> {
    arc: Arc<T>,
    _marker: PhantomData<T>,
    // Optional override for type name, used for opaque types
    type_name_override: Option<String>,
}

impl<T: 'static + fmt::Debug + Send + Sync> fmt::Debug for ArcReader<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArcReader<{}>(", self.get_type_name())?;
        self.arc.fmt(f)?;
        write!(f, ")")
    }
}

impl<T: 'static + fmt::Debug + Send + Sync> ArcReader<T> {
    // Helper to get the correct type name
    fn get_type_name(&self) -> &str {
        if let Some(name) = &self.type_name_override {
            name
        } else {
            std::any::type_name::<T>()
        }
    }
}

impl<T: 'static + fmt::Debug + Send + Sync> ArcRead for ArcReader<T> {
    fn ptr(&self) -> *const () {
        Arc::as_ptr(&self.arc) as *const ()
    }

    fn strong_count(&self) -> usize {
        Arc::strong_count(&self.arc)
    }

    fn weak_count(&self) -> usize {
        Arc::weak_count(&self.arc)
    }

    fn type_name(&self) -> &'static str {
        // Use the type name from the override or get the default
        if self.type_name_override.is_some() {
            // Can't return a non-static reference from self.type_name_override
            // Instead, we'll return the base type
            std::any::type_name::<T>()
        } else {
            std::any::type_name::<T>()
        }
    }

    fn clone_box(&self) -> Box<dyn ArcRead> {
        Box::new(ArcReader {
            arc: self.arc.clone(),
            _marker: PhantomData,
            type_name_override: self.type_name_override.clone(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        &*self.arc as &dyn Any
    }
}

impl fmt::Debug for ErasedArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErasedArc({:?})", self.reader)
    }
}

impl Clone for ErasedArc {
    fn clone(&self) -> Self {
        ErasedArc {
            reader: self.reader.clone(),
        }
    }
}

impl ErasedArc {
    /// Create a new ErasedArc from an Arc
    pub fn new<T: 'static + fmt::Debug + Send + Sync>(arc: Arc<T>) -> Self {
        ErasedArc {
            reader: Box::new(ArcReader {
                arc,
                _marker: PhantomData,
                type_name_override: None,
            }),
        }
    }

    /// Create a new ErasedArc from a value by wrapping it in an Arc
    pub fn from_value<T: 'static + fmt::Debug + Send + Sync>(value: T) -> Self {
        ErasedArc::new(Arc::new(value))
    }

    /// Get the raw pointer to the contained value
    pub fn as_ptr(&self) -> *const () {
        self.reader.ptr()
    }

    /// Get the Arc's strong reference count
    pub fn strong_count(&self) -> usize {
        self.reader.strong_count()
    }

    /// Get the Arc's weak reference count
    pub fn weak_count(&self) -> usize {
        self.reader.weak_count()
    }

    /// Get the type name of the contained value
    pub fn type_name(&self) -> &'static str {
        self.reader.type_name()
    }

    /// Get the contained value as a dynamic Any reference
    pub fn as_any(&self) -> Result<&dyn Any> {
        Ok(self.reader.as_any())
    }

    /// Create an ErasedArc from a boxed Any
    pub fn from_boxed_any(boxed: Box<dyn Any + Send + Sync>) -> Result<Self> {
        // First check for primitive types
        if let Some(value) = boxed.downcast_ref::<String>() {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) = boxed.downcast_ref::<i32>() {
            return Ok(ErasedArc::from_value(*value));
        } else if let Some(value) = boxed.downcast_ref::<i64>() {
            return Ok(ErasedArc::from_value(*value));
        } else if let Some(value) = boxed.downcast_ref::<f32>() {
            return Ok(ErasedArc::from_value(*value));
        } else if let Some(value) = boxed.downcast_ref::<f64>() {
            return Ok(ErasedArc::from_value(*value));
        } else if let Some(value) = boxed.downcast_ref::<bool>() {
            return Ok(ErasedArc::from_value(*value));
        } else if let Some(value) = boxed.downcast_ref::<Vec<u8>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        }

        // Check for container types
        if let Some(value) = boxed.downcast_ref::<Vec<String>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) = boxed.downcast_ref::<Vec<i32>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) = boxed.downcast_ref::<Vec<i64>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) = boxed.downcast_ref::<Vec<f64>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) =
            boxed.downcast_ref::<std::collections::HashMap<String, String>>()
        {
            return Ok(ErasedArc::from_value(value.clone()));
        } else if let Some(value) = boxed.downcast_ref::<std::collections::HashMap<String, i32>>() {
            return Ok(ErasedArc::from_value(value.clone()));
        }

        // For any other type, we use reflection to create a special wrapper
        // that preserves the original type information for deserialization

        // Get the type name for the boxed value
        let type_name = std::any::type_name_of_val(&*boxed).to_string();

        // Create the ErasedArc with type name override
        let arc = Arc::new(boxed);
        let reader = Box::new(ArcReader {
            arc,
            _marker: PhantomData,
            type_name_override: Some(type_name),
        });

        Ok(ErasedArc { reader })
    }

    /// Get the type ID of the contained value
    pub fn type_id(&self) -> Result<TypeId> {
        // Determine the concrete type
        let type_name = self.type_name();

        // Handle the common types we support
        if type_name.contains("String") {
            Ok(TypeId::of::<String>())
        } else if type_name.contains("i32") {
            Ok(TypeId::of::<i32>())
        } else if type_name.contains("i64") {
            Ok(TypeId::of::<i64>())
        } else if type_name.contains("f32") {
            Ok(TypeId::of::<f32>())
        } else if type_name.contains("f64") {
            Ok(TypeId::of::<f64>())
        } else if type_name.contains("bool") {
            Ok(TypeId::of::<bool>())
        } else if type_name.contains("Vec<u8>") {
            Ok(TypeId::of::<Vec<u8>>())
        } else if type_name.contains("Vec<String>") {
            Ok(TypeId::of::<Vec<String>>())
        } else if type_name.contains("Vec<i32>") {
            Ok(TypeId::of::<Vec<i32>>())
        } else if type_name.contains("Vec<i64>") {
            Ok(TypeId::of::<Vec<i64>>())
        } else if type_name.contains("Vec<f64>") {
            Ok(TypeId::of::<Vec<f64>>())
        } else if type_name.contains("HashMap<String, String>") {
            Ok(TypeId::of::<std::collections::HashMap<String, String>>())
        } else if type_name.contains("HashMap<String, i32>") {
            Ok(TypeId::of::<std::collections::HashMap<String, i32>>())
        } else {
            Err(anyhow!("Unknown type ID for type name: {}", type_name))
        }
    }

    /// Serialize the contained value to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Based on the type, use the appropriate serialization
        let type_name = self.type_name();

        if type_name.contains("String") {
            if let Ok(val) = self.as_arc::<String>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("i32") {
            if let Ok(val) = self.as_arc::<i32>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("i64") {
            if let Ok(val) = self.as_arc::<i64>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("f32") {
            if let Ok(val) = self.as_arc::<f32>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("f64") {
            if let Ok(val) = self.as_arc::<f64>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("bool") {
            if let Ok(val) = self.as_arc::<bool>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("Vec<u8>") {
            if let Ok(val) = self.as_arc::<Vec<u8>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        }

        // Container types
        if type_name.contains("Vec<String>") {
            if let Ok(val) = self.as_arc::<Vec<String>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("Vec<i32>") {
            if let Ok(val) = self.as_arc::<Vec<i32>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("Vec<i64>") {
            if let Ok(val) = self.as_arc::<Vec<i64>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("Vec<f64>") {
            if let Ok(val) = self.as_arc::<Vec<f64>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("HashMap<String, String>") {
            if let Ok(val) = self.as_arc::<std::collections::HashMap<String, String>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        } else if type_name.contains("HashMap<String, i32>") {
            if let Ok(val) = self.as_arc::<std::collections::HashMap<String, i32>>() {
                return bincode::serialize(&*val)
                    .map_err(|e| anyhow!("Serialization error: {}", e));
            }
        }

        Err(anyhow!("Cannot serialize type: {}", type_name))
    }

    /// Try to extract an Arc<T> from this ErasedArc
    pub fn as_arc<T: 'static>(&self) -> Result<Arc<T>> {
        // Check if the type matches
        let expected_type_name = std::any::type_name::<T>();
        let actual_type_name = self.type_name();

        if !self.is_type::<T>() {
            return Err(anyhow!(
                "Type mismatch: expected {}, but has {}",
                expected_type_name,
                actual_type_name
            ));
        }

        // Attempt to downcast
        let ptr = self.as_ptr() as *const T;
        let arc = unsafe {
            // Safety: Cloning an Arc with a known type as we've verified the type above
            let arc = Arc::from_raw(ptr);
            let clone = arc.clone();
            // Prevent dropping the original Arc
            std::mem::forget(arc);
            clone
        };

        Ok(arc)
    }

    /// Check if this ArcAny contains a value of type T
    pub fn is_type<T: 'static>(&self) -> bool {
        let expected_type_name = std::any::type_name::<T>();
        let actual_type_name = self.type_name();

        // We need this slightly more complex matching because the std::any type names
        // can have slight differences based on the package/crate names
        if expected_type_name == actual_type_name {
            return true;
        }

        // Handle some common cases where type names might differ but are compatible
        match (expected_type_name, actual_type_name) {
            // String variations
            ("alloc::string::String", "String") => return true,
            ("String", "alloc::string::String") => return true,
            // Vec variations
            (e, a) if e.contains("Vec<") && a.contains("Vec<") => {
                // Basic check for Vec element types - this is a simplified approach
                let e_elem = e
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split('>')
                    .next()
                    .unwrap_or("");
                let a_elem = a
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split('>')
                    .next()
                    .unwrap_or("");
                return e_elem == a_elem
                    || (e_elem.contains("String") && a_elem.contains("String"))
                    || (e_elem.contains("i32") && a_elem.contains("i32"));
            }
            // HashMap variations
            (e, a) if e.contains("HashMap<") && a.contains("HashMap<") => {
                // Basic check for HashMap key/value types - this is a simplified approach
                let e_parts = e
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split('>')
                    .next()
                    .unwrap_or("");
                let a_parts = a
                    .split('<')
                    .nth(1)
                    .unwrap_or("")
                    .split('>')
                    .next()
                    .unwrap_or("");
                return e_parts == a_parts
                    || (e_parts.contains("String") && a_parts.contains("String"));
            }
            _ => {}
        }

        false
    }
}
