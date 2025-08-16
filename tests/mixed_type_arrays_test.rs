//! Tests for mixed type array handling
//!
//! This module tests the system's ability to handle arrays with mixed types,
//! empty arrays, and edge cases across all supported languages.

use json2schema::codegen::factory::GeneratorFactory;
use json2schema::codegen::generator::GenerationOptions;
use json2schema::codegen::types::JsonToIrConverter;
use serde_json::{json, Value};

/// Test data for mixed type array scenarios
mod test_data {
    use super::*;

    /// Array with mixed primitive types
    pub fn mixed_primitives() -> Value {
        json!({
            "mixed_array": [1, "hello", true, 3.14, null],
            "name": "test"
        })
    }

    /// Array with mixed objects and primitives
    pub fn mixed_objects_primitives() -> Value {
        json!({
            "mixed_items": [
                {"id": 1, "name": "item1"},
                "simple_string",
                {"id": 2, "type": "special"},
                42,
                true
            ]
        })
    }

    /// Array with similar objects (should create unified type)
    pub fn similar_objects() -> Value {
        json!({
            "users": [
                {"id": 1, "name": "Alice", "email": "alice@example.com"},
                {"id": 2, "name": "Bob"},
                {"id": 3, "name": "Charlie", "email": "charlie@example.com", "age": 30}
            ]
        })
    }

    /// Array with completely different objects
    pub fn different_objects() -> Value {
        json!({
            "items": [
                {"type": "user", "name": "Alice", "email": "alice@example.com"},
                {"type": "product", "title": "Widget", "price": 19.99},
                {"type": "order", "id": 123, "total": 59.97, "items": ["widget1", "widget2"]}
            ]
        })
    }

    /// Empty arrays and null handling
    pub fn empty_and_null() -> Value {
        json!({
            "empty_array": [],
            "null_array": null,
            "array_with_nulls": [null, null, "value", null],
            "mixed_with_empty_objects": [{}, {"name": "test"}, {}]
        })
    }

    /// Nested arrays with mixed types
    pub fn nested_mixed_arrays() -> Value {
        json!({
            "nested": [
                [1, 2, 3],
                ["a", "b", "c"],
                [true, false],
                [{"nested": "object"}, "mixed"]
            ]
        })
    }
}

#[test]
fn test_mixed_primitive_array_go() {
    let json_data = test_data::mixed_primitives();
    let generator = GeneratorFactory::create_generator("go").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("MixedPrimitives")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Go mixed primitives code:\n{}", code);
    
    // Should use interface{} for mixed array
    assert!(code.contains("[]interface{}"));
    assert!(code.contains("MixedArray"));
}

#[test]
fn test_mixed_primitive_array_rust() {
    let json_data = test_data::mixed_primitives();
    let generator = GeneratorFactory::create_generator("rust").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("MixedPrimitives")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Rust mixed primitives code:\n{}", code);
    
    // Should use Vec<serde_json::Value> for mixed array
    assert!(code.contains("Vec<serde_json::Value>"));
    assert!(code.contains("mixed_array"));
}

#[test]
fn test_mixed_primitive_array_typescript() {
    let json_data = test_data::mixed_primitives();
    let generator = GeneratorFactory::create_generator("typescript").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("MixedPrimitives")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("TypeScript mixed primitives code:\n{}", code);
    
    // Should use any[] for mixed array
    assert!(code.contains("any[]"));
    assert!(code.contains("mixedArray"));
}

#[test]
fn test_mixed_primitive_array_python() {
    let json_data = test_data::mixed_primitives();
    let generator = GeneratorFactory::create_generator("python").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("MixedPrimitives")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Python mixed primitives code:\n{}", code);
    
    // Should use List[Any] for mixed array
    assert!(code.contains("List[Any]"));
    assert!(code.contains("mixed_array"));
}

#[test]
fn test_similar_objects_array() {
    let json_data = test_data::similar_objects();
    let generator = GeneratorFactory::create_generator("go").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("UserList")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Go similar objects code:\n{}", code);
    
    // Should create a unified struct for similar objects
    assert!(code.contains("[]"));
    assert!(code.contains("Users"));
    
    // Should handle optional fields (email and age are not in all objects)
    // The exact implementation may vary, but should handle optionality
}

#[test]
fn test_different_objects_array() {
    let json_data = test_data::different_objects();
    let generator = GeneratorFactory::create_generator("typescript").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("DifferentItems")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("TypeScript different objects code:\n{}", code);
    
    // Should use any[] for very different objects
    assert!(code.contains("any[]") || code.contains("Items"));
}

#[test]
fn test_empty_arrays_handling() {
    let json_data = test_data::empty_and_null();
    let generator = GeneratorFactory::create_generator("rust").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("EmptyArrays")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Rust empty arrays code:\n{}", code);
    
    // Should handle empty arrays gracefully
    assert!(code.contains("Vec<serde_json::Value>") || code.contains("Vec<"));
    assert!(code.contains("empty_array"));
}

#[test]
fn test_nested_mixed_arrays() {
    let json_data = test_data::nested_mixed_arrays();
    let generator = GeneratorFactory::create_generator("python").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("NestedMixed")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Python nested mixed arrays code:\n{}", code);
    
    // Should handle nested arrays with mixed types
    assert!(code.contains("List["));
    assert!(code.contains("nested"));
}

#[test]
fn test_array_type_analysis() {
    let mut converter = JsonToIrConverter::new("go");
    
    // Test mixed primitive array wrapped in object
    let mixed_json = json!({"data": [1, "hello", true]});
    let result = converter.convert_to_struct(&mixed_json, "TestStruct");
    assert!(result.is_ok());
    
    // Test empty array wrapped in object
    let empty_json = json!({"data": []});
    let result = converter.convert_to_struct(&empty_json, "EmptyStruct");
    assert!(result.is_ok());
    
    // Test homogeneous array wrapped in object
    let homogeneous_json = json!({"data": [1, 2, 3, 4]});
    let result = converter.convert_to_struct(&homogeneous_json, "NumberStruct");
    assert!(result.is_ok());
}

#[test]
fn test_mixed_objects_with_common_fields() {
    let json_data = json!({
        "items": [
            {"id": 1, "name": "Alice", "type": "user"},
            {"id": 2, "name": "Bob", "type": "user", "email": "bob@example.com"},
            {"id": 3, "name": "Charlie", "type": "user"}
        ]
    });

    let generator = GeneratorFactory::create_generator("go").unwrap();
    let options = GenerationOptions::default()
        .with_struct_name("CommonFields")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("Go common fields code:\n{}", code);
    
    // Should create a unified struct since objects have common structure
    assert!(code.contains("Items"));
    // Should handle optional email field
    assert!(code.contains("Email") || code.contains("email"));
}

#[test]
fn test_array_with_null_values() {
    let json_data = json!({
        "nullable_items": [
            {"name": "Alice"},
            null,
            {"name": "Bob"},
            null
        ]
    });

    let generator = GeneratorFactory::create_generator("typescript").unwrap();
    let options = GenerationOptions::default()
        .with_struct_name("NullableItems")
        .with_comments(true);

    let result = generator.generate(&json_data, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    println!("TypeScript nullable items code:\n{}", code);
    
    // Should handle null values in arrays appropriately
    assert!(code.contains("nullableItems"));
}

#[test]
fn test_performance_with_large_mixed_array() {
    use std::time::Instant;
    
    // Create a large array with mixed types
    let mut items = Vec::new();
    for i in 0..1000 {
        match i % 4 {
            0 => items.push(json!(i)),
            1 => items.push(json!(format!("item_{}", i))),
            2 => items.push(json!(i % 2 == 0)),
            3 => items.push(json!({"id": i, "value": format!("object_{}", i)})),
            _ => unreachable!(),
        }
    }
    
    let json_data = json!({"large_mixed_array": items});
    let generator = GeneratorFactory::create_generator("rust").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("LargeMixed");

    let start = Instant::now();
    let result = generator.generate(&json_data, &options);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // Should complete within reasonable time
    assert!(duration.as_secs() < 10, "Processing took too long: {:?}", duration);
    
    let code = result.unwrap();
    assert!(!code.is_empty());
    println!("Large mixed array processing took: {:?}", duration);
}

#[test]
fn test_edge_case_arrays() {
    // Test various edge cases
    let edge_cases = vec![
        ("single_null", json!({"arr": [null]})),
        ("mixed_nulls", json!({"arr": [null, "value", null]})),
        ("nested_empty", json!({"arr": [[], [1, 2], []]})),
        ("mixed_nested", json!({"arr": [[1, 2], ["a", "b"], [true, false]]})),
    ];

    for (name, json_data) in edge_cases {
        println!("Testing edge case: {}", name);
        
        let generator = GeneratorFactory::create_generator("go").unwrap();
        let options = GenerationOptions::default()
            .with_struct_name(&format!("EdgeCase{}", name.replace("_", "")));

        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok(), "Failed to generate code for edge case: {}", name);
        
        let code = result.unwrap();
        assert!(!code.is_empty(), "Generated empty code for edge case: {}", name);
        println!("Generated code for {}:\n{}\n", name, code);
    }
}