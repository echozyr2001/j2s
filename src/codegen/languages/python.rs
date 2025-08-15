//! # Python Language Code Generator
//!
//! This module implements the code generator for the Python programming language.
//! It generates Python dataclasses with type annotations, appropriate type mappings,
//! and follows Python naming conventions and best practices.

use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::NameConverter;
use crate::error::{J2sError, Result};
use serde_json::Value;
use std::collections::HashSet;

/// Python language code generator
///
/// This generator creates Python dataclass definitions from JSON data, including:
/// - Proper Python naming conventions (PascalCase for classes, snake_case for fields)
/// - Type annotations with typing module imports
/// - Optional[T] types for nullable/missing fields
/// - Dataclass decorators for automatic method generation
pub struct PythonGenerator {
    /// Python reserved keywords that need to be avoided in generated identifiers
    keywords: HashSet<String>,
}

impl PythonGenerator {
    /// Create a new Python code generator
    pub fn new() -> Self {
        let mut keywords = HashSet::new();
        
        // Add Python reserved keywords
        let python_keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break",
            "class", "continue", "def", "del", "elif", "else", "except", "finally",
            "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal",
            "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
            // Built-in functions that should be avoided
            "abs", "all", "any", "ascii", "bin", "bool", "bytearray", "bytes", "callable",
            "chr", "classmethod", "compile", "complex", "delattr", "dict", "dir", "divmod",
            "enumerate", "eval", "exec", "filter", "float", "format", "frozenset",
            "getattr", "globals", "hasattr", "hash", "help", "hex", "id", "input", "int",
            "isinstance", "issubclass", "iter", "len", "list", "locals", "map", "max",
            "memoryview", "min", "next", "object", "oct", "open", "ord", "pow", "print",
            "property", "range", "repr", "reversed", "round", "set", "setattr", "slice",
            "sorted", "staticmethod", "str", "sum", "super", "tuple", "type", "vars", "zip",
        ];
        
        for keyword in &python_keywords {
            keywords.insert(keyword.to_string());
        }
        
        Self { keywords }
    }

    /// Map a FieldType to the appropriate Python type string
    fn map_field_type(&self, field_type: &FieldType, is_optional: bool, is_array: bool) -> String {
        let base_type = match field_type {
            FieldType::String => "str",
            FieldType::Integer => "int",
            FieldType::Number => "float",
            FieldType::Boolean => "bool",
            FieldType::Custom(name) => name,
            FieldType::Any => "Any",
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("List[{}]", result);
        }

        // Handle optional fields with Optional[T]
        if is_optional {
            result = format!("Optional[{}]", result);
        }

        result
    }

    /// Generate a Python dataclass field declaration
    fn generate_field(&self, field: &FieldDefinition) -> String {
        let field_name = NameConverter::to_snake_case(&field.code_name);
        let sanitized_name = NameConverter::sanitize_identifier(&field_name, &self.keywords);
        
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Add comments if present
        let mut result = String::new();
        for comment in &field.comments {
            result.push_str(&format!("    # {}\n", comment));
        }

        // Generate field with type annotation
        if field.is_optional {
            result.push_str(&format!("    {}: {} = None", sanitized_name, field_type));
        } else {
            result.push_str(&format!("    {}: {}", sanitized_name, field_type));
        }

        result
    }

    /// Generate a complete Python dataclass definition
    fn generate_dataclass(&self, struct_def: &StructDefinition) -> String {
        let class_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&class_name, &self.keywords);

        let mut result = String::new();

        // Add class comments as docstring
        if !struct_def.comments.is_empty() {
            result.push_str(&format!("@dataclass\nclass {}:\n", sanitized_name));
            result.push_str("    \"\"\"\n");
            for comment in &struct_def.comments {
                result.push_str(&format!("    {}\n", comment));
            }
            result.push_str("    \"\"\"\n");
        } else {
            result.push_str(&format!("@dataclass\nclass {}:\n", sanitized_name));
        }

        // Add fields
        if struct_def.fields.is_empty() {
            result.push_str("    pass\n");
        } else {
            for field in &struct_def.fields {
                result.push_str(&self.generate_field(field));
                result.push('\n');
            }
        }

        result
    }

    /// Generate the necessary import statements for the generated code
    fn generate_imports(&self, struct_def: &StructDefinition) -> String {
        let mut imports = Vec::new();
        let mut has_optional = false;
        let mut has_list = false;
        let mut has_any = false;

        // Check what types are used to determine imports
        for field in &struct_def.fields {
            if field.is_optional {
                has_optional = true;
            }
            if field.is_array {
                has_list = true;
            }
            if matches!(field.field_type, FieldType::Any) {
                has_any = true;
            }
        }

        // Add dataclass import
        imports.push("from dataclasses import dataclass".to_string());

        // Add typing imports
        let mut typing_imports = Vec::new();
        if has_optional {
            typing_imports.push("Optional");
        }
        if has_list {
            typing_imports.push("List");
        }
        if has_any {
            typing_imports.push("Any");
        }

        if !typing_imports.is_empty() {
            imports.push(format!("from typing import {}", typing_imports.join(", ")));
        }

        imports.join("\n") + "\n\n"
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PythonGenerator {
    fn generate(&self, _json_value: &Value, _options: &GenerationOptions) -> Result<String> {
        // This is a stub implementation for now
        // The actual implementation will be done in a later task
        Err(J2sError::codegen_error(
            "Python code generation not yet implemented. This will be implemented in task 7.1".to_string()
        ))
    }

    fn file_extension(&self) -> &'static str {
        "py"
    }

    fn language_name(&self) -> &'static str {
        "Python"
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
    fn test_python_generator_creation() {
        let generator = PythonGenerator::new();
        assert_eq!(generator.language_name(), "Python");
        assert_eq!(generator.file_extension(), "py");
    }

    #[test]
    fn test_map_field_type() {
        let generator = PythonGenerator::new();
        
        assert_eq!(generator.map_field_type(&FieldType::String, false, false), "str");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, false), "int");
        assert_eq!(generator.map_field_type(&FieldType::Number, false, false), "float");
        assert_eq!(generator.map_field_type(&FieldType::Boolean, false, false), "bool");
        assert_eq!(generator.map_field_type(&FieldType::Any, false, false), "Any");
        
        // Test optional types
        assert_eq!(generator.map_field_type(&FieldType::String, true, false), "Optional[str]");
        assert_eq!(generator.map_field_type(&FieldType::Integer, true, false), "Optional[int]");
        
        // Test array types
        assert_eq!(generator.map_field_type(&FieldType::String, false, true), "List[str]");
        assert_eq!(generator.map_field_type(&FieldType::Integer, false, true), "List[int]");
        
        // Test optional array types
        assert_eq!(generator.map_field_type(&FieldType::String, true, true), "Optional[List[str]]");
    }

    #[test]
    fn test_generate_field() {
        let generator = PythonGenerator::new();
        
        let field = FieldDefinition::new("userName", "user_name", FieldType::String)
            .add_comment("The user's name");
        
        let result = generator.generate_field(&field);
        assert!(result.contains("user_name"));
        assert!(result.contains("str"));
        assert!(result.contains("# The user's name"));
        assert!(!result.contains("= None"));  // Not optional
    }

    #[test]
    fn test_generate_field_optional() {
        let generator = PythonGenerator::new();
        
        let field = FieldDefinition::new("email", "email", FieldType::String)
            .optional(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("email"));
        assert!(result.contains("Optional[str]"));
        assert!(result.contains("= None"));
    }

    #[test]
    fn test_generate_field_array() {
        let generator = PythonGenerator::new();
        
        let field = FieldDefinition::new("tags", "tags", FieldType::String)
            .array(true);
        
        let result = generator.generate_field(&field);
        assert!(result.contains("tags"));
        assert!(result.contains("List[str]"));
    }

    #[test]
    fn test_generate_dataclass() {
        let generator = PythonGenerator::new();
        
        let field1 = FieldDefinition::new("id", "id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "name", FieldType::String);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_comment("User represents a user in the system");
        
        let result = generator.generate_dataclass(&struct_def);
        assert!(result.contains("@dataclass"));
        assert!(result.contains("class User:"));
        assert!(result.contains("id_: int"));  // Fixed: "id" is a Python builtin, so it becomes "id_"
        assert!(result.contains("name: str"));
        assert!(result.contains("User represents a user in the system"));
    }

    #[test]
    fn test_generate_dataclass_empty() {
        let generator = PythonGenerator::new();
        
        let struct_def = StructDefinition::new("Empty");
        
        let result = generator.generate_dataclass(&struct_def);
        assert!(result.contains("@dataclass"));
        assert!(result.contains("class Empty:"));
        assert!(result.contains("pass"));
    }

    #[test]
    fn test_generate_imports() {
        let generator = PythonGenerator::new();
        
        let field1 = FieldDefinition::new("id", "id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "name", FieldType::String).optional(true);
        let field3 = FieldDefinition::new("tags", "tags", FieldType::String).array(true);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_field(field3);
        
        let result = generator.generate_imports(&struct_def);
        assert!(result.contains("from dataclasses import dataclass"));
        assert!(result.contains("from typing import"));
        assert!(result.contains("Optional"));
        assert!(result.contains("List"));
    }

    #[test]
    fn test_keyword_sanitization() {
        let generator = PythonGenerator::new();
        
        let field = FieldDefinition::new("class", "class", FieldType::String);
        let result = generator.generate_field(&field);
        
        // Should be sanitized to avoid Python keyword conflict
        assert!(result.contains("class_"));
    }

    #[test]
    fn test_validate_options() {
        let generator = PythonGenerator::new();
        let options = GenerationOptions::default();
        
        assert!(generator.validate_options(&options).is_ok());
    }
}