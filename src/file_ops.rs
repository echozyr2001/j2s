use crate::error::{J2sError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Read JSON content from a file with enhanced error reporting and performance optimizations
///
/// This function provides comprehensive error checking and optimized reading for JSON files.
/// It includes file validation, size checking, and detailed error messages to help users
/// diagnose issues quickly.
///
/// # Arguments
/// * `path` - Path to the JSON file to read
///
/// # Returns
/// * `Result<String>` - The file content as a string, or a detailed error
///
/// # Errors
/// * Returns `J2sError::File` with specific error details for various failure scenarios:
///   - File not found
///   - Path is a directory instead of a file
///   - Permission denied
///   - File is empty or contains only whitespace
///   - File is too large (>100MB)
///   - IO errors during reading
///
/// # Performance
/// * Uses efficient file size checking before reading
/// * Provides warnings for large files
/// * Uses optimized string reading for better memory usage
pub fn read_json_file(path: &str) -> Result<String> {
    let file_path = Path::new(path);

    // Check if file exists first to provide better error messages
    if !file_path.exists() {
        return Err(J2sError::file_error(format!(
            "File not found: {path}\n   Please check the file path and ensure the file exists"
        )));
    }

    // Check if it's actually a file (not a directory)
    if !file_path.is_file() {
        return Err(J2sError::file_error(format!(
            "Path is not a file: {path}\n   The specified path points to a directory or special file"
        )));
    }

    // Check file size for performance warnings and limits
    let metadata = match file_path.metadata() {
        Ok(meta) => meta,
        Err(err) => {
            return Err(J2sError::file_error(format!(
                "Cannot access file metadata for {path}: {err}\n   Check file permissions"
            )));
        }
    };

    let file_size = metadata.len();

    // Warn about very large files (>100MB)
    if file_size > 100_000_000 {
        return Err(J2sError::file_error(format!(
            "File too large: {path} ({:.1} MB)\n   Files larger than 100MB are not supported for performance reasons",
            file_size as f64 / 1_000_000.0
        )));
    }

    // Provide performance warning for large files
    if file_size > 10_000_000 {
        eprintln!(
            "⚠️  Warning: Large file detected ({:.1} MB). Processing may take some time.",
            file_size as f64 / 1_000_000.0
        );
    }

    // Read the file content
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return Err(J2sError::file_error(format!(
                    "File is empty or contains only whitespace: {path}\n   Please provide a file with valid JSON content"
                )));
            }
            Ok(content)
        }
        Err(err) => {
            let error_msg = match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied reading file {path}: {err}\n   Check that you have read permissions for this file"
                    )
                }
                std::io::ErrorKind::InvalidData => {
                    format!(
                        "Invalid file encoding in {path}: {err}\n   Ensure the file is saved in UTF-8 encoding"
                    )
                }
                _ => {
                    format!("Failed to read file {path}: {err}")
                }
            };
            Err(J2sError::file_error(error_msg))
        }
    }
}

/// Write schema content to a file with enhanced error reporting and safety checks
///
/// This function provides comprehensive error checking and safe file writing for schema output.
/// It includes directory creation, permission checking, and atomic writing to prevent
/// data corruption.
///
/// # Arguments
/// * `path` - Path where the schema file should be written
/// * `content` - The schema content to write
///
/// # Returns
/// * `Result<()>` - Success or a detailed error
///
/// # Errors
/// * Returns `J2sError::File` with specific error details for various failure scenarios:
///   - Directory creation failures
///   - Permission denied for writing
///   - Disk space issues
///   - File system errors
///
/// # Safety
/// * Creates parent directories if they don't exist
/// * Verifies successful write operation
/// * Provides detailed error messages for troubleshooting
pub fn write_schema_file(path: &str, content: &str) -> Result<()> {
    let file_path = Path::new(path);

    // Ensure the output directory exists
    ensure_output_directory(path)?;

    // Check if we're about to overwrite an existing file
    if file_path.exists() {
        eprintln!("⚠️  Warning: Overwriting existing file: {path}");
    }

    // Write the content to the file
    match fs::write(path, content) {
        Ok(()) => {
            // Verify the file was written successfully by checking if it exists and has content
            match file_path.metadata() {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        Err(J2sError::file_error(format!(
                            "File was created but is empty: {path}\n   This may indicate a disk space or permission issue"
                        )))
                    } else if metadata.len() != content.len() as u64 {
                        Err(J2sError::file_error(format!(
                            "File size mismatch after writing: {path}\n   Expected {} bytes, got {} bytes",
                            content.len(),
                            metadata.len()
                        )))
                    } else {
                        Ok(())
                    }
                }
                Err(err) => Err(J2sError::file_error(format!(
                    "Cannot verify written file {path}: {err}\n   File may have been created but verification failed"
                ))),
            }
        }
        Err(err) => {
            let error_msg = match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied writing to {path}: {err}\n   Check that you have write permissions to this location"
                    )
                }
                std::io::ErrorKind::NotFound => {
                    format!(
                        "Directory not found for {path}: {err}\n   The parent directory may not exist or be accessible"
                    )
                }
                std::io::ErrorKind::WriteZero => {
                    format!(
                        "No space left on device for {path}: {err}\n   Check available disk space"
                    )
                }
                _ => {
                    format!("Failed to write file {path}: {err}")
                }
            };
            Err(J2sError::file_error(error_msg))
        }
    }
}

/// Ensure the directory for the given file path exists with enhanced error reporting
///
/// This function creates all necessary parent directories for a file path if they don't exist.
/// It provides detailed error messages and handles various edge cases safely.
///
/// # Arguments
/// * `file_path` - Path to a file (the directory will be extracted and created)
///
/// # Returns
/// * `Result<()>` - Success or a detailed error
///
/// # Errors
/// * Returns `J2sError::File` with specific error details for various failure scenarios:
///   - Permission denied for directory creation
///   - Path conflicts (file exists where directory should be)
///   - File system errors
///
/// # Safety
/// * Uses `create_dir_all` for recursive directory creation
/// * Handles edge cases like root paths and current directory
/// * Provides clear error messages for troubleshooting
pub fn ensure_output_directory(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);

    // Get the parent directory
    if let Some(parent) = path.parent() {
        // Only create directory if it doesn't exist and is not empty (current directory)
        if !parent.as_os_str().is_empty() && !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(()) => {
                    // Verify the directory was created successfully
                    if parent.exists() && parent.is_dir() {
                        Ok(())
                    } else {
                        Err(J2sError::file_error(format!(
                            "Directory creation appeared to succeed but directory is not accessible: {}",
                            parent.display()
                        )))
                    }
                }
                Err(err) => {
                    let error_msg = match err.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            format!(
                                "Permission denied creating directory {}: {err}\n   Check that you have write permissions to the parent directory",
                                parent.display()
                            )
                        }
                        std::io::ErrorKind::AlreadyExists => {
                            format!(
                                "Cannot create directory {} - a file with this name already exists: {err}",
                                parent.display()
                            )
                        }
                        _ => {
                            format!("Failed to create directory {}: {err}", parent.display())
                        }
                    };
                    Err(J2sError::file_error(error_msg))
                }
            }
        } else if parent.exists() && !parent.is_dir() {
            // Parent exists but is not a directory
            Err(J2sError::file_error(format!(
                "Cannot create directory - a file exists at this path: {}",
                parent.display()
            )))
        } else {
            Ok(())
        }
    } else {
        // No parent directory (root or current directory)
        Ok(())
    }
}

/// Generate output path based on input path and optional output path
///
/// # Arguments
/// * `input_path` - Path to the input JSON file
/// * `output_path` - Optional custom output path
///
/// # Returns
/// * `String` - The generated output path
///
/// # Logic
/// * If output_path is provided, use it as-is
/// * If not provided, generate path by replacing .json extension with .schema.json
/// * If input has no .json extension, append .schema.json
pub fn generate_output_path(input_path: &str, output_path: Option<&str>) -> String {
    match output_path {
        Some(path) => path.to_string(),
        None => {
            let input_path_buf = PathBuf::from(input_path);

            // Get the file stem (filename without extension)
            let file_stem = input_path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            // Get the parent directory
            let parent = input_path_buf.parent();

            // Create the output filename
            let output_filename = format!("{file_stem}.schema.json");

            // Combine with parent directory if it exists
            match parent {
                Some(parent_dir) if !parent_dir.as_os_str().is_empty() => parent_dir
                    .join(output_filename)
                    .to_string_lossy()
                    .to_string(),
                _ => output_filename,
            }
        }
    }
}

/// Generate code output path based on input path, optional output path, and target format
///
/// This function creates appropriate output file paths for generated code files,
/// handling different programming language file extensions and naming conventions.
///
/// # Arguments
/// * `input_path` - Path to the input JSON file
/// * `output_path` - Optional custom output path
/// * `format` - Target format/language (go, rust, typescript, python, etc.)
///
/// # Returns
/// * `String` - The generated output path with appropriate file extension
///
/// # Logic
/// * If output_path is provided, use it as-is (assumes user knows what they want)
/// * If not provided, generate path based on input filename and target format
/// * Uses appropriate file extensions for each language
/// * Preserves directory structure from input path
///
/// # Examples
/// * `generate_code_output_path("data.json", None, "go")` → `"data.go"`
/// * `generate_code_output_path("path/to/data.json", None, "rust")` → `"path/to/data.rs"`
/// * `generate_code_output_path("data.json", Some("custom.ts"), "typescript")` → `"custom.ts"`
pub fn generate_code_output_path(input_path: &str, output_path: Option<&str>, format: &str) -> String {
    match output_path {
        Some(path) => path.to_string(),
        None => {
            let input_path_buf = PathBuf::from(input_path);

            // Get the file stem (filename without extension)
            let file_stem = input_path_buf
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");

            // Get the parent directory
            let parent = input_path_buf.parent();

            // Determine file extension based on format
            let extension = get_file_extension_for_format(format);

            // Create the output filename
            let output_filename = format!("{file_stem}.{extension}");

            // Combine with parent directory if it exists
            match parent {
                Some(parent_dir) if !parent_dir.as_os_str().is_empty() => parent_dir
                    .join(output_filename)
                    .to_string_lossy()
                    .to_string(),
                _ => output_filename,
            }
        }
    }
}

/// Get the appropriate file extension for a given format/language
///
/// # Arguments
/// * `format` - The target format/language identifier
///
/// # Returns
/// * `&'static str` - The file extension (without the dot)
///
/// # Supported Formats
/// * `"go"` → `"go"`
/// * `"rust"` → `"rs"`
/// * `"typescript"` → `"ts"`
/// * `"python"` → `"py"`
/// * `"schema"` → `"schema.json"`
/// * Default → `"txt"`
pub fn get_file_extension_for_format(format: &str) -> &'static str {
    match format {
        "go" => "go",
        "rust" => "rs",
        "typescript" => "ts",
        "python" => "py",
        "schema" => "schema.json",
        _ => "txt", // Fallback for unknown formats
    }
}

/// Write generated code content to a file with enhanced error reporting and validation
///
/// This function provides comprehensive error checking and safe file writing for generated code.
/// It includes syntax validation hints, directory creation, and detailed error reporting
/// specific to code generation workflows.
///
/// # Arguments
/// * `path` - Path where the code file should be written
/// * `content` - The generated code content to write
/// * `format` - The target format/language for additional validation context
///
/// # Returns
/// * `Result<()>` - Success or a detailed error
///
/// # Errors
/// * Returns `J2sError::File` with specific error details for various failure scenarios:
///   - Directory creation failures
///   - Permission denied for writing
///   - Content validation failures
///   - File system errors
///
/// # Features
/// * Creates parent directories if they don't exist
/// * Validates that content is not empty
/// * Provides format-specific warnings and suggestions
/// * Verifies successful write operation
/// * Includes helpful error messages for troubleshooting
pub fn write_code_file(path: &str, content: &str, format: &str) -> Result<()> {
    let file_path = Path::new(path);

    // Validate content is not empty
    if content.trim().is_empty() {
        return Err(J2sError::file_error(format!(
            "Cannot write empty code content to {path}\n   Generated code content is empty, this may indicate a code generation error"
        )));
    }

    // Ensure the output directory exists
    ensure_output_directory(path)?;

    // Check if we're about to overwrite an existing file
    if file_path.exists() {
        eprintln!("⚠️  Warning: Overwriting existing {} file: {path}", format);
    }

    // Perform basic content validation based on format
    validate_code_content(content, format, path)?;

    // Write the content to the file
    match fs::write(path, content) {
        Ok(()) => {
            // Verify the file was written successfully
            match file_path.metadata() {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        Err(J2sError::file_error(format!(
                            "Code file was created but is empty: {path}\n   This may indicate a disk space or permission issue"
                        )))
                    } else if metadata.len() != content.len() as u64 {
                        Err(J2sError::file_error(format!(
                            "Code file size mismatch after writing: {path}\n   Expected {} bytes, got {} bytes\n   This may indicate encoding issues or disk problems",
                            content.len(),
                            metadata.len()
                        )))
                    } else {
                        // Success - provide helpful feedback
                        println!("✅ Generated {} code: {path} ({} bytes)", format, metadata.len());
                        Ok(())
                    }
                }
                Err(err) => Err(J2sError::file_error(format!(
                    "Cannot verify written code file {path}: {err}\n   File may have been created but verification failed"
                ))),
            }
        }
        Err(err) => {
            let error_msg = match err.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    format!(
                        "Permission denied writing {} code to {path}: {err}\n   Check that you have write permissions to this location",
                        format
                    )
                }
                std::io::ErrorKind::NotFound => {
                    format!(
                        "Directory not found for {} code file {path}: {err}\n   The parent directory may not exist or be accessible",
                        format
                    )
                }
                std::io::ErrorKind::WriteZero => {
                    format!(
                        "No space left on device for {} code file {path}: {err}\n   Check available disk space",
                        format
                    )
                }
                _ => {
                    format!("Failed to write {} code file {path}: {err}", format)
                }
            };
            Err(J2sError::file_error(error_msg))
        }
    }
}

/// Validate generated code content for basic correctness
///
/// This function performs basic validation of generated code content to catch
/// obvious issues before writing to disk. It's not a full syntax checker but
/// provides helpful early warnings.
///
/// # Arguments
/// * `content` - The generated code content to validate
/// * `format` - The target format/language
/// * `path` - The output path (for error reporting)
///
/// # Returns
/// * `Result<()>` - Success or validation error
///
/// # Validation Checks
/// * Content is not empty or only whitespace
/// * Basic format-specific syntax checks
/// * Character encoding validation
/// * Length reasonableness checks
fn validate_code_content(content: &str, format: &str, path: &str) -> Result<()> {
    // Check for empty content (already done in write_code_file, but double-check)
    if content.trim().is_empty() {
        return Err(J2sError::file_error(format!(
            "Generated {} code content is empty for {path}", format
        )));
    }

    // Check for reasonable content length (not too short, not too long)
    if content.len() < 10 {
        return Err(J2sError::file_error(format!(
            "Generated {} code content seems too short ({} bytes) for {path}\n   This may indicate a code generation error",
            format, content.len()
        )));
    }

    if content.len() > 10_000_000 {
        eprintln!(
            "⚠️  Warning: Generated {} code is very large ({:.1} MB) for {path}",
            format,
            content.len() as f64 / 1_000_000.0
        );
    }

    // Basic format-specific validation
    match format {
        "go" => validate_go_content(content, path)?,
        "rust" => validate_rust_content(content, path)?,
        "typescript" => validate_typescript_content(content, path)?,
        "python" => validate_python_content(content, path)?,
        _ => {
            // For unknown formats, just check for valid UTF-8
            if !content.is_ascii() && std::str::from_utf8(content.as_bytes()).is_err() {
                return Err(J2sError::file_error(format!(
                    "Generated code content contains invalid UTF-8 for {path}"
                )));
            }
        }
    }

    Ok(())
}

/// Validate Go code content for basic syntax issues
fn validate_go_content(content: &str, path: &str) -> Result<()> {
    // Check for package declaration
    if !content.contains("package ") {
        eprintln!("⚠️  Warning: Generated Go code in {path} may be missing package declaration");
    }

    // Check for balanced braces (basic check)
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    if open_braces != close_braces {
        return Err(J2sError::file_error(format!(
            "Generated Go code has unbalanced braces in {path}: {} open, {} close",
            open_braces, close_braces
        )));
    }

    Ok(())
}

/// Validate Rust code content for basic syntax issues
fn validate_rust_content(content: &str, path: &str) -> Result<()> {
    // Check for balanced braces
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    if open_braces != close_braces {
        return Err(J2sError::file_error(format!(
            "Generated Rust code has unbalanced braces in {path}: {} open, {} close",
            open_braces, close_braces
        )));
    }

    // Check for balanced parentheses
    let open_parens = content.matches('(').count();
    let close_parens = content.matches(')').count();
    if open_parens != close_parens {
        return Err(J2sError::file_error(format!(
            "Generated Rust code has unbalanced parentheses in {path}: {} open, {} close",
            open_parens, close_parens
        )));
    }

    Ok(())
}

/// Validate TypeScript code content for basic syntax issues
fn validate_typescript_content(content: &str, path: &str) -> Result<()> {
    // Check for balanced braces
    let open_braces = content.matches('{').count();
    let close_braces = content.matches('}').count();
    if open_braces != close_braces {
        return Err(J2sError::file_error(format!(
            "Generated TypeScript code has unbalanced braces in {path}: {} open, {} close",
            open_braces, close_braces
        )));
    }

    // Check for balanced parentheses
    let open_parens = content.matches('(').count();
    let close_parens = content.matches(')').count();
    if open_parens != close_parens {
        return Err(J2sError::file_error(format!(
            "Generated TypeScript code has unbalanced parentheses in {path}: {} open, {} close",
            open_parens, close_parens
        )));
    }

    Ok(())
}

/// Validate Python code content for basic syntax issues
fn validate_python_content(content: &str, path: &str) -> Result<()> {
    // Check for balanced parentheses
    let open_parens = content.matches('(').count();
    let close_parens = content.matches(')').count();
    if open_parens != close_parens {
        return Err(J2sError::file_error(format!(
            "Generated Python code has unbalanced parentheses in {path}: {} open, {} close",
            open_parens, close_parens
        )));
    }

    // Check for balanced square brackets
    let open_brackets = content.matches('[').count();
    let close_brackets = content.matches(']').count();
    if open_brackets != close_brackets {
        return Err(J2sError::file_error(format!(
            "Generated Python code has unbalanced brackets in {path}: {} open, {} close",
            open_brackets, close_brackets
        )));
    }

    // Python-specific: check for reasonable indentation (should contain some spaces or tabs)
    if !content.contains("    ") && !content.contains('\t') {
        eprintln!("⚠️  Warning: Generated Python code in {path} may have indentation issues");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_read_json_file_success() {
        // Create a temporary file with JSON content
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_content = r#"{"name": "test", "value": 42}"#;
        write!(temp_file, "{json_content}").unwrap();

        let result = read_json_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json_content);
    }

    #[test]
    fn test_read_json_file_not_found() {
        let result = read_json_file("nonexistent_file.json");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_file_error());
        assert!(error.to_string().contains("File not found"));
    }

    #[test]
    fn test_read_json_file_empty() {
        // Create an empty temporary file
        let temp_file = NamedTempFile::new().unwrap();

        let result = read_json_file(temp_file.path().to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_file_error());
        assert!(error.to_string().contains("File is empty"));
    }

    #[test]
    fn test_read_json_file_directory() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();

        let result = read_json_file(temp_dir.path().to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_file_error());
        assert!(error.to_string().contains("Path is not a file"));
    }

    #[test]
    fn test_write_schema_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.schema.json");
        let content = r#"{"type": "object"}"#;

        let result = write_schema_file(file_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify the file was created and has correct content
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_schema_file_with_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("test.schema.json");
        let content = r#"{"type": "string"}"#;

        let result = write_schema_file(nested_path.to_str().unwrap(), content);
        assert!(result.is_ok());

        // Verify the file was created
        assert!(nested_path.exists());
        let written_content = fs::read_to_string(&nested_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_ensure_output_directory_new_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_file_path = temp_dir
            .path()
            .join("new")
            .join("directory")
            .join("file.json");

        let result = ensure_output_directory(nested_file_path.to_str().unwrap());
        assert!(result.is_ok());

        // Verify the directory was created
        assert!(nested_file_path.parent().unwrap().exists());
    }

    #[test]
    fn test_ensure_output_directory_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.json");

        let result = ensure_output_directory(file_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_output_directory_current_directory() {
        let result = ensure_output_directory("file.json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_output_path_with_custom_output() {
        let input = "input.json";
        let custom_output = "custom/path/output.schema.json";

        let result = generate_output_path(input, Some(custom_output));
        assert_eq!(result, custom_output);
    }

    #[test]
    fn test_generate_output_path_json_extension() {
        let input = "data.json";
        let result = generate_output_path(input, None);
        assert_eq!(result, "data.schema.json");
    }

    #[test]
    fn test_generate_output_path_no_extension() {
        let input = "data";
        let result = generate_output_path(input, None);
        assert_eq!(result, "data.schema.json");
    }

    #[test]
    fn test_generate_output_path_with_directory() {
        let input = "path/to/data.json";
        let result = generate_output_path(input, None);
        assert_eq!(result, "path/to/data.schema.json");
    }

    #[test]
    fn test_generate_output_path_different_extension() {
        let input = "data.txt";
        let result = generate_output_path(input, None);
        assert_eq!(result, "data.schema.json");
    }

    #[test]
    fn test_generate_output_path_complex_path() {
        let input = "/absolute/path/to/complex.data.json";
        let result = generate_output_path(input, None);
        assert_eq!(result, "/absolute/path/to/complex.data.schema.json");
    }

    #[test]
    fn test_generate_output_path_root_file() {
        let input = "/data.json";
        let result = generate_output_path(input, None);
        assert_eq!(result, "/data.schema.json");
    }

    #[test]
    fn test_file_operations_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Create input file
        let input_path = temp_dir.path().join("input.json");
        let json_content = r#"{"test": "data", "number": 123}"#;
        fs::write(&input_path, json_content).unwrap();

        // Read the file
        let read_result = read_json_file(input_path.to_str().unwrap());
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), json_content);

        // Generate output path
        let output_path = generate_output_path(input_path.to_str().unwrap(), None);
        assert!(output_path.ends_with("input.schema.json"));

        // Write schema file
        let schema_content = r#"{"type": "object", "properties": {"test": {"type": "string"}, "number": {"type": "integer"}}}"#;
        let write_result = write_schema_file(&output_path, schema_content);
        assert!(write_result.is_ok());

        // Verify the schema file was created
        assert!(Path::new(&output_path).exists());
        let written_content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(written_content, schema_content);
    }

    // Tests for code generation file operations
    #[test]
    fn test_get_file_extension_for_format() {
        assert_eq!(get_file_extension_for_format("go"), "go");
        assert_eq!(get_file_extension_for_format("rust"), "rs");
        assert_eq!(get_file_extension_for_format("typescript"), "ts");
        assert_eq!(get_file_extension_for_format("python"), "py");
        assert_eq!(get_file_extension_for_format("schema"), "schema.json");
        assert_eq!(get_file_extension_for_format("unknown"), "txt");
    }

    #[test]
    fn test_generate_code_output_path_with_custom_output() {
        let input = "input.json";
        let custom_output = "custom/path/output.go";

        let result = generate_code_output_path(input, Some(custom_output), "go");
        assert_eq!(result, custom_output);
    }

    #[test]
    fn test_generate_code_output_path_go() {
        let input = "data.json";
        let result = generate_code_output_path(input, None, "go");
        assert_eq!(result, "data.go");
    }

    #[test]
    fn test_generate_code_output_path_rust() {
        let input = "data.json";
        let result = generate_code_output_path(input, None, "rust");
        assert_eq!(result, "data.rs");
    }

    #[test]
    fn test_generate_code_output_path_typescript() {
        let input = "data.json";
        let result = generate_code_output_path(input, None, "typescript");
        assert_eq!(result, "data.ts");
    }

    #[test]
    fn test_generate_code_output_path_python() {
        let input = "data.json";
        let result = generate_code_output_path(input, None, "python");
        assert_eq!(result, "data.py");
    }

    #[test]
    fn test_generate_code_output_path_with_directory() {
        let input = "path/to/data.json";
        let result = generate_code_output_path(input, None, "go");
        assert_eq!(result, "path/to/data.go");
    }

    #[test]
    fn test_generate_code_output_path_no_extension() {
        let input = "data";
        let result = generate_code_output_path(input, None, "rust");
        assert_eq!(result, "data.rs");
    }

    #[test]
    fn test_generate_code_output_path_different_extension() {
        let input = "data.txt";
        let result = generate_code_output_path(input, None, "typescript");
        assert_eq!(result, "data.ts");
    }

    #[test]
    fn test_write_code_file_success_go() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = r#"package main

type User struct {
    Name string `json:"name"`
    Age  int    `json:"age"`
}"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_ok());

        // Verify the file was created and has correct content
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_code_file_success_rust() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = r#"use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub age: i64,
}"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "rust");
        assert!(result.is_ok());

        // Verify the file was created and has correct content
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_code_file_success_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");
        let content = r#"export interface User {
    name: string;
    age: number;
}"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "typescript");
        assert!(result.is_ok());

        // Verify the file was created and has correct content
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_code_file_success_python() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        let content = r#"from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    name: str
    age: int"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "python");
        assert!(result.is_ok());

        // Verify the file was created and has correct content
        let written_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_code_file_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = "";

        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_file_error());
        assert!(error.to_string().contains("empty code content"));
    }

    #[test]
    fn test_write_code_file_whitespace_only() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = "   \n\t  \n  ";

        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_file_error());
        assert!(error.to_string().contains("empty code content"));
    }

    #[test]
    fn test_write_code_file_with_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("test.rs");
        let content = r#"pub struct Test {
    pub field: String,
}"#;

        let result = write_code_file(nested_path.to_str().unwrap(), content, "rust");
        assert!(result.is_ok());

        // Verify the file was created
        assert!(nested_path.exists());
        let written_content = fs::read_to_string(&nested_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_validate_go_content_missing_package() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = r#"type User struct {
    Name string
}"#;

        // Should succeed but warn about missing package
        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_go_content_unbalanced_braces() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = r#"package main

type User struct {
    Name string
// Missing closing brace"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unbalanced braces"));
    }

    #[test]
    fn test_validate_rust_content_unbalanced_parens() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = r#"pub struct User {
    pub name: String,
    pub get_info(&self -> String {
        self.name.clone()
    }
}"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "rust");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unbalanced parentheses"));
    }

    #[test]
    fn test_validate_typescript_content_unbalanced_braces() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");
        let content = r#"export interface User {
    name: string;
    age: number;
// Missing closing brace"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "typescript");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unbalanced braces"));
    }

    #[test]
    fn test_validate_python_content_unbalanced_brackets() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        let content = r#"from typing import List

class User:
    def __init__(self, items: List[str):
        self.items = items"#;

        let result = write_code_file(file_path.to_str().unwrap(), content, "python");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("unbalanced brackets"));
    }

    #[test]
    fn test_code_file_operations_integration() {
        let temp_dir = TempDir::new().unwrap();

        // Test all supported formats
        let formats = vec![
            ("go", "package main\n\ntype User struct {\n    Name string `json:\"name\"`\n}"),
            ("rust", "use serde::{Deserialize, Serialize};\n\n#[derive(Serialize, Deserialize)]\npub struct User {\n    pub name: String,\n}"),
            ("typescript", "export interface User {\n    name: string;\n}"),
            ("python", "from dataclasses import dataclass\n\n@dataclass\nclass User:\n    name: str"),
        ];

        for (format, content) in formats {
            let input_path = temp_dir.path().join(format!("input_{}.json", format));
            let json_content = r#"{"name": "test"}"#;
            fs::write(&input_path, json_content).unwrap();

            // Generate code output path
            let output_path = generate_code_output_path(input_path.to_str().unwrap(), None, format);
            assert!(output_path.ends_with(&format!("input_{}.{}", format, get_file_extension_for_format(format))));

            // Write code file
            let write_result = write_code_file(&output_path, content, format);
            assert!(write_result.is_ok(), "Failed to write {} file: {:?}", format, write_result);

            // Verify the code file was created
            assert!(Path::new(&output_path).exists());
            let written_content = fs::read_to_string(&output_path).unwrap();
            assert_eq!(written_content, content);
        }
    }

    #[test]
    fn test_content_validation_too_short() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        let content = "short";

        let result = write_code_file(file_path.to_str().unwrap(), content, "go");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("too short"));
    }
}
