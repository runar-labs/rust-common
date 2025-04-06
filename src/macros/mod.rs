// runar_common/src/macros/mod.rs
//
// Common macros that don't require procedural macro functionality

// This module will contain macros that are used commonly but don't require
// the complexity of procedural macros. These include simple utility macros,
// helper macros, and formatting macros.

// Note: Most complex macros should go in the rust-macros crate instead.

// Import additional macro modules
mod vmap_macros;
mod deprecated;

// Re-export macros from other modules
// These macros are already #[macro_export] marked, which means they
// are automatically available at the crate root namespace
// We don't need to re-export them specifically

/// Create a ValueType::Map with key-value pairs
///
/// This macro allows you to create a ValueType::Map with key-value pairs.
/// The keys are converted to strings, and the values are converted to ValueType.
///
/// ## Map Creation Usage:
///
/// ```
/// use runar_common::vmap;
/// use runar_common::types::ValueType;
/// // Create a new ValueType::Map:
/// let params = vmap!{"name" => "John", "age" => 30, "active" => true};
/// ```
///
/// ## Empty Map:
///
/// ```
/// use runar_common::vmap;
/// use runar_common::types::ValueType;
/// // Create an empty map
/// let empty = vmap!{};
/// ```
// vmap! is defined in vmap_macros.rs

// Define and export the vjson macro (JSON to ValueType)
#[macro_export]
macro_rules! vjson {
    ($($json:tt)+) => {
        $crate::types::ValueType::from(serde_json::json!($($json)+))
    };
}