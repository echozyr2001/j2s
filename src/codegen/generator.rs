//! # Code Generator Trait and Core Types
//!
//! This module defines the core `CodeGenerator` trait that all language-specific generators
//! must implement, along with the configuration types used throughout the code generation
//! process.

use crate::error::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Core trait that all language-specific code generators must implement
///
/// This trait provides a consistent interface for generating code in different programming
/// languages from JSON data. Each implementation handles the specifics of its target language
/// while maintaining a common API.
pub trait CodeGenerator {
    /// Generate code from a JSON value
    ///
    /// This is the main method that transforms JSON data into source code for the target language.
    /// The implementation should handle all aspects of code generation including type mapping,
    /// naming conventions, and language-specific syntax.
    ///
    /// # Arguments
    /// * `json_value` - The JSON data to generate code from
    /// * `options` - Configuration options for the generation process
    ///
    /// # Returns
    /// * `Result<String>` - The generated source code or an error
    fn generate(&self, json_value: &Value, options: &GenerationOptions) -> Result<String>;

    /// Get the file extension for the target language
    ///
    /// Returns the appropriate file extension (without the dot) for files in the target language.
    /// This is used when generating output file names.
    ///
    /// # Returns
    /// * `&'static str` - The file extension (e.g., "go", "rs", "ts", "py")
    fn file_extension(&self) -> &'static str;

    /// Get the human-readable name of the target language
    ///
    /// Returns a display name for the language that can be used in user-facing messages
    /// and documentation.
    ///
    /// # Returns
    /// * `&'static str` - The language name (e.g., "Go", "Rust", "TypeScript", "Python")
    fn language_name(&self) -> &'static str;

    /// Validate generation options for this specific generator
    ///
    /// This method allows each generator to validate that the provided options are
    /// compatible with its capabilities and requirements. It should return an error
    /// if any options are invalid or unsupported.
    ///
    /// # Arguments
    /// * `options` - The generation options to validate
    ///
    /// # Returns
    /// * `Result<()>` - Success or an error describing validation failures
    fn validate_options(&self, options: &GenerationOptions) -> Result<()>;
}

/// Configuration options for code generation
///
/// This structure contains all the configurable options that affect how code is generated.
/// Different generators may use different subsets of these options based on their
/// language-specific requirements.
#[derive(Debug, Clone)]
pub struct GenerationOptions {
    /// The name to use for the root struct/type/interface
    ///
    /// If not provided, generators will derive a name from the input file name or use
    /// a default name appropriate for their language.
    pub struct_name: Option<String>,

    /// Whether to include descriptive comments in the generated code
    ///
    /// When enabled, generators will add comments explaining the purpose of types
    /// and fields, as well as metadata about the generation process.
    pub include_comments: bool,

    /// Whether to treat missing JSON fields as optional in the generated types
    ///
    /// This affects how generators handle fields that might not be present in all
    /// instances of the JSON data. Different languages handle optionality differently
    /// (e.g., pointers in Go, Option<T> in Rust, optional properties in TypeScript).
    pub optional_fields: bool,

    /// Custom type mappings for specific JSON field names or patterns
    ///
    /// This allows users to override the default type inference for specific fields.
    /// The key is the JSON field path (e.g., "user.id") and the value is the target
    /// language type (e.g., "UserId" in Go, "user_id::UserId" in Rust).
    pub type_mappings: HashMap<String, String>,

    /// Additional language-specific options
    ///
    /// This provides a flexible way to pass language-specific configuration options
    /// without modifying the core GenerationOptions structure. Each generator can
    /// define its own set of recognized options.
    pub language_options: HashMap<String, String>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            struct_name: None,
            include_comments: true,
            optional_fields: true,
            type_mappings: HashMap::new(),
            language_options: HashMap::new(),
        }
    }
}

impl GenerationOptions {
    /// Create a new GenerationOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the struct name for the generated code
    pub fn with_struct_name<S: Into<String>>(mut self, name: S) -> Self {
        self.struct_name = Some(name.into());
        self
    }

    /// Set whether to include comments in the generated code
    pub fn with_comments(mut self, include: bool) -> Self {
        self.include_comments = include;
        self
    }

    /// Set whether to treat fields as optional
    pub fn with_optional_fields(mut self, optional: bool) -> Self {
        self.optional_fields = optional;
        self
    }

    /// Add a custom type mapping
    pub fn with_type_mapping<K: Into<String>, V: Into<String>>(
        mut self,
        field_path: K,
        target_type: V,
    ) -> Self {
        self.type_mappings
            .insert(field_path.into(), target_type.into());
        self
    }

    /// Add a language-specific option
    pub fn with_language_option<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.language_options.insert(key.into(), value.into());
        self
    }

    /// Get the effective struct name, using a default if none is specified
    pub fn get_struct_name(&self, default: &str) -> String {
        self.struct_name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Check if a language-specific option is set
    pub fn has_language_option(&self, key: &str) -> bool {
        self.language_options.contains_key(key)
    }

    /// Get a language-specific option value
    pub fn get_language_option(&self, key: &str) -> Option<&String> {
        self.language_options.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_options_default() {
        let options = GenerationOptions::default();
        assert!(options.struct_name.is_none());
        assert!(options.include_comments);
        assert!(options.optional_fields);
        assert!(options.type_mappings.is_empty());
        assert!(options.language_options.is_empty());
    }

    #[test]
    fn test_generation_options_builder() {
        let options = GenerationOptions::new()
            .with_struct_name("TestStruct")
            .with_comments(false)
            .with_optional_fields(false)
            .with_type_mapping("user.id", "UserId")
            .with_language_option("package", "main");

        assert_eq!(options.struct_name, Some("TestStruct".to_string()));
        assert!(!options.include_comments);
        assert!(!options.optional_fields);
        assert_eq!(
            options.type_mappings.get("user.id"),
            Some(&"UserId".to_string())
        );
        assert_eq!(
            options.language_options.get("package"),
            Some(&"main".to_string())
        );
    }

    #[test]
    fn test_get_struct_name() {
        let options_with_name = GenerationOptions::new().with_struct_name("CustomName");
        assert_eq!(options_with_name.get_struct_name("Default"), "CustomName");

        let options_without_name = GenerationOptions::new();
        assert_eq!(options_without_name.get_struct_name("Default"), "Default");
    }

    #[test]
    fn test_language_options() {
        let options = GenerationOptions::new()
            .with_language_option("key1", "value1")
            .with_language_option("key2", "value2");

        assert!(options.has_language_option("key1"));
        assert!(options.has_language_option("key2"));
        assert!(!options.has_language_option("key3"));

        assert_eq!(options.get_language_option("key1"), Some(&"value1".to_string()));
        assert_eq!(options.get_language_option("key2"), Some(&"value2".to_string()));
        assert_eq!(options.get_language_option("key3"), None);
    }
}