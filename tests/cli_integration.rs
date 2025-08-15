use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test basic program execution with help flag
#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("generates JSON Schema files from JSON input files"))
        .stdout(predicate::str::contains("--input"))
        .stdout(predicate::str::contains("--output"));
}

/// Test short help flag
#[test]
fn test_help_flag_short() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate JSON Schema from JSON files"));
}

/// Test version flag
#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("j2s"));
}

/// Test short version flag
#[test]
fn test_version_flag_short() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("j2s"));
}

/// Test error when no input file is provided
#[test]
fn test_no_input_file_error() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No input file specified"));
}

/// Test error when input file doesn't exist
#[test]
fn test_nonexistent_input_file() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("nonexistent.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input file"));
}

/// Test simple JSON object processing with positional argument
#[test]
fn test_simple_json_object_positional() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("simple.json");
    let output_path = temp_dir.path().join("simple.schema.json");
    
    // Create simple JSON file
    fs::write(&input_path, r#"{"name": "John", "age": 30}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Reading JSON file"))
        .stdout(predicate::str::contains("Parsing JSON content"))
        .stdout(predicate::str::contains("Generating JSON Schema"))
        .stdout(predicate::str::contains("Successfully generated schema file"));
    
    // Verify output file was created
    assert!(output_path.exists());
    
    // Verify schema content
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"].is_object());
    assert!(schema["properties"]["name"].is_object());
    assert!(schema["properties"]["age"].is_object());
}

/// Test simple JSON array processing
#[test]
fn test_simple_json_array() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("array.json");
    let output_path = temp_dir.path().join("array.schema.json");
    
    // Create JSON array file
    fs::write(&input_path, r#"[1, 2, 3, 4, 5]"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    // Verify output file was created
    assert!(output_path.exists());
    
    // Verify schema content
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "array");
    assert!(schema["items"].is_object());
}

/// Test complex nested JSON structure
#[test]
fn test_complex_nested_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("complex.json");
    let output_path = temp_dir.path().join("complex.schema.json");
    
    // Create complex nested JSON
    let complex_json = r#"{
        "user": {
            "id": 123,
            "name": "John Doe",
            "email": "john@example.com",
            "active": true,
            "metadata": null,
            "scores": [95, 87, 92],
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "zipcode": "12345"
            }
        },
        "timestamp": "2023-01-01T00:00:00Z",
        "tags": ["user", "premium", "verified"]
    }"#;
    
    fs::write(&input_path, complex_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    // Verify output file was created
    assert!(output_path.exists());
    
    // Verify schema structure
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["user"]["properties"]["address"]["properties"].is_object());
    assert!(schema["properties"]["tags"]["items"].is_object());
}

/// Test with --input flag
#[test]
fn test_input_flag() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input_flag.json");
    let output_path = temp_dir.path().join("input_flag.schema.json");
    
    fs::write(&input_path, r#"{"test": true}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--input").arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test with short -i flag
#[test]
fn test_input_flag_short() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input_short.json");
    let output_path = temp_dir.path().join("input_short.schema.json");
    
    fs::write(&input_path, r#"{"test": true}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("-i").arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test with custom output path using --output flag
#[test]
fn test_custom_output_path() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("custom_output.json");
    
    fs::write(&input_path, r#"{"custom": "output"}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .arg("--output").arg(&output_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    // Verify the custom output path was used
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    assert_eq!(schema["type"], "object");
}

/// Test with short -o flag for output
#[test]
fn test_custom_output_path_short() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("short_output.json");
    
    fs::write(&input_path, r#"{"short": "flag"}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .arg("-o").arg(&output_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test combination of --input and --output flags
#[test]
fn test_input_and_output_flags() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("source.json");
    let output_path = temp_dir.path().join("destination.schema.json");
    
    fs::write(&input_path, r#"{"combined": "flags"}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--input").arg(&input_path)
        .arg("--output").arg(&output_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test combination of short flags -i and -o
#[test]
fn test_short_flags_combination() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("source.json");
    let output_path = temp_dir.path().join("dest.json");
    
    fs::write(&input_path, r#"{"short": "flags"}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("-i").arg(&input_path)
        .arg("-o").arg(&output_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test invalid JSON input
#[test]
fn test_invalid_json_input() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("invalid.json");
    
    // Create invalid JSON file
    fs::write(&input_path, r#"{"invalid": json, missing quotes}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing JSON"));
}

/// Test empty JSON object
#[test]
fn test_empty_json_object() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("empty.json");
    let output_path = temp_dir.path().join("empty.schema.json");
    
    fs::write(&input_path, r#"{}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    assert_eq!(schema["type"], "object");
}

/// Test empty JSON array
#[test]
fn test_empty_json_array() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("empty_array.json");
    let output_path = temp_dir.path().join("empty_array.schema.json");
    
    fs::write(&input_path, r#"[]"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    assert_eq!(schema["type"], "array");
}

/// Test JSON with all primitive types
#[test]
fn test_all_primitive_types() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("primitives.json");
    let output_path = temp_dir.path().join("primitives.schema.json");
    
    let json_with_primitives = r#"{
        "string_field": "hello",
        "number_field": 42,
        "float_field": 3.14,
        "boolean_field": true,
        "null_field": null
    }"#;
    
    fs::write(&input_path, json_with_primitives).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["string_field"].is_object());
    assert!(schema["properties"]["number_field"].is_object());
    assert!(schema["properties"]["boolean_field"].is_object());
    assert!(schema["properties"]["null_field"].is_object());
}

/// Test deeply nested JSON structure
#[test]
fn test_deeply_nested_structure() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("deep.json");
    let output_path = temp_dir.path().join("deep.schema.json");
    
    let deep_json = r#"{
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "deep_value": "found"
                        }
                    }
                }
            }
        }
    }"#;
    
    fs::write(&input_path, deep_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    // Verify deep nesting is preserved in schema
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["level1"]["properties"]["level2"]["properties"]["level3"]["properties"]["level4"]["properties"]["level5"]["properties"]["deep_value"].is_object());
}

/// Test array with mixed types
#[test]
fn test_mixed_type_array() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("mixed_array.json");
    let output_path = temp_dir.path().join("mixed_array.schema.json");
    
    let mixed_array_json = r#"[
        "string",
        42,
        true,
        null,
        {"nested": "object"},
        [1, 2, 3]
    ]"#;
    
    fs::write(&input_path, mixed_array_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "array");
    assert!(schema["items"].is_object());
}

/// Test array of objects
#[test]
fn test_array_of_objects() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("object_array.json");
    let output_path = temp_dir.path().join("object_array.schema.json");
    
    let object_array_json = r#"[
        {"id": 1, "name": "Alice"},
        {"id": 2, "name": "Bob"},
        {"id": 3, "name": "Charlie"}
    ]"#;
    
    fs::write(&input_path, object_array_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "array");
    assert!(schema["items"]["properties"]["id"].is_object());
    assert!(schema["items"]["properties"]["name"].is_object());
}

/// Test large JSON file processing
#[test]
fn test_large_json_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("large.json");
    let output_path = temp_dir.path().join("large.schema.json");
    
    // Create a large JSON structure
    let mut large_json = String::from("{\"items\": [");
    for i in 0..100 {
        if i > 0 {
            large_json.push(',');
        }
        large_json.push_str(&format!(
            r#"{{"id": {}, "name": "Item {}", "active": {}, "score": {}}}"#,
            i, i, i % 2 == 0, i as f64 * 1.5
        ));
    }
    large_json.push_str("]}");
    
    fs::write(&input_path, large_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["items"]["items"]["properties"]["id"].is_object());
}

/// Test file permission error (read-only directory)
#[test]
fn test_output_permission_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    
    // Create a subdirectory and make it read-only
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();
    
    // Set directory to read-only (this might not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }
    
    let output_path = readonly_dir.join("output.json");
    
    fs::write(&input_path, r#"{"test": "permission"}"#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .arg("--output").arg(&output_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error writing schema file"));
}

/// Test with JSON containing Unicode characters
#[test]
fn test_unicode_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("unicode.json");
    let output_path = temp_dir.path().join("unicode.schema.json");
    
    let unicode_json = r#"{
        "name": "José María",
        "emoji": "🚀",
        "chinese": "你好",
        "arabic": "مرحبا",
        "special": "café naïve résumé"
    }"#;
    
    fs::write(&input_path, unicode_json).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["name"].is_object());
    assert!(schema["properties"]["emoji"].is_object());
}

/// Test JSON with very long strings
#[test]
fn test_long_strings() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("long_strings.json");
    let output_path = temp_dir.path().join("long_strings.schema.json");
    
    let long_string = "a".repeat(10000);
    let json_with_long_string = format!(r#"{{"long_field": "{}"}}"#, long_string);
    
    fs::write(&input_path, json_with_long_string).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
}

/// Test edge case: JSON with only null value
#[test]
fn test_null_only_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("null.json");
    let output_path = temp_dir.path().join("null.schema.json");
    
    fs::write(&input_path, "null").unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    // Should handle null as a valid JSON value
    assert!(schema.is_object());
}

/// Test edge case: JSON with only boolean value
#[test]
fn test_boolean_only_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("boolean.json");
    let output_path = temp_dir.path().join("boolean.schema.json");
    
    fs::write(&input_path, "true").unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "boolean");
}

/// Test edge case: JSON with only number value
#[test]
fn test_number_only_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("number.json");
    let output_path = temp_dir.path().join("number.schema.json");
    
    fs::write(&input_path, "42").unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    // The schema generator might return "integer" for whole numbers
    assert!(schema["type"] == "number" || schema["type"] == "integer");
}

/// Test edge case: JSON with only string value
#[test]
fn test_string_only_json() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("string.json");
    let output_path = temp_dir.path().join("string.schema.json");
    
    fs::write(&input_path, r#""hello world""#).unwrap();
    
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .success();
    
    assert!(output_path.exists());
    
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();
    
    assert_eq!(schema["type"], "string");
}