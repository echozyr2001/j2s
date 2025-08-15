use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test handling of completely empty file
#[test]
fn test_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("empty.json");

    // Create completely empty file
    fs::write(&input_path, "").unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input file"));
}

/// Test handling of file with only whitespace
#[test]
fn test_whitespace_only_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("whitespace.json");

    // Create file with only whitespace
    fs::write(&input_path, "   \n\t  \r\n  ").unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input file"));
}

/// Test handling of malformed JSON - missing closing brace
#[test]
fn test_malformed_json_missing_brace() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("malformed.json");

    fs::write(&input_path, r#"{"name": "test", "value": 123"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing JSON"));
}

/// Test handling of malformed JSON - trailing comma
#[test]
fn test_malformed_json_trailing_comma() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("trailing_comma.json");

    fs::write(&input_path, r#"{"name": "test", "value": 123,}"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing JSON"));
}

/// Test handling of malformed JSON - unquoted keys
#[test]
fn test_malformed_json_unquoted_keys() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("unquoted.json");

    fs::write(&input_path, r#"{name: "test", value: 123}"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing JSON"));
}

/// Test handling of malformed JSON - single quotes instead of double
#[test]
fn test_malformed_json_single_quotes() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("single_quotes.json");

    fs::write(&input_path, r#"{'name': 'test', 'value': 123}"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing JSON"));
}

/// Test handling of binary file input
#[test]
fn test_binary_file_input() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("binary.json");

    // Create a binary file with non-UTF8 content
    let binary_data = vec![0xFF, 0xFE, 0x00, 0x01, 0x80, 0x90];
    fs::write(&input_path, binary_data).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

/// Test handling of very large file path
#[test]
fn test_very_long_file_path() {
    let temp_dir = TempDir::new().unwrap();

    // Create a very long filename (but within filesystem limits)
    let long_name = "a".repeat(100);
    let input_path = temp_dir.path().join(format!("{long_name}.json"));

    fs::write(&input_path, r#"{"test": "long_path"}"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();
}

/// Test handling of directory instead of file
#[test]
fn test_directory_as_input() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().join("subdir");
    fs::create_dir(&dir_path).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&dir_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error reading input file"));
}

/// Test handling of special characters in file path
#[test]
fn test_special_chars_in_path() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir
        .path()
        .join("test file with spaces & symbols!.json");

    fs::write(&input_path, r#"{"special": "chars"}"#).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();
}

/// Test handling of output to existing file (should overwrite)
#[test]
fn test_overwrite_existing_output() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("existing.json");

    // Create input file
    fs::write(&input_path, r#"{"new": "content"}"#).unwrap();

    // Create existing output file with different content
    fs::write(&output_path, "old content").unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    // Verify the file was overwritten with new schema content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("schema"));
    assert!(!content.contains("old content"));
}

/// Test handling of output to read-only file
#[test]
fn test_readonly_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("readonly.json");

    fs::write(&input_path, r#"{"test": "readonly"}"#).unwrap();

    // Create read-only output file
    fs::write(&output_path, "readonly content").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&output_path).unwrap().permissions();
        perms.set_mode(0o444); // read-only
        fs::set_permissions(&output_path, perms).unwrap();
    }

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error writing schema file"));
}

/// Test handling of invalid command line arguments
#[test]
fn test_invalid_arguments() {
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg("--invalid-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

/// Test handling of conflicting arguments
#[test]
fn test_conflicting_arguments() {
    let temp_dir = TempDir::new().unwrap();
    let input1 = temp_dir.path().join("input1.json");
    let input2 = temp_dir.path().join("input2.json");

    fs::write(&input1, r#"{"file": 1}"#).unwrap();
    fs::write(&input2, r#"{"file": 2}"#).unwrap();

    // Test with both positional and --input flag (--input should take precedence)
    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input1) // positional
        .arg("--input")
        .arg(&input2) // flag
        .assert()
        .success()
        .stdout(predicate::str::contains(
            input2.to_string_lossy().to_string(),
        ));
}

/// Test handling of JSON with control characters
#[test]
fn test_json_with_control_chars() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("control_chars.json");

    // JSON with escaped control characters
    let json_with_controls = r#"{"text": "line1\nline2\ttab\rcarriage"}"#;
    fs::write(&input_path, json_with_controls).unwrap();

    let mut cmd = Command::cargo_bin("json2schema").unwrap();
    cmd.arg(&input_path).assert().success();
}
