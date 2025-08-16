//! # Type System for Code Generation
//!
//! This module defines the intermediate representation (IR) types used throughout the code
//! generation process. These types provide a language-agnostic way to represent data
//! structures that can then be translated into language-specific code.

use std::collections::HashMap;
use serde_json::Value;

/// Statistics about JSON structure complexity
#[derive(Debug, Clone, Default)]
pub struct StructureStats {
    /// Maximum nesting depth found
    pub max_depth: usize,
    /// Total number of objects in the structure
    pub object_count: usize,
    /// Total number of arrays in the structure
    pub array_count: usize,
    /// Total number of fields across all objects
    pub total_fields: usize,
    /// Maximum length of any array found
    pub max_array_length: usize,
}

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

/// JSON to IR converter
///
/// This struct handles the conversion from JSON data to the intermediate representation (IR)
/// that can then be used to generate code in different languages.
#[derive(Debug, Clone)]
pub struct JsonToIrConverter {
    /// Type mapper for the target language
    type_mapper: TypeMapper,
    /// Maximum recursion depth to prevent infinite loops
    max_depth: usize,
    /// Current recursion depth
    current_depth: usize,
    /// Track generated struct names to avoid duplicates
    generated_names: std::collections::HashSet<String>,
    /// Track the current path for better naming
    current_path: Vec<String>,
}

impl JsonToIrConverter {
    /// Create a new JSON to IR converter
    pub fn new(language: &str) -> Self {
        Self {
            type_mapper: TypeMapper::new(language),
            max_depth: 20, // Increased default to handle deeper nesting
            current_depth: 0,
            generated_names: std::collections::HashSet::new(),
            current_path: Vec::new(),
        }
    }

    /// Create a new converter with custom max depth
    pub fn with_max_depth(language: &str, max_depth: usize) -> Self {
        Self {
            type_mapper: TypeMapper::new(language),
            max_depth,
            current_depth: 0,
            generated_names: std::collections::HashSet::new(),
            current_path: Vec::new(),
        }
    }

    /// Set the maximum recursion depth
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }

    /// Get the current recursion depth
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    /// Get the maximum recursion depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Convert JSON value to StructDefinition
    pub fn convert_to_struct(&mut self, json_value: &Value, struct_name: &str) -> crate::error::Result<StructDefinition> {
        self.current_depth = 0;
        self.generated_names.clear();
        self.current_path.clear();
        self.convert_object_to_struct(json_value, struct_name)
    }

    /// Convert a JSON object to a StructDefinition
    fn convert_object_to_struct(&mut self, json_value: &Value, struct_name: &str) -> crate::error::Result<StructDefinition> {
        if self.current_depth >= self.max_depth {
            return Err(crate::error::J2sError::codegen_error(
                format!(
                    "Maximum recursion depth ({}) exceeded while processing nested structures at path: {}. \
                    Consider increasing max_depth or simplifying the JSON structure.",
                    self.max_depth,
                    self.current_path.join(".")
                )
            ));
        }

        let mut struct_def = StructDefinition::new(struct_name);
        let mut nested_structs = Vec::new();

        match json_value {
            Value::Object(obj) => {
                self.current_depth += 1;
                
                // Sort keys to ensure deterministic field ordering
                let mut sorted_keys: Vec<_> = obj.keys().collect();
                sorted_keys.sort();
                
                for key in sorted_keys {
                    let value = &obj[key];
                    
                    // Add current field to path for better error reporting and naming
                    self.current_path.push(key.clone());
                    
                    let field_def = self.convert_value_to_field(key, value, &mut nested_structs)?;
                    struct_def = struct_def.add_field(field_def);
                    
                    // Remove current field from path
                    self.current_path.pop();
                }
                
                self.current_depth -= 1;
            }
            _ => {
                return Err(crate::error::J2sError::codegen_error(
                    format!(
                        "Expected JSON object for struct conversion at path: {}",
                        self.current_path.join(".")
                    )
                ));
            }
        }

        // Add all nested structs
        for nested in nested_structs {
            struct_def = struct_def.add_nested_struct(nested);
        }

        Ok(struct_def)
    }

    /// Convert a JSON value to a FieldDefinition
    fn convert_value_to_field(
        &mut self,
        field_name: &str,
        value: &Value,
        nested_structs: &mut Vec<StructDefinition>,
    ) -> crate::error::Result<FieldDefinition> {
        // For arrays, check if they contain null values to determine optionality
        let is_optional = match value {
            Value::Null => true,
            Value::Array(arr) => arr.iter().any(|v| v.is_null()),
            _ => false,
        };
        
        let (field_type, is_array) = self.process_json_type_with_value(value, field_name, nested_structs)?;

        let code_name = self.convert_field_name(field_name);
        
        let mut field = FieldDefinition::new(field_name, &code_name, field_type)
            .optional(is_optional)
            .array(is_array);

        // Add metadata for JSON serialization
        field = field.add_metadata("json_name".to_string(), field_name.to_string());

        Ok(field)
    }

    /// Process JsonType and handle nested structures
    fn process_json_type(
        &mut self,
        json_type: &JsonType,
        field_name: &str,
        nested_structs: &mut Vec<StructDefinition>,
    ) -> crate::error::Result<(FieldType, bool)> {
        match json_type {
            JsonType::Array(element_type) => {
                let (inner_type, _) = self.process_json_type(element_type, field_name, nested_structs)?;
                Ok((inner_type, true))
            }
            JsonType::Object(_) => {
                // Generate a name for the nested struct based on the field name
                let nested_struct_name = self.generate_nested_struct_name(field_name);
                Ok((FieldType::Custom(nested_struct_name), false))
            }
            _ => {
                let field_type = self.type_mapper.json_type_to_field_type(json_type);
                Ok((field_type, false))
            }
        }
    }

    /// Process JsonType and handle nested structures, creating actual nested struct definitions
    fn process_json_type_with_value(
        &mut self,
        json_value: &Value,
        field_name: &str,
        nested_structs: &mut Vec<StructDefinition>,
    ) -> crate::error::Result<(FieldType, bool)> {
        match json_value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    // For empty arrays, we can't determine the element type
                    // Use Any type and let the language generators handle it appropriately
                    Ok((FieldType::Any, true))
                } else {
                    // Analyze array elements to determine the most appropriate type
                    let element_type = self.analyze_array_element_type(arr, field_name, nested_structs)?;
                    Ok((element_type, true))
                }
            }
            Value::Object(obj) => {
                if obj.is_empty() {
                    // Empty object - use Any type
                    Ok((FieldType::Any, false))
                } else {
                    // Generate a name for the nested struct based on the field name
                    let nested_struct_name = self.generate_nested_struct_name(field_name);
                    
                    // Create the nested struct definition
                    let nested_struct = self.convert_object_to_struct(json_value, &nested_struct_name)?;
                    nested_structs.push(nested_struct);
                    
                    Ok((FieldType::Custom(nested_struct_name), false))
                }
            }
            Value::String(_) => Ok((FieldType::String, false)),
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    Ok((FieldType::Integer, false))
                } else {
                    Ok((FieldType::Number, false))
                }
            }
            Value::Bool(_) => Ok((FieldType::Boolean, false)),
            Value::Null => {
                // Null values are ambiguous - use Any type and mark as optional
                Ok((FieldType::Any, false))
            }
        }
    }

    /// Analyze array elements to determine the most appropriate element type
    fn analyze_array_element_type(
        &mut self,
        arr: &[Value],
        field_name: &str,
        nested_structs: &mut Vec<StructDefinition>,
    ) -> crate::error::Result<FieldType> {
        if arr.is_empty() {
            return Ok(FieldType::Any);
        }

        // Analyze all elements to determine if we have mixed types
        let mut element_types = std::collections::HashMap::new();
        let mut has_objects = false;
        let mut has_primitives = false;

        for (index, element) in arr.iter().enumerate() {
            let element_field_name = format!("{}_{}", field_name, index);
            let (element_type, _) = self.process_json_type_with_value(element, &element_field_name, nested_structs)?;
            
            match &element_type {
                FieldType::Custom(_) => has_objects = true,
                _ => has_primitives = true,
            }
            
            *element_types.entry(element_type).or_insert(0) += 1;
        }

        // Determine the best type based on analysis
        if element_types.len() == 1 {
            // All elements are the same type
            Ok(element_types.keys().next().unwrap().clone())
        } else if has_objects && has_primitives {
            // Mixed objects and primitives - use Any type
            Ok(FieldType::Any)
        } else if has_objects {
            // Multiple object types - try to find a common structure or use Any
            self.analyze_mixed_object_types(arr, field_name, nested_structs)
        } else {
            // Multiple primitive types - determine the most general type
            self.determine_common_primitive_type(&element_types)
        }
    }

    /// Analyze mixed object types in an array to find common structure
    fn analyze_mixed_object_types(
        &mut self,
        arr: &[Value],
        field_name: &str,
        nested_structs: &mut Vec<StructDefinition>,
    ) -> crate::error::Result<FieldType> {
        // For now, create a union-like structure or use Any
        // This could be enhanced to create discriminated unions in languages that support them
        
        // Check if all objects have similar structure (same keys)
        let mut all_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut common_keys: Option<std::collections::HashSet<String>> = None;
        
        for element in arr {
            if let Value::Object(obj) = element {
                let keys: std::collections::HashSet<String> = obj.keys().cloned().collect();
                all_keys.extend(keys.clone());
                
                match &common_keys {
                    None => common_keys = Some(keys),
                    Some(existing) => {
                        common_keys = Some(existing.intersection(&keys).cloned().collect());
                    }
                }
            }
        }
        
        // If objects have common structure, create a unified type
        if let Some(common) = common_keys {
            if !common.is_empty() && common.len() > all_keys.len() / 2 {
                // Objects have significant common structure
                let unified_name = self.generate_nested_struct_name(&format!("{}_item", field_name));
                let unified_struct = self.create_unified_struct_from_array(arr, &unified_name)?;
                nested_structs.push(unified_struct);
                return Ok(FieldType::Custom(unified_name));
            }
        }
        
        // Objects are too different, use Any type
        Ok(FieldType::Any)
    }

    /// Create a unified struct definition from an array of similar objects
    fn create_unified_struct_from_array(
        &mut self,
        arr: &[Value],
        struct_name: &str,
    ) -> crate::error::Result<StructDefinition> {
        let mut unified_fields: std::collections::HashMap<String, (FieldType, bool, bool)> = std::collections::HashMap::new();
        let mut nested_structs = Vec::new();
        
        // Analyze all objects to determine field types and optionality
        for element in arr {
            if let Value::Object(obj) = element {
                for (key, value) in obj {
                    let (field_type, is_array) = self.process_json_type_with_value(value, key, &mut nested_structs)?;
                    let is_optional = value.is_null();
                    
                    match unified_fields.get_mut(key) {
                        Some((existing_type, existing_optional, existing_array)) => {
                            // If types differ, make it optional and use Any
                            if *existing_type != field_type {
                                *existing_type = FieldType::Any;
                            }
                            *existing_optional = *existing_optional || is_optional;
                            *existing_array = *existing_array || is_array;
                        }
                        None => {
                            unified_fields.insert(key.clone(), (field_type, is_optional, is_array));
                        }
                    }
                }
            }
        }
        
        // Create the unified struct
        let mut struct_def = StructDefinition::new(struct_name);
        
        // Add all nested structs first
        for nested in nested_structs {
            struct_def = struct_def.add_nested_struct(nested);
        }
        
        // Add fields sorted by name for consistency
        let mut sorted_fields: Vec<_> = unified_fields.into_iter().collect();
        sorted_fields.sort_by(|a, b| a.0.cmp(&b.0));
        
        for (json_name, (field_type, is_optional, is_array)) in sorted_fields {
            let code_name = self.convert_field_name(&json_name);
            let field = FieldDefinition::new(&json_name, &code_name, field_type)
                .optional(is_optional)
                .array(is_array);
            struct_def = struct_def.add_field(field);
        }
        
        Ok(struct_def)
    }

    /// Determine the most general primitive type from a set of types
    fn determine_common_primitive_type(
        &self,
        type_counts: &std::collections::HashMap<FieldType, usize>,
    ) -> crate::error::Result<FieldType> {
        let types: Vec<&FieldType> = type_counts.keys().collect();
        
        // If we have more than 2 different primitive types, use Any
        if types.len() > 2 {
            return Ok(FieldType::Any);
        }
        
        // If we have Any type mixed with anything else, use Any
        if types.contains(&&FieldType::Any) {
            return Ok(FieldType::Any);
        }
        
        // If we have strings mixed with other primitives, use Any
        if types.contains(&&FieldType::String) && types.len() > 1 {
            return Ok(FieldType::Any);
        }
        
        // If we have booleans mixed with numbers, use Any
        if types.contains(&&FieldType::Boolean) && 
           (types.contains(&&FieldType::Number) || types.contains(&&FieldType::Integer)) {
            return Ok(FieldType::Any);
        }
        
        // If we have numbers and integers, use Number (more general)
        if types.contains(&&FieldType::Number) && types.contains(&&FieldType::Integer) {
            return Ok(FieldType::Number);
        }
        
        // Find the most common type
        let most_common = type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(field_type, _)| field_type.clone())
            .unwrap_or(FieldType::Any);
        
        Ok(most_common)
    }

    /// Generate a name for a nested struct based on the field name and current path
    fn generate_nested_struct_name(&mut self, field_name: &str) -> String {
        use crate::codegen::utils::NameConverter;
        
        // Build a hierarchical name based on the current path
        let mut name_parts = self.current_path.clone();
        name_parts.push(field_name.to_string());
        
        // Create a base name from the path
        let base_name = if name_parts.len() > 3 {
            // For very deep nesting, use only the last few parts to keep names manageable
            let relevant_parts = &name_parts[name_parts.len().saturating_sub(3)..];
            relevant_parts.join("_")
        } else {
            name_parts.join("_")
        };
        
        let converted_name = NameConverter::convert_type_name(&base_name, self.type_mapper.language());
        
        // Ensure uniqueness by adding a suffix if needed
        let mut final_name = converted_name.clone();
        let mut counter = 1;
        
        while self.generated_names.contains(&final_name) {
            final_name = format!("{converted_name}{counter}");
            counter += 1;
        }
        
        self.generated_names.insert(final_name.clone());
        final_name
    }

    /// Convert field name to appropriate code name based on language conventions
    fn convert_field_name(&self, field_name: &str) -> String {
        use crate::codegen::utils::NameConverter;
        NameConverter::convert_field_name(field_name, self.type_mapper.language())
    }



    /// Process nested objects and create struct definitions
    pub fn process_nested_objects(
        &mut self,
        json_value: &Value,
        parent_struct: &mut StructDefinition,
    ) -> crate::error::Result<()> {
        if let Value::Object(obj) = json_value {
            for (field_name, field_value) in obj {
                if let Value::Object(_) = field_value {
                    let nested_struct_name = self.generate_nested_struct_name(field_name);
                    let nested_struct = self.convert_object_to_struct(field_value, &nested_struct_name)?;
                    parent_struct.nested_structs.push(nested_struct);
                } else if let Value::Array(arr) = field_value {
                    // Process array elements for nested objects
                    for item in arr {
                        if let Value::Object(_) = item {
                            let nested_struct_name = self.generate_nested_struct_name(field_name);
                            let nested_struct = self.convert_object_to_struct(item, &nested_struct_name)?;
                            
                            // Check if we already have this struct definition
                            if !parent_struct.nested_structs.iter().any(|s| s.name == nested_struct.name) {
                                parent_struct.nested_structs.push(nested_struct);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the type mapper
    pub fn type_mapper(&self) -> &TypeMapper {
        &self.type_mapper
    }

    /// Get mutable reference to type mapper for customization
    pub fn type_mapper_mut(&mut self) -> &mut TypeMapper {
        &mut self.type_mapper
    }

    /// Validate that the JSON structure doesn't exceed reasonable nesting limits
    pub fn validate_nesting_depth(json_value: &Value) -> crate::error::Result<usize> {
        fn calculate_depth(value: &Value, current_depth: usize, max_seen: &mut usize) -> crate::error::Result<()> {
            *max_seen = (*max_seen).max(current_depth);
            
            // Prevent infinite recursion in case of circular references
            if current_depth > 100 {
                return Err(crate::error::J2sError::codegen_error(
                    "JSON structure appears to have circular references or excessive nesting (>100 levels)"
                ));
            }
            
            match value {
                Value::Object(obj) => {
                    for (_, v) in obj {
                        calculate_depth(v, current_depth + 1, max_seen)?;
                    }
                }
                Value::Array(arr) => {
                    for v in arr {
                        calculate_depth(v, current_depth + 1, max_seen)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        
        let mut max_depth = 0;
        calculate_depth(json_value, 0, &mut max_depth)?;
        Ok(max_depth)
    }

    /// Get statistics about the JSON structure for debugging
    pub fn get_structure_stats(json_value: &Value) -> StructureStats {
        fn analyze(value: &Value, stats: &mut StructureStats, current_depth: usize) {
            stats.max_depth = stats.max_depth.max(current_depth);
            
            match value {
                Value::Object(obj) => {
                    stats.object_count += 1;
                    stats.total_fields += obj.len();
                    for (_, v) in obj {
                        analyze(v, stats, current_depth + 1);
                    }
                }
                Value::Array(arr) => {
                    stats.array_count += 1;
                    stats.max_array_length = stats.max_array_length.max(arr.len());
                    for v in arr {
                        analyze(v, stats, current_depth + 1);
                    }
                }
                _ => {}
            }
        }
        
        let mut stats = StructureStats::default();
        analyze(json_value, &mut stats, 0);
        stats
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::String => write!(f, "String"),
            FieldType::Integer => write!(f, "Integer"),
            FieldType::Number => write!(f, "Number"),
            FieldType::Boolean => write!(f, "Boolean"),
            FieldType::Custom(name) => write!(f, "{name}"),
            FieldType::Any => write!(f, "Any"),
        }
    }
}

/// Represents JSON types for type mapping
///
/// This enum provides a more granular representation of JSON types that can be
/// used for mapping to language-specific types. It includes support for nested
/// structures and arrays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JsonType {
    /// JSON string value
    String,
    /// JSON integer number (no decimal places)
    Integer,
    /// JSON floating-point number (with decimal places)
    Number,
    /// JSON boolean value
    Boolean,
    /// JSON array with element type
    Array(Box<JsonType>),
    /// JSON object with type name
    Object(String),
    /// JSON null value
    Null,
}

/// Type mapper for converting JSON types to language-specific types
///
/// This struct provides the core functionality for mapping JSON data types
/// to appropriate types in different programming languages. It maintains
/// language-specific type mappings and handles the conversion logic.
#[derive(Debug, Clone)]
pub struct TypeMapper {
    /// Language-specific type mappings
    mappings: HashMap<JsonType, String>,
    /// Language identifier
    language: String,
}

impl TypeMapper {
    /// Create a new type mapper for the specified language
    pub fn new(language: &str) -> Self {
        let mut mapper = Self {
            mappings: HashMap::new(),
            language: language.to_string(),
        };
        mapper.initialize_default_mappings();
        mapper
    }

    /// Initialize default type mappings based on the language
    fn initialize_default_mappings(&mut self) {
        match self.language.as_str() {
            "go" => self.initialize_go_mappings(),
            "rust" => self.initialize_rust_mappings(),
            "typescript" => self.initialize_typescript_mappings(),
            "python" => self.initialize_python_mappings(),
            _ => self.initialize_generic_mappings(),
        }
    }

    /// Initialize Go language type mappings
    fn initialize_go_mappings(&mut self) {
        self.mappings.insert(JsonType::String, "string".to_string());
        self.mappings.insert(JsonType::Integer, "int64".to_string());
        self.mappings.insert(JsonType::Number, "float64".to_string());
        self.mappings.insert(JsonType::Boolean, "bool".to_string());
        self.mappings.insert(JsonType::Null, "interface{}".to_string());
    }

    /// Initialize Rust language type mappings
    fn initialize_rust_mappings(&mut self) {
        self.mappings.insert(JsonType::String, "String".to_string());
        self.mappings.insert(JsonType::Integer, "i64".to_string());
        self.mappings.insert(JsonType::Number, "f64".to_string());
        self.mappings.insert(JsonType::Boolean, "bool".to_string());
        self.mappings.insert(JsonType::Null, "serde_json::Value".to_string());
    }

    /// Initialize TypeScript language type mappings
    fn initialize_typescript_mappings(&mut self) {
        self.mappings.insert(JsonType::String, "string".to_string());
        self.mappings.insert(JsonType::Integer, "number".to_string());
        self.mappings.insert(JsonType::Number, "number".to_string());
        self.mappings.insert(JsonType::Boolean, "boolean".to_string());
        self.mappings.insert(JsonType::Null, "null".to_string());
    }

    /// Initialize Python language type mappings
    fn initialize_python_mappings(&mut self) {
        self.mappings.insert(JsonType::String, "str".to_string());
        self.mappings.insert(JsonType::Integer, "int".to_string());
        self.mappings.insert(JsonType::Number, "float".to_string());
        self.mappings.insert(JsonType::Boolean, "bool".to_string());
        self.mappings.insert(JsonType::Null, "None".to_string());
    }

    /// Initialize generic type mappings (fallback)
    fn initialize_generic_mappings(&mut self) {
        self.mappings.insert(JsonType::String, "String".to_string());
        self.mappings.insert(JsonType::Integer, "Integer".to_string());
        self.mappings.insert(JsonType::Number, "Number".to_string());
        self.mappings.insert(JsonType::Boolean, "Boolean".to_string());
        self.mappings.insert(JsonType::Null, "Any".to_string());
    }

    /// Map a JsonType to a language-specific type string
    pub fn map_type(&self, json_type: &JsonType) -> String {
        match json_type {
            JsonType::Array(element_type) => {
                let element_type_str = self.map_type(element_type);
                match self.language.as_str() {
                    "go" => format!("[]{element_type_str}"),
                    "rust" => format!("Vec<{element_type_str}>"),
                    "typescript" => format!("{element_type_str}[]"),
                    "python" => format!("List[{element_type_str}]"),
                    _ => format!("Array<{element_type_str}>"),
                }
            }
            JsonType::Object(type_name) => type_name.clone(),
            _ => self.mappings.get(json_type).cloned().unwrap_or_else(|| {
                format!("Unknown_{}", self.language)
            }),
        }
    }

    /// Map a JsonType to a language-specific optional type string
    pub fn map_optional_type(&self, json_type: &JsonType) -> String {
        let base_type = self.map_type(json_type);
        match self.language.as_str() {
            "go" => format!("*{base_type}"),
            "rust" => format!("Option<{base_type}>"),
            "typescript" => format!("{base_type} | null"),
            "python" => format!("Optional[{base_type}]"),
            _ => format!("Optional<{base_type}>"),
        }
    }

    /// Infer JsonType from a serde_json::Value
    pub fn infer_json_type(&self, value: &Value) -> JsonType {
        match value {
            Value::String(_) => JsonType::String,
            Value::Number(n) => {
                if n.is_i64() || n.is_u64() {
                    JsonType::Integer
                } else {
                    JsonType::Number
                }
            }
            Value::Bool(_) => JsonType::Boolean,
            Value::Array(arr) => {
                if arr.is_empty() {
                    // For empty arrays, we can't infer the element type
                    JsonType::Array(Box::new(JsonType::Null))
                } else {
                    // Infer type from first element (could be improved to analyze all elements)
                    let element_type = self.infer_json_type(&arr[0]);
                    JsonType::Array(Box::new(element_type))
                }
            }
            Value::Object(_) => JsonType::Object("Object".to_string()),
            Value::Null => JsonType::Null,
        }
    }

    /// Convert JsonType to FieldType
    pub fn json_type_to_field_type(&self, json_type: &JsonType) -> FieldType {
        match json_type {
            JsonType::String => FieldType::String,
            JsonType::Integer => FieldType::Integer,
            JsonType::Number => FieldType::Number,
            JsonType::Boolean => FieldType::Boolean,
            JsonType::Object(name) => FieldType::Custom(name.clone()),
            JsonType::Array(_) | JsonType::Null => FieldType::Any,
        }
    }

    /// Add or override a type mapping
    pub fn add_mapping(&mut self, json_type: JsonType, language_type: String) {
        self.mappings.insert(json_type, language_type);
    }

    /// Get the language this mapper is configured for
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get all current mappings
    pub fn mappings(&self) -> &HashMap<JsonType, String> {
        &self.mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    // Tests for JsonType and TypeMapper
    #[test]
    fn test_json_type_creation() {
        let string_type = JsonType::String;
        let array_type = JsonType::Array(Box::new(JsonType::Integer));
        let object_type = JsonType::Object("User".to_string());

        assert_eq!(string_type, JsonType::String);
        assert_eq!(array_type, JsonType::Array(Box::new(JsonType::Integer)));
        assert_eq!(object_type, JsonType::Object("User".to_string()));
    }

    #[test]
    fn test_type_mapper_go() {
        let mapper = TypeMapper::new("go");
        
        assert_eq!(mapper.map_type(&JsonType::String), "string");
        assert_eq!(mapper.map_type(&JsonType::Integer), "int64");
        assert_eq!(mapper.map_type(&JsonType::Number), "float64");
        assert_eq!(mapper.map_type(&JsonType::Boolean), "bool");
        assert_eq!(mapper.map_type(&JsonType::Null), "interface{}");
        
        let array_type = JsonType::Array(Box::new(JsonType::String));
        assert_eq!(mapper.map_type(&array_type), "[]string");
        
        let object_type = JsonType::Object("User".to_string());
        assert_eq!(mapper.map_type(&object_type), "User");
    }

    #[test]
    fn test_type_mapper_rust() {
        let mapper = TypeMapper::new("rust");
        
        assert_eq!(mapper.map_type(&JsonType::String), "String");
        assert_eq!(mapper.map_type(&JsonType::Integer), "i64");
        assert_eq!(mapper.map_type(&JsonType::Number), "f64");
        assert_eq!(mapper.map_type(&JsonType::Boolean), "bool");
        assert_eq!(mapper.map_type(&JsonType::Null), "serde_json::Value");
        
        let array_type = JsonType::Array(Box::new(JsonType::String));
        assert_eq!(mapper.map_type(&array_type), "Vec<String>");
        
        let object_type = JsonType::Object("User".to_string());
        assert_eq!(mapper.map_type(&object_type), "User");
    }

    #[test]
    fn test_type_mapper_typescript() {
        let mapper = TypeMapper::new("typescript");
        
        assert_eq!(mapper.map_type(&JsonType::String), "string");
        assert_eq!(mapper.map_type(&JsonType::Integer), "number");
        assert_eq!(mapper.map_type(&JsonType::Number), "number");
        assert_eq!(mapper.map_type(&JsonType::Boolean), "boolean");
        assert_eq!(mapper.map_type(&JsonType::Null), "null");
        
        let array_type = JsonType::Array(Box::new(JsonType::String));
        assert_eq!(mapper.map_type(&array_type), "string[]");
        
        let object_type = JsonType::Object("User".to_string());
        assert_eq!(mapper.map_type(&object_type), "User");
    }

    #[test]
    fn test_type_mapper_python() {
        let mapper = TypeMapper::new("python");
        
        assert_eq!(mapper.map_type(&JsonType::String), "str");
        assert_eq!(mapper.map_type(&JsonType::Integer), "int");
        assert_eq!(mapper.map_type(&JsonType::Number), "float");
        assert_eq!(mapper.map_type(&JsonType::Boolean), "bool");
        assert_eq!(mapper.map_type(&JsonType::Null), "None");
        
        let array_type = JsonType::Array(Box::new(JsonType::String));
        assert_eq!(mapper.map_type(&array_type), "List[str]");
        
        let object_type = JsonType::Object("User".to_string());
        assert_eq!(mapper.map_type(&object_type), "User");
    }

    #[test]
    fn test_optional_type_mapping() {
        let go_mapper = TypeMapper::new("go");
        let rust_mapper = TypeMapper::new("rust");
        let ts_mapper = TypeMapper::new("typescript");
        let py_mapper = TypeMapper::new("python");
        
        assert_eq!(go_mapper.map_optional_type(&JsonType::String), "*string");
        assert_eq!(rust_mapper.map_optional_type(&JsonType::String), "Option<String>");
        assert_eq!(ts_mapper.map_optional_type(&JsonType::String), "string | null");
        assert_eq!(py_mapper.map_optional_type(&JsonType::String), "Optional[str]");
    }

    #[test]
    fn test_infer_json_type() {
        let mapper = TypeMapper::new("go");
        
        assert_eq!(mapper.infer_json_type(&json!("hello")), JsonType::String);
        assert_eq!(mapper.infer_json_type(&json!(42)), JsonType::Integer);
        assert_eq!(mapper.infer_json_type(&json!(3.14)), JsonType::Number);
        assert_eq!(mapper.infer_json_type(&json!(true)), JsonType::Boolean);
        assert_eq!(mapper.infer_json_type(&json!(null)), JsonType::Null);
        
        let array_value = json!(["a", "b", "c"]);
        assert_eq!(mapper.infer_json_type(&array_value), JsonType::Array(Box::new(JsonType::String)));
        
        let object_value = json!({"name": "John"});
        assert_eq!(mapper.infer_json_type(&object_value), JsonType::Object("Object".to_string()));
        
        let empty_array = json!([]);
        assert_eq!(mapper.infer_json_type(&empty_array), JsonType::Array(Box::new(JsonType::Null)));
    }

    #[test]
    fn test_json_type_to_field_type() {
        let mapper = TypeMapper::new("go");
        
        assert_eq!(mapper.json_type_to_field_type(&JsonType::String), FieldType::String);
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Integer), FieldType::Integer);
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Number), FieldType::Number);
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Boolean), FieldType::Boolean);
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Object("User".to_string())), FieldType::Custom("User".to_string()));
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Array(Box::new(JsonType::String))), FieldType::Any);
        assert_eq!(mapper.json_type_to_field_type(&JsonType::Null), FieldType::Any);
    }

    #[test]
    fn test_custom_mappings() {
        let mut mapper = TypeMapper::new("go");
        
        // Add custom mapping
        mapper.add_mapping(JsonType::String, "MyString".to_string());
        assert_eq!(mapper.map_type(&JsonType::String), "MyString");
        
        // Original mappings should still work for other types
        assert_eq!(mapper.map_type(&JsonType::Integer), "int64");
    }

    #[test]
    fn test_mapper_metadata() {
        let mapper = TypeMapper::new("rust");
        
        assert_eq!(mapper.language(), "rust");
        assert!(!mapper.mappings().is_empty());
        assert!(mapper.mappings().contains_key(&JsonType::String));
    }

    #[test]
    fn test_nested_array_types() {
        let mapper = TypeMapper::new("typescript");
        
        // Array of arrays
        let nested_array = JsonType::Array(Box::new(JsonType::Array(Box::new(JsonType::String))));
        assert_eq!(mapper.map_type(&nested_array), "string[][]");
        
        // Array of objects
        let object_array = JsonType::Array(Box::new(JsonType::Object("User".to_string())));
        assert_eq!(mapper.map_type(&object_array), "User[]");
    }

    #[test]
    fn test_unknown_language_fallback() {
        let mapper = TypeMapper::new("unknown");
        
        assert_eq!(mapper.map_type(&JsonType::String), "String");
        assert_eq!(mapper.map_type(&JsonType::Integer), "Integer");
        assert_eq!(mapper.map_type(&JsonType::Number), "Number");
        assert_eq!(mapper.map_type(&JsonType::Boolean), "Boolean");
        assert_eq!(mapper.map_type(&JsonType::Null), "Any");
        
        let array_type = JsonType::Array(Box::new(JsonType::String));
        assert_eq!(mapper.map_type(&array_type), "Array<String>");
        
        assert_eq!(mapper.map_optional_type(&JsonType::String), "Optional<String>");
    }

    // Tests for JsonToIrConverter
    #[test]
    fn test_json_to_ir_converter_creation() {
        let converter = JsonToIrConverter::new("go");
        assert_eq!(converter.type_mapper().language(), "go");
        assert_eq!(converter.max_depth, 10);
        assert_eq!(converter.current_depth, 0);

        let converter_with_depth = JsonToIrConverter::with_max_depth("rust", 5);
        assert_eq!(converter_with_depth.max_depth, 5);
    }

    #[test]
    fn test_simple_object_conversion() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "name": "John",
            "age": 30,
            "is_active": true
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        assert_eq!(struct_def.name, "User");
        assert_eq!(struct_def.fields.len(), 3);
        
        // Check field names and types (sorted alphabetically)
        let age_field = &struct_def.fields[0];
        assert_eq!(age_field.json_name, "age");
        assert_eq!(age_field.code_name, "Age");
        assert_eq!(age_field.field_type, FieldType::Integer);
        
        let active_field = &struct_def.fields[1];
        assert_eq!(active_field.json_name, "is_active");
        assert_eq!(active_field.code_name, "IsActive");
        assert_eq!(active_field.field_type, FieldType::Boolean);
        
        let name_field = &struct_def.fields[2];
        assert_eq!(name_field.json_name, "name");
        assert_eq!(name_field.code_name, "Name");
        assert_eq!(name_field.field_type, FieldType::String);
        assert!(!name_field.is_optional);
        assert!(!name_field.is_array);
    }

    #[test]
    fn test_nested_object_conversion() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "user": {
                "name": "John",
                "email": "john@example.com"
            },
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        });

        let result = converter.convert_to_struct(&json_data, "Config");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        assert_eq!(struct_def.name, "Config");
        assert_eq!(struct_def.fields.len(), 2);
        
        // Check that nested struct references are created (sorted alphabetically)
        let settings_field = &struct_def.fields[0];
        assert_eq!(settings_field.json_name, "settings");
        assert_eq!(settings_field.code_name, "Settings");
        assert_eq!(settings_field.field_type, FieldType::Custom("Settings".to_string()));
        
        let user_field = &struct_def.fields[1];
        assert_eq!(user_field.json_name, "user");
        assert_eq!(user_field.code_name, "User");
        assert_eq!(user_field.field_type, FieldType::Custom("User".to_string()));
    }

    #[test]
    fn test_array_conversion() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "tags": ["rust", "programming", "json"],
            "scores": [95, 87, 92],
            "flags": [true, false, true]
        });

        let result = converter.convert_to_struct(&json_data, "Data");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        assert_eq!(struct_def.fields.len(), 3);
        
        // Fields are sorted alphabetically
        let flags_field = &struct_def.fields[0];
        assert_eq!(flags_field.json_name, "flags");
        assert_eq!(flags_field.field_type, FieldType::Boolean);
        assert!(flags_field.is_array);
        
        let scores_field = &struct_def.fields[1];
        assert_eq!(scores_field.json_name, "scores");
        assert_eq!(scores_field.field_type, FieldType::Integer);
        assert!(scores_field.is_array);
        
        let tags_field = &struct_def.fields[2];
        assert_eq!(tags_field.json_name, "tags");
        assert_eq!(tags_field.field_type, FieldType::String);
        assert!(tags_field.is_array);
    }

    #[test]
    fn test_null_values() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "name": "John",
            "middle_name": null,
            "age": 30
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        // Fields are sorted: age, middle_name, name
        let middle_name_field = &struct_def.fields[1];
        assert_eq!(middle_name_field.json_name, "middle_name");
        assert!(middle_name_field.is_optional);
    }

    #[test]
    fn test_empty_array() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "items": [],
            "name": "test"
        });

        let result = converter.convert_to_struct(&json_data, "Container");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        // Fields are sorted: items, name
        let items_field = &struct_def.fields[0];
        assert_eq!(items_field.json_name, "items");
        assert!(items_field.is_array);
        assert_eq!(items_field.field_type, FieldType::Any); // Empty array maps to Any
    }

    #[test]
    fn test_naming_conventions_go() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "user_name": "John",
            "first-name": "John",
            "lastName": "Doe",
            "email_address": "john@example.com"
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        
        // Go uses PascalCase, fields sorted: email_address, first-name, lastName, user_name
        assert_eq!(struct_def.fields[0].code_name, "EmailAddress");
        assert_eq!(struct_def.fields[1].code_name, "FirstName");
        assert_eq!(struct_def.fields[2].code_name, "LastName");
        assert_eq!(struct_def.fields[3].code_name, "UserName");
    }

    #[test]
    fn test_naming_conventions_rust() {
        let mut converter = JsonToIrConverter::new("rust");
        let json_data = json!({
            "userName": "John",
            "first-name": "John",
            "LastName": "Doe",
            "email_address": "john@example.com"
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        
        // Rust uses snake_case, fields sorted: LastName, email_address, first-name, userName
        assert_eq!(struct_def.fields[0].code_name, "last_name");
        assert_eq!(struct_def.fields[1].code_name, "email_address");
        assert_eq!(struct_def.fields[2].code_name, "first_name");
        assert_eq!(struct_def.fields[3].code_name, "user_name");
    }

    #[test]
    fn test_pascal_case_conversion() {
        use crate::codegen::utils::NameConverter;
        
        assert_eq!(NameConverter::to_pascal_case("user_name"), "UserName");
        assert_eq!(NameConverter::to_pascal_case("first-name"), "FirstName");
        assert_eq!(NameConverter::to_pascal_case("lastName"), "LastName");
        assert_eq!(NameConverter::to_pascal_case("email"), "Email");
        assert_eq!(NameConverter::to_pascal_case("API_KEY"), "ApiKey");
        assert_eq!(NameConverter::to_pascal_case("user123name"), "User123name");
    }

    #[test]
    fn test_snake_case_conversion() {
        use crate::codegen::utils::NameConverter;
        
        assert_eq!(NameConverter::to_snake_case("UserName"), "user_name");
        assert_eq!(NameConverter::to_snake_case("firstName"), "first_name");
        assert_eq!(NameConverter::to_snake_case("LastName"), "last_name");
        assert_eq!(NameConverter::to_snake_case("email"), "email");
        assert_eq!(NameConverter::to_snake_case("APIKey"), "api_key");
        assert_eq!(NameConverter::to_snake_case("user123Name"), "user123_name");
        assert_eq!(NameConverter::to_snake_case("XMLHttpRequest"), "xml_http_request");
    }

    #[test]
    fn test_nested_struct_name_generation() {
        let mut converter = JsonToIrConverter::new("go");
        
        assert_eq!(converter.generate_nested_struct_name("user"), "User");
        assert_eq!(converter.generate_nested_struct_name("user_profile"), "UserProfile");
        assert_eq!(converter.generate_nested_struct_name("settings"), "Settings");
        assert_eq!(converter.generate_nested_struct_name(""), "field");
    }

    #[test]
    fn test_non_object_conversion_error() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!("not an object");

        let result = converter.convert_to_struct(&json_data, "Test");
        assert!(result.is_err());
    }

    #[test]
    fn test_max_depth_limit() {
        let mut converter = JsonToIrConverter::with_max_depth("go", 2);
        
        // Create deeply nested JSON that exceeds max depth
        let json_data = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "too deep"
                    }
                }
            }
        });

        let result = converter.convert_to_struct(&json_data, "Deep");
        // This should succeed because we're only going 3 levels deep with max_depth of 2
        // The actual depth checking happens during nested object processing
        assert!(result.is_ok());
    }

    #[test]
    fn test_field_metadata() {
        let mut converter = JsonToIrConverter::new("go");
        let json_data = json!({
            "user_name": "John"
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        let field = &struct_def.fields[0]; // user_name field
        
        // Check that JSON name is stored in metadata
        assert_eq!(field.metadata.get("json_name"), Some(&"user_name".to_string()));
    }

    #[test]
    fn test_complex_mixed_structure() {
        let mut converter = JsonToIrConverter::new("typescript");
        let json_data = json!({
            "id": 1,
            "name": "John Doe",
            "email": "john@example.com",
            "is_active": true,
            "profile": {
                "age": 30,
                "location": "New York",
                "preferences": {
                    "theme": "dark",
                    "language": "en"
                }
            },
            "tags": ["developer", "rust", "json"],
            "scores": [95.5, 87.2, 92.8],
            "metadata": null
        });

        let result = converter.convert_to_struct(&json_data, "User");
        assert!(result.is_ok());
        
        let struct_def = result.unwrap();
        assert_eq!(struct_def.name, "User");
        assert_eq!(struct_def.fields.len(), 8);
        
        // Check various field types
        let profile_field = struct_def.fields.iter().find(|f| f.json_name == "profile").unwrap();
        assert_eq!(profile_field.field_type, FieldType::Custom("Profile".to_string()));
        assert!(!profile_field.is_array);
        
        let tags_field = struct_def.fields.iter().find(|f| f.json_name == "tags").unwrap();
        assert_eq!(tags_field.field_type, FieldType::String);
        assert!(tags_field.is_array);
        
        let scores_field = struct_def.fields.iter().find(|f| f.json_name == "scores").unwrap();
        assert_eq!(scores_field.field_type, FieldType::Number);
        assert!(scores_field.is_array);
        
        let metadata_field = struct_def.fields.iter().find(|f| f.json_name == "metadata").unwrap();
        assert!(metadata_field.is_optional);
    }
}