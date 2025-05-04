use std::any::{Any, TypeId};
use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Result};

/// A type-erased Arc container that preserves pointer identity
///
/// This structure stores a raw Arc pointer with type information
/// and specialized clone/drop functions to ensure proper memory management.
/// It allows type-safe access to the original Arc without cloning the inner data.
pub struct ErasedArc {
    /// The TypeId of the contained type for runtime type checks
    type_id: TypeId,
    /// The raw Arc pointer (type-erased)
    arc_ptr: *const (),
    /// Type-specific clone function that knows how to clone this specific Arc
    clone_fn: fn(*const ()) -> *const (),
    /// Type-specific drop function that knows how to drop this specific Arc
    drop_fn: fn(*const ()),
    /// Type name for debugging
    type_name: &'static str,
}

// Safety: ErasedArc handles its own synchronization via Arc
unsafe impl Send for ErasedArc {}
unsafe impl Sync for ErasedArc {}

impl ErasedArc {
    /// Create a new ErasedArc from any type
    pub fn new<T: 'static + Send + Sync>(value: T) -> Self {
        // Store value in Arc
        let arc = Arc::new(value);

        // Convert to raw pointer (type erased)
        let arc_ptr = Arc::into_raw(arc) as *const ();

        // Create clone function for this specific type
        let clone_fn = |ptr: *const ()| {
            // Safety: We know the pointer type from construction
            let typed_ptr = ptr as *const T;
            let arc = unsafe { Arc::from_raw(typed_ptr) };

            // Clone the arc and convert back to raw
            let cloned = Arc::clone(&arc);

            // Forget original to avoid dropping
            std::mem::forget(arc);

            // Return raw pointer of clone
            Arc::into_raw(cloned) as *const ()
        };

        // Create drop function for this specific type
        let drop_fn = |ptr: *const ()| {
            // Safety: We know the pointer type from construction
            let typed_ptr = ptr as *const T;

            // Convert back to Arc which will be dropped
            unsafe { Arc::from_raw(typed_ptr) };
        };

        Self {
            type_id: TypeId::of::<T>(),
            arc_ptr,
            clone_fn,
            drop_fn,
            type_name: std::any::type_name::<T>(),
        }
    }

    /// Get the Arc reference with proper type
    pub fn as_arc<T: 'static>(&self) -> Result<Arc<T>> {
        if self.type_id == TypeId::of::<T>() {
            // Safety: We verified the type matches
            let arc_ptr = self.arc_ptr as *const T;
            let arc = unsafe { Arc::from_raw(arc_ptr) };

            // Clone the Arc
            let result = Arc::clone(&arc);

            // Forget original to avoid dropping
            std::mem::forget(arc);

            Ok(result)
        } else {
            Err(anyhow!(
                "Type mismatch: expected {:?}, found {:?}",
                std::any::type_name::<T>(),
                self.type_name
            ))
        }
    }

    /// Get the TypeId of the contained value
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Get the type name of the contained value
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Attempt to downcast to a specific type and get a reference to the inner value
    /// This avoids an Arc clone when you just need to check a value
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        if self.type_id == TypeId::of::<T>() {
            // Safety: We verified the type matches
            let arc_ptr = self.arc_ptr as *const T;
            let value_ref = unsafe { &*arc_ptr };
            Some(value_ref)
        } else {
            None
        }
    }
}

/// Proper cleanup on drop
impl Drop for ErasedArc {
    fn drop(&mut self) {
        // Call the type-specific drop function
        (self.drop_fn)(self.arc_ptr);
    }
}

/// Allow cloning the ErasedArc
impl Clone for ErasedArc {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            // Call type-specific clone function
            arc_ptr: (self.clone_fn)(self.arc_ptr),
            clone_fn: self.clone_fn,
            drop_fn: self.drop_fn,
            type_name: self.type_name,
        }
    }
}

impl fmt::Debug for ErasedArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedArc")
            .field("type", &self.type_name)
            .field("ptr", &self.arc_ptr)
            .finish()
    }
}
