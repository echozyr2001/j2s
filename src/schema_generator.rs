use serde::Serialize;
use std::collections::HashMap;

/// Represents a JSON Schema structure according to JSON Schema Draft 2020-12
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct JsonSchema {
    /// The JSON Schema version identifier
    #[serde(rename = "$schema", skip_serializing_if = "String::is_empty")]
    pub schema: String,

    /// The type of the schema
    #[serde(rename = "type")]
    pub type_name: SchemaType,

    /// Properties for object types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonSchema>>,

    /// Items schema for array types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,

    /// Required properties for object types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Optional title for the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Optional description for the schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Represents the different types supported in JSON Schema
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    Object,
    Array,
    String,
    Number,
    Integer,
    Boolean,
    Null,
}

impl JsonSchema {
    /// Creates a new JsonSchema with the specified type
    pub fn new(type_name: SchemaType) -> Self {
        Self {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            type_name,
            properties: None,
            items: None,
            required: None,
            title: None,
            description: None,
        }
    }

    /// Creates a new JsonSchema without the $schema field (for nested schemas)
    pub fn new_nested(type_name: SchemaType) -> Self {
        Self {
            schema: String::new(),
            type_name,
            properties: None,
            items: None,
            required: None,
            title: None,
            description: None,
        }
    }

    /// Creates a new object schema with properties
    pub fn new_object(properties: HashMap<String, JsonSchema>, required: Vec<String>) -> Self {
        Self {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            type_name: SchemaType::Object,
            properties: Some(properties),
            items: None,
            required: if required.is_empty() {
                None
            } else {
                Some(required)
            },
            title: None,
            description: None,
        }
    }

    /// Creates a new nested object schema with properties (without $schema field)
    pub fn new_nested_object(
        properties: HashMap<String, JsonSchema>,
        required: Vec<String>,
    ) -> Self {
        Self {
            schema: String::new(),
            type_name: SchemaType::Object,
            properties: Some(properties),
            items: None,
            required: if required.is_empty() {
                None
            } else {
                Some(required)
            },
            title: None,
            description: None,
        }
    }

    /// Creates a new array schema with items type
    pub fn new_array(items: JsonSchema) -> Self {
        Self {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            type_name: SchemaType::Array,
            properties: None,
            items: Some(Box::new(items)),
            required: None,
            title: None,
            description: None,
        }
    }

    /// Creates a new nested array schema with items type (without $schema field)
    pub fn new_nested_array(items: JsonSchema) -> Self {
        Self {
            schema: String::new(),
            type_name: SchemaType::Array,
            properties: None,
            items: Some(Box::new(items)),
            required: None,
            title: None,
            description: None,
        }
    }

    /// Sets the title of the schema
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Sets the description of the schema
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Maximum recursion depth to prevent stack overflow
const MAX_RECURSION_DEPTH: usize = 100;

/// Generates a JSON Schema from a JSON value
///
/// This function analyzes the structure of the provided JSON value and creates
/// a corresponding JSON Schema that describes the data structure, types, and constraints.
///
/// # Arguments
/// * `json_value` - The JSON value to analyze and generate a schema for
///
/// # Returns
/// A `JsonSchema` struct representing the schema for the input JSON
///
/// # Performance
/// For large or deeply nested JSON structures, this function includes optimizations:
/// - Recursion depth limiting to prevent stack overflow
/// - Efficient type inference and deduplication
/// - Memory-conscious processing of large arrays and objects
pub fn generate_schema(json_value: &serde_json::Value) -> JsonSchema {
    generate_schema_with_depth(json_value, 0, true)
}

/// Generates a JSON Schema with progress indication for large structures
///
/// This function provides the same functionality as `generate_schema` but includes
/// progress callbacks for processing large JSON structures.
///
/// # Arguments
/// * `json_value` - The JSON value to analyze
/// * `show_progress` - Whether to show progress messages
///
/// # Returns
/// A `JsonSchema` struct representing the schema for the input JSON
pub fn generate_schema_with_progress(
    json_value: &serde_json::Value,
    show_progress: bool,
) -> JsonSchema {
    if show_progress {
        println!("   📊 Progress: 0% - Starting schema generation...");
    }

    let schema = generate_schema_with_depth_and_progress(json_value, 0, true, show_progress);

    if show_progress {
        println!("   📊 Progress: 100% - Schema generation complete");
    }

    schema
}

/// Internal function that generates a JSON Schema with recursion depth tracking
fn generate_schema_with_depth(
    json_value: &serde_json::Value,
    depth: usize,
    is_root: bool,
) -> JsonSchema {
    generate_schema_with_depth_and_progress(json_value, depth, is_root, false)
}

/// Internal function that generates a JSON Schema with recursion depth tracking and progress indication
fn generate_schema_with_depth_and_progress(
    json_value: &serde_json::Value,
    depth: usize,
    is_root: bool,
    show_progress: bool,
) -> JsonSchema {
    // Check recursion depth to prevent stack overflow
    if depth > MAX_RECURSION_DEPTH {
        if show_progress {
            println!(
                "   📊 Progress: 90% - Maximum recursion depth reached - creating fallback schema"
            );
        }

        // Return a generic schema when max depth is reached
        if is_root {
            return JsonSchema::new(SchemaType::Object).with_description(
                "Schema generation stopped due to maximum recursion depth".to_string(),
            );
        } else {
            return JsonSchema::new_nested(SchemaType::Object);
        }
    }

    match json_value {
        serde_json::Value::Object(obj) => {
            if show_progress && depth < 3 {
                let progress = ((depth as f32 / MAX_RECURSION_DEPTH as f32) * 80.0) as usize;
                println!(
                    "   📊 Progress: {}% - Processing object at depth {depth} with {} properties",
                    progress,
                    obj.len()
                );
            }

            let processed_schema =
                process_object_with_depth_and_progress(obj, depth + 1, show_progress);
            if is_root {
                // For root schema, add the $schema field
                JsonSchema {
                    schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
                    type_name: processed_schema.type_name,
                    properties: processed_schema.properties,
                    items: processed_schema.items,
                    required: processed_schema.required,
                    title: processed_schema.title,
                    description: processed_schema.description,
                }
            } else {
                processed_schema
            }
        }
        serde_json::Value::Array(arr) => {
            if show_progress && depth < 3 {
                let progress = ((depth as f32 / MAX_RECURSION_DEPTH as f32) * 80.0) as usize;
                println!(
                    "   📊 Progress: {}% - Processing array at depth {depth} with {} elements",
                    progress,
                    arr.len()
                );
            }

            let processed_schema =
                process_array_with_depth_and_progress(arr, depth + 1, show_progress);
            if is_root {
                // For root schema, add the $schema field
                JsonSchema {
                    schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
                    type_name: processed_schema.type_name,
                    properties: processed_schema.properties,
                    items: processed_schema.items,
                    required: processed_schema.required,
                    title: processed_schema.title,
                    description: processed_schema.description,
                }
            } else {
                processed_schema
            }
        }
        _ => {
            // For primitive types
            let schema_type = infer_type(json_value);
            if is_root {
                JsonSchema::new(schema_type)
            } else {
                JsonSchema::new_nested(schema_type)
            }
        }
    }
}

/// Infers the schema type from a JSON value
fn infer_type(value: &serde_json::Value) -> SchemaType {
    match value {
        serde_json::Value::Null => SchemaType::Null,
        serde_json::Value::Bool(_) => SchemaType::Boolean,
        serde_json::Value::Number(n) => {
            // Distinguish between integer and floating point numbers
            if n.is_i64() || n.is_u64() {
                SchemaType::Integer
            } else {
                SchemaType::Number
            }
        }
        serde_json::Value::String(_) => SchemaType::String,
        serde_json::Value::Array(_) => SchemaType::Array,
        serde_json::Value::Object(_) => SchemaType::Object,
    }
}

/// Processes a JSON object and generates its schema (for testing)
#[cfg(test)]
fn process_object(obj: &serde_json::Map<String, serde_json::Value>) -> JsonSchema {
    process_object_with_depth_and_progress(obj, 0, false)
}

/// Processes a JSON object and generates its schema with depth tracking and progress indication
fn process_object_with_depth_and_progress(
    obj: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    show_progress: bool,
) -> JsonSchema {
    let mut properties = HashMap::new();
    let mut required = Vec::new();
    let total_props = obj.len();

    // Process each property in the object
    for (index, (key, value)) in obj.iter().enumerate() {
        // Update progress for large objects
        if show_progress && total_props > 100 && index % 20 == 0 {
            let progress = (index * 80 / total_props) + 10;
            println!(
                "   📊 Progress: {}% - Processing property '{key}' ({}/{total_props})",
                progress,
                index + 1
            );
        }

        // Recursively generate schema for each property using depth-aware function
        let property_schema =
            generate_schema_with_depth_and_progress(value, depth, false, show_progress);

        properties.insert(key.clone(), property_schema);

        // Add to required list if the value is not null
        if !value.is_null() {
            required.push(key.clone());
        }
    }

    JsonSchema::new_nested_object(properties, required)
}

/// Processes a JSON array and generates its schema (for testing)
#[cfg(test)]
fn process_array(arr: &[serde_json::Value]) -> JsonSchema {
    process_array_with_depth_and_progress(arr, 0, false)
}

/// Processes a JSON array and generates its schema with depth tracking and progress indication
fn process_array_with_depth_and_progress(
    arr: &[serde_json::Value],
    depth: usize,
    show_progress: bool,
) -> JsonSchema {
    // Handle empty array case
    if arr.is_empty() {
        // For empty arrays, we can't infer the item type, so we create a generic array schema
        // The items field will be None, which means any type is allowed
        return JsonSchema {
            schema: String::new(),
            type_name: SchemaType::Array,
            properties: None,
            items: None,
            required: None,
            title: None,
            description: None,
        };
    }

    let total_items = arr.len();

    // For very large arrays, sample items instead of processing all
    let sample_size = if total_items > 10000 {
        if show_progress {
            println!(
                "   📊 Progress: 20% - Large array detected ({total_items} items) - using sampling for performance"
            );
        }
        1000 // Sample first 1000 items for very large arrays
    } else {
        total_items
    };

    // Collect all unique types in the array (or sample)
    let mut type_schemas: Vec<JsonSchema> = Vec::new();
    let mut seen_types: Vec<String> = Vec::new();

    for (index, value) in arr.iter().take(sample_size).enumerate() {
        // Update progress for large arrays
        if show_progress && sample_size > 100 && index % 100 == 0 {
            let progress = (index * 60 / sample_size) + 20;
            println!(
                "   📊 Progress: {}% - Analyzing array item {}/{sample_size}",
                progress,
                index + 1
            );
        }

        // Use depth-aware schema generation
        let item_schema =
            generate_schema_with_depth_and_progress(value, depth, false, show_progress);

        // Create a unique key for this schema to avoid duplicates
        let schema_key = format!("{item_schema:?}");

        if !seen_types.contains(&schema_key) {
            seen_types.push(schema_key);
            type_schemas.push(item_schema);
        }

        // Early exit if we've found enough unique types for mixed arrays
        if type_schemas.len() > 5 {
            if show_progress {
                println!(
                    "   📊 Progress: 80% - Multiple types detected - using first type for schema"
                );
            }
            break;
        }
    }

    // If all items have the same type, use that type directly
    if type_schemas.len() == 1 {
        JsonSchema::new_nested_array(type_schemas.into_iter().next().unwrap())
    } else {
        // For mixed types, we need to use anyOf
        // For now, we'll use the first type as a fallback
        // TODO: Implement proper anyOf support in a future enhancement
        if show_progress {
            println!(
                "   📊 Progress: 85% - Mixed array types detected ({} unique types) - using primary type",
                type_schemas.len()
            );
        }
        JsonSchema::new_nested_array(type_schemas.into_iter().next().unwrap())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_type_serialization() {
        // Test that SchemaType serializes to lowercase strings
        assert_eq!(
            serde_json::to_string(&SchemaType::Object).unwrap(),
            "\"object\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::Array).unwrap(),
            "\"array\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::String).unwrap(),
            "\"string\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::Number).unwrap(),
            "\"number\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::Integer).unwrap(),
            "\"integer\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::Boolean).unwrap(),
            "\"boolean\""
        );
        assert_eq!(
            serde_json::to_string(&SchemaType::Null).unwrap(),
            "\"null\""
        );
    }

    #[test]
    fn test_json_schema_new() {
        let schema = JsonSchema::new(SchemaType::String);

        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema.type_name, SchemaType::String);
        assert!(schema.properties.is_none());
        assert!(schema.items.is_none());
        assert!(schema.required.is_none());
        assert!(schema.title.is_none());
        assert!(schema.description.is_none());
    }

    #[test]
    fn test_json_schema_new_object() {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));
        properties.insert("age".to_string(), JsonSchema::new(SchemaType::Integer));

        let required = vec!["name".to_string(), "age".to_string()];
        let schema = JsonSchema::new_object(properties.clone(), required.clone());

        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(schema.properties.unwrap(), properties);
        assert_eq!(schema.required.unwrap(), required);
        assert!(schema.items.is_none());
    }

    #[test]
    fn test_json_schema_new_object_empty_required() {
        let mut properties = HashMap::new();
        properties.insert("optional".to_string(), JsonSchema::new(SchemaType::String));

        let schema = JsonSchema::new_object(properties, vec![]);

        assert_eq!(schema.type_name, SchemaType::Object);
        assert!(schema.required.is_none()); // Empty required should be None
    }

    #[test]
    fn test_json_schema_new_array() {
        let items_schema = JsonSchema::new(SchemaType::String);
        let schema = JsonSchema::new_array(items_schema.clone());

        assert_eq!(schema.type_name, SchemaType::Array);
        assert_eq!(*schema.items.unwrap(), items_schema);
        assert!(schema.properties.is_none());
        assert!(schema.required.is_none());
    }

    #[test]
    fn test_json_schema_with_title() {
        let schema = JsonSchema::new(SchemaType::String).with_title("User Name".to_string());

        assert_eq!(schema.title.unwrap(), "User Name");
    }

    #[test]
    fn test_json_schema_with_description() {
        let schema = JsonSchema::new(SchemaType::String)
            .with_description("The user's full name".to_string());

        assert_eq!(schema.description.unwrap(), "The user's full name");
    }

    #[test]
    fn test_json_schema_chaining() {
        let schema = JsonSchema::new(SchemaType::String)
            .with_title("User Name".to_string())
            .with_description("The user's full name".to_string());

        assert_eq!(schema.title.unwrap(), "User Name");
        assert_eq!(schema.description.unwrap(), "The user's full name");
    }

    #[test]
    fn test_json_schema_serialization_simple() {
        let schema = JsonSchema::new(SchemaType::String);
        let json = serde_json::to_value(&schema).unwrap();

        assert_eq!(
            json["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(json["type"], "string");

        // Optional fields should not be present when None
        assert!(!json.as_object().unwrap().contains_key("properties"));
        assert!(!json.as_object().unwrap().contains_key("items"));
        assert!(!json.as_object().unwrap().contains_key("required"));
        assert!(!json.as_object().unwrap().contains_key("title"));
        assert!(!json.as_object().unwrap().contains_key("description"));
    }

    #[test]
    fn test_json_schema_serialization_object() {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));
        properties.insert("age".to_string(), JsonSchema::new(SchemaType::Integer));

        let required = vec!["name".to_string()];
        let schema = JsonSchema::new_object(properties, required)
            .with_title("User".to_string())
            .with_description("A user object".to_string());

        let json = serde_json::to_value(&schema).unwrap();

        assert_eq!(
            json["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(json["type"], "object");
        assert_eq!(json["title"], "User");
        assert_eq!(json["description"], "A user object");
        assert_eq!(json["required"], serde_json::json!(["name"]));

        let properties_obj = json["properties"].as_object().unwrap();
        assert_eq!(properties_obj["name"]["type"], "string");
        assert_eq!(properties_obj["age"]["type"], "integer");
    }

    #[test]
    fn test_json_schema_serialization_array() {
        let items_schema = JsonSchema::new(SchemaType::String);
        let schema = JsonSchema::new_array(items_schema);

        let json = serde_json::to_value(&schema).unwrap();

        assert_eq!(
            json["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(json["type"], "array");
        assert_eq!(json["items"]["type"], "string");
    }

    #[test]
    fn test_json_schema_equality() {
        let schema1 = JsonSchema::new(SchemaType::String);
        let schema2 = JsonSchema::new(SchemaType::String);
        let schema3 = JsonSchema::new(SchemaType::Integer);

        assert_eq!(schema1, schema2);
        assert_ne!(schema1, schema3);
    }

    #[test]
    fn test_nested_object_schema() {
        // Test creating a nested object schema
        let mut address_properties = HashMap::new();
        address_properties.insert("street".to_string(), JsonSchema::new(SchemaType::String));
        address_properties.insert("city".to_string(), JsonSchema::new(SchemaType::String));

        let address_schema = JsonSchema::new_object(
            address_properties,
            vec!["street".to_string(), "city".to_string()],
        );

        let mut user_properties = HashMap::new();
        user_properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));
        user_properties.insert("address".to_string(), address_schema);

        let user_schema = JsonSchema::new_object(user_properties, vec!["name".to_string()]);

        let json = serde_json::to_value(&user_schema).unwrap();

        assert_eq!(json["type"], "object");
        assert_eq!(json["properties"]["name"]["type"], "string");
        assert_eq!(json["properties"]["address"]["type"], "object");
        assert_eq!(
            json["properties"]["address"]["properties"]["street"]["type"],
            "string"
        );
        assert_eq!(
            json["properties"]["address"]["properties"]["city"]["type"],
            "string"
        );
    }

    #[test]
    fn test_array_of_objects_schema() {
        // Test creating an array of objects schema
        let mut item_properties = HashMap::new();
        item_properties.insert("id".to_string(), JsonSchema::new(SchemaType::Integer));
        item_properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));

        let item_schema =
            JsonSchema::new_object(item_properties, vec!["id".to_string(), "name".to_string()]);

        let array_schema = JsonSchema::new_array(item_schema);

        let json = serde_json::to_value(&array_schema).unwrap();

        assert_eq!(json["type"], "array");
        assert_eq!(json["items"]["type"], "object");
        assert_eq!(json["items"]["properties"]["id"]["type"], "integer");
        assert_eq!(json["items"]["properties"]["name"]["type"], "string");
    }

    // Type inference tests
    #[test]
    fn test_infer_type_null() {
        let value = serde_json::Value::Null;
        assert_eq!(infer_type(&value), SchemaType::Null);
    }

    #[test]
    fn test_infer_type_boolean() {
        let value_true = serde_json::json!(true);
        let value_false = serde_json::json!(false);

        assert_eq!(infer_type(&value_true), SchemaType::Boolean);
        assert_eq!(infer_type(&value_false), SchemaType::Boolean);
    }

    #[test]
    fn test_infer_type_string() {
        let value = serde_json::json!("hello world");
        assert_eq!(infer_type(&value), SchemaType::String);

        let empty_string = serde_json::json!("");
        assert_eq!(infer_type(&empty_string), SchemaType::String);

        let unicode_string = serde_json::json!("你好世界");
        assert_eq!(infer_type(&unicode_string), SchemaType::String);
    }

    #[test]
    fn test_infer_type_integer() {
        // Test positive integers
        let positive_int = serde_json::json!(42);
        assert_eq!(infer_type(&positive_int), SchemaType::Integer);

        // Test negative integers
        let negative_int = serde_json::json!(-123);
        assert_eq!(infer_type(&negative_int), SchemaType::Integer);

        // Test zero
        let zero = serde_json::json!(0);
        assert_eq!(infer_type(&zero), SchemaType::Integer);

        // Test large integers
        let large_int = serde_json::json!(9223372036854775807i64);
        assert_eq!(infer_type(&large_int), SchemaType::Integer);

        // Test unsigned integers
        let unsigned_int = serde_json::json!(18446744073709551615u64);
        assert_eq!(infer_type(&unsigned_int), SchemaType::Integer);
    }

    #[test]
    fn test_infer_type_number() {
        // Test floating point numbers
        let float = serde_json::json!(3.14);
        assert_eq!(infer_type(&float), SchemaType::Number);

        let negative_float = serde_json::json!(-2.718);
        assert_eq!(infer_type(&negative_float), SchemaType::Number);

        // Test scientific notation
        let scientific = serde_json::json!(1.23e-4);
        assert_eq!(infer_type(&scientific), SchemaType::Number);

        // Test very small decimal
        let small_decimal = serde_json::json!(0.001);
        assert_eq!(infer_type(&small_decimal), SchemaType::Number);
    }

    #[test]
    fn test_infer_type_array() {
        let empty_array = serde_json::json!([]);
        assert_eq!(infer_type(&empty_array), SchemaType::Array);

        let string_array = serde_json::json!(["a", "b", "c"]);
        assert_eq!(infer_type(&string_array), SchemaType::Array);

        let number_array = serde_json::json!([1, 2, 3]);
        assert_eq!(infer_type(&number_array), SchemaType::Array);

        let mixed_array = serde_json::json!([1, "hello", true, null]);
        assert_eq!(infer_type(&mixed_array), SchemaType::Array);

        let nested_array = serde_json::json!([[1, 2], [3, 4]]);
        assert_eq!(infer_type(&nested_array), SchemaType::Array);
    }

    #[test]
    fn test_infer_type_object() {
        let empty_object = serde_json::json!({});
        assert_eq!(infer_type(&empty_object), SchemaType::Object);

        let simple_object = serde_json::json!({
            "name": "John",
            "age": 30
        });
        assert_eq!(infer_type(&simple_object), SchemaType::Object);

        let nested_object = serde_json::json!({
            "user": {
                "profile": {
                    "name": "John"
                }
            }
        });
        assert_eq!(infer_type(&nested_object), SchemaType::Object);

        let complex_object = serde_json::json!({
            "name": "John",
            "age": 30,
            "active": true,
            "scores": [95, 87, 92],
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            },
            "metadata": null
        });
        assert_eq!(infer_type(&complex_object), SchemaType::Object);
    }

    #[test]
    fn test_infer_type_edge_cases() {
        // Test edge cases for number distinction

        // Numbers that look like integers but are stored as floats
        let whole_number_float = serde_json::json!(42.0);
        // Note: serde_json may represent 42.0 as an integer internally
        // The actual behavior depends on how serde_json parses the number
        let inferred_type = infer_type(&whole_number_float);
        // This could be either Integer or Number depending on serde_json's internal representation
        assert!(inferred_type == SchemaType::Integer || inferred_type == SchemaType::Number);

        // Very large numbers that might overflow
        let very_large = serde_json::json!(1.7976931348623157e+308);
        assert_eq!(infer_type(&very_large), SchemaType::Number);

        // Very small numbers
        let very_small = serde_json::json!(2.2250738585072014e-308);
        assert_eq!(infer_type(&very_small), SchemaType::Number);
    }

    #[test]
    fn test_infer_type_comprehensive() {
        // Test a comprehensive JSON structure with all types
        let complex_json = serde_json::json!({
            "null_field": null,
            "boolean_field": true,
            "string_field": "hello",
            "integer_field": 42,
            "number_field": 3.14,
            "array_field": [1, 2, 3],
            "object_field": {
                "nested": "value"
            }
        });

        // Test the root object
        assert_eq!(infer_type(&complex_json), SchemaType::Object);

        // Test individual fields
        if let serde_json::Value::Object(obj) = &complex_json {
            assert_eq!(infer_type(&obj["null_field"]), SchemaType::Null);
            assert_eq!(infer_type(&obj["boolean_field"]), SchemaType::Boolean);
            assert_eq!(infer_type(&obj["string_field"]), SchemaType::String);
            assert_eq!(infer_type(&obj["integer_field"]), SchemaType::Integer);
            assert_eq!(infer_type(&obj["number_field"]), SchemaType::Number);
            assert_eq!(infer_type(&obj["array_field"]), SchemaType::Array);
            assert_eq!(infer_type(&obj["object_field"]), SchemaType::Object);
        }
    }

    // Object processing tests
    #[test]
    fn test_process_object_empty() {
        let empty_obj = serde_json::Map::new();
        let schema = process_object(&empty_obj);

        assert_eq!(schema.type_name, SchemaType::Object);
        assert!(schema.properties.is_some());
        assert!(schema.properties.as_ref().unwrap().is_empty());
        assert!(schema.required.is_none()); // Empty required should be None
        assert!(schema.schema.is_empty()); // Nested schema should not have $schema field
    }

    #[test]
    fn test_process_object_simple() {
        let json_obj = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "active": true
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);

            assert_eq!(schema.type_name, SchemaType::Object);
            assert!(schema.properties.is_some());
            assert!(schema.schema.is_empty()); // Nested schema should not have $schema field

            let properties = schema.properties.as_ref().unwrap();
            assert_eq!(properties.len(), 3);

            // Check property types
            assert_eq!(properties["name"].type_name, SchemaType::String);
            assert_eq!(properties["age"].type_name, SchemaType::Integer);
            assert_eq!(properties["active"].type_name, SchemaType::Boolean);

            // Check that all properties are required (none are null)
            let required = schema.required.as_ref().unwrap();
            assert_eq!(required.len(), 3);
            assert!(required.contains(&"name".to_string()));
            assert!(required.contains(&"age".to_string()));
            assert!(required.contains(&"active".to_string()));

            // Check that nested properties don't have $schema field
            assert!(properties["name"].schema.is_empty());
            assert!(properties["age"].schema.is_empty());
            assert!(properties["active"].schema.is_empty());
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_process_object_with_null_values() {
        let json_obj = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "email": null,
            "active": true
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);

            assert_eq!(schema.type_name, SchemaType::Object);

            let properties = schema.properties.as_ref().unwrap();
            assert_eq!(properties.len(), 4);

            // Check that null property has correct type
            assert_eq!(properties["email"].type_name, SchemaType::Null);

            // Check that only non-null properties are required
            let required = schema.required.as_ref().unwrap();
            assert_eq!(required.len(), 3);
            assert!(required.contains(&"name".to_string()));
            assert!(required.contains(&"age".to_string()));
            assert!(required.contains(&"active".to_string()));
            assert!(!required.contains(&"email".to_string()));
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_process_object_nested() {
        let json_obj = serde_json::json!({
            "user": {
                "name": "John Doe",
                "profile": {
                    "bio": "Software developer",
                    "age": 30
                }
            },
            "active": true
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);

            assert_eq!(schema.type_name, SchemaType::Object);

            let properties = schema.properties.as_ref().unwrap();
            assert_eq!(properties.len(), 2);

            // Check root level properties
            assert_eq!(properties["user"].type_name, SchemaType::Object);
            assert_eq!(properties["active"].type_name, SchemaType::Boolean);

            // Check nested user object
            let user_properties = properties["user"].properties.as_ref().unwrap();
            assert_eq!(user_properties.len(), 2);
            assert_eq!(user_properties["name"].type_name, SchemaType::String);
            assert_eq!(user_properties["profile"].type_name, SchemaType::Object);

            // Check deeply nested profile object
            let profile_properties = user_properties["profile"].properties.as_ref().unwrap();
            assert_eq!(profile_properties.len(), 2);
            assert_eq!(profile_properties["bio"].type_name, SchemaType::String);
            assert_eq!(profile_properties["age"].type_name, SchemaType::Integer);

            // Check required fields at each level
            let root_required = schema.required.as_ref().unwrap();
            assert!(root_required.contains(&"user".to_string()));
            assert!(root_required.contains(&"active".to_string()));

            let user_required = properties["user"].required.as_ref().unwrap();
            assert!(user_required.contains(&"name".to_string()));
            assert!(user_required.contains(&"profile".to_string()));

            let profile_required = user_properties["profile"].required.as_ref().unwrap();
            assert!(profile_required.contains(&"bio".to_string()));
            assert!(profile_required.contains(&"age".to_string()));

            // Verify nested schemas don't have $schema field
            assert!(properties["user"].schema.is_empty());
            assert!(user_properties["profile"].schema.is_empty());
            assert!(profile_properties["bio"].schema.is_empty());
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_process_object_all_types() {
        let json_obj = serde_json::json!({
            "null_field": null,
            "boolean_field": true,
            "string_field": "hello",
            "integer_field": 42,
            "number_field": 3.14159,
            "array_field": [1, 2, 3],
            "object_field": {
                "nested": "value"
            }
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);

            assert_eq!(schema.type_name, SchemaType::Object);

            let properties = schema.properties.as_ref().unwrap();
            assert_eq!(properties.len(), 7);

            // Check all property types
            assert_eq!(properties["null_field"].type_name, SchemaType::Null);
            assert_eq!(properties["boolean_field"].type_name, SchemaType::Boolean);
            assert_eq!(properties["string_field"].type_name, SchemaType::String);
            assert_eq!(properties["integer_field"].type_name, SchemaType::Integer);
            assert_eq!(properties["number_field"].type_name, SchemaType::Number);
            assert_eq!(properties["array_field"].type_name, SchemaType::Array);
            assert_eq!(properties["object_field"].type_name, SchemaType::Object);

            // Check required fields (all except null_field)
            let required = schema.required.as_ref().unwrap();
            assert_eq!(required.len(), 6);
            assert!(!required.contains(&"null_field".to_string()));
            assert!(required.contains(&"boolean_field".to_string()));
            assert!(required.contains(&"string_field".to_string()));
            assert!(required.contains(&"integer_field".to_string()));
            assert!(required.contains(&"number_field".to_string()));
            assert!(required.contains(&"array_field".to_string()));
            assert!(required.contains(&"object_field".to_string()));
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_process_object_only_null_values() {
        let json_obj = serde_json::json!({
            "field1": null,
            "field2": null,
            "field3": null
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);

            assert_eq!(schema.type_name, SchemaType::Object);

            let properties = schema.properties.as_ref().unwrap();
            assert_eq!(properties.len(), 3);

            // All properties should be null type
            assert_eq!(properties["field1"].type_name, SchemaType::Null);
            assert_eq!(properties["field2"].type_name, SchemaType::Null);
            assert_eq!(properties["field3"].type_name, SchemaType::Null);

            // No required fields since all are null
            assert!(schema.required.is_none());
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_process_object_serialization() {
        let json_obj = serde_json::json!({
            "name": "John",
            "age": 30,
            "email": null
        });

        if let serde_json::Value::Object(obj) = json_obj {
            let schema = process_object(&obj);
            let serialized = serde_json::to_value(&schema).unwrap();

            // Check basic structure
            assert_eq!(serialized["type"], "object");

            // Should not have $schema field for nested object
            assert!(!serialized.as_object().unwrap().contains_key("$schema"));

            // Check properties
            let properties = &serialized["properties"];
            assert_eq!(properties["name"]["type"], "string");
            assert_eq!(properties["age"]["type"], "integer");
            assert_eq!(properties["email"]["type"], "null");

            // Check required array
            let required = serialized["required"].as_array().unwrap();
            assert_eq!(required.len(), 2);
            assert!(required.contains(&serde_json::json!("name")));
            assert!(required.contains(&serde_json::json!("age")));
            assert!(!required.contains(&serde_json::json!("email")));

            // Nested properties should not have $schema field
            assert!(
                !properties["name"]
                    .as_object()
                    .unwrap()
                    .contains_key("$schema")
            );
            assert!(
                !properties["age"]
                    .as_object()
                    .unwrap()
                    .contains_key("$schema")
            );
            assert!(
                !properties["email"]
                    .as_object()
                    .unwrap()
                    .contains_key("$schema")
            );
        } else {
            panic!("Expected JSON object");
        }
    }

    // Array processing tests
    #[test]
    fn test_process_array_empty() {
        let empty_array: Vec<serde_json::Value> = vec![];
        let schema = process_array(&empty_array);

        assert_eq!(schema.type_name, SchemaType::Array);
        assert!(schema.items.is_none()); // Empty array has no items constraint
        assert!(schema.schema.is_empty()); // Nested schema should not have $schema field
        assert!(schema.properties.is_none());
        assert!(schema.required.is_none());
    }

    #[test]
    fn test_process_array_uniform_strings() {
        let json_array = serde_json::json!(["hello", "world", "test"]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());
            assert!(schema.schema.is_empty()); // Nested schema should not have $schema field

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::String);
            assert!(items_schema.schema.is_empty()); // Items schema should also be nested
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_uniform_integers() {
        let json_array = serde_json::json!([1, 2, 3, 42, -5]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Integer);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_uniform_numbers() {
        let json_array = serde_json::json!([1.5, 2.7, 3.14, -2.5]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Number);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_uniform_booleans() {
        let json_array = serde_json::json!([true, false, true]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Boolean);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_uniform_nulls() {
        let json_array = serde_json::json!([null, null, null]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Null);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_uniform_objects() {
        let json_array = serde_json::json!([
            {"name": "John", "age": 30},
            {"name": "Jane", "age": 25},
            {"name": "Bob", "age": 35}
        ]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Object);

            // Check that the object schema has the expected properties
            let properties = items_schema.properties.as_ref().unwrap();
            assert!(properties.contains_key("name"));
            assert!(properties.contains_key("age"));
            assert_eq!(properties["name"].type_name, SchemaType::String);
            assert_eq!(properties["age"].type_name, SchemaType::Integer);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_nested_arrays() {
        let json_array = serde_json::json!([[1, 2, 3], [4, 5, 6], [7, 8, 9]]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Array);

            // Check nested array items
            let nested_items = items_schema.items.as_ref().unwrap();
            assert_eq!(nested_items.type_name, SchemaType::Integer);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_mixed_types_simple() {
        let json_array = serde_json::json!([1, "hello", true]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            // For mixed types, we currently use the first type as fallback
            // This is a simplified implementation - in the future we should use anyOf
            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Integer);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_mixed_types_complex() {
        let json_array = serde_json::json!([
            {"type": "user", "name": "John"},
            {"type": "admin", "permissions": ["read", "write"]},
            "simple_string",
            42,
            null
        ]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            // For mixed types, we currently use the first type as fallback
            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Object);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_duplicate_types() {
        // Test that duplicate types are handled correctly
        let json_array = serde_json::json!(["hello", "world", "test", "another", "string"]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::String);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_single_element() {
        let json_array = serde_json::json!(["single"]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::String);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_complex_nested_structure() {
        let json_array = serde_json::json!([
            {
                "user": {
                    "name": "John",
                    "contacts": ["email@example.com", "phone"]
                },
                "metadata": {
                    "created": "2023-01-01",
                    "tags": ["important", "user"]
                }
            },
            {
                "user": {
                    "name": "Jane",
                    "contacts": ["jane@example.com"]
                },
                "metadata": {
                    "created": "2023-01-02",
                    "tags": ["user"]
                }
            }
        ]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);

            assert_eq!(schema.type_name, SchemaType::Array);
            assert!(schema.items.is_some());

            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Object);

            // Verify the complex nested structure
            let properties = items_schema.properties.as_ref().unwrap();
            assert!(properties.contains_key("user"));
            assert!(properties.contains_key("metadata"));

            // Check user object structure
            let user_schema = &properties["user"];
            assert_eq!(user_schema.type_name, SchemaType::Object);
            let user_properties = user_schema.properties.as_ref().unwrap();
            assert!(user_properties.contains_key("name"));
            assert!(user_properties.contains_key("contacts"));
            assert_eq!(user_properties["name"].type_name, SchemaType::String);
            assert_eq!(user_properties["contacts"].type_name, SchemaType::Array);

            // Check metadata object structure
            let metadata_schema = &properties["metadata"];
            assert_eq!(metadata_schema.type_name, SchemaType::Object);
            let metadata_properties = metadata_schema.properties.as_ref().unwrap();
            assert!(metadata_properties.contains_key("created"));
            assert!(metadata_properties.contains_key("tags"));
            assert_eq!(metadata_properties["created"].type_name, SchemaType::String);
            assert_eq!(metadata_properties["tags"].type_name, SchemaType::Array);
        } else {
            panic!("Expected JSON array");
        }
    }

    #[test]
    fn test_process_array_edge_cases() {
        // Test array with only null values
        let null_array = serde_json::json!([null, null]);
        if let serde_json::Value::Array(arr) = null_array {
            let schema = process_array(&arr);
            assert_eq!(schema.type_name, SchemaType::Array);
            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Null);
        }

        // Test array with mixed numbers (integers and floats)
        let mixed_numbers = serde_json::json!([1, 2.5, 3, 4.7]);
        if let serde_json::Value::Array(arr) = mixed_numbers {
            let schema = process_array(&arr);
            assert_eq!(schema.type_name, SchemaType::Array);
            // Should use the first type encountered (Integer in this case)
            let items_schema = schema.items.as_ref().unwrap();
            assert_eq!(items_schema.type_name, SchemaType::Integer);
        }

        // Test deeply nested array
        let deep_nested = serde_json::json!([[[["deep"]]]]);
        if let serde_json::Value::Array(arr) = deep_nested {
            let schema = process_array(&arr);
            assert_eq!(schema.type_name, SchemaType::Array);

            // Follow the nesting
            let level1 = schema.items.as_ref().unwrap();
            assert_eq!(level1.type_name, SchemaType::Array);

            let level2 = level1.items.as_ref().unwrap();
            assert_eq!(level2.type_name, SchemaType::Array);

            let level3 = level2.items.as_ref().unwrap();
            assert_eq!(level3.type_name, SchemaType::Array);

            let level4 = level3.items.as_ref().unwrap();
            assert_eq!(level4.type_name, SchemaType::String);
        }
    }

    #[test]
    fn test_process_array_serialization() {
        let json_array = serde_json::json!(["hello", "world"]);

        if let serde_json::Value::Array(arr) = json_array {
            let schema = process_array(&arr);
            let serialized = serde_json::to_value(&schema).unwrap();

            // Check basic structure
            assert_eq!(serialized["type"], "array");

            // Should not have $schema field for nested array
            assert!(!serialized.as_object().unwrap().contains_key("$schema"));

            // Check items
            let items = &serialized["items"];
            assert_eq!(items["type"], "string");

            // Items should not have $schema field
            assert!(!items.as_object().unwrap().contains_key("$schema"));
        } else {
            panic!("Expected JSON array");
        }
    }

    // Main generate_schema function tests
    #[test]
    fn test_generate_schema_primitive_types() {
        // Test null
        let null_value = serde_json::Value::Null;
        let schema = generate_schema(&null_value);
        assert_eq!(schema.type_name, SchemaType::Null);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Test boolean
        let bool_value = serde_json::json!(true);
        let schema = generate_schema(&bool_value);
        assert_eq!(schema.type_name, SchemaType::Boolean);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Test string
        let string_value = serde_json::json!("hello");
        let schema = generate_schema(&string_value);
        assert_eq!(schema.type_name, SchemaType::String);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Test integer
        let int_value = serde_json::json!(42);
        let schema = generate_schema(&int_value);
        assert_eq!(schema.type_name, SchemaType::Integer);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Test number
        let num_value = serde_json::json!(3.14);
        let schema = generate_schema(&num_value);
        assert_eq!(schema.type_name, SchemaType::Number);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
    }

    #[test]
    fn test_generate_schema_simple_object() {
        let json_obj = serde_json::json!({
            "name": "John Doe",
            "age": 30,
            "active": true
        });

        let schema = generate_schema(&json_obj);

        // Check root schema properties
        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema.properties.is_some());

        let properties = schema.properties.as_ref().unwrap();
        assert_eq!(properties.len(), 3);

        // Check property types
        assert_eq!(properties["name"].type_name, SchemaType::String);
        assert_eq!(properties["age"].type_name, SchemaType::Integer);
        assert_eq!(properties["active"].type_name, SchemaType::Boolean);

        // Check that nested properties don't have $schema field
        assert!(properties["name"].schema.is_empty());
        assert!(properties["age"].schema.is_empty());
        assert!(properties["active"].schema.is_empty());

        // Check required fields
        let required = schema.required.as_ref().unwrap();
        assert_eq!(required.len(), 3);
        assert!(required.contains(&"name".to_string()));
        assert!(required.contains(&"age".to_string()));
        assert!(required.contains(&"active".to_string()));
    }

    #[test]
    fn test_generate_schema_simple_array() {
        let json_array = serde_json::json!([1, 2, 3, 4, 5]);

        let schema = generate_schema(&json_array);

        // Check root schema properties
        assert_eq!(schema.type_name, SchemaType::Array);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema.items.is_some());

        let items = schema.items.as_ref().unwrap();
        assert_eq!(items.type_name, SchemaType::Integer);
        assert!(items.schema.is_empty()); // Nested items should not have $schema field
    }

    #[test]
    fn test_generate_schema_nested_object() {
        let json_obj = serde_json::json!({
            "user": {
                "name": "John Doe",
                "profile": {
                    "bio": "Software developer",
                    "age": 30
                }
            },
            "active": true
        });

        let schema = generate_schema(&json_obj);

        // Check root schema
        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        let properties = schema.properties.as_ref().unwrap();
        assert_eq!(properties.len(), 2);

        // Check user object
        assert_eq!(properties["user"].type_name, SchemaType::Object);
        assert!(properties["user"].schema.is_empty()); // Nested should not have $schema

        let user_properties = properties["user"].properties.as_ref().unwrap();
        assert_eq!(user_properties["name"].type_name, SchemaType::String);
        assert_eq!(user_properties["profile"].type_name, SchemaType::Object);

        // Check deeply nested profile
        let profile_properties = user_properties["profile"].properties.as_ref().unwrap();
        assert_eq!(profile_properties["bio"].type_name, SchemaType::String);
        assert_eq!(profile_properties["age"].type_name, SchemaType::Integer);

        // Verify all nested schemas don't have $schema field
        assert!(user_properties["name"].schema.is_empty());
        assert!(user_properties["profile"].schema.is_empty());
        assert!(profile_properties["bio"].schema.is_empty());
        assert!(profile_properties["age"].schema.is_empty());
    }

    #[test]
    fn test_generate_schema_array_of_objects() {
        let json_array = serde_json::json!([
            {
                "id": 1,
                "name": "Item 1"
            },
            {
                "id": 2,
                "name": "Item 2"
            }
        ]);

        let schema = generate_schema(&json_array);

        // Check root array schema
        assert_eq!(schema.type_name, SchemaType::Array);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        let items = schema.items.as_ref().unwrap();
        assert_eq!(items.type_name, SchemaType::Object);
        assert!(items.schema.is_empty()); // Items should not have $schema field

        // Check object properties in array items
        let item_properties = items.properties.as_ref().unwrap();
        assert_eq!(item_properties["id"].type_name, SchemaType::Integer);
        assert_eq!(item_properties["name"].type_name, SchemaType::String);

        // Check required fields
        let item_required = items.required.as_ref().unwrap();
        assert!(item_required.contains(&"id".to_string()));
        assert!(item_required.contains(&"name".to_string()));
    }

    #[test]
    fn test_generate_schema_complex_structure() {
        let complex_json = serde_json::json!({
            "metadata": {
                "version": "1.0",
                "created": "2023-01-01"
            },
            "users": [
                {
                    "id": 1,
                    "name": "John",
                    "contacts": ["email@example.com", "phone"],
                    "active": true
                }
            ],
            "settings": {
                "theme": "dark",
                "notifications": {
                    "email": true,
                    "push": false
                }
            },
            "count": 42,
            "rate": 3.14,
            "enabled": true,
            "notes": null
        });

        let schema = generate_schema(&complex_json);

        // Verify root schema
        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        let properties = schema.properties.as_ref().unwrap();

        // Check metadata object
        assert_eq!(properties["metadata"].type_name, SchemaType::Object);
        let metadata_props = properties["metadata"].properties.as_ref().unwrap();
        assert_eq!(metadata_props["version"].type_name, SchemaType::String);
        assert_eq!(metadata_props["created"].type_name, SchemaType::String);

        // Check users array
        assert_eq!(properties["users"].type_name, SchemaType::Array);
        let user_items = properties["users"].items.as_ref().unwrap();
        assert_eq!(user_items.type_name, SchemaType::Object);

        let user_props = user_items.properties.as_ref().unwrap();
        assert_eq!(user_props["id"].type_name, SchemaType::Integer);
        assert_eq!(user_props["name"].type_name, SchemaType::String);
        assert_eq!(user_props["contacts"].type_name, SchemaType::Array);
        assert_eq!(user_props["active"].type_name, SchemaType::Boolean);

        // Check contacts array items
        let contacts_items = user_props["contacts"].items.as_ref().unwrap();
        assert_eq!(contacts_items.type_name, SchemaType::String);

        // Check settings nested object
        assert_eq!(properties["settings"].type_name, SchemaType::Object);
        let settings_props = properties["settings"].properties.as_ref().unwrap();
        assert_eq!(settings_props["theme"].type_name, SchemaType::String);
        assert_eq!(
            settings_props["notifications"].type_name,
            SchemaType::Object
        );

        let notifications_props = settings_props["notifications"].properties.as_ref().unwrap();
        assert_eq!(notifications_props["email"].type_name, SchemaType::Boolean);
        assert_eq!(notifications_props["push"].type_name, SchemaType::Boolean);

        // Check primitive fields
        assert_eq!(properties["count"].type_name, SchemaType::Integer);
        assert_eq!(properties["rate"].type_name, SchemaType::Number);
        assert_eq!(properties["enabled"].type_name, SchemaType::Boolean);
        assert_eq!(properties["notes"].type_name, SchemaType::Null);

        // Verify required fields (all except notes which is null)
        let required = schema.required.as_ref().unwrap();
        assert!(required.contains(&"metadata".to_string()));
        assert!(required.contains(&"users".to_string()));
        assert!(required.contains(&"settings".to_string()));
        assert!(required.contains(&"count".to_string()));
        assert!(required.contains(&"rate".to_string()));
        assert!(required.contains(&"enabled".to_string()));
        assert!(!required.contains(&"notes".to_string()));
    }

    #[test]
    fn test_generate_schema_empty_structures() {
        // Test empty object
        let empty_obj = serde_json::json!({});
        let schema = generate_schema(&empty_obj);
        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema.properties.as_ref().unwrap().is_empty());
        assert!(schema.required.is_none());

        // Test empty array
        let empty_array = serde_json::json!([]);
        let schema = generate_schema(&empty_array);
        assert_eq!(schema.type_name, SchemaType::Array);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(schema.items.is_none()); // Empty array has no items schema
    }

    #[test]
    fn test_generate_schema_with_null_values() {
        let json_with_nulls = serde_json::json!({
            "name": "John",
            "email": null,
            "age": 30,
            "address": null
        });

        let schema = generate_schema(&json_with_nulls);

        assert_eq!(schema.type_name, SchemaType::Object);
        let properties = schema.properties.as_ref().unwrap();

        // Check types
        assert_eq!(properties["name"].type_name, SchemaType::String);
        assert_eq!(properties["email"].type_name, SchemaType::Null);
        assert_eq!(properties["age"].type_name, SchemaType::Integer);
        assert_eq!(properties["address"].type_name, SchemaType::Null);

        // Check required fields (only non-null values)
        let required = schema.required.as_ref().unwrap();
        assert_eq!(required.len(), 2);
        assert!(required.contains(&"name".to_string()));
        assert!(required.contains(&"age".to_string()));
        assert!(!required.contains(&"email".to_string()));
        assert!(!required.contains(&"address".to_string()));
    }

    #[test]
    fn test_generate_schema_recursion_depth_limit() {
        // Create a deeply nested structure that would exceed the recursion limit
        let mut deep_json = serde_json::json!("base");

        // Create a structure deeper than MAX_RECURSION_DEPTH
        for i in 0..MAX_RECURSION_DEPTH + 10 {
            deep_json = serde_json::json!({
                format!("level_{}", i): deep_json
            });
        }

        let schema = generate_schema(&deep_json);

        // Should still generate a valid schema without crashing
        assert_eq!(schema.type_name, SchemaType::Object);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // The schema should be generated successfully, even if depth limited
        // Let's verify it has properties (at least the first level)
        assert!(schema.properties.is_some());
        let properties = schema.properties.as_ref().unwrap();
        assert!(!properties.is_empty());

        // The first property should exist
        let first_key = format!("level_{}", MAX_RECURSION_DEPTH + 9);
        assert!(properties.contains_key(&first_key));
    }

    #[test]
    fn test_generate_schema_with_depth_internal() {
        // Test the internal depth-aware function directly
        let simple_obj = serde_json::json!({"test": "value"});

        // Test with normal depth
        let schema_normal = generate_schema_with_depth(&simple_obj, 0, true);
        assert_eq!(schema_normal.type_name, SchemaType::Object);
        assert_eq!(
            schema_normal.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Test with depth over limit - this should trigger the depth limit
        let schema_over_limit =
            generate_schema_with_depth(&simple_obj, MAX_RECURSION_DEPTH + 1, true);
        assert_eq!(schema_over_limit.type_name, SchemaType::Object);
        assert!(schema_over_limit.description.is_some());
        assert!(
            schema_over_limit
                .description
                .as_ref()
                .unwrap()
                .contains("maximum recursion depth")
        );

        // Test that the function doesn't crash with very high depth
        let schema_very_deep =
            generate_schema_with_depth(&simple_obj, MAX_RECURSION_DEPTH + 100, true);
        assert_eq!(schema_very_deep.type_name, SchemaType::Object);
        assert!(schema_very_deep.description.is_some());
    }

    #[test]
    fn test_generate_schema_serialization() {
        let json_obj = serde_json::json!({
            "name": "Test",
            "items": [1, 2, 3],
            "config": {
                "enabled": true
            }
        });

        let schema = generate_schema(&json_obj);
        let serialized = serde_json::to_value(&schema).unwrap();

        // Check root level serialization
        assert_eq!(
            serialized["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(serialized["type"], "object");

        // Check that nested objects don't have $schema field
        let properties = &serialized["properties"];
        assert_eq!(properties["name"]["type"], "string");
        assert!(
            !properties["name"]
                .as_object()
                .unwrap()
                .contains_key("$schema")
        );

        assert_eq!(properties["items"]["type"], "array");
        assert!(
            !properties["items"]
                .as_object()
                .unwrap()
                .contains_key("$schema")
        );

        assert_eq!(properties["config"]["type"], "object");
        assert!(
            !properties["config"]
                .as_object()
                .unwrap()
                .contains_key("$schema")
        );

        // Check deeply nested
        let config_props = &properties["config"]["properties"];
        assert_eq!(config_props["enabled"]["type"], "boolean");
        assert!(
            !config_props["enabled"]
                .as_object()
                .unwrap()
                .contains_key("$schema")
        );
    }

    #[test]
    fn test_generate_schema_mixed_array_types() {
        let mixed_array = serde_json::json!([
            "string",
            42,
            true,
            null,
            {"key": "value"},
            [1, 2, 3]
        ]);

        let schema = generate_schema(&mixed_array);

        assert_eq!(schema.type_name, SchemaType::Array);
        assert_eq!(
            schema.schema,
            "https://json-schema.org/draft/2020-12/schema"
        );

        // Should have items schema (currently uses first type as fallback)
        assert!(schema.items.is_some());
        let items = schema.items.as_ref().unwrap();
        assert_eq!(items.type_name, SchemaType::String); // First type encountered
        assert!(items.schema.is_empty());
    }
}
