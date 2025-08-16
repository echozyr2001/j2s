//! # Python Language Code Generator
//!
//! This module implements the code generator for the Python programming language.
//! It generates Python dataclasses with type annotations, appropriate type mappings,
//! and follows Python naming conventions and best practices.

use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::NameConverter;
use crate::error::Result;
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
            FieldType::Any => {
                if is_array {
                    // For arrays with mixed types, use Any as element type
                    // Could be enhanced to use Union types like List[Union[str, int, bool]]
                    "Any"
                } else {
                    // For single Any fields, use Any
                    "Any"
                }
            }
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("List[{result}]");
        }

        // Handle optional fields with Optional[T]
        if is_optional {
            result = format!("Optional[{result}]");
        }

        result
    }

    /// Generate a Python dataclass field declaration
    fn generate_field(&self, field: &FieldDefinition) -> String {
        // The code_name is already converted to the proper case by JsonToIrConverter
        let sanitized_name = NameConverter::sanitize_identifier(&field.code_name, &self.keywords);
        
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Add comments if present
        let mut result = String::new();
        for comment in &field.comments {
            result.push_str(&format!("    # {comment}\n"));
        }

        // Generate field with type annotation
        if field.is_optional {
            result.push_str(&format!("    {sanitized_name}: {field_type} = None"));
        } else {
            result.push_str(&format!("    {sanitized_name}: {field_type}"));
        }

        result
    }

    /// Generate a complete Python dataclass definition
    fn generate_dataclass(&self, struct_def: &StructDefinition) -> String {
        let class_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&class_name, &self.keywords);

        let mut result = String::new();

        // Add dataclass decorator
        result.push_str("@dataclass\n");
        result.push_str(&format!("class {sanitized_name}:\n"));

        // Add class docstring
        result.push_str("    \"\"\"\n");
        if !struct_def.comments.is_empty() {
            for comment in &struct_def.comments {
                result.push_str(&format!("    {comment}\n"));
            }
        } else {
            result.push_str(&format!("    {sanitized_name} dataclass.\n"));
            result.push_str("    \n");
            result.push_str("    Auto-generated from JSON data.\n");
        }

        // Add field documentation to docstring if fields exist
        if !struct_def.fields.is_empty() {
            result.push_str("    \n");
            result.push_str("    Attributes:\n");
            for field in &struct_def.fields {
                let field_name = NameConverter::sanitize_identifier(&field.code_name, &self.keywords);
                let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
                result.push_str(&format!("        {field_name} ({field_type}): "));
                
                if !field.comments.is_empty() {
                    result.push_str(&field.comments.join(" "));
                } else {
                    result.push_str(&format!("Field from JSON key '{}'", field.json_name));
                }
                result.push('\n');
            }
        }
        result.push_str("    \"\"\"\n");

        // Add fields
        if struct_def.fields.is_empty() {
            result.push_str("    pass\n");
        } else {
            for field in &struct_def.fields {
                result.push_str(&self.generate_field_with_default(field));
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

        // Check what types are used to determine imports (including nested structs)
        self.check_types_recursive(struct_def, &mut has_optional, &mut has_list, &mut has_any);

        // Add dataclass import
        if self.needs_field_import(struct_def) {
            imports.push("from dataclasses import dataclass, field".to_string());
        } else {
            imports.push("from dataclasses import dataclass".to_string());
        }

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

    /// Generate file header with generation information and module docstring
    fn generate_file_header(&self, struct_name: &str) -> String {
        use crate::codegen::utils::generate_timestamp;
        
        let timestamp = generate_timestamp();
        format!(
            "\"\"\"
Generated Python dataclasses from JSON data.

This module contains dataclass definitions automatically generated from JSON data
using the j2s (JSON to Struct) tool.

Generated at: {timestamp}
Main class: {struct_name}

DO NOT EDIT - This file was automatically generated.
\"\"\"

"
        )
    }

    /// Recursively check types in struct and nested structs
    fn check_types_recursive(&self, struct_def: &StructDefinition, has_optional: &mut bool, has_list: &mut bool, has_any: &mut bool) {
        // Check fields in current struct
        for field in &struct_def.fields {
            if field.is_optional {
                *has_optional = true;
            }
            if field.is_array {
                *has_list = true;
            }
            if matches!(field.field_type, FieldType::Any) {
                *has_any = true;
            }
        }

        // Check nested structs
        for nested in &struct_def.nested_structs {
            self.check_types_recursive(nested, has_optional, has_list, has_any);
        }
    }

    /// Generate a Python-style field with proper type annotation and default value
    fn generate_field_with_default(&self, field: &FieldDefinition) -> String {
        let sanitized_name = NameConverter::sanitize_identifier(&field.code_name, &self.keywords);
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        let mut result = String::new();
        
        // Add inline comments if present
        for comment in &field.comments {
            result.push_str(&format!("    # {comment}\n"));
        }

        // Generate field with type annotation and appropriate default
        if field.is_optional {
            result.push_str(&format!("    {sanitized_name}: {field_type} = None"));
        } else if field.is_array {
            // For non-optional arrays, use field(default_factory=list) to avoid mutable defaults
            result.push_str(&format!("    {sanitized_name}: {field_type} = field(default_factory=list)"));
        } else {
            result.push_str(&format!("    {sanitized_name}: {field_type}"));
        }

        result
    }

    /// Check if we need to import field from dataclasses
    fn needs_field_import(&self, struct_def: &StructDefinition) -> bool {
        // Check if any non-optional array fields exist (they need field(default_factory=list))
        for field in &struct_def.fields {
            if field.is_array && !field.is_optional {
                return true;
            }
        }
        
        // Check nested structs
        for nested in &struct_def.nested_structs {
            if self.needs_field_import(nested) {
                return true;
            }
        }
        
        false
    }
}

impl Default for PythonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PythonGenerator {
    fn generate(&self, json_value: &Value, options: &GenerationOptions) -> Result<String> {
        use crate::codegen::types::JsonToIrConverter;
        
        // Create converter for Python language
        let mut converter = JsonToIrConverter::new("python");
        
        // Determine struct name
        let struct_name = options.get_struct_name("GeneratedClass");
        let sanitized_struct_name = NameConverter::convert_type_name(&struct_name, "python");
        
        // Convert JSON to intermediate representation
        let struct_def = converter.convert_to_struct(json_value, &sanitized_struct_name)?;
        
        // Generate Python code
        let mut result = String::new();
        
        // Add file header comment if comments are enabled
        if options.include_comments {
            result.push_str(&self.generate_file_header(&sanitized_struct_name));
        }
        
        // Generate imports
        result.push_str(&self.generate_imports(&struct_def));
        
        // Generate nested classes first
        for nested_struct in &struct_def.nested_structs {
            result.push_str(&self.generate_dataclass(nested_struct));
            result.push('\n');
        }
        
        // Generate main class
        result.push_str(&self.generate_dataclass(&struct_def));
        
        Ok(result)
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

    #[test]
    fn test_generate_simple_dataclass() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "age": 30,
            "is_active": true
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_comments(true);
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("from dataclasses import dataclass"));
        assert!(code.contains("@dataclass"));
        assert!(code.contains("class User:"));
        assert!(code.contains("name: str"));
        assert!(code.contains("age: int"));
        assert!(code.contains("is_active: bool"));
        assert!(code.contains("Generated Python dataclasses from JSON data"));
    }

    #[test]
    fn test_generate_with_optional_fields() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "email": null,
            "age": 30
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_optional_fields(true);
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("from typing import Optional"));
        assert!(code.contains("name: str"));
        assert!(code.contains("age: int"));
        // Email should be optional since it was null in JSON
        assert!(code.contains("email: Optional["));
        assert!(code.contains("= None"));
    }

    #[test]
    fn test_generate_with_arrays() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "tags": ["developer", "python", "rust"],
            "scores": [95, 87, 92]
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User");
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("from typing import List"));
        assert!(code.contains("tags: List[str]"));
        assert!(code.contains("scores: List[int]"));
    }

    #[test]
    fn test_generate_with_nested_objects() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "zip_code": 12345
            }
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User");
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("@dataclass"));
        assert!(code.contains("class Address:"));
        assert!(code.contains("class User:"));
        assert!(code.contains("address: Address"));
        assert!(code.contains("street: str"));
        assert!(code.contains("city: str"));
        assert!(code.contains("zip_code: int"));
    }

    #[test]
    fn test_generate_without_comments() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe"
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_comments(false);
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(!code.contains("Generated Python dataclasses from JSON data"));
        assert!(code.contains("@dataclass"));
        assert!(code.contains("class User:"));
        assert!(code.contains("name: str"));
    }

    #[test]
    fn test_generate_with_array_default_factory() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "tags": ["developer", "python"]
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User");
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("from dataclasses import dataclass, field"));
        assert!(code.contains("tags: List[str] = field(default_factory=list)"));
    }

    #[test]
    fn test_generate_enhanced_docstrings() {
        use serde_json::json;
        
        let generator = PythonGenerator::new();
        let json_data = json!({
            "user_id": 123,
            "email": "test@example.com"
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_comments(true);
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("User dataclass."));
        assert!(code.contains("Auto-generated from JSON data."));
        assert!(code.contains("Attributes:"));
        assert!(code.contains("user_id (int): Field from JSON key 'user_id'"));
        assert!(code.contains("email (str): Field from JSON key 'email'"));
    }

    #[test]
    fn test_file_header_generation() {
        let generator = PythonGenerator::new();
        let header = generator.generate_file_header("TestClass");
        
        assert!(header.contains("Generated Python dataclasses from JSON data"));
        assert!(header.contains("Main class: TestClass"));
        assert!(header.contains("DO NOT EDIT"));
        assert!(header.starts_with("\"\"\""));
        assert!(header.ends_with("\"\"\"\n\n"));
    }

    #[test]
    fn test_needs_field_import() {
        let generator = PythonGenerator::new();
        
        // Test with array field (should need field import)
        let field1 = FieldDefinition::new("tags", "tags", FieldType::String).array(true);
        let struct_def1 = StructDefinition::new("User").add_field(field1);
        assert!(generator.needs_field_import(&struct_def1));
        
        // Test with optional array field (should not need field import)
        let field2 = FieldDefinition::new("tags", "tags", FieldType::String)
            .array(true)
            .optional(true);
        let struct_def2 = StructDefinition::new("User").add_field(field2);
        assert!(!generator.needs_field_import(&struct_def2));
        
        // Test with regular fields (should not need field import)
        let field3 = FieldDefinition::new("name", "name", FieldType::String);
        let struct_def3 = StructDefinition::new("User").add_field(field3);
        assert!(!generator.needs_field_import(&struct_def3));
    }

    #[test]
    fn test_complex_python_generation() {
        use serde_json::json;
        
        let json_data = json!({
            "user_id": 123,
            "name": "John Doe", 
            "email": "john@example.com",
            "is_active": true,
            "tags": ["developer", "python", "rust"],
            "profile": {
                "bio": "Software developer",
                "location": "San Francisco", 
                "website": null
            },
            "scores": [95, 87, 92]
        });
        
        let generator = PythonGenerator::new();
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_comments(true);
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        
        // Verify the generated code contains expected elements
        assert!(code.contains("from dataclasses import dataclass, field"));
        assert!(code.contains("from typing import Optional, List"));
        assert!(code.contains("class Profile:"));
        assert!(code.contains("class User:"));
        assert!(code.contains("user_id: int"));
        assert!(code.contains("name: str"));
        assert!(code.contains("email: str"));
        assert!(code.contains("is_active: bool"));
        assert!(code.contains("tags: List[str] = field(default_factory=list)"));
        assert!(code.contains("profile: Profile"));
        assert!(code.contains("scores: List[int] = field(default_factory=list)"));
        // The website field should be optional since it was null in JSON
        assert!(code.contains("website: Optional["));
        assert!(code.contains("= None"));
        
        // Print the generated code for manual inspection
        println!("Generated Python code:\n{}", code);
    }
}