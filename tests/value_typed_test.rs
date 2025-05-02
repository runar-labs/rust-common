// runar_common/tests/value_typed_test.rs
//
// Tests for the type-preserving ValueType system

use std::collections::HashMap;
use std::any::Any;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use runar_common::types::{Value, MapValue, ValueBase, ValueConvert, CustomStruct};

#[test]
fn test_primitives() -> Result<()> {
    // Create basic primitive values with unified constructor
    let s = Value::<String>::new("Hello".to_string());
    let i = Value::<i32>::new(42);
    let f = Value::<f64>::new(3.14159);
    let b = Value::<bool>::new(true);
    
    // Create null value
    let null = Value::<()>::null();
    
    // Verify values are stored correctly
    if let Value::Value(val) = &s {
        assert_eq!(val, "Hello");
    } else {
        panic!("Expected Value::Value variant for string");
    }
    
    if let Value::Value(val) = &i {
        assert_eq!(*val, 42);
    } else {
        panic!("Expected Value::Value variant for integer");
    }
    
    if let Value::Value(val) = &f {
        assert_eq!(*val, 3.14159);
    } else {
        panic!("Expected Value::Value variant for float");
    }
    
    if let Value::Value(val) = &b {
        assert_eq!(*val, true);
    } else {
        panic!("Expected Value::Value variant for boolean");
    }
    
    if let Value::Null = &null {
        // This is expected
    } else {
        panic!("Expected Value::Null variant");
    }
    
    // Test type conversion - expected to fail
    let i32_result: Result<i32> = s.as_type();
    assert!(i32_result.is_err());
    
    // Test type conversion - expected to succeed
    let i32_value: i32 = i.as_type()?;
    assert_eq!(i32_value, 42);
    
    Ok(())
}

#[test]
fn test_lists() -> Result<()> {
    // Create lists with the list constructor
    let str_list = Value::<String>::new_list(vec!["one".to_string(), "two".to_string(), "three".to_string()]);
    let int_list = Value::<i32>::new_list(vec![1, 2, 3, 4, 5]);
    
    // Verify lists are stored correctly
    if let Value::List(values) = &str_list {
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], "one");
        assert_eq!(values[1], "two");
        assert_eq!(values[2], "three");
    } else {
        panic!("Expected Value::List variant for string list");
    }
    
    if let Value::List(values) = &int_list {
        assert_eq!(values.len(), 5);
        assert_eq!(values[0], 1);
        assert_eq!(values[4], 5);
    } else {
        panic!("Expected Value::List variant for integer list");
    }
    
    // Using the list conversion API
    let str_vec: Vec<String> = str_list.as_list()?;
    assert_eq!(str_vec, vec!["one".to_string(), "two".to_string(), "three".to_string()]);
    
    let int_vec: Vec<i32> = int_list.as_list()?;
    assert_eq!(int_vec, vec![1, 2, 3, 4, 5]);
    
    // Try a conversion that should fail
    let result: Result<Vec<f64>> = int_list.as_list();
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_maps() -> Result<()> {
    // Create string to int map
    let mut str_to_int = HashMap::new();
    str_to_int.insert("one".to_string(), 1);
    str_to_int.insert("two".to_string(), 2);
    str_to_int.insert("three".to_string(), 3);
    
    // Create int to string map
    let mut int_to_str = HashMap::new();
    int_to_str.insert(1, "one".to_string());
    int_to_str.insert(2, "two".to_string());
    int_to_str.insert(3, "three".to_string());
    
    // Create map values
    let str_to_int_map = MapValue::<String, i32>::new(str_to_int.clone());
    let int_to_str_map = MapValue::<i32, String>::new(int_to_str.clone());
    
    // Verify maps are stored correctly
    assert_eq!(str_to_int_map.entries.len(), 3);
    assert_eq!(str_to_int_map.entries.get("one"), Some(&1));
    assert_eq!(str_to_int_map.entries.get("three"), Some(&3));
    
    assert_eq!(int_to_str_map.entries.len(), 3);
    assert_eq!(int_to_str_map.entries.get(&1), Some(&"one".to_string()));
    assert_eq!(int_to_str_map.entries.get(&3), Some(&"three".to_string()));
    
    // Using the map conversion API
    let map1: HashMap<String, i32> = str_to_int_map.as_map()?;
    assert_eq!(map1, str_to_int);
    
    let map2: HashMap<i32, String> = int_to_str_map.as_map()?;
    assert_eq!(map2, int_to_str);
    
    // Try a conversion that should fail
    let result: Result<HashMap<i32, i32>> = str_to_int_map.as_map();
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_serialization() -> Result<()> {
    // Create a value to serialize
    let original = Value::<i32>::new(42);
    
    // Serialize to bytes
    let bytes = original.to_bytes()?;
    assert!(!bytes.is_empty());
    
    // Create a value from the serialized bytes
    let deserialized = runar_common::types::value_from_bytes(&bytes)?;
    
    // Type-safe access to the deserialized value requires casting the trait object to its concrete type
    if let Some(value_typed) = deserialized.as_any().downcast_ref::<Value<()>>() {
        if let Value::Bytes(typed_bytes) = value_typed {
            let val: i32 = typed_bytes.deserialize()?;
            assert_eq!(val, 42);
        } else {
            panic!("Expected Value::Bytes variant");
        }
    } else {
        panic!("Failed to downcast to Value<()>");
    }
    
    Ok(())
}

#[test]
fn test_zero_copy() -> Result<()> {
    // Create complex object
    let mut map = HashMap::new();
    map.insert("name".to_string(), "John Doe".to_string());
    map.insert("email".to_string(), "john@example.com".to_string());
    map.insert("role".to_string(), "Admin".to_string());
    
    let user_map = MapValue::<String, String>::new(map);
    
    // Direct access without serialization/deserialization overhead
    assert_eq!(user_map.entries.get("name"), Some(&"John Doe".to_string()));
    assert_eq!(user_map.entries.get("email"), Some(&"john@example.com".to_string()));
    assert_eq!(user_map.entries.get("role"), Some(&"Admin".to_string()));
    
    Ok(())
}

// Define a custom struct for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: i32,
    email: String,
}

// The CustomStruct trait is automatically implemented for Person
// because it derives Debug, Clone, Serialize, and implements Send + Sync

#[test]
fn test_custom_structs() -> Result<()> {
    // Create a Person instance
    let person = Person {
        name: "Alice Smith".to_string(),
        age: 32,
        email: "alice@example.com".to_string(),
    };
    
    // Create a Value that contains our custom struct
    // No need to explicitly implement CustomStruct - it's automatically implemented
    let person_value = Value::<()>::Struct(Box::new(person.clone()));
    
    // Verify type information
    if let Value::Struct(s) = &person_value {
        assert!(s.type_name().contains("Person"));
    } else {
        panic!("Expected Value::Struct variant");
    }
    
    // Serialize the struct
    let bytes = person_value.to_bytes()?;
    assert!(!bytes.is_empty());
    
    // Deserialize
    let value = runar_common::types::value_from_bytes(&bytes)?;
    
    // Access the struct
    if let Value::Struct(s) = value.as_any().downcast_ref::<Value<()>>().unwrap() {
        if let Some(p) = s.as_any().downcast_ref::<Person>() {
            assert_eq!(p.name, "Alice Smith");
            assert_eq!(p.age, 32);
            assert_eq!(p.email, "alice@example.com");
        } else {
            panic!("Failed to downcast to Person");
        }
    } else {
        panic!("Not a struct value");
    }
    
    Ok(())
} 