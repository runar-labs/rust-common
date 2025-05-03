// runar_common/tests/value_typed_test.rs
//
// Tests for the type-preserving ValueType system

use anyhow::Result;
use runar_common::types::value_from_bytes;
use runar_common::types::TypedValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Import implementation details only where needed for advanced testing
use runar_common::types::internal::{MapValue, Value, ValueBase};

#[test]
fn test_primitives() -> Result<()> {
    // Create basic primitive values with the TypedValue API
    let s = TypedValue::from_value("Hello".to_string());
    let i = TypedValue::from_value(42);
    let f = TypedValue::from_value(3.14159);
    let b = TypedValue::from_value(true);

    // Create null value
    let null = TypedValue::null();

    // Test type conversion
    let str_value: String = s.as_type()?;
    assert_eq!(str_value, "Hello");

    let i32_value: i32 = i.as_type()?;
    assert_eq!(i32_value, 42);

    let f64_value: f64 = f.as_type()?;
    assert_eq!(f64_value, 3.14159);

    let bool_value: bool = b.as_type()?;
    assert_eq!(bool_value, true);

    // Test if null is correctly identified
    assert!(null.is_null());

    // Test type conversion - demonstrate that our as_type is strongly typed
    // by trying to extract a completely incompatible, custom type
    #[derive(Debug, Clone, Deserialize)]
    struct ComplexType {
        field1: Vec<HashMap<String, i32>>,
        field2: [u8; 16],
    }

    let complex_result: Result<ComplexType> = s.as_type();
    assert!(complex_result.is_err());

    Ok(())
}

#[test]
fn test_lists() -> Result<()> {
    // Create lists with the TypedValue API
    let str_list = TypedValue::from_list(vec![
        "one".to_string(),
        "two".to_string(),
        "three".to_string(),
    ]);
    let int_list = TypedValue::from_list(vec![1, 2, 3, 4, 5]);

    // Using the list conversion API
    let str_vec: Vec<String> = str_list.as_list()?;
    assert_eq!(
        str_vec,
        vec!["one".to_string(), "two".to_string(), "three".to_string()]
    );

    let int_vec: Vec<i32> = int_list.as_list()?;
    assert_eq!(int_vec, vec![1, 2, 3, 4, 5]);

    // Try a conversion that should fail
    let result: Result<Vec<f64>> = int_list.as_list();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_maps() -> Result<()> {
    // Create maps
    let mut str_to_int = HashMap::new();
    str_to_int.insert("one".to_string(), 1);
    str_to_int.insert("two".to_string(), 2);
    str_to_int.insert("three".to_string(), 3);

    let mut int_to_str = HashMap::new();
    int_to_str.insert(1, "one".to_string());
    int_to_str.insert(2, "two".to_string());
    int_to_str.insert(3, "three".to_string());

    // Create map values with TypedValue API
    let typed_str_to_int = TypedValue::from_map(str_to_int.clone());
    let typed_int_to_str = TypedValue::from_map(int_to_str.clone());

    // Using the map conversion API
    let map1: HashMap<String, i32> = typed_str_to_int.as_map()?;
    assert_eq!(map1, str_to_int);

    let map2: HashMap<i32, String> = typed_int_to_str.as_map()?;
    assert_eq!(map2, int_to_str);

    // Try a conversion that should fail
    let result: Result<HashMap<bool, Vec<u8>>> = typed_str_to_int.as_map();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_serialization() -> Result<()> {
    // Create a value to serialize
    let original = TypedValue::from_value(42);

    // Serialize to bytes
    let bytes = original.to_bytes()?;
    assert!(!bytes.is_empty());

    // Create a value from the serialized bytes
    let deserialized = runar_common::types::value_from_bytes(&bytes)?;

    // Type-safe access to the deserialized value
    let val: i32 = deserialized.as_type()?;
    assert_eq!(val, 42);

    Ok(())
}

#[test]
fn test_zero_copy() -> Result<()> {
    // Create complex object
    let mut map = HashMap::new();
    map.insert("name".to_string(), "John Doe".to_string());
    map.insert("email".to_string(), "john@example.com".to_string());
    map.insert("role".to_string(), "Admin".to_string());

    // Create with TypedValue API
    let typed_user_map = TypedValue::from_map(map.clone());

    // Use the API to get the map
    let map_data: HashMap<String, String> = typed_user_map.as_map()?;
    assert_eq!(map_data.get("name"), Some(&"John Doe".to_string()));
    assert_eq!(map_data.get("email"), Some(&"john@example.com".to_string()));
    assert_eq!(map_data.get("role"), Some(&"Admin".to_string()));

    Ok(())
}

// Define a custom struct for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: i32,
    email: String,
}

#[test]
fn test_custom_structs() -> Result<()> {
    // Create a Person instance
    let person = Person {
        name: "Alice Smith".to_string(),
        age: 32,
        email: "alice@example.com".to_string(),
    };

    // Create a TypedValue containing our custom struct
    let typed_person = TypedValue::from_struct(person.clone());

    // Serialize the struct
    let bytes = typed_person.to_bytes()?;
    assert!(!bytes.is_empty());

    // Deserialize
    let typed_value = runar_common::types::value_from_bytes(&bytes)?;

    // Access the struct using the simplified API
    let p: Person = typed_value.as_type()?;
    assert_eq!(p.name, "Alice Smith");
    assert_eq!(p.age, 32);
    assert_eq!(p.email, "alice@example.com");

    Ok(())
}

#[test]
fn test_improved_as_type_api() -> Result<()> {
    // Test direct value conversion
    let typed_value = TypedValue::from_value(42);
    let i: i32 = typed_value.as_type()?;
    assert_eq!(i, 42);

    // Test struct conversion
    let person = Person {
        name: "Bob Johnson".to_string(),
        age: 45,
        email: "bob@example.com".to_string(),
    };
    let typed_value = TypedValue::from_struct(person.clone());
    let p: Person = typed_value.as_type()?;
    assert_eq!(p.name, "Bob Johnson");

    // Test bytes conversion
    let typed_value = TypedValue::from_value(3.14159);
    let bytes = typed_value.to_bytes()?;
    let value_from_bytes = runar_common::types::value_from_bytes(&bytes)?;
    let f: f64 = value_from_bytes.as_type()?;
    assert_eq!(f, 3.14159);

    // Test list conversion
    let typed_list = TypedValue::from_list(vec!["one".to_string(), "two".to_string()]);
    let bytes = typed_list.to_bytes()?;
    let list_from_bytes = runar_common::types::value_from_bytes(&bytes)?;
    let str_vec: Vec<String> = list_from_bytes.as_list()?;
    assert_eq!(str_vec, vec!["one".to_string(), "two".to_string()]);

    Ok(())
}

#[test]
fn test_chained_operations() -> Result<()> {
    // Create a complex nested structure and test various ways to extract data

    // Create a person with map of attributes
    let person = Person {
        name: "Charlie Brown".to_string(),
        age: 25,
        email: "charlie@example.com".to_string(),
    };

    // check in memory access (no serialization)
    let wrapped = TypedValue::from_struct(person.clone());
    let p: Person = wrapped.as_type()?;
    assert_eq!(p.name, "Charlie Brown");
    assert_eq!(p.age, 25);
    assert_eq!(p.email, "charlie@example.com");

    //testing serialization
    let bytes = wrapped.to_bytes()?;
    let typed_value = runar_common::types::value_from_bytes(&bytes)?;

    // Extract using as_type
    let p: Person = typed_value.as_type()?;
    assert_eq!(p.name, "Charlie Brown");

    // Create a more complex structure - map of person data
    let mut person_map = HashMap::new();
    person_map.insert("name".to_string(), "Alice".to_string());
    person_map.insert("email".to_string(), "alice@example.com".to_string());

    // Create a MapValue
    let typed_map = TypedValue::from_map(person_map.clone());

    // Serialize and deserialize
    let bytes = typed_map.to_bytes()?;
    let typed_map = runar_common::types::value_from_bytes(&bytes)?;

    // Extract
    let extracted_map: HashMap<String, String> = typed_map.as_map()?;
    assert_eq!(extracted_map.get("name"), Some(&"Alice".to_string()));

    // Test multiple layers of serialization/deserialization
    let re_serialized = typed_map.to_bytes()?;
    let re_typed_map = runar_common::types::value_from_bytes(&re_serialized)?;
    let final_map: HashMap<String, String> = re_typed_map.as_map()?;
    assert_eq!(
        final_map.get("email"),
        Some(&"alice@example.com".to_string())
    );

    Ok(())
}
