//! # Go Language Code Generator
//!
//! This module implements the code generator for the Go programming language.
//! It generates Go structs with appropriate JSON tags, type mappings, and
//! follows Go naming conventions and best practices.

use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::NameConverter;
use crate::error::{J2sError, Result};
use serde_json::Value;
use std::collections::HashSet;

/// Go language code generator
///
/// This generator creates Go struct definitions from JSON data, including:
/// - Proper Go naming conventions (PascalCase for types, camelCase for fields)
/// - JSON struct tags for serialization
/// - Pointer types for optional fields
/// - Appropriate type mappings for Go's type system
pub struct GoGenerator {
    /// Go reserved keywords that need to be avoided in generated identifiers
    keywords: HashSet<String>,
}

impl GoGenerator {
    /// Create a new Go code generator
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        
        // Add Go reserved keywords
        let go_keywords = [
            "break", "case", "chan", "const", "continue", "default", "defer", "else",
            "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
            "map", "package", "range", "return", "select", "struct", "switch", "type",
            "var", "bool", "byte", "complex64", "complex128", "error", "float32",
            "float64", "int", "int8", "int16", "int32", "int64", "rune", "string",
            "uint", "uint8", "uint16", "uint32", "uint64", "uintptr", "true", "false",
            "iota", "nil", "append", "cap", "close", "complex", "copy", "delete",
            "imag", "len", "make", "new", "panic", "print", "println", "real", "recover",
        ];
        
        for keyword in &go_keywords {
            keywords.insert(keyword.to_string());
        }
        
        Self { keywords }
    }

    /// Map a FieldType to the appropriate Go type string
    fn map_field_type(&self, field_type: &FieldType, is_optional: bool, is_array: bool) -> String {
        let base_type = match field_type {
            FieldType::String => "string",
            FieldType::Integer => "int64",
            FieldType::Number => "float64",
            FieldType::Boolean => "bool",
            FieldType::Custom(name) => name,
            FieldType::Any => "interface{}",
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("[]{}",result);
        }

        // Handle optional fields with pointers
        if is_optional {
            result = format!("*{}", result);
        }

        result
    }

    /// Generate a Go struct field declaration
    fn generate_field(&self, field: &FieldDefinition) -> String {
        let field_name = NameConverter::to_pascal_case(&field.code_name);
        let sanitized_name = NameConverter::sanitize_identifier(&field_name, &self.keywords);
        
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Generate JSON tag
        let json_tag = if field.is_optional {
            format!("`json:\"{},omitempty\"`", field.json_name)
        } else {
            format!("`json:\"{}\"`", field.json_name)
        };

        // Add comments if present
        let mut result = String::new();
        for comment in &field.comments {
            result.push_str(&format!("\t// {}\n", comment));
        }

        result.push_str(&format!("\t{} {} {}", sanitized_name, field_type, json_tag));
        result
    }

    /// Generate a complete Go struct definition
    fn generate_struct(&self, struct_def: &StructDefinition) -> String {
        let struct_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&struct_name, &self.keywords);

        let mut result = String::new();

        // Add struct comments
        for comment in &struct_def.comments {
            result.push_str(&format!("// {}\n", comment));
        }

        // Start struct definition
        result.push_str(&format!("type {} struct {{\n", sanitized_name));

        // Add fields
        for field in &struct_def.fields {
            result.push_str(&self.generate_field(field));
            result.push('\n');
        }

        // Close struct definition
        result.push_str("}\n");

        result
    }
}

impl Default for GoGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for GoGenerator {
    fn generate(&self, _json_value: &Value, _options: &GenerationOptions) -> Result<String> {
        // This is a stub implementation for now
        // The actual implementation will be done in a later task
        Err(J2sError::codegen_error(
            "Go code generation not yet implemented. This will be implemented in task 4.1".to_string()
        ))
    }

    fn file_extension(&self) -> &'static str {
        "go"
    }

    fn language_name(&self) -> &'static str {
        "Go"
    }

    fn validate_options(&self, _options: &GenerationOptions) -> Result<()> {
        // Basic validation - can be extended later
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::types::FieldType;

    #[test]
    fn test_go_generator_creation() {
        let generator = GoGenerator::new();
        assert_eq!(generator.language_name(), "Go");
        assert_eq!(generator.file_extension(), "go");
    }

    #[test]
    fn test_map_field_type() {
        let generator = GoGenerator::new();
        
        assert_eq!(generator.map_field_type(&FieldType::String, false, false), "string");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, false), "int64");
        assert_eq!(generator.map_field_type(&FieldType::Number, false, false), "float64");
        assert_eq!(generator.map_field_type(&FieldType::Boolean, false, false), "bool");
        assert_eq!(generator.map_field_type(&FieldType::Any, false, false), "interface{}");
        
        // Test optional types
        assert_eq!(generator.map_field_type(&FieldType::String, true, false), "*string");
        assert_eq!(generator.map_field_type(&FieldType::Integer, true, false), "*int64");
        
        // Test array types
        assert_eq!(generator.map_field_type(&FieldType::String, false, true), "[]string");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, true), "[]int64");
        
        // Test optional array types
        assert_eq!(generator.map_field_type(&FieldType::String, true, true), "*[]string");
    }

    #[test]
    fn test_generate_field() {
        let generator = GoGenerator::new();
        
        let field = FieldDefinition::new("user_name", "user_name", FieldType::String)
            .add_comment("The user's name");
        
        let result = generator.generate_field(&field);
        assert!(result.contains("UserName"));
        assert!(result.contains("string"));
        assert!(result.contains("`json:\"user_name\"`"));
        assert!(result.contains("// The user's name"));
    }

    #[test]
    fn test_generate_field_optional() {
        let generator = GoGenerator::new();
        
        let field = FieldDefinition::new("email", "email", FieldType::String)
            .optional(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("Email"));
        assert!(result.contains("*string"));
        assert!(result.contains("`json:\"email,omitempty\"`"));
    }

    #[test]
    fn test_generate_field_array() {
        let generator = GoGenerator::new();
        
        let field = FieldDefinition::new("tags", "tags", FieldType::String)
            .array(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("Tags"));
        assert!(result.contains("[]string"));
        assert!(result.contains("`json:\"tags\"`"));
    }

    #[test]
    fn test_generate_struct() {
        let generator = GoGenerator::new();
        
        let field1 = FieldDefinition::new("id", "id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "name", FieldType::String);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_comment("User represents a user in the system");
        
        let result = generator.generate_struct(&struct_def);
        assert!(result.contains("type User struct {"));
        assert!(result.contains("Id int64"));
        assert!(result.contains("Name string"));
        assert!(result.contains("// User represents a user in the system"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_keyword_sanitization() {
        let generator = GoGenerator::new();
        
        let field = FieldDefinition::new("type", "type", FieldType::String);
        let result = generator.generate_field(&field);
        
        // Should be sanitized to avoid Go keyword conflict
        assert!(result.contains("Type_"));
    }

    #[test]
    fn test_validate_options() {
        let generator = GoGenerator::new();
        let options = GenerationOptions::default();
        
        assert!(generator.validate_options(&options).is_ok());
    }
}