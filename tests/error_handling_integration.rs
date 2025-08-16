use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test error handling for invalid format
#[test]
fn test_invalid_format_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.json");
    fs::write(&input_file, r#"{"name": "test"}"#).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "invalid"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unsupported format"));
    assert!(stderr.contains("Supported formats:"));
}

/// Test error handling for missing input file
#[test]
fn test_missing_input_file_error() {
    let output = Command::new("cargo")
        .args(&["run", "--", "nonexistent.json", "--format", "go"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found"));
}

/// Test error handling for invalid JSON
#[test]
fn test_invalid_json_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("invalid.json");
    fs::write(&input_file, r#"{"name": "test", invalid}"#).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "go"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error parsing JSON"));
    assert!(stderr.contains("valid syntax"));
}

/// Test error handling for empty JSON file
#[test]
fn test_empty_json_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("empty.json");
    fs::write(&input_file, "").unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "go"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty") || stderr.contains("whitespace"));
}

/// Test successful code generation with progress indication
#[test]
fn test_successful_code_generation_with_progress() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.json");
    
    // Create a moderately complex JSON to test progress indication
    let json_content = r#"{
        "user": {
            "id": 123,
            "name": "John Doe",
            "email": "john@example.com",
            "profile": {
                "age": 30,
                "location": "New York",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            }
        },
        "posts": [
            {
                "id": 1,
                "title": "First Post",
                "content": "Hello World",
                "tags": ["intro", "hello"]
            }
        ]
    }"#;
    
    fs::write(&input_file, json_content).unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "go"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check for progress indicators
    assert!(stdout.contains("Reading JSON file"));
    assert!(stdout.contains("Parsing JSON content"));
    assert!(stdout.contains("Creating go code generator"));
    assert!(stdout.contains("Generating Go code"));
    assert!(stdout.contains("Writing Go code"));
    assert!(stdout.contains("Successfully generated"));
    assert!(stdout.contains("Usage hints"));
}

/// Test struct name validation
#[test]
fn test_struct_name_validation() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.json");
    fs::write(&input_file, r#"{"name": "test"}"#).unwrap();

    // Test with valid struct name
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "go", "--struct-name", "ValidName"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    
    // Check that the generated file contains the custom struct name
    let output_file = temp_dir.path().join("test.go");
    if output_file.exists() {
        let content = fs::read_to_string(&output_file).unwrap();
        assert!(content.contains("type ValidName struct"));
    }
}

/// Test backward compatibility with schema generation
#[test]
fn test_backward_compatibility_schema() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.json");
    fs::write(&input_file, r#"{"name": "test", "age": 25}"#).unwrap();

    // Test without format (should default to schema)
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generating JSON Schema"));
    assert!(stdout.contains("schema file"));
    
    // Check that schema file was created
    let schema_file = temp_dir.path().join("test.schema.json");
    if schema_file.exists() {
        let content = fs::read_to_string(&schema_file).unwrap();
        assert!(content.contains("$schema"));
        assert!(content.contains("properties"));
    }
}

/// Test format-specific usage hints
#[test]
fn test_format_specific_usage_hints() {
    let temp_dir = TempDir::new().unwrap();
    let input_file = temp_dir.path().join("test.json");
    fs::write(&input_file, r#"{"name": "test"}"#).unwrap();

    // Test Go hints
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "go"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage hints for Go"));
    assert!(stdout.contains("json.Unmarshal"));

    // Test Rust hints
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "rust"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage hints for Rust"));
    assert!(stdout.contains("serde"));

    // Test TypeScript hints
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "typescript"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage hints for TypeScript"));
    assert!(stdout.contains("JSON.parse"));

    // Test Python hints
    let output = Command::new("cargo")
        .args(&["run", "--", input_file.to_str().unwrap(), "--format", "python"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage hints for Python"));
    assert!(stdout.contains("json.loads"));
}