// runar_common/src/bin/value_typed_example.rs
//
// Example demonstrating the use of the new type-preserving ValueType system

use std::collections::HashMap;
use std::any::Any;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use runar_common::types::{Value, MapValue, ValueBase, ValueConvert, CustomStruct};

fn main() -> Result<()> {
    println!("Value<T> and MapValue<K, V> Type-Preserving Serialization Example");
    println!("-------------------------------------------------------------");
    
    // Example 1: Basic Primitive Values
    example_primitives()?;
    
    // Example 2: Lists
    example_lists()?;
    
    // Example 3: Maps
    example_maps()?;
    
    // Example 4: Serialization and deserialization
    example_serialization()?;
    
    // Example 5: Zero-copy access
    example_zero_copy()?;
    
    // Example 6: Custom structs
    example_custom_structs()?;
    
    Ok(())
}

fn example_primitives() -> Result<()> {
    println!("\n### Example 1: Primitive Values ###");
    
    // Create basic primitive values with unified constructor
    let s = Value::<String>::new("Hello".to_string());
    let i = Value::<i32>::new(42);
    let f = Value::<f64>::new(3.14159);
    let b = Value::<bool>::new(true);
    
    // Create null value
    let null = Value::<()>::null();
    
    // Access and use the values
    if let Value::Value(val) = &s {
        println!("String value: {}", val);
    }
    
    if let Value::Value(val) = &i {
        println!("Integer value: {}", val);
    }
    
    if let Value::Value(val) = &f {
        println!("Float value: {}", val);
    }
    
    if let Value::Value(val) = &b {
        println!("Boolean value: {}", val);
    }
    
    if let Value::Null = &null {
        println!("Null value");
    }
    
    // Using the type conversion API
    let i32_value: i32 = s.as_type().unwrap_or(0);
    println!("Attempting to convert string to i32 (should fail): {}", i32_value);
    
    let i32_value: i32 = i.as_type()?;
    println!("Converting i32 to i32 (should succeed): {}", i32_value);
    
    Ok(())
}

fn example_lists() -> Result<()> {
    println!("\n### Example 2: Lists ###");
    
    // Create lists with the list constructor
    let str_list = Value::<String>::new_list(vec!["one".to_string(), "two".to_string(), "three".to_string()]);
    let int_list = Value::<i32>::new_list(vec![1, 2, 3, 4, 5]);
    
    // Access and use the lists
    if let Value::List(values) = &str_list {
        println!("String list with {} items:", values.len());
        for (i, val) in values.iter().enumerate() {
            println!("  [{}]: {}", i, val);
        }
    }
    
    if let Value::List(values) = &int_list {
        println!("Integer list with {} items:", values.len());
        for (i, val) in values.iter().enumerate() {
            println!("  [{}]: {}", i, val);
        }
    }
    
    // Using the list conversion API
    let str_vec: Vec<String> = str_list.as_list()?;
    println!("Converted string list: {:?}", str_vec);
    
    let int_vec: Vec<i32> = int_list.as_list()?;
    println!("Converted integer list: {:?}", int_vec);
    
    // Try a conversion that should fail
    let result: Result<Vec<f64>> = int_list.as_list();
    println!("Attempting to convert int list to float list (should fail): {:?}", result.is_err());
    
    Ok(())
}

fn example_maps() -> Result<()> {
    println!("\n### Example 3: Maps ###");
    
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
    let str_to_int_map = MapValue::<String, i32>::new(str_to_int);
    let int_to_str_map = MapValue::<i32, String>::new(int_to_str);
    
    // Access and use the maps
    println!("String to int map with {} entries:", str_to_int_map.entries.len());
    for (k, v) in &str_to_int_map.entries {
        println!("  {}: {}", k, v);
    }
    
    println!("Int to string map with {} entries:", int_to_str_map.entries.len());
    for (k, v) in &int_to_str_map.entries {
        println!("  {}: {}", k, v);
    }
    
    // Using the map conversion API
    let map1: HashMap<String, i32> = str_to_int_map.as_map()?;
    println!("Converted string to int map: {:?}", map1);
    
    let map2: HashMap<i32, String> = int_to_str_map.as_map()?;
    println!("Converted int to string map: {:?}", map2);
    
    // Try a conversion that should fail
    let result: Result<HashMap<i32, i32>> = str_to_int_map.as_map();
    println!("Attempting to convert to wrong map type (should fail): {:?}", result.is_err());
    
    Ok(())
}

fn example_serialization() -> Result<()> {
    println!("\n### Example 4: Serialization and Deserialization ###");
    
    // Create a value to serialize
    let original = Value::<i32>::new(42);
    
    // Serialize to bytes
    let bytes = original.to_bytes()?;
    println!("Serialized {} bytes", bytes.len());
    
    // In a real scenario, these bytes would be sent over the network or stored
    // For this example, we'll just use them immediately
    
    // Create a value from the serialized bytes
    let deserialized = runar_common::types::value_from_bytes(&bytes)?;
    
    // Type-safe access to the deserialized value
    if let Ok(val) = deserialized.as_type::<i32>() {
        println!("Successfully deserialized value: {}", val);
    } else {
        println!("Failed to deserialize value");
    }
    
    Ok(())
}

fn example_zero_copy() -> Result<()> {
    println!("\n### Example 5: Zero-Copy Access ###");
    
    // Create complex object
    let mut map = HashMap::new();
    map.insert("name".to_string(), "John Doe".to_string());
    map.insert("email".to_string(), "john@example.com".to_string());
    map.insert("role".to_string(), "Admin".to_string());
    
    let user_map = MapValue::<String, String>::new(map);
    
    // Direct access without serialization/deserialization overhead
    println!("User information:");
    println!("  Name: {}", user_map.entries.get("name").unwrap_or(&"Unknown".to_string()));
    println!("  Email: {}", user_map.entries.get("email").unwrap_or(&"Unknown".to_string()));
    println!("  Role: {}", user_map.entries.get("role").unwrap_or(&"Unknown".to_string()));
    
    // In same-runtime service calls, we can just pass the MapValue directly
    // and the receiving service can access the entries without any overhead
    simulate_same_runtime_service_call(user_map)?;
    
    Ok(())
}

// Define a custom struct for example 6
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Person {
    name: String,
    age: i32,
    email: String,
}

// The CustomStruct trait is automatically implemented for Person
// because it derives Debug, Clone, Serialize, and implements Send + Sync

fn example_custom_structs() -> Result<()> {
    println!("\n### Example 6: Custom Structs ###");
    
    // Create a Person instance
    let person = Person {
        name: "Alice Smith".to_string(),
        age: 32,
        email: "alice@example.com".to_string(),
    };
    
    // Create a Value that contains our custom struct
    // No need to explicitly implement CustomStruct - it's automatically implemented
    let person_value = Value::<()>::Struct(Box::new(person.clone()));
    
    // Print information about the struct
    println!("Created a custom struct value of type: {}", 
             if let Value::Struct(s) = &person_value { 
                 s.type_name() 
             } else { 
                 "Unknown" 
             });
    
    // Serialize the struct
    let bytes = person_value.to_bytes()?;
    println!("Serialized custom struct in {} bytes", bytes.len());
    
    // In a real scenario, we'd send these bytes over the network
    // For this example, we'll deserialize it right away
    
    // Deserialize
    let value = runar_common::types::value_from_bytes(&bytes)?;
    
    // Access the struct (in a real scenario, we'd use ValueConvert apis)
    if let Value::Struct(s) = value.as_any().downcast_ref::<Value<()>>().unwrap() {
        if let Some(p) = s.as_any().downcast_ref::<Person>() {
            println!("Successfully deserialized Person:");
            println!("  Name: {}", p.name);
            println!("  Age: {}", p.age);
            println!("  Email: {}", p.email);
        } else {
            println!("Failed to downcast to Person");
        }
    } else {
        println!("Not a struct value");
    }
    
    Ok(())
}

// Simulates a service call within the same runtime
fn simulate_same_runtime_service_call(params: MapValue<String, String>) -> Result<Box<dyn ValueBase>> {
    println!("Service received user data with zero overhead:");
    println!("  Name: {}", params.entries.get("name").unwrap_or(&"Unknown".to_string()));
    
    // Create a response
    let response = Value::<String>::new("Success".to_string());
    Ok(Box::new(response))
} 