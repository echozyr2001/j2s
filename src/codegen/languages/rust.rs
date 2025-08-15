//! # Rust Language Code Generator
//!
//! This module implements the code generator for the Rust programming language.
//! It generates Rust structs with serde derive macros, appropriate type mappings,
//! and follows Rust naming conventions and best practices.

use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::NameConverter;
use crate::error::{J2sError, Result};
use serde_json::Value;
use std::collections::HashSet;

/// Rust language code generator
///
/// This generator creates Rust struct definitions from JSON data, including:
/// - Proper Rust naming conventions (PascalCase for types, snake_case for fields)
/// - Serde derive macros for serialization/deserialization
/// - Option<T> types for optional fields
/// - Appropriate type mappings for Rust's type system
pub struct RustGenerator {
    /// Rust reserved keywords that need to be avoided in generated identifiers
    keywords: HashSet<String>,
}

impl RustGenerator {
    /// Create a new Rust code generator
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        
        // Add Rust reserved keywords
        let rust_keywords = [
            "as", "break", "const", "continue", "crate", "else", "enum", "extern",
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
            "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            "async", "await", "dyn", "abstract", "become", "box", "do", "final",
            "macro", "override", "priv", "typeof", "unsized", "virtual", "yield",
            "try", "union", "raw",
        ];
        
        for keyword in &rust_keywords {
            keywords.insert(keyword.to_string());
        }
        
        Self { keywords }
    }

    /// Map a FieldType to the appropriate Rust type string
    fn map_field_type(&self, field_type: &FieldType, is_optional: bool, is_array: bool) -> String {
        let base_type = match field_type {
            FieldType::String => "String",
            FieldType::Integer => "i64",
            FieldType::Number => "f64",
            FieldType::Boolean => "bool",
            FieldType::Custom(name) => name,
            FieldType::Any => "serde_json::Value",
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("Vec<{}>", result);
        }

        // Handle optional fields with Option<T>
        if is_optional {
            result = format!("Option<{}>", result);
        }

        result
    }

    /// Generate a Rust struct field declaration
    fn generate_field(&self, field: &FieldDefinition) -> String {
        let field_name = NameConverter::to_snake_case(&field.code_name);
        let sanitized_name = NameConverter::sanitize_identifier(&field_name, &self.keywords);
        
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Add comments if present
        let mut result = String::new();
        for comment in &field.comments {
            result.push_str(&format!("    /// {}\n", comment));
        }

        // Add serde rename attribute if field name differs from JSON name
        if field.json_name != sanitized_name {
            result.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.json_name));
        }

        // Add serde skip_serializing_if for optional fields
        if field.is_optional {
            result.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
        }

        result.push_str(&format!("    pub {}: {},", sanitized_name, field_type));
        result
    }

    /// Generate a complete Rust struct definition
    fn generate_struct(&self, struct_def: &StructDefinition) -> String {
        let struct_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&struct_name, &self.keywords);

        let mut result = String::new();

        // Add struct comments
        for comment in &struct_def.comments {
            result.push_str(&format!("/// {}\n", comment));
        }

        // Add derive macros
        result.push_str("#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]\n");

        // Start struct definition
        result.push_str(&format!("pub struct {} {{\n", sanitized_name));

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

impl Default for RustGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for RustGenerator {
    fn generate(&self, _json_value: &Value, _options: &GenerationOptions) -> Result<String> {
        // This is a stub implementation for now
        // The actual implementation will be done in a later task
        Err(J2sError::codegen_error(
            "Rust code generation not yet implemented. This will be implemented in task 5.1".to_string()
        ))
    }

    fn file_extension(&self) -> &'static str {
        "rs"
    }

    fn language_name(&self) -> &'static str {
        "Rust"
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
    fn test_rust_generator_creation() {
        let generator = RustGenerator::new();
        assert_eq!(generator.language_name(), "Rust");
        assert_eq!(generator.file_extension(), "rs");
    }

    #[test]
    fn test_map_field_type() {
        let generator = RustGenerator::new();
        
        assert_eq!(generator.map_field_type(&FieldType::String, false, false), "String");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, false), "i64");
        assert_eq!(generator.map_field_type(&FieldType::Number, false, false), "f64");
        assert_eq!(generator.map_field_type(&FieldType::Boolean, false, false), "bool");
        assert_eq!(generator.map_field_type(&FieldType::Any, false, false), "serde_json::Value");
        
        // Test optional types
        assert_eq!(generator.map_field_type(&FieldType::String, true, false), "Option<String>");
        assert_eq!(generator.map_field_type(&FieldType::Integer, true, false), "Option<i64>");
        
        // Test array types
        assert_eq!(generator.map_field_type(&FieldType::String, false, true), "Vec<String>");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, true), "Vec<i64>");
        
        // Test optional array types
        assert_eq!(generator.map_field_type(&FieldType::String, true, true), "Option<Vec<String>>");
    }

    #[test]
    fn test_generate_field() {
        let generator = RustGenerator::new();
        
        let field = FieldDefinition::new("userName", "user_name", FieldType::String)
            .add_comment("The user's name");
        
        let result = generator.generate_field(&field);
        assert!(result.contains("user_name"));
        assert!(result.contains("String"));
        assert!(result.contains("#[serde(rename = \"userName\")]"));
        assert!(result.contains("/// The user's name"));
    }

    #[test]
    fn test_generate_field_optional() {
        let generator = RustGenerator::new();
        
        let field = FieldDefinition::new("email", "email", FieldType::String)
            .optional(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("email"));
        assert!(result.contains("Option<String>"));
        assert!(result.contains("#[serde(skip_serializing_if = \"Option::is_none\")]"));
    }

    #[test]
    fn test_generate_field_array() {
        let generator = RustGenerator::new();
        
        let field = FieldDefinition::new("tags", "tags", FieldType::String)
            .array(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("tags"));
        assert!(result.contains("Vec<String>"));
    }

    #[test]
    fn test_generate_struct() {
        let generator = RustGenerator::new();
        
        let field1 = FieldDefinition::new("id", "id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "name", FieldType::String);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_comment("User represents a user in the system");
        
        let result = generator.generate_struct(&struct_def);
        assert!(result.contains("pub struct User {"));
        assert!(result.contains("pub id: i64"));
        assert!(result.contains("pub name: String"));
        assert!(result.contains("/// User represents a user in the system"));
        assert!(result.contains("#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_keyword_sanitization() {
        let generator = RustGenerator::new();
        
        let field = FieldDefinition::new("type", "type", FieldType::String);
        let result = generator.generate_field(&field);
        
        // Should be sanitized to avoid Rust keyword conflict
        assert!(result.contains("type_"));
    }

    #[test]
    fn test_validate_options() {
        let generator = RustGenerator::new();
        let options = GenerationOptions::default();
        
        assert!(generator.validate_options(&options).is_ok());
    }
}