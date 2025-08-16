use thiserror::Error;

/// Main error type for the j2s application
#[derive(Debug, Error)]
pub enum J2sError {
    /// File operation errors (reading, writing, path issues)
    #[error("File operation failed: {0}")]
    File(String),

    /// JSON parsing and validation errors
    #[error("JSON parsing failed: {0}")]
    Json(String),

    /// Schema generation errors
    #[error("Schema generation failed: {0}")]
    Schema(String),

    /// Command line argument errors
    #[error("Invalid arguments: {0}")]
    Argument(String),

    /// Code generation errors
    #[error("Code generation failed: {0}")]
    Codegen(String),

    /// Performance-related errors
    #[error("Performance issue: {0}")]
    Performance(String),
}

/// Convenience type alias for Results with J2sError
pub type Result<T> = std::result::Result<T, J2sError>;

// Automatic conversion from std::io::Error to J2sError
impl From<std::io::Error> for J2sError {
    fn from(err: std::io::Error) -> Self {
        J2sError::File(format!("IO error: {err}"))
    }
}

// Automatic conversion from serde_json::Error to J2sError
impl From<serde_json::Error> for J2sError {
    fn from(err: serde_json::Error) -> Self {
        J2sError::Json(format!("JSON error: {err}"))
    }
}

impl J2sError {
    /// Create a new file error with a custom message
    pub fn file_error(msg: impl Into<String>) -> Self {
        J2sError::File(msg.into())
    }

    /// Create a new JSON error with a custom message
    pub fn json_error(msg: impl Into<String>) -> Self {
        J2sError::Json(msg.into())
    }

    /// Create a new schema generation error with a custom message
    pub fn schema_error(msg: impl Into<String>) -> Self {
        J2sError::Schema(msg.into())
    }

    /// Create a new argument error with a custom message
    pub fn argument_error(msg: impl Into<String>) -> Self {
        J2sError::Argument(msg.into())
    }

    /// Create a new code generation error with a custom message
    pub fn codegen_error(msg: impl Into<String>) -> Self {
        J2sError::Codegen(msg.into())
    }

    /// Create a new performance error with a custom message
    pub fn performance_error(msg: impl Into<String>) -> Self {
        J2sError::Performance(msg.into())
    }

    /// Check if this is a file-related error
    #[allow(dead_code)]
    pub fn is_file_error(&self) -> bool {
        matches!(self, J2sError::File(_))
    }

    /// Check if this is a JSON-related error
    #[allow(dead_code)]
    pub fn is_json_error(&self) -> bool {
        matches!(self, J2sError::Json(_))
    }

    /// Check if this is a schema generation error
    #[allow(dead_code)]
    pub fn is_schema_error(&self) -> bool {
        matches!(self, J2sError::Schema(_))
    }

    /// Check if this is an argument error
    #[allow(dead_code)]
    pub fn is_argument_error(&self) -> bool {
        matches!(self, J2sError::Argument(_))
    }

    /// Check if this is a code generation error
    #[allow(dead_code)]
    pub fn is_codegen_error(&self) -> bool {
        matches!(self, J2sError::Codegen(_))
    }

    /// Check if this is a performance error
    #[allow(dead_code)]
    pub fn is_performance_error(&self) -> bool {
        matches!(self, J2sError::Performance(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let file_err = J2sError::File("test file error".to_string());
        assert_eq!(
            file_err.to_string(),
            "File operation failed: test file error"
        );

        let json_err = J2sError::Json("test json error".to_string());
        assert_eq!(json_err.to_string(), "JSON parsing failed: test json error");

        let schema_err = J2sError::Schema("test schema error".to_string());
        assert_eq!(
            schema_err.to_string(),
            "Schema generation failed: test schema error"
        );

        let arg_err = J2sError::Argument("test argument error".to_string());
        assert_eq!(
            arg_err.to_string(),
            "Invalid arguments: test argument error"
        );
    }

    #[test]
    fn test_error_constructors() {
        let file_err = J2sError::file_error("custom file error");
        assert!(file_err.is_file_error());
        assert_eq!(
            file_err.to_string(),
            "File operation failed: custom file error"
        );

        let json_err = J2sError::json_error("custom json error");
        assert!(json_err.is_json_error());
        assert_eq!(
            json_err.to_string(),
            "JSON parsing failed: custom json error"
        );

        let schema_err = J2sError::schema_error("custom schema error");
        assert!(schema_err.is_schema_error());
        assert_eq!(
            schema_err.to_string(),
            "Schema generation failed: custom schema error"
        );

        let arg_err = J2sError::argument_error("custom argument error");
        assert!(arg_err.is_argument_error());
        assert_eq!(
            arg_err.to_string(),
            "Invalid arguments: custom argument error"
        );
    }

    #[test]
    fn test_error_type_checks() {
        let file_err = J2sError::File("test".to_string());
        assert!(file_err.is_file_error());
        assert!(!file_err.is_json_error());
        assert!(!file_err.is_schema_error());
        assert!(!file_err.is_argument_error());

        let json_err = J2sError::Json("test".to_string());
        assert!(!json_err.is_file_error());
        assert!(json_err.is_json_error());
        assert!(!json_err.is_schema_error());
        assert!(!json_err.is_argument_error());

        let schema_err = J2sError::Schema("test".to_string());
        assert!(!schema_err.is_file_error());
        assert!(!schema_err.is_json_error());
        assert!(schema_err.is_schema_error());
        assert!(!schema_err.is_argument_error());

        let arg_err = J2sError::Argument("test".to_string());
        assert!(!arg_err.is_file_error());
        assert!(!arg_err.is_json_error());
        assert!(!arg_err.is_schema_error());
        assert!(arg_err.is_argument_error());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let j2s_err: J2sError = io_err.into();

        assert!(j2s_err.is_file_error());
        assert!(j2s_err.to_string().contains("IO error"));
        assert!(j2s_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid_json = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let j2s_err: J2sError = json_err.into();

        assert!(j2s_err.is_json_error());
        assert!(j2s_err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_debug() {
        let err = J2sError::File("debug test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("File"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        fn test_error_function() -> Result<String> {
            Err(J2sError::file_error("test error"))
        }

        let error_result = test_error_function();
        assert!(error_result.is_err());
        assert!(error_result.unwrap_err().is_file_error());
    }

    #[test]
    fn test_error_chain() {
        // Test that errors can be chained properly
        let original_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let j2s_err: J2sError = original_err.into();

        // The error should contain information about the original error
        let error_string = j2s_err.to_string();
        assert!(error_string.contains("File operation failed"));
        assert!(error_string.contains("IO error"));
        assert!(error_string.contains("access denied"));
    }
}
