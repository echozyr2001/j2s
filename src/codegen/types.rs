//! # Type System for Code Generation
//!
//! This module defines the intermediate representation (IR) types used throughout the code
//! generation process. These types provide a language-agnostic way to represent data
//! structures that can then be translated into language-specific code.

use std::collections::HashMap;

/// Represents a complete struct/class/interface definition
///
/// This is the top-level container for a generated type definition. It includes
/// the type name, its fields, any nested type definitions, and associated metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    /// The name of the struct/class/interface
    pub name: String,

    /// The fields/properties that belong to this type
    pub fields: Vec<FieldDefinition>,

    /// Nested struct definitions that are referenced by this struct
    ///
    /// These are typically generated from nested JSON objects and are defined
    /// as separate types that can be referenced by name.
    pub nested_structs: Vec<StructDefinition>,

    /// Comments and documentation for this struct
    ///
    /// These will be formatted according to the target language's documentation
    /// conventions (e.g., /// in Rust, // in Go, /** */ in TypeScript).
    pub comments: Vec<String>,

    /// Additional metadata for language-specific generation
    ///
    /// This allows storing language-specific information that doesn't fit into
    /// the common structure, such as derive attributes for Rust or package
    /// information for Go.
    pub metadata: HashMap<String, String>,
}

/// Represents a single field/property within a struct
///
/// This captures all the information needed to generate a field declaration
/// in the target language, including its name, type, optionality, and documentation.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    /// The name of the field as it appears in the JSON
    pub json_name: String,

    /// The name of the field as it should appear in the generated code
    ///
    /// This may be different from json_name due to naming convention conversions
    /// (e.g., "user_name" in JSON might become "UserName" in Go or "userName" in TypeScript).
    pub code_name: String,

    /// The type of this field
    pub field_type: FieldType,

    /// Whether this field is optional (may be null or missing)
    ///
    /// This affects how the field is represented in different languages:
    /// - Go: pointer type (*T) or omitempty tag
    /// - Rust: Option<T>
    /// - TypeScript: optional property (field?: T) or union with null
    /// - Python: Optional[T]
    pub is_optional: bool,

    /// Whether this field represents an array/list
    ///
    /// When true, the field_type represents the element type, and the actual
    /// field type is an array/slice/vector of that type.
    pub is_array: bool,

    /// Comments and documentation for this field
    pub comments: Vec<String>,

    /// Additional metadata for language-specific generation
    pub metadata: HashMap<String, String>,
}

/// Represents the different types that can be inferred from JSON data
///
/// This enum provides a language-agnostic representation of types that can be
/// mapped to appropriate types in each target language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// String type (JSON string)
    String,

    /// Integer type (JSON number without decimal places)
    ///
    /// Different languages may map this to different specific integer types
    /// (e.g., int64 in Go, i64 in Rust, number in TypeScript, int in Python).
    Integer,

    /// Floating-point number type (JSON number with decimal places)
    Number,

    /// Boolean type (JSON boolean)
    Boolean,

    /// Reference to a custom/nested type
    ///
    /// The string contains the name of the referenced type, which should
    /// correspond to a StructDefinition in the nested_structs collection.
    Custom(String),

    /// Any/unknown type for cases where type inference is ambiguous
    ///
    /// This is used when the JSON structure doesn't provide enough information
    /// to determine a specific type (e.g., null values, empty arrays).
    Any,
}

impl StructDefinition {
    /// Create a new struct definition with the given name
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            nested_structs: Vec::new(),
            comments: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a field to this struct
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }

    /// Add a nested struct definition
    pub fn add_nested_struct(mut self, nested: StructDefinition) -> Self {
        self.nested_structs.push(nested);
        self
    }

    /// Add a comment to this struct
    pub fn add_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comments.push(comment.into());
        self
    }

    /// Add metadata to this struct
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this struct has any fields
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Get all nested struct names referenced by this struct's fields
    pub fn get_referenced_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        for field in &self.fields {
            if let FieldType::Custom(type_name) = &field.field_type {
                if !types.contains(type_name) {
                    types.push(type_name.clone());
                }
            }
        }
        types
    }
}

impl FieldDefinition {
    /// Create a new field definition
    pub fn new<S: Into<String>>(json_name: S, code_name: S, field_type: FieldType) -> Self {
        Self {
            json_name: json_name.into(),
            code_name: code_name.into(),
            field_type,
            is_optional: false,
            is_array: false,
            comments: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set whether this field is optional
    pub fn optional(mut self, optional: bool) -> Self {
        self.is_optional = optional;
        self
    }

    /// Set whether this field is an array
    pub fn array(mut self, is_array: bool) -> Self {
        self.is_array = is_array;
        self
    }

    /// Add a comment to this field
    pub fn add_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comments.push(comment.into());
        self
    }

    /// Add metadata to this field
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl FieldType {
    /// Check if this type represents a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            FieldType::String | FieldType::Integer | FieldType::Number | FieldType::Boolean
        )
    }

    /// Check if this type represents a custom/complex type
    pub fn is_custom(&self) -> bool {
        matches!(self, FieldType::Custom(_))
    }

    /// Get the name of a custom type, if this is a custom type
    pub fn custom_type_name(&self) -> Option<&str> {
        match self {
            FieldType::Custom(name) => Some(name),
            _ => None,
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::String => write!(f, "String"),
            FieldType::Integer => write!(f, "Integer"),
            FieldType::Number => write!(f, "Number"),
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::Custom(name) => write!(f, "{}", name),
            FieldType::Any => write!(f, "Any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_definition_creation() {
        let struct_def = StructDefinition::new("TestStruct")
            .add_comment("A test struct")
            .add_metadata("package", "main");

        assert_eq!(struct_def.name, "TestStruct");
        assert_eq!(struct_def.comments, vec!["A test struct"]);
        assert_eq!(struct_def.metadata.get("package"), Some(&"main".to_string()));
        assert!(struct_def.is_empty());
    }

    #[test]
    fn test_field_definition_creation() {
        let field = FieldDefinition::new("user_name", "UserName", FieldType::String)
            .optional(true)
            .array(false)
            .add_comment("The user's name")
            .add_metadata("json_tag", "user_name");

        assert_eq!(field.json_name, "user_name");
        assert_eq!(field.code_name, "UserName");
        assert_eq!(field.field_type, FieldType::String);
        assert!(field.is_optional);
        assert!(!field.is_array);
        assert_eq!(field.comments, vec!["The user's name"]);
        assert_eq!(field.metadata.get("json_tag"), Some(&"user_name".to_string()));
    }

    #[test]
    fn test_struct_with_fields() {
        let field1 = FieldDefinition::new("id", "ID", FieldType::Integer);
        let field2 = FieldDefinition::new("name", "Name", FieldType::String);

        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2);

        assert!(!struct_def.is_empty());
        assert_eq!(struct_def.fields.len(), 2);
        assert_eq!(struct_def.fields[0].json_name, "id");
        assert_eq!(struct_def.fields[1].json_name, "name");
    }

    #[test]
    fn test_nested_structs() {
        let nested = StructDefinition::new("Address");
        let struct_def = StructDefinition::new("User").add_nested_struct(nested);

        assert_eq!(struct_def.nested_structs.len(), 1);
        assert_eq!(struct_def.nested_structs[0].name, "Address");
    }

    #[test]
    fn test_get_referenced_types() {
        let field1 = FieldDefinition::new("id", "ID", FieldType::Integer);
        let field2 = FieldDefinition::new("address", "Address", FieldType::Custom("Address".to_string()));
        let field3 = FieldDefinition::new("company", "Company", FieldType::Custom("Company".to_string()));
        let field4 = FieldDefinition::new("backup_address", "BackupAddress", FieldType::Custom("Address".to_string()));

        let struct_def = StructDefinition::new("User")
            .add_field(field1)
            .add_field(field2)
            .add_field(field3)
            .add_field(field4);

        let referenced_types = struct_def.get_referenced_types();
        assert_eq!(referenced_types.len(), 2);
        assert!(referenced_types.contains(&"Address".to_string()));
        assert!(referenced_types.contains(&"Company".to_string()));
    }

    #[test]
    fn test_field_type_methods() {
        assert!(FieldType::String.is_primitive());
        assert!(FieldType::Integer.is_primitive());
        assert!(FieldType::Number.is_primitive());
        assert!(FieldType::Boolean.is_primitive());
        assert!(!FieldType::Custom("Test".to_string()).is_primitive());
        assert!(!FieldType::Any.is_primitive());

        assert!(!FieldType::String.is_custom());
        assert!(FieldType::Custom("Test".to_string()).is_custom());

        assert_eq!(FieldType::Custom("Test".to_string()).custom_type_name(), Some("Test"));
        assert_eq!(FieldType::String.custom_type_name(), None);
    }

    #[test]
    fn test_field_type_display() {
        assert_eq!(format!("{}", FieldType::String), "String");
        assert_eq!(format!("{}", FieldType::Integer), "Integer");
        assert_eq!(format!("{}", FieldType::Number), "Number");
        assert_eq!(format!("{}", FieldType::Boolean), "Boolean");
        assert_eq!(format!("{}", FieldType::Custom("CustomType".to_string())), "CustomType");
        assert_eq!(format!("{}", FieldType::Any), "Any");
    }
}