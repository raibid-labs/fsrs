//! JSON Schema type definitions

use std::collections::HashMap;

/// JSON Schema type enum
#[derive(Debug, Clone, PartialEq)]
pub enum JsonSchemaType {
    String,
    Integer,
    Number,
    Boolean,
    Null,
    Array,
    Object,
}

/// Parsed JSON Schema representation
#[derive(Debug, Clone, Default)]
pub struct JsonSchema {
    /// Schema title
    pub title: Option<String>,
    /// Schema description
    pub description: Option<String>,
    /// The type of this schema
    pub schema_type: Option<JsonSchemaType>,
    /// Required field names
    pub required: Vec<String>,
    /// Object properties
    pub properties: HashMap<String, JsonSchema>,
    /// Array items schema
    pub items: Option<Box<JsonSchema>>,
    /// Enum values
    pub enum_values: Vec<serde_json::Value>,
    /// oneOf schemas
    pub one_of: Vec<JsonSchema>,
    /// anyOf schemas
    pub any_of: Vec<JsonSchema>,
    /// allOf schemas
    pub all_of: Vec<JsonSchema>,
    /// $ref to another schema
    pub reference: Option<String>,
    /// Definitions/components
    pub definitions: HashMap<String, JsonSchema>,
    /// Const value (for discriminators)
    pub const_value: Option<serde_json::Value>,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Format hint (e.g., "date-time", "email")
    pub format: Option<String>,
}
