//! # TypeScript Language Code Generator
//!
//! This module implements the code generator for the TypeScript programming language.
//! It generates TypeScript interfaces with appropriate type mappings, optional properties,
//! and follows TypeScript naming conventions and best practices.

use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::NameConverter;
use crate::error::{J2sError, Result};
use serde_json::Value;
use std::collections::HashSet;

/// TypeScript language code generator
///
/// This generator creates TypeScript interface definitions from JSON data, including:
/// - Proper TypeScript naming conventions (PascalCase for interfaces, camelCase for properties)
/// - Optional properties for nullable/missing fields
/// - Union types for flexible type handling
/// - Appropriate type mappings for TypeScript's type system
pub struct TypeScriptGenerator {
    /// TypeScript reserved keywords that need to be avoided in generated identifiers
    keywords: HashSet<String>,
}

impl TypeScriptGenerator {
    /// Create a new TypeScript code generator
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        
        // Add TypeScript/JavaScript reserved keywords
        let ts_keywords = [
            "abstract", "any", "as", "asserts", "bigint", "boolean", "break", "case",
            "catch", "class", "const", "constructor", "continue", "debugger", "declare",
            "default", "delete", "do", "else", "enum", "export", "extends", "false",
            "finally", "for", "from", "function", "get", "if", "implements", "import",
            "in", "infer", "instanceof", "interface", "is", "keyof", "let", "module",
            "namespace", "never", "new", "null", "number", "object", "of", "package",
            "private", "protected", "public", "readonly", "require", "return", "set",
            "static", "string", "super", "switch", "symbol", "this", "throw", "true",
            "try", "type", "typeof", "undefined", "unique", "unknown", "var", "void",
            "while", "with", "yield",
        ];
        
        for keyword in &ts_keywords {
            keywords.insert(keyword.to_string());
        }
        
        Self { keywords }
    }

    /// Map a FieldType to the appropriate TypeScript type string
    fn map_field_type(&self, field_type: &FieldType, is_optional: bool, is_array: bool) -> String {
        let base_type = match field_type {
            FieldType::String => "string",
            FieldType::Integer => "number",
            FieldType::Number => "number",
            FieldType::Boolean => "boolean",
            FieldType::Custom(name) => name,
            FieldType::Any => "any",
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("{}[]", result);
        }

        // Handle optional fields with union types
        if is_optional {
            result = format!("{} | null", result);
        }

        result
    }

    /// Generate a TypeScript interface property declaration
    fn generate_property(&self, field: &FieldDefinition) -> String {
        let property_name = NameConverter::to_camel_case(&field.code_name);
        let sanitized_name = NameConverter::sanitize_identifier(&property_name, &self.keywords);
        
        let property_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Add comments if present
        let mut result = String::new();
        for comment in &field.comments {
            result.push_str(&format!("  /** {} */\n", comment));
        }

        // Determine if property should be optional (using ? syntax)
        let optional_marker = if field.is_optional { "?" } else { "" };

        result.push_str(&format!("  {}{}: {};", sanitized_name, optional_marker, property_type));
        result
    }

    /// Generate a complete TypeScript interface definition
    fn generate_interface(&self, struct_def: &StructDefinition) -> String {
        let interface_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&interface_name, &self.keywords);

        let mut result = String::new();

        // Add interface comments
        for comment in &struct_def.comments {
            result.push_str(&format!("/** {} */\n", comment));
        }

        // Start interface definition
        result.push_str(&format!("export interface {} {{\n", sanitized_name));

        // Add properties
        for field in &struct_def.fields {
            result.push_str(&self.generate_property(field));
            result.push('\n');
        }

        // Close interface definition
        result.push_str("}\n");

        result
    }
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for TypeScriptGenerator {
    fn generate(&self, _json_value: &Value, _options: &GenerationOptions) -> Result<String> {
        // This is a stub implementation for now
        // The actual implementation will be done in a later task
        Err(J2sError::codegen_error(
            "TypeScript code generation not yet implemented. This will be implemented in task 6.1".to_string()
        ))
    }

    fn file_extension(&self) -> &'static str {
        "ts"
    }

    fn language_name(&self) -> &'static str {
        "TypeScript"
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
    fn test_typescript_generator_creation() {
        let generator = TypeScriptGenerator::new();
        assert_eq!(generator.language_name(), "TypeScript");
        assert_eq!(generator.file_extension(), "ts");
    }

    #[test]
    fn test_map_field_type() {
        let generator = TypeScriptGenerator::new();
        
        assert_eq!(generator.map_field_type(&FieldType::String, false, false), "string");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, false), "number");
        assert_eq!(generator.map_field_type(&FieldType::Number, false, false), "number");
        assert_eq!(generator.map_field_type(&FieldType::Boolean, false, false), "boolean");
        assert_eq!(generator.map_field_type(&FieldType::Any, false, false), "any");
        
        // Test optional types
        assert_eq!(generator.map_field_type(&FieldType::String, true, false), "string | null");
        assert_eq!(generator.map_field_type(&FieldType::Integer, true, false), "number | null");
        
        // Test array types
        assert_eq!(generator.map_field_type(&FieldType::String, false, true), "string[]");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, true), "number[]");
        
        // Test optional array types
        assert_eq!(generator.map_field_type(&FieldType::String, true, true), "string[] | null");
    }

    #[test]
    fn test_generate_property() {
        let generator = TypeScriptGenerator::new();
        
        let field = FieldDefinition::new("user_name", "user_name", FieldType::String)
            .add_comment("The user's name");
        
        let result = generator.generate_property(&field);
        assert!(result.contains("userName"));
        assert!(result.contains("string"));
        assert!(result.contains("/** The user's name */"));
        assert!(!result.contains("?"));  // Not optional
    }

    #[test]
    fn test_generate_property_optional() {
        let generator = TypeScriptGenerator::new();
        
        let field = FieldDefinition::new("email", "email", FieldType::String)
            .optional(true);
        
        let result = generator.generate_property(&field);
        assert!(result.contains("email?"));
        assert!(result.contains("string | null"));
    }

    #[test]
    fn test_generate_property_array() {
        let generator = TypeScriptGenerator::new();
        
        let field = FieldDefinition::new("tags", "tags", FieldType::String)
            .array(true);
        
        let result = generator.generate_property(&field);
        assert!(result.contains("tags"));
        assert!(result.contains("string[]"));
    }

    #[test]
    fn test_generate_interface() {
        let generator = TypeScriptGenerator::new();
        
        let field1 = FieldDefinition::new("id", "id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "name", FieldType::String);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_comment("User represents a user in the system");
        
        let result = generator.generate_interface(&struct_def);
        assert!(result.contains("export interface User {"));
        assert!(result.contains("id: number"));
        assert!(result.contains("name: string"));
        assert!(result.contains("/** User represents a user in the system */"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_keyword_sanitization() {
        let generator = TypeScriptGenerator::new();
        
        let field = FieldDefinition::new("type", "type", FieldType::String);
        let result = generator.generate_property(&field);
        
        // Should be sanitized to avoid TypeScript keyword conflict
        assert!(result.contains("type_"));
    }

    #[test]
    fn test_validate_options() {
        let generator = TypeScriptGenerator::new();
        let options = GenerationOptions::default();
        
        assert!(generator.validate_options(&options).is_ok());
    }
}