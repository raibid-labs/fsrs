//! OpenTelemetry Semantic Conventions Parser
//!
//! Parses the YAML format used by the OpenTelemetry semantic conventions:
//! https://github.com/open-telemetry/semantic-conventions

use serde::Deserialize;
use std::collections::HashMap;

/// Root structure of a semantic conventions YAML file
#[derive(Debug, Clone, Deserialize)]
pub struct SemanticConventionsFile {
    pub groups: Vec<AttributeGroup>,
}

/// A group of related attributes (e.g., http, db, messaging)
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeGroup {
    /// Unique identifier (e.g., "http.client", "db.sql")
    pub id: String,

    /// Attribute name prefix (e.g., "http", "db")
    #[serde(default)]
    pub prefix: Option<String>,

    /// Type of telemetry (span, resource, metric, event)
    #[serde(rename = "type", default)]
    pub group_type: Option<String>,

    /// Brief description
    #[serde(default)]
    pub brief: Option<String>,

    /// Extended attributes from another group
    #[serde(default)]
    pub extends: Option<String>,

    /// Attributes in this group
    #[serde(default)]
    pub attributes: Vec<SemanticAttribute>,

    /// Stability level
    #[serde(default)]
    pub stability: Option<String>,
}

/// A single semantic attribute definition
#[derive(Debug, Clone, Deserialize)]
pub struct SemanticAttribute {
    /// Attribute ID (e.g., "method", "status_code")
    pub id: String,

    /// Attribute type
    #[serde(rename = "type")]
    pub attr_type: AttributeType,

    /// Brief description
    #[serde(default)]
    pub brief: Option<String>,

    /// Example values
    #[serde(default)]
    pub examples: Vec<serde_yaml::Value>,

    /// Requirement level
    #[serde(default)]
    pub requirement_level: RequirementLevel,

    /// Stability (stable, experimental, deprecated)
    #[serde(default)]
    pub stability: Option<String>,

    /// Deprecation note
    #[serde(default)]
    pub deprecated: Option<String>,

    /// Additional notes
    #[serde(default)]
    pub note: Option<String>,
}

/// Attribute type - can be simple or enum
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum AttributeType {
    /// Simple type: string, int, double, boolean, string[], int[], double[], boolean[]
    Simple(String),
    /// Enum type with allowed values
    Enum {
        allow_custom_values: Option<bool>,
        members: Vec<EnumMember>,
    },
    /// Template type (e.g., template[string])
    Template {
        template: String,
    },
}

impl AttributeType {
    /// Convert to Fusabi type name
    pub fn to_fusabi_type(&self) -> String {
        match self {
            AttributeType::Simple(s) => match s.as_str() {
                "string" => "string".to_string(),
                "int" => "int".to_string(),
                "double" => "float".to_string(),
                "boolean" => "bool".to_string(),
                "string[]" => "string list".to_string(),
                "int[]" => "int list".to_string(),
                "double[]" => "float list".to_string(),
                "boolean[]" => "bool list".to_string(),
                other => format!("string /* {} */", other),
            },
            AttributeType::Enum { .. } => "string".to_string(), // Enums are strings
            AttributeType::Template { .. } => "string".to_string(),
        }
    }
}

/// Enum member for enum-typed attributes
#[derive(Debug, Clone, Deserialize)]
pub struct EnumMember {
    pub id: String,
    pub value: serde_yaml::Value,
    #[serde(default)]
    pub brief: Option<String>,
    #[serde(default)]
    pub stability: Option<String>,
    #[serde(default)]
    pub deprecated: Option<String>,
}

/// Requirement level for an attribute
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum RequirementLevel {
    /// Simple level: required, recommended, opt_in
    Simple(String),
    /// Conditional requirement with explanation
    Conditional {
        #[serde(rename = "conditionally_required")]
        conditionally_required: Option<String>,
        #[serde(rename = "recommended")]
        recommended: Option<String>,
    },
    #[default]
    Unspecified,
}

impl RequirementLevel {
    pub fn is_required(&self) -> bool {
        match self {
            RequirementLevel::Simple(s) => s == "required",
            RequirementLevel::Conditional { conditionally_required, .. } => {
                conditionally_required.is_some()
            }
            RequirementLevel::Unspecified => false,
        }
    }

    pub fn is_recommended(&self) -> bool {
        match self {
            RequirementLevel::Simple(s) => s == "recommended",
            RequirementLevel::Conditional { recommended, .. } => recommended.is_some(),
            RequirementLevel::Unspecified => false,
        }
    }
}

/// Parsed and organized semantic conventions
#[derive(Debug, Clone, Default)]
pub struct ParsedConventions {
    /// Groups organized by category (http, db, messaging, etc.)
    pub categories: HashMap<String, ConventionCategory>,
}

/// A category of conventions (e.g., HTTP, Database)
#[derive(Debug, Clone, Default)]
pub struct ConventionCategory {
    pub name: String,
    pub groups: Vec<AttributeGroup>,
}

impl ParsedConventions {
    /// Parse from YAML content
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let file: SemanticConventionsFile = serde_yaml::from_str(yaml)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;

        let mut conventions = ParsedConventions::default();

        for group in file.groups {
            // Extract category from group ID (e.g., "http.client" -> "http")
            let category_name = group.id.split('.').next()
                .unwrap_or(&group.id)
                .to_string();

            let category = conventions.categories
                .entry(category_name.clone())
                .or_insert_with(|| ConventionCategory {
                    name: category_name,
                    groups: Vec::new(),
                });

            category.groups.push(group);
        }

        Ok(conventions)
    }

    /// Parse from JSON content (alternative format)
    pub fn from_json(json: &str) -> Result<Self, String> {
        let file: SemanticConventionsFile = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut conventions = ParsedConventions::default();

        for group in file.groups {
            let category_name = group.id.split('.').next()
                .unwrap_or(&group.id)
                .to_string();

            let category = conventions.categories
                .entry(category_name.clone())
                .or_insert_with(|| ConventionCategory {
                    name: category_name,
                    groups: Vec::new(),
                });

            category.groups.push(group);
        }

        Ok(conventions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_convention() {
        let yaml = r#"
groups:
  - id: http.client
    prefix: http
    type: span
    brief: HTTP client spans
    attributes:
      - id: request.method
        type: string
        brief: HTTP request method
        examples: ["GET", "POST"]
        requirement_level: required
      - id: response.status_code
        type: int
        brief: HTTP response status code
        examples: [200, 404]
        requirement_level:
          conditionally_required: If response was received
"#;

        let conventions = ParsedConventions::from_yaml(yaml).unwrap();
        assert!(conventions.categories.contains_key("http"));

        let http = &conventions.categories["http"];
        assert_eq!(http.groups.len(), 1);
        assert_eq!(http.groups[0].attributes.len(), 2);

        let method = &http.groups[0].attributes[0];
        assert_eq!(method.id, "request.method");
        assert!(method.requirement_level.is_required());
    }

    #[test]
    fn test_attribute_type_conversion() {
        assert_eq!(AttributeType::Simple("string".to_string()).to_fusabi_type(), "string");
        assert_eq!(AttributeType::Simple("int".to_string()).to_fusabi_type(), "int");
        assert_eq!(AttributeType::Simple("double".to_string()).to_fusabi_type(), "float");
        assert_eq!(AttributeType::Simple("boolean".to_string()).to_fusabi_type(), "bool");
        assert_eq!(AttributeType::Simple("string[]".to_string()).to_fusabi_type(), "string list");
    }
}
