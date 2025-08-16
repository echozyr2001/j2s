//! # Go Language Code Generator
//!
//! This module implements the code generator for the Go programming language.
//! It generates Go structs with appropriate JSON tags, type mappings, and
//! follows Go naming conventions and best practices.

use crate::codegen::comments::{CommentGenerator, GoCommentGenerator};
use crate::codegen::generator::{CodeGenerator, GenerationOptions};
use crate::codegen::types::{FieldDefinition, FieldType, StructDefinition};
use crate::codegen::utils::{NameConverter, escape_comment_string};
use crate::error::Result;
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
    /// Comment generator for Go-specific comments
    comment_generator: GoCommentGenerator,
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
        
        Self { 
            keywords,
            comment_generator: GoCommentGenerator,
        }
    }

    /// Map a FieldType to the appropriate Go type string
    fn map_field_type(&self, field_type: &FieldType, is_optional: bool, is_array: bool) -> String {
        let base_type = match field_type {
            FieldType::String => "string",
            FieldType::Integer => "int64",
            FieldType::Number => "float64",
            FieldType::Boolean => "bool",
            FieldType::Custom(name) => name,
            FieldType::Any => {
                if is_array {
                    // For arrays with mixed types, use interface{} as element type
                    "interface{}"
                } else {
                    // For single Any fields, use interface{}
                    "interface{}"
                }
            }
        };

        let mut result = base_type.to_string();

        // Handle arrays
        if is_array {
            result = format!("[]{result}");
        }

        // Handle optional fields with pointers
        if is_optional {
            result = format!("*{result}");
        }

        result
    }

    /// Generate a Go struct field declaration
    fn generate_field(&self, field: &FieldDefinition, include_comments: bool) -> String {
        // The code_name is already converted to the proper case by JsonToIrConverter
        let sanitized_name = NameConverter::sanitize_identifier(&field.code_name, &self.keywords);
        
        let field_type = self.map_field_type(&field.field_type, field.is_optional, field.is_array);
        
        // Generate JSON tag
        let json_tag = if field.is_optional {
            format!("`json:\"{},omitempty\"`", field.json_name)
        } else {
            format!("`json:\"{}\"`", field.json_name)
        };

        // Add comments if enabled
        let mut result = String::new();
        if include_comments {
            if !field.comments.is_empty() {
                for comment in &field.comments {
                    let escaped_comment = escape_comment_string(comment);
                    result.push_str(&format!("\t// {escaped_comment}\n"));
                }
            } else {
                // Generate automatic field comment using enhanced inference
                use crate::codegen::comments::utils::infer_field_description;
                let description = infer_field_description(&field.json_name, &field_type);
                let field_comment = self.comment_generator.generate_field_comment(
                    &field.json_name, 
                    &field_type, 
                    Some(&description)
                );
                result.push_str(&field_comment);
            }
        }

        result.push_str(&format!("\t{sanitized_name} {field_type} {json_tag}"));
        result
    }

    /// Generate file header with generation information
    fn generate_file_header(&self) -> String {
        use crate::codegen::comments::utils::current_timestamp;
        let timestamp = current_timestamp();
        self.comment_generator.generate_file_header("j2s", &timestamp)
    }

    /// Order nested structs to ensure dependencies are defined before use
    fn order_structs_by_dependency<'a>(&self, main_struct: &'a StructDefinition) -> Vec<&'a StructDefinition> {
        let mut ordered = Vec::new();
        let mut visited = HashSet::new();
        
        // Simple topological sort - for now just return in original order
        // This can be enhanced later if circular dependencies become an issue
        for nested in &main_struct.nested_structs {
            if !visited.contains(&nested.name) {
                ordered.push(nested);
                visited.insert(&nested.name);
            }
        }
        
        ordered
    }

    /// Generate a complete Go struct definition
    fn generate_struct(&self, struct_def: &StructDefinition, include_comments: bool) -> String {
        let struct_name = NameConverter::to_pascal_case(&struct_def.name);
        let sanitized_name = NameConverter::sanitize_identifier(&struct_name, &self.keywords);

        let mut result = String::new();

        // Add struct comments if enabled
        if include_comments {
            if !struct_def.comments.is_empty() {
                for comment in &struct_def.comments {
                    let escaped_comment = escape_comment_string(comment);
                    result.push_str(&format!("// {escaped_comment}\n"));
                }
            } else {
                // Generate automatic struct comment
                let struct_comment = self.comment_generator.generate_struct_comment(&sanitized_name, None);
                result.push_str(&struct_comment);
            }
        }

        // Start struct definition
        result.push_str(&format!("type {sanitized_name} struct {{\n"));

        // Add fields
        for field in &struct_def.fields {
            result.push_str(&self.generate_field(field, include_comments));
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
    fn generate(&self, json_value: &Value, options: &GenerationOptions) -> Result<String> {
        use crate::codegen::types::JsonToIrConverter;
        
        // Validate nesting depth before processing
        let max_depth = JsonToIrConverter::validate_nesting_depth(json_value)?;
        if max_depth > 50 {
            return Err(crate::error::J2sError::codegen_error(
                format!(
                    "JSON structure is too deeply nested ({} levels). Consider simplifying the structure or increasing the recursion limit.",
                    max_depth
                )
            ));
        }
        
        // Get structure statistics for better error reporting
        let stats = JsonToIrConverter::get_structure_stats(json_value);
        
        // Create converter for Go language with appropriate max depth
        let mut converter = if max_depth > 20 {
            JsonToIrConverter::with_max_depth("go", max_depth + 5)
        } else {
            JsonToIrConverter::new("go")
        };
        
        // Determine struct name
        let struct_name = options.get_struct_name("GeneratedStruct");
        let sanitized_struct_name = NameConverter::convert_type_name(&struct_name, "go");
        
        // Convert JSON to intermediate representation
        let struct_def = converter.convert_to_struct(json_value, &sanitized_struct_name)?;
        
        // Generate Go code
        let mut result = String::new();
        
        // Add file header comment if comments are enabled
        if options.include_comments {
            result.push_str(&self.generate_file_header());
            
            // Add structure complexity information as comments
            if stats.max_depth > 5 || stats.object_count > 10 {
                result.push_str(&format!(
                    "// Structure complexity: {} levels deep, {} objects, {} arrays, {} total fields\n",
                    stats.max_depth, stats.object_count, stats.array_count, stats.total_fields
                ));
            }
        }
        
        // Add package declaration
        let package_name = options.get_language_option("package").unwrap_or(&"main".to_string()).clone();
        result.push_str(&format!("package {}\n\n", package_name));
        
        // Generate nested structs first (in dependency order)
        let ordered_structs = self.order_structs_by_dependency(&struct_def);
        for nested_struct in &ordered_structs {
            result.push_str(&self.generate_struct(nested_struct, options.include_comments));
            result.push('\n');
        }
        
        // Generate main struct
        result.push_str(&self.generate_struct(&struct_def, options.include_comments));
        
        Ok(result)
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
        
        // The code_name should already be converted to Go naming convention
        let field = FieldDefinition::new("user_name", "UserName", FieldType::String)
            .add_comment("The user's name");
        
        let result = generator.generate_field(&field, true);
        assert!(result.contains("UserName"));
        assert!(result.contains("string"));
        assert!(result.contains("`json:\"user_name\"`"));
        assert!(result.contains("// The user's name"));
    }

    #[test]
    fn test_generate_field_optional() {
        let generator = GoGenerator::new();
        
        // The code_name should already be converted to Go naming convention
        let field = FieldDefinition::new("email", "Email", FieldType::String)
            .optional(true);
        
        let result = generator.generate_field(&field, true);
        assert!(result.contains("Email"));
        assert!(result.contains("*string"));
        assert!(result.contains("`json:\"email,omitempty\"`"));
    }

    #[test]
    fn test_generate_field_array() {
        let generator = GoGenerator::new();
        
        // The code_name should already be converted to Go naming convention
        let field = FieldDefinition::new("tags", "Tags", FieldType::String)
            .array(true);
        
        let result = generator.generate_field(&field, true);
        assert!(result.contains("Tags"));
        assert!(result.contains("[]string"));
        assert!(result.contains("`json:\"tags\"`"));
    }

    #[test]
    fn test_generate_struct() {
        let generator = GoGenerator::new();
        
        // The code_name should already be converted to Go naming convention
        let field1 = FieldDefinition::new("id", "Id", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "Name", FieldType::String);
        
        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_comment("User represents a user in the system");
        
        let result = generator.generate_struct(&struct_def, true);
        assert!(result.contains("type User struct {"));
        assert!(result.contains("Id int64"));
        assert!(result.contains("Name string"));
        assert!(result.contains("// User represents a user in the system"));
        assert!(result.ends_with("}\n"));
    }

    #[test]
    fn test_keyword_sanitization() {
        let generator = GoGenerator::new();
        
        // The code_name should already be converted and sanitized
        let field = FieldDefinition::new("type", "Type_", FieldType::String);
        let result = generator.generate_field(&field, true);
        
        // Should be sanitized to avoid Go keyword conflict
        assert!(result.contains("Type_"));
    }

    #[test]
    fn test_validate_options() {
        let generator = GoGenerator::new();
        let options = GenerationOptions::default();
        
        assert!(generator.validate_options(&options).is_ok());
    }

    #[test]
    fn test_generate_simple_struct() {
        use serde_json::json;
        
        let generator = GoGenerator::new();
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
        assert!(code.contains("package main"));
        assert!(code.contains("type User struct"));
        assert!(code.contains("Name string"));
        assert!(code.contains("Age int64"));
        assert!(code.contains("IsActive bool"));
        assert!(code.contains("`json:\"name\"`"));
        assert!(code.contains("`json:\"age\"`"));
        assert!(code.contains("`json:\"is_active\"`"));
    }

    #[test]
    fn test_generate_with_optional_fields() {
        use serde_json::json;
        
        let generator = GoGenerator::new();
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
        assert!(code.contains("Name string"));
        assert!(code.contains("Age int64"));
        // Email should be optional since it was null in JSON
        assert!(code.contains("Email *"));
        assert!(code.contains("`json:\"email,omitempty\"`"));
    }

    #[test]
    fn test_generate_with_arrays() {
        use serde_json::json;
        
        let generator = GoGenerator::new();
        let json_data = json!({
            "name": "John Doe",
            "tags": ["developer", "rust", "go"],
            "scores": [95, 87, 92]
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User");
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("Tags []string"));
        assert!(code.contains("Scores []int64"));
        assert!(code.contains("`json:\"tags\"`"));
        assert!(code.contains("`json:\"scores\"`"));
    }

    #[test]
    fn test_generate_with_custom_package() {
        use serde_json::json;
        
        let generator = GoGenerator::new();
        let json_data = json!({
            "name": "John Doe"
        });
        
        let options = GenerationOptions::default()
            .with_struct_name("User")
            .with_language_option("package", "models");
        
        let result = generator.generate(&json_data, &options);
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert!(code.contains("package models"));
    }
}