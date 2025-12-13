//! JSON Schema Type Provider
//!
//! Generates Fusabi types from JSON Schema definitions.

mod parser;
mod types;

pub use types::JsonSchemaType;

use crate::ast::{RecordTypeDef, DuTypeDef, VariantDef, TypeExpr, TypeDefinition};
use crate::type_provider::{
    TypeProvider, ProviderParams, Schema,
    GeneratedTypes, GeneratedModule, TypeGenerator, NamingStrategy,
    error::{ProviderError, ProviderResult},
};

/// JSON Schema type provider
pub struct JsonSchemaProvider {
    generator: TypeGenerator,
}

impl JsonSchemaProvider {
    pub fn new() -> Self {
        Self {
            generator: TypeGenerator::new(NamingStrategy::PascalCase),
        }
    }

    /// Parse JSON Schema from string
    fn parse_schema(&self, json: &str) -> ProviderResult<types::JsonSchema> {
        parser::parse_json_schema(json)
    }

    /// Generate types from parsed schema
    fn generate_from_schema(
        &self,
        schema: &types::JsonSchema,
        namespace: &str
    ) -> ProviderResult<GeneratedTypes> {
        let mut result = GeneratedTypes::new();
        let mut definitions_module = GeneratedModule::new(vec![namespace.to_string()]);

        // Process definitions first
        for (name, def_schema) in &schema.definitions {
            if let Some(type_def) = self.schema_to_typedef(name, def_schema)? {
                definitions_module.types.push(type_def);
            }
        }

        // Process root schema
        if let Some(root_type) = self.schema_to_typedef("Root", schema)? {
            result.root_types.push(root_type);
        }

        if !definitions_module.types.is_empty() {
            result.modules.push(definitions_module);
        }

        Ok(result)
    }

    /// Convert a JSON Schema to a Fusabi TypeDefinition
    fn schema_to_typedef(
        &self,
        name: &str,
        schema: &types::JsonSchema,
    ) -> ProviderResult<Option<TypeDefinition>> {
        match &schema.schema_type {
            Some(JsonSchemaType::Object) => {
                let fields = self.object_to_fields(schema)?;
                Ok(Some(TypeDefinition::Record(RecordTypeDef {
                    name: self.generator.naming.apply(name),
                    fields,
                })))
            }
            Some(JsonSchemaType::String) if !schema.enum_values.is_empty() => {
                // String enum -> DU
                let variants = schema.enum_values.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| VariantDef::new_simple(self.generator.naming.apply(s)))
                    .collect();
                Ok(Some(TypeDefinition::Du(DuTypeDef {
                    name: self.generator.naming.apply(name),
                    variants,
                })))
            }
            _ if !schema.one_of.is_empty() => {
                // oneOf -> DU
                let variants = self.one_of_to_variants(&schema.one_of)?;
                Ok(Some(TypeDefinition::Du(DuTypeDef {
                    name: self.generator.naming.apply(name),
                    variants,
                })))
            }
            _ => Ok(None), // Primitive types don't need typedef
        }
    }

    /// Convert object properties to record fields
    fn object_to_fields(
        &self,
        schema: &types::JsonSchema,
    ) -> ProviderResult<Vec<(String, TypeExpr)>> {
        let mut fields = Vec::new();

        for (prop_name, prop_schema) in &schema.properties {
            let type_expr = self.schema_to_type_expr(prop_schema)?;
            let is_required = schema.required.contains(prop_name);

            let final_type = if is_required {
                type_expr
            } else {
                // Wrap in Option for optional fields
                TypeExpr::Named(format!("{} option", type_expr))
            };

            fields.push((prop_name.clone(), final_type));
        }

        Ok(fields)
    }

    /// Convert JSON Schema to TypeExpr
    fn schema_to_type_expr(&self, schema: &types::JsonSchema) -> ProviderResult<TypeExpr> {
        // Handle $ref
        if let Some(ref_path) = &schema.reference {
            let type_name = ref_path.split('/').last().unwrap_or("Unknown");
            return Ok(TypeExpr::Named(self.generator.naming.apply(type_name)));
        }

        match &schema.schema_type {
            Some(JsonSchemaType::String) => Ok(TypeExpr::Named("string".to_string())),
            Some(JsonSchemaType::Integer) => Ok(TypeExpr::Named("int".to_string())),
            Some(JsonSchemaType::Number) => Ok(TypeExpr::Named("float".to_string())),
            Some(JsonSchemaType::Boolean) => Ok(TypeExpr::Named("bool".to_string())),
            Some(JsonSchemaType::Null) => Ok(TypeExpr::Named("unit".to_string())),
            Some(JsonSchemaType::Array) => {
                if let Some(items) = &schema.items {
                    let item_type = self.schema_to_type_expr(items)?;
                    Ok(TypeExpr::Named(format!("{} list", item_type)))
                } else {
                    Ok(TypeExpr::Named("any list".to_string()))
                }
            }
            Some(JsonSchemaType::Object) => {
                // Inline object - use dynamic map type
                Ok(TypeExpr::Named("Map<string, any>".to_string()))
            }
            None => Ok(TypeExpr::Named("any".to_string())),
        }
    }

    /// Convert oneOf to DU variants
    fn one_of_to_variants(
        &self,
        schemas: &[types::JsonSchema],
    ) -> ProviderResult<Vec<VariantDef>> {
        let mut variants = Vec::new();

        for (i, schema) in schemas.iter().enumerate() {
            // Try to find a discriminator field (like "type" or "kind")
            let variant_name = schema.properties.get("type")
                .or_else(|| schema.properties.get("kind"))
                .and_then(|s| s.const_value.as_ref())
                .and_then(|v| v.as_str())
                .map(|s| self.generator.naming.apply(s))
                .unwrap_or_else(|| format!("Variant{}", i));

            // Get fields excluding discriminator
            let fields: Vec<TypeExpr> = schema.properties.iter()
                .filter(|(k, _)| *k != "type" && *k != "kind")
                .map(|(_, v)| self.schema_to_type_expr(v))
                .collect::<ProviderResult<_>>()?;

            variants.push(VariantDef::new(variant_name, fields));
        }

        Ok(variants)
    }
}

impl Default for JsonSchemaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeProvider for JsonSchemaProvider {
    fn name(&self) -> &str {
        "JsonSchemaProvider"
    }

    fn resolve_schema(&self, source: &str, _params: &ProviderParams) -> ProviderResult<Schema> {
        // For now, treat source as inline JSON or file path
        let json_str = if source.starts_with('{') {
            source.to_string()
        } else if source.starts_with("file://") {
            let path = source.strip_prefix("file://").unwrap();
            std::fs::read_to_string(path)
                .map_err(|e| ProviderError::IoError(e.to_string()))?
        } else {
            // Treat as file path without prefix
            std::fs::read_to_string(source)
                .map_err(|e| ProviderError::IoError(e.to_string()))?
        };

        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        Ok(Schema::JsonSchema(value))
    }

    fn generate_types(&self, schema: &Schema, namespace: &str) -> ProviderResult<GeneratedTypes> {
        match schema {
            Schema::JsonSchema(value) => {
                let json_str = serde_json::to_string(value)
                    .map_err(|e| ProviderError::ParseError(e.to_string()))?;
                let parsed = self.parse_schema(&json_str)?;
                self.generate_from_schema(&parsed, namespace)
            }
            _ => Err(ProviderError::ParseError("Expected JSON Schema".to_string())),
        }
    }
}
