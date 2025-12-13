//! Kubernetes Type Provider
//!
//! Generates Fusabi types from Kubernetes OpenAPI specifications.
//!
//! Usage:
//! ```fsharp
//! type K8s = KubernetesProvider<"1.28">
//! type K8s = KubernetesProvider<"cluster">
//! type K8s = KubernetesProvider<"file://./k8s-openapi.json">
//! ```

mod openapi;

use crate::ast::{RecordTypeDef, TypeExpr, TypeDefinition};
use crate::type_provider::{
    TypeProvider, ProviderParams, Schema,
    GeneratedTypes, GeneratedModule, TypeGenerator, NamingStrategy,
    error::{ProviderError, ProviderResult},
    json_schema::JsonSchemaProvider,
};
use std::collections::HashMap;

/// Kubernetes type provider
pub struct KubernetesProvider {
    json_provider: JsonSchemaProvider,
    generator: TypeGenerator,
    /// Maps Kubernetes API group paths to Fusabi module names
    /// e.g., "io.k8s.api.core.v1" -> "Core.V1"
    group_mapping: HashMap<String, String>,
}

impl KubernetesProvider {
    pub fn new() -> Self {
        let mut group_mapping = HashMap::new();

        // Core API groups
        group_mapping.insert("io.k8s.api.core.v1".to_string(), "Core.V1".to_string());
        group_mapping.insert("io.k8s.api.apps.v1".to_string(), "Apps.V1".to_string());
        group_mapping.insert("io.k8s.api.batch.v1".to_string(), "Batch.V1".to_string());
        group_mapping.insert("io.k8s.api.networking.v1".to_string(), "Networking.V1".to_string());
        group_mapping.insert("io.k8s.api.storage.v1".to_string(), "Storage.V1".to_string());
        group_mapping.insert("io.k8s.api.rbac.v1".to_string(), "Rbac.V1".to_string());
        group_mapping.insert("io.k8s.api.autoscaling.v1".to_string(), "Autoscaling.V1".to_string());
        group_mapping.insert("io.k8s.api.autoscaling.v2".to_string(), "Autoscaling.V2".to_string());
        group_mapping.insert("io.k8s.api.policy.v1".to_string(), "Policy.V1".to_string());

        // Metadata types
        group_mapping.insert("io.k8s.apimachinery.pkg.apis.meta.v1".to_string(), "Meta.V1".to_string());
        group_mapping.insert("io.k8s.apimachinery.pkg.api.resource".to_string(), "Resource".to_string());
        group_mapping.insert("io.k8s.apimachinery.pkg.util.intstr".to_string(), "Util".to_string());

        Self {
            json_provider: JsonSchemaProvider::new(),
            generator: TypeGenerator::new(NamingStrategy::PascalCase),
            group_mapping,
        }
    }

    /// Parse Kubernetes type name into module path and type name
    /// e.g., "io.k8s.api.core.v1.Pod" -> ("Core.V1", "Pod")
    fn parse_k8s_type_name(&self, full_name: &str) -> (String, String) {
        // Split into parts
        let parts: Vec<&str> = full_name.split('.').collect();

        if parts.len() < 2 {
            return ("Types".to_string(), full_name.to_string());
        }

        // Type name is always the last part
        let type_name = parts.last().unwrap().to_string();

        // Try to find matching group prefix
        for (prefix, module) in &self.group_mapping {
            if full_name.starts_with(prefix) {
                return (module.clone(), type_name);
            }
        }

        // Fallback: use last two parts before type name as module
        if parts.len() >= 3 {
            let module = format!("{}.{}",
                self.generator.naming.apply(parts[parts.len() - 3]),
                self.generator.naming.apply(parts[parts.len() - 2])
            );
            return (module, type_name);
        }

        ("Types".to_string(), type_name)
    }

    /// Resolve version string to OpenAPI URL
    fn resolve_version_url(&self, version: &str) -> ProviderResult<String> {
        // Common Kubernetes versions
        let base = "https://raw.githubusercontent.com/kubernetes/kubernetes";

        if version.starts_with("v") || version.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            // Version like "1.28" or "v1.28.0"
            let normalized = if version.starts_with("v") { version.to_string() } else { format!("v{}.0", version) };
            Ok(format!("{}/{}/api/openapi-spec/swagger.json", base, normalized))
        } else {
            Err(ProviderError::InvalidSource(format!("Invalid version: {}", version)))
        }
    }

    /// Generate types from OpenAPI definitions
    fn generate_from_openapi(
        &self,
        openapi: &serde_json::Value,
        namespace: &str,
    ) -> ProviderResult<GeneratedTypes> {
        let definitions = openapi.get("definitions")
            .or_else(|| openapi.get("components").and_then(|c| c.get("schemas")))
            .and_then(|d| d.as_object())
            .ok_or_else(|| ProviderError::ParseError("No definitions found in OpenAPI spec".to_string()))?;

        let mut modules: HashMap<String, Vec<TypeDefinition>> = HashMap::new();

        for (full_name, schema) in definitions {
            // Skip internal types
            if full_name.contains("WatchEvent") || full_name.ends_with("List") {
                continue;
            }

            let (module_path, type_name) = self.parse_k8s_type_name(full_name);
            let full_module_path = format!("{}.{}", namespace, module_path);

            if let Some(type_def) = self.openapi_schema_to_typedef(&type_name, schema)? {
                modules.entry(full_module_path)
                    .or_default()
                    .push(type_def);
            }
        }

        let generated_modules = modules.into_iter()
            .map(|(path, types)| GeneratedModule {
                path: path.split('.').map(String::from).collect(),
                types,
            })
            .collect();

        Ok(GeneratedTypes {
            modules: generated_modules,
            root_types: vec![],
        })
    }

    /// Convert OpenAPI schema to TypeDefinition
    fn openapi_schema_to_typedef(
        &self,
        name: &str,
        schema: &serde_json::Value,
    ) -> ProviderResult<Option<TypeDefinition>> {
        let schema_type = schema.get("type").and_then(|t| t.as_str());

        match schema_type {
            Some("object") => {
                let fields = self.openapi_object_to_fields(schema)?;
                if fields.is_empty() {
                    return Ok(None);
                }
                Ok(Some(TypeDefinition::Record(RecordTypeDef {
                    name: name.to_string(),
                    fields,
                })))
            }
            _ => Ok(None),
        }
    }

    /// Convert OpenAPI object properties to record fields
    fn openapi_object_to_fields(
        &self,
        schema: &serde_json::Value,
    ) -> ProviderResult<Vec<(String, TypeExpr)>> {
        let mut fields = Vec::new();

        let properties = match schema.get("properties").and_then(|p| p.as_object()) {
            Some(p) => p,
            None => return Ok(fields),
        };

        let required: Vec<String> = schema.get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        for (prop_name, prop_schema) in properties {
            let type_expr = self.openapi_type_to_expr(prop_schema)?;
            let is_required = required.contains(prop_name);

            let final_type = if is_required {
                type_expr
            } else {
                TypeExpr::Named(format!("{} option", type_expr))
            };

            fields.push((prop_name.clone(), final_type));
        }

        Ok(fields)
    }

    /// Convert OpenAPI type to TypeExpr
    fn openapi_type_to_expr(&self, schema: &serde_json::Value) -> ProviderResult<TypeExpr> {
        // Handle $ref
        if let Some(ref_path) = schema.get("$ref").and_then(|r| r.as_str()) {
            let type_name = ref_path.split('/').last()
                .map(|s| s.split('.').last().unwrap_or(s))
                .unwrap_or("Unknown");
            return Ok(TypeExpr::Named(type_name.to_string()));
        }

        let schema_type = schema.get("type").and_then(|t| t.as_str());

        match schema_type {
            Some("string") => Ok(TypeExpr::Named("string".to_string())),
            Some("integer") => Ok(TypeExpr::Named("int".to_string())),
            Some("number") => Ok(TypeExpr::Named("float".to_string())),
            Some("boolean") => Ok(TypeExpr::Named("bool".to_string())),
            Some("array") => {
                if let Some(items) = schema.get("items") {
                    let item_type = self.openapi_type_to_expr(items)?;
                    Ok(TypeExpr::Named(format!("{} list", item_type)))
                } else {
                    Ok(TypeExpr::Named("any list".to_string()))
                }
            }
            Some("object") => {
                // Check for additionalProperties (map type)
                if schema.get("additionalProperties").is_some() {
                    Ok(TypeExpr::Named("Map<string, any>".to_string()))
                } else {
                    Ok(TypeExpr::Named("any".to_string()))
                }
            }
            _ => Ok(TypeExpr::Named("any".to_string())),
        }
    }
}

impl Default for KubernetesProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeProvider for KubernetesProvider {
    fn name(&self) -> &str {
        "KubernetesProvider"
    }

    fn resolve_schema(&self, source: &str, _params: &ProviderParams) -> ProviderResult<Schema> {
        let json_str = if source == "cluster" {
            // TODO: Implement in-cluster config
            return Err(ProviderError::FetchError("In-cluster config not yet implemented".to_string()));
        } else if source.starts_with("http://") || source.starts_with("https://") {
            // Direct URL
            // TODO: Implement HTTP fetch
            return Err(ProviderError::FetchError("HTTP fetch not yet implemented. Use file:// or version string.".to_string()));
        } else if source.starts_with("file://") {
            // Local file
            let path = source.strip_prefix("file://").unwrap();
            std::fs::read_to_string(path)
                .map_err(|e| ProviderError::IoError(e.to_string()))?
        } else if source.chars().next().map(|c| c.is_numeric() || c == 'v').unwrap_or(false) {
            // Version string - for now, require a file
            return Err(ProviderError::FetchError(format!(
                "Version '{}' specified but HTTP fetch not implemented. Please provide a local file with file://path/to/openapi.json",
                source
            )));
        } else {
            // Treat as file path
            std::fs::read_to_string(source)
                .map_err(|e| ProviderError::IoError(e.to_string()))?
        };

        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        Ok(Schema::OpenApi(value))
    }

    fn generate_types(&self, schema: &Schema, namespace: &str) -> ProviderResult<GeneratedTypes> {
        match schema {
            Schema::OpenApi(value) => self.generate_from_openapi(value, namespace),
            _ => Err(ProviderError::ParseError("Expected OpenAPI schema".to_string())),
        }
    }

    fn get_documentation(&self, type_path: &str) -> Option<String> {
        // Could return Kubernetes documentation for types
        // For now, return None
        None
    }
}
