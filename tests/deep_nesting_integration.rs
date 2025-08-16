//! Integration tests for deep nesting handling
//!
//! This module tests the system's ability to handle deeply nested JSON structures
//! across all supported languages, ensuring proper error handling, naming strategies,
//! and performance characteristics.

use json2schema::codegen::factory::GeneratorFactory;
use json2schema::codegen::generator::GenerationOptions;
use json2schema::codegen::types::JsonToIrConverter;
use serde_json::{json, Value};

/// Test data for deep nesting scenarios
mod test_data {
    use super::*;

    /// Create a deeply nested JSON structure for testing
    pub fn create_deep_nested_json(depth: usize) -> Value {
        fn create_nested_object(current_depth: usize, max_depth: usize) -> Value {
            if current_depth >= max_depth {
                return json!({
                    "leaf_value": format!("depth_{}", current_depth),
                    "leaf_number": current_depth,
                    "leaf_bool": current_depth % 2 == 0
                });
            }

            json!({
                "level": current_depth,
                "name": format!("level_{}", current_depth),
                "nested": create_nested_object(current_depth + 1, max_depth),
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "level_info": format!("This is level {}", current_depth)
                }
            })
        }

        create_nested_object(0, depth)
    }

    /// Create a JSON structure with multiple nested branches
    pub fn create_branched_nested_json(depth: usize, branches: usize) -> Value {
        let max_sub_branches = if branches > 3 { 3 } else { branches }; // Limit to 3 sub-branches to avoid exponential growth
        
        fn create_branch(branch_id: usize, current_depth: usize, max_depth: usize, max_sub_branches: usize) -> Value {
            if current_depth >= max_depth {
                return json!({
                    "branch_id": branch_id,
                    "depth": current_depth,
                    "value": format!("branch_{}_depth_{}", branch_id, current_depth)
                });
            }

            let mut branch = json!({
                "branch_id": branch_id,
                "level": current_depth,
                "data": format!("branch_{}_level_{}", branch_id, current_depth)
            });

            // Add nested branches
            for i in 0..max_sub_branches {
                let sub_branch = create_branch(i, current_depth + 1, max_depth, max_sub_branches);
                branch[format!("sub_branch_{}", i)] = sub_branch;
            }

            branch
        }

        let mut root = json!({
            "root": true,
            "total_branches": branches,
            "max_depth": depth
        });

        for i in 0..branches {
            root[format!("branch_{}", i)] = create_branch(i, 0, depth, max_sub_branches);
        }

        root
    }

    /// Create a JSON structure with arrays at different nesting levels
    pub fn create_nested_arrays_json(depth: usize) -> Value {
        fn create_nested_array_level(current_depth: usize, max_depth: usize) -> Value {
            if current_depth >= max_depth {
                return json!([
                    format!("item_1_depth_{}", current_depth),
                    format!("item_2_depth_{}", current_depth),
                    format!("item_3_depth_{}", current_depth)
                ]);
            }

            json!({
                "level": current_depth,
                "items": [
                    {
                        "id": 1,
                        "nested": create_nested_array_level(current_depth + 1, max_depth)
                    },
                    {
                        "id": 2,
                        "nested": create_nested_array_level(current_depth + 1, max_depth)
                    }
                ],
                "metadata": {
                    "level_info": format!("Array level {}", current_depth)
                }
            })
        }

        create_nested_array_level(0, depth)
    }
}

#[test]
fn test_structure_stats_calculation() {
    let simple_json = json!({
        "name": "test",
        "nested": {
            "value": 42
        }
    });

    let stats = JsonToIrConverter::get_structure_stats(&simple_json);
    assert_eq!(stats.max_depth, 2);
    assert_eq!(stats.object_count, 2);
    assert_eq!(stats.array_count, 0);
    assert_eq!(stats.total_fields, 3); // name, nested, value

    let array_json = json!({
        "items": [1, 2, 3],
        "nested_items": [
            {"id": 1},
            {"id": 2}
        ]
    });

    let array_stats = JsonToIrConverter::get_structure_stats(&array_json);
    assert_eq!(array_stats.max_depth, 2);
    assert_eq!(array_stats.object_count, 3); // root + 2 nested objects
    assert_eq!(array_stats.array_count, 2);
    assert_eq!(array_stats.max_array_length, 3);
}

#[test]
fn test_nesting_depth_validation() {
    // Test reasonable depth
    let shallow_json = test_data::create_deep_nested_json(5);
    let result = JsonToIrConverter::validate_nesting_depth(&shallow_json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    // Test moderate depth
    let moderate_json = test_data::create_deep_nested_json(15);
    let result = JsonToIrConverter::validate_nesting_depth(&moderate_json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 15);

    // Test very deep nesting (should still work but be flagged)
    let deep_json = test_data::create_deep_nested_json(50);
    let result = JsonToIrConverter::validate_nesting_depth(&deep_json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 50);
}

#[test]
fn test_deep_nesting_go_generation() {
    let deep_json = test_data::create_deep_nested_json(10);
    let generator = GeneratorFactory::create_generator("go").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("DeepNested")
        .with_comments(true);

    let result = generator.generate(&deep_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should contain multiple struct definitions
    assert!(code.contains("type DeepNested struct"));
    assert!(code.contains("type Nested struct"));
    assert!(code.contains("type Metadata struct"));
    
    // Should have proper nesting structure
    assert!(code.contains("Nested Nested"));
    assert!(code.contains("Metadata Metadata"));
    
    // Should include complexity information in comments
    assert!(code.contains("Structure complexity:"));
}

#[test]
fn test_deep_nesting_rust_generation() {
    let deep_json = test_data::create_deep_nested_json(8);
    let generator = GeneratorFactory::create_generator("rust").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("DeepNested")
        .with_comments(true);

    let result = generator.generate(&deep_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should contain multiple struct definitions with serde
    assert!(code.contains("#[derive(Serialize, Deserialize"));
    assert!(code.contains("pub struct DeepNested"));
    assert!(code.contains("pub struct Nested"));
    
    // Should use proper Rust naming conventions
    assert!(code.contains("pub level: i64"));
    assert!(code.contains("pub nested: Nested"));
}

#[test]
fn test_deep_nesting_typescript_generation() {
    let deep_json = test_data::create_deep_nested_json(6);
    let generator = GeneratorFactory::create_generator("typescript").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("DeepNested")
        .with_comments(true);

    let result = generator.generate(&deep_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should contain multiple interface definitions
    assert!(code.contains("export interface DeepNested"));
    assert!(code.contains("export interface Nested"));
    
    // Should use proper TypeScript syntax
    assert!(code.contains("level: number;"));
    assert!(code.contains("nested: Nested;"));
}

#[test]
fn test_deep_nesting_python_generation() {
    let deep_json = test_data::create_deep_nested_json(7);
    let generator = GeneratorFactory::create_generator("python").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("DeepNested")
        .with_comments(true);

    let result = generator.generate(&deep_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should contain multiple dataclass definitions
    assert!(code.contains("@dataclass"));
    assert!(code.contains("class DeepNested:"));
    assert!(code.contains("class Nested:"));
    
    // Should use proper Python type hints
    assert!(code.contains("level: int"));
    assert!(code.contains("nested: Nested"));
}

#[test]
fn test_branched_nesting_handling() {
    let branched_json = test_data::create_branched_nested_json(5, 3);
    let generator = GeneratorFactory::create_generator("go").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("BranchedStruct")
        .with_comments(true);

    let result = generator.generate(&branched_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should handle multiple branches correctly
    assert!(code.contains("type BranchedStruct struct"));
    assert!(code.contains("Branch0"));
    assert!(code.contains("Branch1"));
    assert!(code.contains("Branch2"));
    
    // Should have proper naming for sub-branches
    assert!(code.contains("SubBranch0"));
    assert!(code.contains("SubBranch1"));
}

#[test]
fn test_nested_arrays_handling() {
    let nested_arrays_json = test_data::create_nested_arrays_json(4);
    let generator = GeneratorFactory::create_generator("rust").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("NestedArrays")
        .with_comments(true);

    let result = generator.generate(&nested_arrays_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should handle nested arrays correctly
    assert!(code.contains("pub struct NestedArrays"));
    assert!(code.contains("pub items: Vec<Items>"));
    assert!(code.contains("pub struct Items"));
    assert!(code.contains("pub nested: "));
}

#[test]
fn test_excessive_nesting_error_handling() {
    // Create extremely deep nesting that should trigger error handling
    let excessive_json = test_data::create_deep_nested_json(60);
    let generator = GeneratorFactory::create_generator("go").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("ExcessiveNesting");

    let result = generator.generate(&excessive_json, &options);
    
    // Should return an error for excessive nesting
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("too deeply nested"));
}

#[test]
fn test_naming_strategy_for_deep_nesting() {
    let deep_json = json!({
        "user": {
            "profile": {
                "personal": {
                    "contact": {
                        "address": {
                            "street": "123 Main St",
                            "city": "Anytown"
                        }
                    }
                }
            }
        }
    });

    let generator = GeneratorFactory::create_generator("go").unwrap();
    let options = GenerationOptions::default()
        .with_struct_name("User")
        .with_comments(true);

    let result = generator.generate(&deep_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should generate reasonable names for deeply nested structures
    assert!(code.contains("type User struct"));
    assert!(code.contains("type UserProfile struct") || code.contains("type Profile struct"));
    assert!(code.contains("Address") || code.contains("ContactAddress"));
    
    // Names should not be excessively long
    let lines: Vec<&str> = code.lines().collect();
    for line in lines {
        if line.contains("type ") && line.contains(" struct") {
            let type_name = line.split_whitespace().nth(1).unwrap_or("");
            assert!(type_name.len() < 50, "Type name too long: {}", type_name);
        }
    }
}

#[test]
fn test_converter_depth_tracking() {
    let mut converter = JsonToIrConverter::new("go");
    
    // Test initial state
    assert_eq!(converter.current_depth(), 0);
    assert_eq!(converter.max_depth(), 20);
    
    // Test depth setting
    converter.set_max_depth(30);
    assert_eq!(converter.max_depth(), 30);
    
    // Test with custom max depth
    let custom_converter = JsonToIrConverter::with_max_depth("rust", 15);
    assert_eq!(custom_converter.max_depth(), 15);
}

#[test]
fn test_performance_with_deep_nesting() {
    use std::time::Instant;
    
    let deep_json = test_data::create_deep_nested_json(20);
    let generator = GeneratorFactory::create_generator("typescript").unwrap();
    
    let options = GenerationOptions::default()
        .with_struct_name("PerformanceTest");

    let start = Instant::now();
    let result = generator.generate(&deep_json, &options);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // Should complete within reasonable time (adjust threshold as needed)
    assert!(duration.as_secs() < 5, "Generation took too long: {:?}", duration);
    
    let code = result.unwrap();
    assert!(!code.is_empty());
    assert!(code.len() > 100); // Should generate substantial code
}

#[test]
fn test_circular_reference_detection() {
    // This test ensures we don't get into infinite loops with circular-like structures
    // Note: JSON itself cannot have true circular references, but we can have
    // structures that might cause issues with naive implementations
    
    let complex_json = json!({
        "a": {
            "b": {
                "c": {
                    "back_to_a_like": {
                        "similar_to_a": "value"
                    }
                }
            }
        },
        "similar_structure": {
            "b": {
                "c": {
                    "different_ending": "value"
                }
            }
        }
    });

    let generator = GeneratorFactory::create_generator("rust").unwrap();
    let options = GenerationOptions::default()
        .with_struct_name("ComplexStruct");

    let result = generator.generate(&complex_json, &options);
    assert!(result.is_ok());

    let code = result.unwrap();
    
    // Should generate unique struct names even for similar structures
    assert!(code.contains("pub struct ComplexStruct"));
    assert!(code.contains("pub struct A"));
    assert!(code.contains("pub struct SimilarStructure"));
    
    // Should not have duplicate struct definitions
    let struct_count = code.matches("pub struct").count();
    let unique_structs: std::collections::HashSet<&str> = code
        .lines()
        .filter(|line| line.contains("pub struct"))
        .map(|line| line.split_whitespace().nth(2).unwrap_or(""))
        .collect();
    
    assert_eq!(struct_count, unique_structs.len(), "Duplicate struct definitions found");
}