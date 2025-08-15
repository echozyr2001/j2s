use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test processing of deeply nested JSON (stress test for recursion)
#[test]
fn test_deeply_nested_json_stress() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("deep_nested.json");
    let output_path = temp_dir.path().join("deep_nested.schema.json");

    // Create deeply nested JSON (50 levels deep)
    let mut json = String::new();
    for i in 0..50 {
        json.push_str(&format!(r#"{{"level{i}": "#));
    }
    json.push_str(r#""deep_value""#);
    for _ in 0..50 {
        json.push('}');
    }

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of wide JSON object (many properties)
#[test]
fn test_wide_json_object() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("wide_object.json");
    let output_path = temp_dir.path().join("wide_object.schema.json");

    // Create JSON object with 500 properties
    let mut json = String::from("{");
    for i in 0..500 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(r#""field{i}": {i}"#));
    }
    json.push('}');

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of large array
#[test]
fn test_large_array() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("large_array.json");
    let output_path = temp_dir.path().join("large_array.schema.json");

    // Create array with 1000 elements
    let mut json = String::from("[");
    for i in 0..1000 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(r#"{{"id": {i}, "value": "item{i}"}}"#));
    }
    json.push(']');

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of JSON with many different data types
#[test]
fn test_complex_mixed_types() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("mixed_complex.json");
    let output_path = temp_dir.path().join("mixed_complex.schema.json");

    // Create complex JSON with various nested structures
    let complex_json = r#"{
        "users": [
            {
                "id": 1,
                "profile": {
                    "name": "Alice",
                    "age": 30,
                    "active": true,
                    "metadata": null,
                    "scores": [95, 87, 92],
                    "preferences": {
                        "theme": "dark",
                        "notifications": {
                            "email": true,
                            "push": false,
                            "sms": null
                        }
                    }
                },
                "posts": [
                    {
                        "id": 101,
                        "title": "First Post",
                        "content": "Hello world!",
                        "tags": ["intro", "hello"],
                        "published": true,
                        "stats": {
                            "views": 150,
                            "likes": 23,
                            "comments": [
                                {"user": "bob", "text": "Nice post!"},
                                {"user": "charlie", "text": "Welcome!"}
                            ]
                        }
                    }
                ]
            }
        ],
        "settings": {
            "version": "1.0.0",
            "features": {
                "beta": ["feature1", "feature2"],
                "stable": ["core", "auth", "api"]
            },
            "limits": {
                "max_users": 1000,
                "max_posts_per_user": 100,
                "rate_limit": 60.5
            }
        },
        "statistics": {
            "total_users": 1,
            "active_sessions": 0,
            "server_uptime": 86400.123,
            "memory_usage": null
        }
    }"#;

    fs::write(&input_path, complex_json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());

    // Verify the schema was generated correctly
    let schema_content = fs::read_to_string(&output_path).unwrap();
    let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();

    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["users"].is_object());
    assert!(schema["properties"]["settings"].is_object());
    assert!(schema["properties"]["statistics"].is_object());
}

/// Test processing of JSON with repeated patterns
#[test]
fn test_repeated_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("repeated.json");
    let output_path = temp_dir.path().join("repeated.schema.json");

    // Create JSON with repeated nested patterns
    let mut json = String::from(r#"{"data": ["#);
    for i in 0..100 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{
            "id": {i},
            "nested": {{
                "level1": {{
                    "level2": {{
                        "values": [1, 2, 3],
                        "metadata": {{
                            "created": "2023-01-01",
                            "updated": null,
                            "tags": ["tag1", "tag2"]
                        }}
                    }}
                }}
            }}
        }}"#
        ));
    }
    json.push_str("]}");

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of JSON with very long string values
#[test]
fn test_very_long_strings() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("long_strings.json");
    let output_path = temp_dir.path().join("long_strings.schema.json");

    // Create JSON with very long string values
    let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(1000);
    let json = format!(
        r#"{{
        "short_text": "hello",
        "medium_text": "{}",
        "very_long_text": "{}",
        "numbers": [1, 2, 3],
        "nested": {{
            "another_long_text": "{}"
        }}
    }}"#,
        "medium ".repeat(100),
        long_text,
        "nested ".repeat(500)
    );

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of JSON with many null values
#[test]
fn test_many_nulls() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("many_nulls.json");
    let output_path = temp_dir.path().join("many_nulls.schema.json");

    // Create JSON with many null values
    let mut json = String::from("{");
    for i in 0..200 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(r#""null_field{i}": null"#));
    }
    json.push_str(r#", "non_null": "value""#);
    json.push('}');

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}

/// Test processing of JSON with alternating array types
#[test]
fn test_alternating_array_types() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("alternating.json");
    let output_path = temp_dir.path().join("alternating.schema.json");

    // Create array with alternating types
    let mut json = String::from("[");
    for i in 0..100 {
        if i > 0 {
            json.push(',');
        }
        match i % 6 {
            0 => json.push_str(&format!("{i}")),
            1 => json.push_str(&format!(r#""string{i}""#)),
            2 => json.push_str("true"),
            3 => json.push_str("false"),
            4 => json.push_str("null"),
            5 => json.push_str(&format!(r#"{{"obj": {i}}}"#)),
            _ => unreachable!(),
        }
    }
    json.push(']');

    fs::write(&input_path, json).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();

    assert!(output_path.exists());
}
