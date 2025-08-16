//! # Code Generator Factory
//!
//! This module provides the factory pattern implementation for creating language-specific
//! code generators. It serves as the central registry for all supported output formats
//! and provides a unified interface for generator creation.

use crate::codegen::generator::CodeGenerator;
use crate::codegen::languages::{
    go::GoGenerator, python::PythonGenerator, rust::RustGenerator, typescript::TypeScriptGenerator,
};
use crate::error::{J2sError, Result};

/// Factory for creating language-specific code generators
///
/// This factory provides a centralized way to create code generators for different
/// programming languages. It abstracts the creation logic and provides a consistent
/// interface for the main application logic.
pub struct GeneratorFactory;

impl GeneratorFactory {
    /// Create a code generator for the specified format
    ///
    /// This method creates and returns a boxed code generator instance for the
    /// requested output format. The generator is ready to use for code generation.
    ///
    /// # Arguments
    /// * `format` - The target format/language (e.g., "go", "rust", "typescript", "python")
    ///
    /// # Returns
    /// * `Result<Box<dyn CodeGenerator>>` - A boxed generator instance or an error
    ///
    /// # Errors
    /// * Returns `J2sError::UnsupportedFormat` if the requested format is not supported
    ///
    /// # Supported Formats
    /// * `"go"` - Go language structs with JSON tags
    /// * `"rust"` - Rust structs with serde annotations
    /// * `"typescript"` - TypeScript interfaces
    /// * `"python"` - Python dataclasses with type hints
    ///
    /// # Examples
    /// ```rust
    /// use crate::codegen::GeneratorFactory;
    ///
    /// let generator = GeneratorFactory::create_generator("go")?;
    /// let code = generator.generate(&json_value, &options)?;
    /// ```
    pub fn create_generator(format: &str) -> Result<Box<dyn CodeGenerator>> {
        match format.to_lowercase().as_str() {
            "go" => Ok(Box::new(GoGenerator::new())),
            "rust" => Ok(Box::new(RustGenerator::new())),
            "typescript" | "ts" => Ok(Box::new(TypeScriptGenerator::new())),
            "python" | "py" => Ok(Box::new(PythonGenerator::new())),
            _ => Err(J2sError::codegen_error(format!(
                "Unsupported format: '{}'. Supported formats are: go, rust, typescript, python",
                format
            ))),
        }
    }

    /// Get a list of all supported output formats
    ///
    /// This method returns a vector of all format strings that can be passed to
    /// `create_generator()`. This is useful for CLI help text, validation, and
    /// user interface generation.
    ///
    /// # Returns
    /// * `Vec<&'static str>` - A list of supported format strings
    pub fn supported_formats() -> Vec<&'static str> {
        vec!["go", "rust", "typescript", "python"]
    }

    /// Check if a format is supported
    ///
    /// This method provides a quick way to validate format strings without
    /// attempting to create a generator instance.
    ///
    /// # Arguments
    /// * `format` - The format string to check
    ///
    /// # Returns
    /// * `bool` - True if the format is supported, false otherwise
    pub fn is_supported_format(format: &str) -> bool {
        matches!(
            format.to_lowercase().as_str(),
            "go" | "rust" | "typescript" | "ts" | "python" | "py"
        )
    }

    /// Get the canonical format name for a given format string
    ///
    /// This method normalizes format strings to their canonical form. For example,
    /// both "ts" and "typescript" will return "typescript".
    ///
    /// # Arguments
    /// * `format` - The format string to normalize
    ///
    /// # Returns
    /// * `Option<&'static str>` - The canonical format name, or None if unsupported
    pub fn canonical_format(format: &str) -> Option<&'static str> {
        match format.to_lowercase().as_str() {
            "go" => Some("go"),
            "rust" => Some("rust"),
            "typescript" | "ts" => Some("typescript"),
            "python" | "py" => Some("python"),
            _ => None,
        }
    }

    /// Get a human-readable description of a format
    ///
    /// This method returns a descriptive string for each supported format that
    /// can be used in help text and user interfaces.
    ///
    /// # Arguments
    /// * `format` - The format to describe
    ///
    /// # Returns
    /// * `Option<&'static str>` - A description of the format, or None if unsupported
    pub fn format_description(format: &str) -> Option<&'static str> {
        match format.to_lowercase().as_str() {
            "go" => Some("Go language structs with JSON tags"),
            "rust" => Some("Rust structs with serde derive macros"),
            "typescript" | "ts" => Some("TypeScript interfaces with optional properties"),
            "python" | "py" => Some("Python dataclasses with type annotations"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_go_generator() {
        let generator = GeneratorFactory::create_generator("go").unwrap();
        assert_eq!(generator.language_name(), "Go");
        assert_eq!(generator.file_extension(), "go");
    }

    #[test]
    fn test_create_rust_generator() {
        let generator = GeneratorFactory::create_generator("rust").unwrap();
        assert_eq!(generator.language_name(), "Rust");
        assert_eq!(generator.file_extension(), "rs");
    }

    #[test]
    fn test_create_typescript_generator() {
        let generator = GeneratorFactory::create_generator("typescript").unwrap();
        assert_eq!(generator.language_name(), "TypeScript");
        assert_eq!(generator.file_extension(), "ts");

        // Test alias
        let generator_alias = GeneratorFactory::create_generator("ts").unwrap();
        assert_eq!(generator_alias.language_name(), "TypeScript");
    }

    #[test]
    fn test_create_python_generator() {
        let generator = GeneratorFactory::create_generator("python").unwrap();
        assert_eq!(generator.language_name(), "Python");
        assert_eq!(generator.file_extension(), "py");

        // Test alias
        let generator_alias = GeneratorFactory::create_generator("py").unwrap();
        assert_eq!(generator_alias.language_name(), "Python");
    }

    #[test]
    fn test_create_generator_case_insensitive() {
        let generator_upper = GeneratorFactory::create_generator("GO").unwrap();
        let generator_mixed = GeneratorFactory::create_generator("Go").unwrap();
        let generator_lower = GeneratorFactory::create_generator("go").unwrap();

        assert_eq!(generator_upper.language_name(), "Go");
        assert_eq!(generator_mixed.language_name(), "Go");
        assert_eq!(generator_lower.language_name(), "Go");
    }

    #[test]
    fn test_create_generator_unsupported() {
        let result = GeneratorFactory::create_generator("java");
        assert!(result.is_err());

        let result = GeneratorFactory::create_generator("cpp");
        assert!(result.is_err());

        let result = GeneratorFactory::create_generator("");
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_formats() {
        let formats = GeneratorFactory::supported_formats();
        assert!(formats.contains(&"go"));
        assert!(formats.contains(&"rust"));
        assert!(formats.contains(&"typescript"));
        assert!(formats.contains(&"python"));
        assert_eq!(formats.len(), 4);
    }

    #[test]
    fn test_is_supported_format() {
        assert!(GeneratorFactory::is_supported_format("go"));
        assert!(GeneratorFactory::is_supported_format("GO"));
        assert!(GeneratorFactory::is_supported_format("rust"));
        assert!(GeneratorFactory::is_supported_format("typescript"));
        assert!(GeneratorFactory::is_supported_format("ts"));
        assert!(GeneratorFactory::is_supported_format("python"));
        assert!(GeneratorFactory::is_supported_format("py"));

        assert!(!GeneratorFactory::is_supported_format("java"));
        assert!(!GeneratorFactory::is_supported_format("cpp"));
        assert!(!GeneratorFactory::is_supported_format(""));
        assert!(!GeneratorFactory::is_supported_format("javascript"));
    }

    #[test]
    fn test_canonical_format() {
        assert_eq!(GeneratorFactory::canonical_format("go"), Some("go"));
        assert_eq!(GeneratorFactory::canonical_format("GO"), Some("go"));
        assert_eq!(GeneratorFactory::canonical_format("rust"), Some("rust"));
        assert_eq!(GeneratorFactory::canonical_format("RUST"), Some("rust"));
        assert_eq!(
            GeneratorFactory::canonical_format("typescript"),
            Some("typescript")
        );
        assert_eq!(GeneratorFactory::canonical_format("ts"), Some("typescript"));
        assert_eq!(GeneratorFactory::canonical_format("TS"), Some("typescript"));
        assert_eq!(GeneratorFactory::canonical_format("python"), Some("python"));
        assert_eq!(GeneratorFactory::canonical_format("py"), Some("python"));
        assert_eq!(GeneratorFactory::canonical_format("PY"), Some("python"));

        assert_eq!(GeneratorFactory::canonical_format("java"), None);
        assert_eq!(GeneratorFactory::canonical_format(""), None);
    }

    #[test]
    fn test_format_description() {
        assert!(GeneratorFactory::format_description("go").is_some());
        assert!(GeneratorFactory::format_description("rust").is_some());
        assert!(GeneratorFactory::format_description("typescript").is_some());
        assert!(GeneratorFactory::format_description("ts").is_some());
        assert!(GeneratorFactory::format_description("python").is_some());
        assert!(GeneratorFactory::format_description("py").is_some());

        assert!(GeneratorFactory::format_description("java").is_none());
        assert!(GeneratorFactory::format_description("").is_none());

        // Verify descriptions are meaningful
        let go_desc = GeneratorFactory::format_description("go").unwrap();
        assert!(go_desc.contains("Go"));
        assert!(go_desc.contains("JSON"));

        let rust_desc = GeneratorFactory::format_description("rust").unwrap();
        assert!(rust_desc.contains("Rust"));
        assert!(rust_desc.contains("serde"));
    }
}
