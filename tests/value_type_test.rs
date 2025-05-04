use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use runar_common::types::ArcValueType;

#[test]
fn test_primitives_arc_preservation() -> Result<()> {
    // Create a value with a string
    let string_value = "Hello, world!".to_string();
    let value = ArcValueType::from_value(string_value);

    // Get reference to the string
    let ref1 = value.as_type_ref::<String>()?;
    let ref2 = value.as_type_ref::<String>()?;

    // Verify identity (same Arc pointer)
    assert!(Arc::ptr_eq(&ref1, &ref2));

    // Verify content
    assert_eq!(&*ref1, "Hello, world!");
    assert_eq!(&*ref2, "Hello, world!");

    Ok(())
}

#[test]
fn test_list_arc_preservation() -> Result<()> {
    // Create a value with a list
    let list = vec![1, 2, 3, 4, 5];
    let value = ArcValueType::from_list(list);

    // Get references
    let ref1 = value.as_list_ref::<i32>()?;
    let ref2 = value.as_list_ref::<i32>()?;

    // Verify identity
    assert!(Arc::ptr_eq(&ref1, &ref2));

    // Verify content
    assert_eq!(*ref1, vec![1, 2, 3, 4, 5]);
    assert_eq!(*ref2, vec![1, 2, 3, 4, 5]);

    Ok(())
}

#[test]
fn test_map_arc_preservation() -> Result<()> {
    // Create a map
    let mut map = HashMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());

    let value = ArcValueType::from_map(map);

    // Get references
    let ref1 = value.as_map_ref::<String, String>()?;
    let ref2 = value.as_map_ref::<String, String>()?;

    // Verify identity
    assert!(Arc::ptr_eq(&ref1, &ref2));

    // Verify content
    assert_eq!(ref1.len(), 2);
    assert_eq!(ref1.get("key1"), Some(&"value1".to_string()));
    assert_eq!(ref1.get("key2"), Some(&"value2".to_string()));

    assert_eq!(ref2.len(), 2);
    assert_eq!(ref2.get("key1"), Some(&"value1".to_string()));
    assert_eq!(ref2.get("key2"), Some(&"value2".to_string()));

    //lets check serialization
    let bytes = value.to_bytes()?;
    let value_from_bytes = ArcValueType::from_bytes(&bytes)?;
    let ref3 = value_from_bytes.as_map_ref::<String, String>()?;
    assert_eq!(ref3.len(), 2);
    assert_eq!(ref3.get("key1"), Some(&"value1".to_string()));
    assert_eq!(ref3.get("key2"), Some(&"value2".to_string()));

    Ok(())
}

#[test]
fn test_struct_arc_preservation() -> Result<()> {
    // Create a struct
    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    let test_struct = TestStruct {
        field1: "Hello".to_string(),
        field2: 42,
    };

    let value = ArcValueType::from_struct(test_struct.clone());

    // Get references
    let ref1 = value.as_struct_ref::<TestStruct>()?;
    let ref2 = value.as_struct_ref::<TestStruct>()?;

    // Verify identity
    assert!(Arc::ptr_eq(&ref1, &ref2));

    // Verify content
    assert_eq!(*ref1, test_struct);
    assert_eq!(*ref2, test_struct);

    assert_eq!(ref1.field1, "Hello");
    assert_eq!(ref1.field2, 42);

    //lets check serialization
    let bytes = value.to_bytes()?;
    let value_from_bytes = ArcValueType::from_bytes(&bytes)?;
    let ref3 = value_from_bytes.as_struct_ref::<TestStruct>()?;
    assert_eq!(*ref3, test_struct);

    Ok(())
}

#[test]
fn test_type_mismatch_errors() -> Result<()> {
    // Create a value with a string
    let value = ArcValueType::from_value("Hello, world!".to_string());

    // Try to get it as an integer - should fail
    let result = value.as_type_ref::<i32>();
    assert!(result.is_err());

    // Try to get it as a list - should fail
    let result = value.as_list_ref::<String>();
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_null_value() -> Result<()> {
    let value = ArcValueType::null();
    assert!(value.is_null());

    Ok(())
}

#[test]
fn test_primitive_cloning() -> Result<()> {
    // Test that as_type (not as_type_ref) does clone the value
    let string_value = "Hello, world!".to_string();
    let value = ArcValueType::from_value(string_value);

    // Get a cloned value
    let cloned_value: String = value.as_type()?;

    // Verify it's a clone, not the original
    let ref_value = value.as_type_ref::<String>()?;
    assert_eq!(cloned_value, *ref_value);

    // Modifying the clone should not affect the original
    let mut cloned_value = cloned_value;
    cloned_value.push_str(" Modified");

    // Original should remain unchanged
    let ref_value = value.as_type_ref::<String>()?;
    assert_eq!(&*ref_value, "Hello, world!");
    assert_eq!(cloned_value, "Hello, world! Modified");

    Ok(())
}
