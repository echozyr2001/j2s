use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{J2sError, Result};

/// Read JSON content from a file
/// 
/// # Arguments
/// * `path` - Path to the JSON file to read
/// 
/// # Returns
/// * `Result<String>` - The file content as a string, or an error
/// 
/// # Errors
/// * Returns `J2sError::File` if the file cannot be read or doesn't exist
pub fn read_json_file(path: &str) -> Result<String> {
    // Check if file exists first to provide better error messages
    if !Path::new(path).exists() {
        return Err(J2sError::file_error(format!("File not found: {path}")));
    }
    
    // Check if it's actually a file (not a directory)
    if !Path::new(path).is_file() {
        return Err(J2sError::file_error(format!("Path is not a file: {path}")));
    }
    
    // Read the file content
    match fs::read_to_string(path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return Err(J2sError::file_error(format!("File is empty: {path}")));
            }
            Ok(content)
        }
        Err(err) => Err(J2sError::file_error(format!("Failed to read file {path}: {err}"))),
    }
}

/// Write schema content to a file
/// 
/// # Arguments
/// * `path` - Path where the schema file should be written
/// * `content` - The schema content to write
/// 
/// # Returns
/// * `Result<()>` - Success or an error
/// 
/// # Errors
/// * Returns `J2sError::File` if the file cannot be written
pub fn write_schema_file(path: &str, content: &str) -> Result<()> {
    // Ensure the output directory exists
    ensure_output_directory(path)?;
    
    // Write the content to the file
    match fs::write(path, content) {
        Ok(()) => {
            // Verify the file was written successfully by checking if it exists
            if Path::new(path).exists() {
                Ok(())
            } else {
                Err(J2sError::file_error(format!("File was not created successfully: {path}")))
            }
        }
        Err(err) => Err(J2sError::file_error(format!("Failed to write file {path}: {err}"))),
    }
}

/// Ensure the directory for the given file path exists
/// 
/// # Arguments
/// * `file_path` - Path to a file (the directory will be extracted and created)
/// 
/// # Returns
/// * `Result<()>` - Success or an error
/// 
/// # Errors
/// * Returns `J2sError::File` if the directory cannot be created
pub fn ensure_output_directory(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);
    
    // Get the parent directory
    if let Some(parent) = path.parent() {
        // Only create directory if it doesn't exist and is not empty (current directory)
        if !parent.as_os_str().is_empty() && !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(()) => Ok(()),
                Err(err) => Err(J2sError::file_error(format!(
                    "Failed to create directory {}: {err}", 
                    parent.display()
                ))),
            }
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
                Some(parent_dir) if !parent_dir.as_os_str().is_empty() => {
                    parent_dir.join(output_filename).to_string_lossy().to_string()
                }
                _ => output_filename,
            }
        }
    }
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
        let nested_path = temp_dir.path().join("nested").join("dir").join("test.schema.json");
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
        let nested_file_path = temp_dir.path().join("new").join("directory").join("file.json");
        
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
}