//! JSON Schema parser

use super::types::{JsonSchema, JsonSchemaType};
use crate::type_provider::error::{ProviderError, ProviderResult};

/// Parse a JSON Schema from a JSON string
pub fn parse_json_schema(json: &str) -> ProviderResult<JsonSchema> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| ProviderError::ParseError(format!("Invalid JSON: {}", e)))?;

    parse_schema_value(&value)
}

/// Parse a JSON Schema from a serde_json::Value
pub fn parse_schema_value(value: &serde_json::Value) -> ProviderResult<JsonSchema> {
    let obj = value.as_object()
        .ok_or_else(|| ProviderError::ParseError("Schema must be an object".to_string()))?;

    let mut schema = JsonSchema::default();

    // Title and description
    if let Some(title) = obj.get("title").and_then(|v| v.as_str()) {
        schema.title = Some(title.to_string());
    }
    if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
        schema.description = Some(desc.to_string());
    }

    // Type
    if let Some(type_val) = obj.get("type") {
        schema.schema_type = parse_schema_type(type_val);
    }

    // Required fields
    if let Some(required) = obj.get("required").and_then(|v| v.as_array()) {
        schema.required = required.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }

    // Properties
    if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
        for (name, prop_value) in props {
            schema.properties.insert(name.clone(), parse_schema_value(prop_value)?);
        }
    }

    // Items (for arrays)
    if let Some(items) = obj.get("items") {
        schema.items = Some(Box::new(parse_schema_value(items)?));
    }

    // Enum
    if let Some(enum_val) = obj.get("enum").and_then(|v| v.as_array()) {
        schema.enum_values = enum_val.clone();
    }

    // oneOf
    if let Some(one_of) = obj.get("oneOf").and_then(|v| v.as_array()) {
        schema.one_of = one_of.iter()
            .map(parse_schema_value)
            .collect::<ProviderResult<_>>()?;
    }

    // anyOf
    if let Some(any_of) = obj.get("anyOf").and_then(|v| v.as_array()) {
        schema.any_of = any_of.iter()
            .map(parse_schema_value)
            .collect::<ProviderResult<_>>()?;
    }

    // allOf
    if let Some(all_of) = obj.get("allOf").and_then(|v| v.as_array()) {
        schema.all_of = all_of.iter()
            .map(parse_schema_value)
            .collect::<ProviderResult<_>>()?;
    }

    // $ref
    if let Some(ref_val) = obj.get("$ref").and_then(|v| v.as_str()) {
        schema.reference = Some(ref_val.to_string());
    }

    // Definitions
    let def_key = if obj.contains_key("definitions") { "definitions" } else { "$defs" };
    if let Some(defs) = obj.get(def_key).and_then(|v| v.as_object()) {
        for (name, def_value) in defs {
            schema.definitions.insert(name.clone(), parse_schema_value(def_value)?);
        }
    }

    // Const
    if let Some(const_val) = obj.get("const") {
        schema.const_value = Some(const_val.clone());
    }

    // Default
    if let Some(default_val) = obj.get("default") {
        schema.default = Some(default_val.clone());
    }

    // Format
    if let Some(format) = obj.get("format").and_then(|v| v.as_str()) {
        schema.format = Some(format.to_string());
    }

    Ok(schema)
}

fn parse_schema_type(value: &serde_json::Value) -> Option<JsonSchemaType> {
    value.as_str().and_then(|s| match s {
        "string" => Some(JsonSchemaType::String),
        "integer" => Some(JsonSchemaType::Integer),
        "number" => Some(JsonSchemaType::Number),
        "boolean" => Some(JsonSchemaType::Boolean),
        "null" => Some(JsonSchemaType::Null),
        "array" => Some(JsonSchemaType::Array),
        "object" => Some(JsonSchemaType::Object),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        }"#;

        let schema = parse_json_schema(json).unwrap();
        assert_eq!(schema.schema_type, Some(JsonSchemaType::Object));
        assert!(schema.properties.contains_key("name"));
        assert!(schema.properties.contains_key("age"));
        assert!(schema.required.contains(&"name".to_string()));
    }

    #[test]
    fn test_parse_enum() {
        let json = r#"{
            "type": "string",
            "enum": ["active", "inactive", "pending"]
        }"#;

        let schema = parse_json_schema(json).unwrap();
        assert_eq!(schema.schema_type, Some(JsonSchemaType::String));
        assert_eq!(schema.enum_values.len(), 3);
    }

    #[test]
    fn test_parse_ref() {
        let json = r##"{
            "$ref": "#/definitions/Person"
        }"##;

        let schema = parse_json_schema(json).unwrap();
        assert_eq!(schema.reference, Some("#/definitions/Person".to_string()));
    }
}
