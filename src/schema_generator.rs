use serde::Serialize;
use std::collections::HashMap;

/// Represents a JSON Schema structure according to JSON Schema Draft 2020-12
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct JsonSchema {
    /// The JSON Schema version identifier
    #[serde(rename = "$schema")]
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
    
    /// Creates a new object schema with properties
    pub fn new_object(properties: HashMap<String, JsonSchema>, required: Vec<String>) -> Self {
        Self {
            schema: "https://json-schema.org/draft/2020-12/schema".to_string(),
            type_name: SchemaType::Object,
            properties: Some(properties),
            items: None,
            required: if required.is_empty() { None } else { Some(required) },
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

/// Generates a JSON Schema from a JSON value
pub fn generate_schema(_json_value: &serde_json::Value) -> JsonSchema {
    // TODO: Implement schema generation in task 9
    JsonSchema::new(SchemaType::Object)
}

/// Infers the schema type from a JSON value
fn infer_type(_value: &serde_json::Value) -> SchemaType {
    // TODO: Implement type inference in task 6
    SchemaType::Object
}

/// Processes a JSON object and generates its schema
fn process_object(_obj: &serde_json::Map<String, serde_json::Value>) -> JsonSchema {
    // TODO: Implement object processing in task 7
    JsonSchema::new(SchemaType::Object)
}

/// Processes a JSON array and generates its schema
fn process_array(_arr: &[serde_json::Value]) -> JsonSchema {
    // TODO: Implement array processing in task 8
    JsonSchema::new(SchemaType::Array)
}
#
[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_schema_type_serialization() {
        // Test that SchemaType serializes to lowercase strings
        assert_eq!(serde_json::to_string(&SchemaType::Object).unwrap(), "\"object\"");
        assert_eq!(serde_json::to_string(&SchemaType::Array).unwrap(), "\"array\"");
        assert_eq!(serde_json::to_string(&SchemaType::String).unwrap(), "\"string\"");
        assert_eq!(serde_json::to_string(&SchemaType::Number).unwrap(), "\"number\"");
        assert_eq!(serde_json::to_string(&SchemaType::Integer).unwrap(), "\"integer\"");
        assert_eq!(serde_json::to_string(&SchemaType::Boolean).unwrap(), "\"boolean\"");
        assert_eq!(serde_json::to_string(&SchemaType::Null).unwrap(), "\"null\"");
    }

    #[test]
    fn test_json_schema_new() {
        let schema = JsonSchema::new(SchemaType::String);
        
        assert_eq!(schema.schema, "https://json-schema.org/draft/2020-12/schema");
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
        let schema = JsonSchema::new(SchemaType::String)
            .with_title("User Name".to_string());
        
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
        
        assert_eq!(json["$schema"], "https://json-schema.org/draft/2020-12/schema");
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
        
        assert_eq!(json["$schema"], "https://json-schema.org/draft/2020-12/schema");
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
        
        assert_eq!(json["$schema"], "https://json-schema.org/draft/2020-12/schema");
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
            vec!["street".to_string(), "city".to_string()]
        );
        
        let mut user_properties = HashMap::new();
        user_properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));
        user_properties.insert("address".to_string(), address_schema);
        
        let user_schema = JsonSchema::new_object(
            user_properties,
            vec!["name".to_string()]
        );
        
        let json = serde_json::to_value(&user_schema).unwrap();
        
        assert_eq!(json["type"], "object");
        assert_eq!(json["properties"]["name"]["type"], "string");
        assert_eq!(json["properties"]["address"]["type"], "object");
        assert_eq!(json["properties"]["address"]["properties"]["street"]["type"], "string");
        assert_eq!(json["properties"]["address"]["properties"]["city"]["type"], "string");
    }

    #[test]
    fn test_array_of_objects_schema() {
        // Test creating an array of objects schema
        let mut item_properties = HashMap::new();
        item_properties.insert("id".to_string(), JsonSchema::new(SchemaType::Integer));
        item_properties.insert("name".to_string(), JsonSchema::new(SchemaType::String));
        
        let item_schema = JsonSchema::new_object(
            item_properties,
            vec!["id".to_string(), "name".to_string()]
        );
        
        let array_schema = JsonSchema::new_array(item_schema);
        
        let json = serde_json::to_value(&array_schema).unwrap();
        
        assert_eq!(json["type"], "array");
        assert_eq!(json["items"]["type"], "object");
        assert_eq!(json["items"]["properties"]["id"]["type"], "integer");
        assert_eq!(json["items"]["properties"]["name"]["type"], "string");
    }
}