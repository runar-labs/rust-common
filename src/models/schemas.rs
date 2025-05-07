use crate::types::ArcValueType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines the type of a schema field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaDataType {
    String,
    Integer, // Represents whole numbers (e.g., i32, i64)
    Number,  // Represents floating-point numbers (e.g., f32, f64)
    Boolean,
    Object,
    Array,
    Null,
}

/// Represents the schema for a single field or a complex data structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    /// The data type of the field. For complex types like Object or Array,
    /// this defines the main type, and 'properties' or 'items' define the substructure.
    pub data_type: SchemaDataType,

    pub description: Option<String>,
    pub nullable: Option<bool>,

    /// String representation of the default value.
    /// Consumers must parse this string based on 'data_type'.
    /// For complex types (Object, Array), this should be a JSON string representation.
    pub default_value: Option<String>,

    /// For `SchemaDataType::Object`: Defines the schema for each property of the object.
    /// Keys are property names.
    pub properties: Option<HashMap<String, Box<FieldSchema>>>, // Boxed to handle recursive type
    pub required: Option<Vec<String>>,

    /// For `SchemaDataType::Array`: Defines the schema for items in the array.
    /// All items in the array must conform to this schema.
    pub items: Option<Box<FieldSchema>>, // Boxed to handle recursive type

    pub pattern: Option<String>,

    /// String representations of allowed enumeration values.
    /// Consumers must parse these strings based on 'data_type'.
    pub enum_values: Option<Vec<String>>,

    // Numeric constraints
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub exclusive_minimum: Option<bool>, // If true, minimum is exclusive
    pub exclusive_maximum: Option<bool>, // If true, maximum is exclusive

    // String length constraints
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,

    // Array length constraints
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,

    /// String representation of an example value.
    /// Consumers must parse this string based on 'data_type'.
    /// For complex types (Object, Array), this should be a JSON string representation.
    pub example: Option<String>,
}

impl FieldSchema {
    // Helper constructors for common types
    pub fn string() -> Self {
        FieldSchema::new(SchemaDataType::String)
    }
    pub fn integer() -> Self {
        FieldSchema::new(SchemaDataType::Integer)
    }
    pub fn number() -> Self {
        FieldSchema::new(SchemaDataType::Number)
    }
    pub fn boolean() -> Self {
        FieldSchema::new(SchemaDataType::Boolean)
    }
    pub fn object(
        properties: HashMap<String, Box<FieldSchema>>,
        required: Option<Vec<String>>,
    ) -> Self {
        FieldSchema {
            data_type: SchemaDataType::Object,
            properties: Some(properties),
            required,
            ..FieldSchema::new(SchemaDataType::Object)
        }
    }
    pub fn array(items: Box<FieldSchema>) -> Self {
        FieldSchema {
            data_type: SchemaDataType::Array,
            items: Some(items),
            ..FieldSchema::new(SchemaDataType::Array)
        }
    }
    pub fn new(data_type: SchemaDataType) -> Self {
        FieldSchema {
            data_type,
            description: None,
            nullable: None,
            default_value: None,
            properties: None,
            required: None,
            items: None,
            pattern: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: None,
            exclusive_maximum: None,
            min_length: None,
            max_length: None,
            min_items: None,
            max_items: None,
            example: None,
        }
    }

    /// Convert a FieldSchema to an ArcValueType representation
    /// This is useful when you need to represent the schema structure as a value
    /// for transporting or displaying
    pub fn to_arc_value_type(&self) -> ArcValueType {
        let mut map = HashMap::new();

        // Add data type
        map.insert(
            "data_type".to_string(),
            ArcValueType::new_primitive(format!("{:?}", self.data_type)),
        );

        // Add description if present
        if let Some(desc) = &self.description {
            map.insert(
                "description".to_string(),
                ArcValueType::new_primitive(desc.clone()),
            );
        }

        // Add nullable flag if present
        if let Some(nullable) = &self.nullable {
            map.insert(
                "nullable".to_string(),
                ArcValueType::new_primitive(*nullable),
            );
        }

        // Add default value if present
        if let Some(default) = &self.default_value {
            map.insert(
                "default".to_string(),
                ArcValueType::new_primitive(default.clone()),
            );
        }

        // Add properties for object types
        if let Some(properties) = &self.properties {
            let mut prop_map = HashMap::new();
            for (key, value) in properties {
                prop_map.insert(key.clone(), value.to_arc_value_type());
            }
            map.insert("properties".to_string(), ArcValueType::from_map(prop_map));
        }

        // Add required fields if present
        if let Some(required) = &self.required {
            let mut required_vec = Vec::new();
            for s in required {
                required_vec.push(ArcValueType::new_primitive(s.clone()));
            }
            map.insert("required".to_string(), ArcValueType::new_list(required_vec));
        }

        // Add items for array types
        if let Some(items) = &self.items {
            map.insert("items".to_string(), items.to_arc_value_type());
        }

        // Add pattern if present
        if let Some(pattern) = &self.pattern {
            map.insert(
                "pattern".to_string(),
                ArcValueType::new_primitive(pattern.clone()),
            );
        }

        // Add enum values if present
        if let Some(enum_values) = &self.enum_values {
            let mut enum_vec = Vec::new();
            for s in enum_values {
                enum_vec.push(ArcValueType::new_primitive(s.clone()));
            }
            map.insert("enum".to_string(), ArcValueType::new_list(enum_vec));
        }

        // Add numeric constraints if present
        if let Some(min) = &self.minimum {
            map.insert("minimum".to_string(), ArcValueType::new_primitive(*min));
        }
        if let Some(max) = &self.maximum {
            map.insert("maximum".to_string(), ArcValueType::new_primitive(*max));
        }
        if let Some(exclusive_min) = &self.exclusive_minimum {
            map.insert(
                "exclusiveMinimum".to_string(),
                ArcValueType::new_primitive(*exclusive_min),
            );
        }
        if let Some(exclusive_max) = &self.exclusive_maximum {
            map.insert(
                "exclusiveMaximum".to_string(),
                ArcValueType::new_primitive(*exclusive_max),
            );
        }

        // Add string length constraints if present
        if let Some(min_length) = &self.min_length {
            map.insert(
                "minLength".to_string(),
                ArcValueType::new_primitive(*min_length as i64),
            );
        }
        if let Some(max_length) = &self.max_length {
            map.insert(
                "maxLength".to_string(),
                ArcValueType::new_primitive(*max_length as i64),
            );
        }

        // Add array length constraints if present
        if let Some(min_items) = &self.min_items {
            map.insert(
                "minItems".to_string(),
                ArcValueType::new_primitive(*min_items as i64),
            );
        }
        if let Some(max_items) = &self.max_items {
            map.insert(
                "maxItems".to_string(),
                ArcValueType::new_primitive(*max_items as i64),
            );
        }

        // Add example if present
        if let Some(example) = &self.example {
            map.insert(
                "example".to_string(),
                ArcValueType::new_primitive(example.clone()),
            );
        }

        ArcValueType::from_map(map)
    }
}

/// Represents metadata for an action.
/// This is a unified struct that replaces both ActionMetadata and ActionCapability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub path: String, // Full action path, e.g., "service_path/action_name"
    pub description: String,
    pub parameters_schema: Option<FieldSchema>,
    pub return_schema: Option<FieldSchema>,
}

/// Represents metadata for an event.
/// This is a unified struct that replaces both EventMetadata and EventCapability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub path: String, // Full event topic path, e.g., "service_path/event_name"
    pub description: String,
    pub data_schema: Option<FieldSchema>,
}

/// Represents metadata for a service.
/// This is a unified struct that replaces ServiceCapability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub network_id: String,
    pub service_path: String, // Path of the service itself
    pub name: String,         // Human-readable name of the service
    pub version: String,
    pub description: String,
    pub actions: Vec<ActionMetadata>,
    pub events: Vec<EventMetadata>,
}
