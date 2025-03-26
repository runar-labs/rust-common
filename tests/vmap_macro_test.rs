use runar_common::types::ValueType;
use runar_common::{vmap, vmap_str, vmap_i32, vmap_i64, vmap_i16, vmap_i8, 
                  vmap_u8, vmap_u32, vmap_u64, vmap_f64, vmap_f32, vmap_bool, 
                  vmap_vec};
// Only include these if the chrono feature is available
#[cfg(feature = "chrono")]
use runar_common::{vmap_date, vmap_datetime};

#[test]
fn test_map_creation() {
    // Test empty map creation
    let empty_map = vmap!{};
    assert!(matches!(empty_map, ValueType::Map(_)));
    if let ValueType::Map(m) = empty_map {
        assert!(m.is_empty());
    }

    // Test map creation with entries
    let map = vmap! {
        "string" => "value",
        "number" => 42,
        "float" => 3.14,
        "bool" => true,
        "null" => ValueType::Null
    };

    // Verify map contents
    if let ValueType::Map(m) = map {
        assert_eq!(m.len(), 5);
        assert_eq!(m.get("string").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(m.get("number").and_then(|v| v.as_f64()), Some(42.0));
        assert_eq!(m.get("float").and_then(|v| v.as_f64()), Some(3.14));
        assert_eq!(m.get("bool").and_then(|v| v.as_bool()), Some(true));
        assert!(m.get("null").map_or(false, |v| matches!(v, ValueType::Null)));
    } else {
        panic!("Expected ValueType::Map");
    }
}

#[test]
fn test_value_extraction() {
    // Create a test map
    let map = vmap! {
        "string" => "value",
        "number" => 42,
        "float" => 3.14,
        "bool" => true,
        "null" => ValueType::Null,
        "nested" => vmap! {
            "key" => "nested value"
        }
    };

    // Test string extraction using the specialized macro
    let string_val = vmap_str!(map.clone(), "string", "default");
    assert_eq!(string_val, "value");

    // Test number extraction using the specialized macro
    let number_val = vmap_i32!(map.clone(), "number", 0);
    assert_eq!(number_val, 42);

    // Test float extraction using the specialized macro
    let float_val = vmap_f64!(map.clone(), "float", 0.0);
    assert_eq!(float_val, 3.14);

    // Test boolean extraction using the specialized macro
    let bool_val = vmap_bool!(map.clone(), "bool", false);
    assert_eq!(bool_val, true);

    // Test missing key with default using the specialized macro
    let missing_val = vmap_str!(map.clone(), "missing", "default");
    assert_eq!(missing_val, "default");

    // Test null value with default using the specialized macro
    let null_val = vmap_str!(map.clone(), "null", "default");
    assert_eq!(null_val, "default");

    // Test nested map extraction using the vmap! macro
    let nested_val = vmap!(map.clone(), "nested");
    assert!(matches!(nested_val, ValueType::Map(_)));
    
    if let ValueType::Map(nested) = nested_val {
        let key_val = vmap_str!(ValueType::Map(nested), "key", "default");
        assert_eq!(key_val, "nested value");
    } else {
        panic!("Expected ValueType::Map");
    }
    
    // Test nested map extraction using dot notation (simpler approach)
    let nested_key_val = vmap_str!(map.clone(), "nested.key", "default");
    assert_eq!(nested_key_val, "nested value");
}

#[test]
fn test_type_conversion() {
    let map = vmap! {
        "number_as_string" => 42,
        "bool_as_string" => true,
        "string_as_number" => "42",
        "string_as_bool" => "true"
    };

    // Test number to string conversion 
    let number_as_string = vmap_str!(map.clone(), "number_as_string", "");
    assert_eq!(number_as_string, "42");

    // Test boolean to string conversion
    let bool_as_string = vmap_str!(map.clone(), "bool_as_string", "");
    assert_eq!(bool_as_string, "true");

    // Test string to number conversion
    let string_as_number = vmap_i32!(map.clone(), "string_as_number", 0);
    assert_eq!(string_as_number, 42);

    // Test string to boolean conversion
    let string_as_bool = vmap_bool!(map.clone(), "string_as_bool", false);
    assert_eq!(string_as_bool, true);

    // Test additional numeric conversions
    let int_val = vmap_i32!(map.clone(), "number_as_string", 0);
    assert_eq!(int_val, 42);
    
    let float_val = vmap_f64!(map.clone(), "number_as_string", 0.0);
    assert_eq!(float_val, 42.0);
}

#[test]
fn test_nested_key_lookup() {
    // Create a deeply nested map
    let nested_map = vmap! {
        "user" => vmap! {
            "profile" => vmap! {
                "name" => "John Doe",
                "contact" => vmap! {
                    "email" => "john@example.com",
                    "phone" => "555-1234"
                }
            },
            "settings" => vmap! {
                "theme" => "dark",
                "notifications" => true
            },
            "stats" => vmap! {
                "score" => 95,
                "rating" => 4.8
            }
        }
    };

    // Test nested string lookups
    let name = vmap_str!(nested_map.clone(), "user.profile.name", "");
    assert_eq!(name, "John Doe");

    let email = vmap_str!(nested_map.clone(), "user.profile.contact.email", "");
    assert_eq!(email, "john@example.com");

    let theme = vmap_str!(nested_map.clone(), "user.settings.theme", "default");
    assert_eq!(theme, "dark");

    // Test nested number lookups
    let score = vmap_i32!(nested_map.clone(), "user.stats.score", 0);
    assert_eq!(score, 95);

    // Test nested float lookups
    let rating = vmap_f64!(nested_map.clone(), "user.stats.rating", 0.0);
    assert_eq!(rating, 4.8);

    // Test nested boolean lookups
    let notifications = vmap_bool!(nested_map.clone(), "user.settings.notifications", false);
    assert_eq!(notifications, true);

    // Test missing keys with defaults
    let missing = vmap_str!(nested_map.clone(), "user.profile.address", "N/A");
    assert_eq!(missing, "N/A");

    let deep_missing = vmap_str!(nested_map.clone(), "user.profile.contact.address.street", "N/A");
    assert_eq!(deep_missing, "N/A");

    // Test partially missing paths
    let invalid_path = vmap_i32!(nested_map.clone(), "user.profile.name.first", 0);
    assert_eq!(invalid_path, 0); // Should return default as "name" is a string, not a map
    
    // Test extremely deep nesting (5+ levels)
    let deep_map = vmap! {
        "level1" => vmap! {
            "level2" => vmap! {
                "level3" => vmap! {
                    "level4" => vmap! {
                        "level5" => vmap! {
                            "level6" => vmap! {
                                "data" => "deep value"
                            },
                            "number" => 42,
                            "active" => true
                        }
                    }
                }
            }
        }
    };
    
    // Access extremely deep value (6 levels)
    let deep_value = vmap_str!(deep_map.clone(), "level1.level2.level3.level4.level5.level6.data", "not found");
    assert_eq!(deep_value, "deep value");
    
    // Access value at level 5
    let level5_number = vmap_i32!(deep_map.clone(), "level1.level2.level3.level4.level5.number", 0);
    assert_eq!(level5_number, 42);
    
    let level5_active = vmap_bool!(deep_map.clone(), "level1.level2.level3.level4.level5.active", false);
    assert_eq!(level5_active, true);
    
    // Test path with more segments than exist in the structure
    let too_deep = vmap_str!(deep_map.clone(), "level1.level2.level3.level4.level5.level6.level7.level8.data", "too deep");
    assert_eq!(too_deep, "too deep");
}

#[test]
fn test_extended_type_conversions() {
    // Create a test map with various data types
    let map = vmap! {
        // Integer types
        "i8_val" => 127,
        "i16_val" => 32767,
        "i64_val" => 9223372036854775807_i64,
        "u8_val" => 255,
        "u32_val" => 4294967295_u32,
        "u64_val" => 18446744073709551615_u64,
        
        // Float types
        "f32_val" => 3.14159,
        
        // Date and time
        "date_val" => "2023-08-15",
        "datetime_val" => "2023-08-15T14:30:00Z",
        
        // Array/Vec
        "string_array" => vec!["one", "two", "three"],
        
        // Nested values
        "nested" => vmap! {
            "i8_nested" => 42,
            "i16_nested" => 1000,
            "i64_nested" => 1234567890123_i64,
            "u8_nested" => 200,
            "u32_nested" => 1000000,
            "u64_nested" => 1000000000000_u64,
            "f32_nested" => 2.71828,
            "date_nested" => "2022-01-01",
            "datetime_nested" => "2022-01-01T00:00:00Z",
            "string_array_nested" => vec!["apple", "banana", "cherry"]
        }
    };
    
    // Test i8 extraction
    let i8_val = vmap_i8!(map.clone(), "i8_val", 0);
    assert_eq!(i8_val, 127);
    
    let i8_nested = vmap_i8!(map.clone(), "nested.i8_nested", 0);
    assert_eq!(i8_nested, 42);
    
    // Test i16 extraction
    let i16_val = vmap_i16!(map.clone(), "i16_val", 0);
    assert_eq!(i16_val, 32767);
    
    let i16_nested = vmap_i16!(map.clone(), "nested.i16_nested", 0);
    assert_eq!(i16_nested, 1000);
    
    // Test i64 extraction
    let i64_val = vmap_i64!(map.clone(), "i64_val", 0);
    assert_eq!(i64_val, 9223372036854775807_i64);
    
    let i64_nested = vmap_i64!(map.clone(), "nested.i64_nested", 0);
    assert_eq!(i64_nested, 1234567890123);
    
    // Test u8 extraction
    let u8_val = vmap_u8!(map.clone(), "u8_val", 0);
    assert_eq!(u8_val, 255);
    
    let u8_nested = vmap_u8!(map.clone(), "nested.u8_nested", 0);
    assert_eq!(u8_nested, 200);
    
    // Test u32 extraction
    let u32_val = vmap_u32!(map.clone(), "u32_val", 0);
    assert_eq!(u32_val, 4294967295_u32);
    
    let u32_nested = vmap_u32!(map.clone(), "nested.u32_nested", 0);
    assert_eq!(u32_nested, 1000000);
    
    // Test u64 extraction
    let u64_val = vmap_u64!(map.clone(), "u64_val", 0);
    assert_eq!(u64_val, 18446744073709551615_u64);
    
    let u64_nested = vmap_u64!(map.clone(), "nested.u64_nested", 0);
    assert_eq!(u64_nested, 1000000000000);
    
    // Test f32 extraction
    let f32_val = vmap_f32!(map.clone(), "f32_val", 0.0);
    assert!((f32_val - 3.14159).abs() < 0.00001);
    
    let f32_nested = vmap_f32!(map.clone(), "nested.f32_nested", 0.0);
    assert!((f32_nested - 2.71828).abs() < 0.00001);
    
    // Test date extraction (requires chrono)
    #[cfg(feature = "chrono")]
    {
        use chrono::{NaiveDate, DateTime, Utc};
        
        let date_val = vmap_date!(map.clone(), "date_val", NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        assert_eq!(date_val.to_string(), "2023-08-15");
        
        let date_nested = vmap_date!(map.clone(), "nested.date_nested", NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        assert_eq!(date_nested.to_string(), "2022-01-01");
        
        // Test datetime extraction
        let datetime_val = vmap_datetime!(map.clone(), "datetime_val", Utc::now());
        assert_eq!(datetime_val.to_rfc3339(), "2023-08-15T14:30:00+00:00");
        
        let datetime_nested = vmap_datetime!(map.clone(), "nested.datetime_nested", Utc::now());
        assert_eq!(datetime_nested.to_rfc3339(), "2022-01-01T00:00:00+00:00");
    }
    
    // Test vector extraction
    let string_array = vmap_vec!(map.clone(), "string_array", Vec::<String>::new());
    assert_eq!(string_array, vec!["one", "two", "three"]);
    
    let string_array_nested = vmap_vec!(map.clone(), "nested.string_array_nested", Vec::<String>::new());
    assert_eq!(string_array_nested, vec!["apple", "banana", "cherry"]);
    
    // Test missing values with defaults
    let missing_i8 = vmap_i8!(map.clone(), "missing", 42);
    assert_eq!(missing_i8, 42);
    
    let missing_i16 = vmap_i16!(map.clone(), "missing", 1000);
    assert_eq!(missing_i16, 1000);
    
    let missing_u8 = vmap_u8!(map.clone(), "missing", 255);
    assert_eq!(missing_u8, 255);
    
    let missing_f32 = vmap_f32!(map.clone(), "missing", 3.14);
    assert!((missing_f32 - 3.14).abs() < 0.00001);
    
    let missing_array = vmap_vec!(map.clone(), "missing", vec![String::from("default")]);
    assert_eq!(missing_array, vec!["default"]);
    
    // Test conversion between types
    let num_as_str = vmap_str!(map.clone(), "i8_val", "");
    assert_eq!(num_as_str, "127");
    
    // Add string as number for conversion test
    let map_with_string_number = vmap! {
        "string_as_number" => "42"
    };
    let str_as_num = vmap_i8!(map_with_string_number.clone(), "string_as_number", 0);
    assert_eq!(str_as_num, 42);
} 