#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;

    use runar_common::types::ArcValueType;
    use runar_common::types::VMap;

    // Test implementation
    fn create_test_vmap() -> VMap<ArcValueType> {
        let mut map = HashMap::new();
        map.insert(
            "key1".to_string(),
            ArcValueType::new_primitive("value1".to_string()),
        );
        map.insert("key2".to_string(), ArcValueType::new_primitive(42.0));
        map.insert("key3".to_string(), ArcValueType::new_primitive(true));
        VMap::from_hashmap(map)
    }

    #[test]
    fn test_basics() -> Result<()> {
        let vmap = create_test_vmap();

        // Test direct key access
        let value1 = vmap.get("key1").unwrap();
        let typed_value1: String = value1.as_type()?;
        assert_eq!(typed_value1, "value1");

        // Test number value
        let value2 = vmap.get("key2").unwrap();
        let typed_value2: f64 = value2.as_type()?;
        assert_eq!(typed_value2, 42.0);

        // Test boolean value
        let value3 = vmap.get("key3").unwrap();
        let typed_value3: bool = value3.as_type()?;
        assert!(typed_value3);

        // Test missing key
        let missing = vmap.get("missing");
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_error_handling() -> Result<()> {
        let vmap = create_test_vmap();

        // Try to get a string as a number
        let value1 = vmap.get("key1").unwrap();
        let result: Result<f64> = value1.as_type();
        assert!(result.is_err());

        // Check that the error message contains information about the wrong type
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("mismatch"),
            "Error message should mention type conversion issue: {}",
            err_msg
        );

        Ok(())
    }

    #[test]
    fn test_clone() -> Result<()> {
        let vmap = create_test_vmap();
        let cloned = vmap.clone();

        // Verify cloned map has the same values
        assert_eq!(vmap.inner.len(), cloned.inner.len());

        // Verify all keys and values are cloned
        for (key, value) in vmap.inner.iter() {
            let cloned_value = cloned.inner.get(key).unwrap();

            // Compare string values
            if let Ok(v1) = value.as_type::<String>() {
                let v2: String = cloned_value.as_type()?;
                assert_eq!(v1, v2);
            }

            // Compare number values
            if let Ok(v1) = value.as_type::<f64>() {
                let v2: f64 = cloned_value.as_type()?;
                assert_eq!(v1, v2);
            }

            // Compare boolean values
            if let Ok(v1) = value.as_type::<bool>() {
                let v2: bool = cloned_value.as_type()?;
                assert_eq!(v1, v2);
            }
        }

        Ok(())
    }
}
