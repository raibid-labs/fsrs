//! OpenTelemetry Type Provider
//!
//! Generates Fusabi types from OpenTelemetry semantic conventions.
//! This enables type-safe telemetry instrumentation where attribute
//! names and types are validated at compile time.
//!
//! # Usage
//!
//! ```fsharp
//! // Load conventions from a version or file
//! type OTel = OpenTelemetryProvider<"1.24.0">
//! type OTel = OpenTelemetryProvider<"file://./semantic-conventions.yaml">
//!
//! // Use typed span builders
//! let span = OTel.Http.Client.create "fetch-user" {
//!     httpRequestMethod = "GET"
//!     httpResponseStatusCode = Some 200
//!     urlFull = "https://api.example.com/users/1"
//! }
//! ```
//!
//! # Generated Structure
//!
//! ```
//! OTel
//! ├── Http
//! │   ├── Client         (record with http.client attributes)
//! │   └── Server         (record with http.server attributes)
//! ├── Db
//! │   ├── Common         (record with db attributes)
//! │   └── Sql            (record with db.sql attributes)
//! ├── Messaging
//! │   ├── Producer
//! │   └── Consumer
//! └── ...
//! ```

pub mod conventions;

use crate::ast::{RecordTypeDef, TypeExpr, TypeDefinition};
use crate::type_provider::{
    TypeProvider, ProviderParams, Schema,
    GeneratedTypes, GeneratedModule, TypeGenerator, NamingStrategy,
    error::{ProviderError, ProviderResult},
};
use conventions::{ParsedConventions, AttributeGroup, RequirementLevel};
use std::collections::HashMap;

/// OpenTelemetry semantic conventions type provider
pub struct OpenTelemetryProvider {
    generator: TypeGenerator,
    /// Embedded conventions for common versions (fallback)
    embedded_conventions: HashMap<String, &'static str>,
}

impl OpenTelemetryProvider {
    pub fn new() -> Self {
        let mut embedded = HashMap::new();

        // Embed a minimal set of common conventions for offline use
        embedded.insert("embedded".to_string(), EMBEDDED_CONVENTIONS);

        Self {
            generator: TypeGenerator::new(NamingStrategy::CamelCase),
            embedded_conventions: embedded,
        }
    }

    /// Generate types from parsed conventions
    fn generate_from_conventions(
        &self,
        conventions: &ParsedConventions,
        namespace: &str,
    ) -> ProviderResult<GeneratedTypes> {
        let mut modules: HashMap<String, Vec<TypeDefinition>> = HashMap::new();

        for (category_name, category) in &conventions.categories {
            let module_name = self.generator.naming.apply(category_name);
            let module_path = format!("{}.{}", namespace, module_name);

            for group in &category.groups {
                if let Some(type_def) = self.group_to_typedef(group)? {
                    modules.entry(module_path.clone())
                        .or_default()
                        .push(type_def);
                }
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

    /// Convert an attribute group to a Fusabi type definition
    fn group_to_typedef(&self, group: &AttributeGroup) -> ProviderResult<Option<TypeDefinition>> {
        if group.attributes.is_empty() {
            return Ok(None);
        }

        // Convert group ID to type name (e.g., "http.client" -> "HttpClient")
        let type_name = group.id.split('.')
            .map(|part| self.generator.naming.apply(part))
            .collect::<Vec<_>>()
            .join("");

        // Capitalize first letter for PascalCase type name
        let type_name = capitalize(&type_name);

        let mut fields = Vec::new();

        for attr in &group.attributes {
            // Convert attribute ID to field name
            // e.g., "request.method" -> "requestMethod"
            let field_name = attr.id.split('.')
                .enumerate()
                .map(|(i, part)| {
                    if i == 0 {
                        part.to_string()
                    } else {
                        capitalize(part)
                    }
                })
                .collect::<String>();

            let base_type = attr.attr_type.to_fusabi_type();

            // Make non-required fields optional
            let field_type = if attr.requirement_level.is_required() {
                TypeExpr::Named(base_type)
            } else {
                TypeExpr::Named(format!("{} option", base_type))
            };

            fields.push((field_name, field_type));
        }

        Ok(Some(TypeDefinition::Record(RecordTypeDef {
            name: type_name,
            fields,
        })))
    }
}

impl Default for OpenTelemetryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeProvider for OpenTelemetryProvider {
    fn name(&self) -> &str {
        "OpenTelemetryProvider"
    }

    fn resolve_schema(&self, source: &str, _params: &ProviderParams) -> ProviderResult<Schema> {
        let content = if source.starts_with("file://") {
            // Load from local file
            let path = source.strip_prefix("file://").unwrap();
            std::fs::read_to_string(path)
                .map_err(|e| ProviderError::IoError(format!("Failed to read {}: {}", path, e)))?
        } else if source == "embedded" || source == "default" {
            // Use embedded conventions
            EMBEDDED_CONVENTIONS.to_string()
        } else if source.starts_with("http://") || source.starts_with("https://") {
            // TODO: Implement HTTP fetch
            return Err(ProviderError::FetchError(
                "HTTP fetch not yet implemented. Use file:// or 'embedded'".to_string()
            ));
        } else {
            // Assume it's a version string, use embedded for now
            // In future: fetch from OTel GitHub releases
            EMBEDDED_CONVENTIONS.to_string()
        };

        Ok(Schema::Custom(content))
    }

    fn generate_types(&self, schema: &Schema, namespace: &str) -> ProviderResult<GeneratedTypes> {
        let content = match schema {
            Schema::Custom(s) => s,
            _ => return Err(ProviderError::ParseError("Expected Custom schema".to_string())),
        };

        // Try YAML first, then JSON
        let conventions = ParsedConventions::from_yaml(content)
            .or_else(|_| ParsedConventions::from_json(content))
            .map_err(|e| ProviderError::ParseError(e))?;

        self.generate_from_conventions(&conventions, namespace)
    }

    fn get_documentation(&self, type_path: &str) -> Option<String> {
        // Could return OTel documentation for attributes
        None
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Embedded minimal semantic conventions for common use cases
/// Based on OpenTelemetry Semantic Conventions v1.24.0
const EMBEDDED_CONVENTIONS: &str = r#"
groups:
  # HTTP Client Spans
  - id: http.client
    prefix: http
    type: span
    brief: Semantic conventions for HTTP client spans
    attributes:
      - id: request.method
        type: string
        brief: HTTP request method
        examples: ["GET", "POST", "PUT", "DELETE"]
        requirement_level: required
      - id: request.url
        type: string
        brief: Full HTTP request URL
        examples: ["https://api.example.com/users"]
        requirement_level: required
      - id: request.headers
        type: string[]
        brief: HTTP request headers
        requirement_level: opt_in
      - id: response.status_code
        type: int
        brief: HTTP response status code
        examples: [200, 404, 500]
        requirement_level:
          conditionally_required: If response was received
      - id: response.headers
        type: string[]
        brief: HTTP response headers
        requirement_level: opt_in
      - id: request.body.size
        type: int
        brief: Size of request body in bytes
        requirement_level: recommended
      - id: response.body.size
        type: int
        brief: Size of response body in bytes
        requirement_level: recommended

  # HTTP Server Spans
  - id: http.server
    prefix: http
    type: span
    brief: Semantic conventions for HTTP server spans
    attributes:
      - id: request.method
        type: string
        brief: HTTP request method
        requirement_level: required
      - id: route
        type: string
        brief: The matched route
        examples: ["/users/:id", "/api/v1/*"]
        requirement_level: recommended
      - id: response.status_code
        type: int
        brief: HTTP response status code
        requirement_level:
          conditionally_required: If response was sent
      - id: client.address
        type: string
        brief: Client IP address
        requirement_level: recommended
      - id: user_agent.original
        type: string
        brief: Original user agent string
        requirement_level: recommended

  # Database Spans
  - id: db
    prefix: db
    type: span
    brief: Semantic conventions for database operations
    attributes:
      - id: system
        type: string
        brief: Database system identifier
        examples: ["postgresql", "mysql", "redis", "mongodb"]
        requirement_level: required
      - id: name
        type: string
        brief: Database name
        examples: ["customers", "main"]
        requirement_level:
          conditionally_required: If applicable
      - id: operation
        type: string
        brief: Database operation name
        examples: ["SELECT", "INSERT", "findAndModify"]
        requirement_level:
          conditionally_required: If readily available
      - id: statement
        type: string
        brief: Database statement (sanitized)
        examples: ["SELECT * FROM users WHERE id = ?"]
        requirement_level: recommended

  # Database SQL Spans
  - id: db.sql
    prefix: db
    type: span
    brief: Semantic conventions for SQL databases
    extends: db
    attributes:
      - id: sql.table
        type: string
        brief: Primary table being operated on
        examples: ["users", "orders"]
        requirement_level: recommended

  # Messaging Spans
  - id: messaging
    prefix: messaging
    type: span
    brief: Semantic conventions for messaging systems
    attributes:
      - id: system
        type: string
        brief: Messaging system identifier
        examples: ["kafka", "rabbitmq", "sqs", "nats"]
        requirement_level: required
      - id: destination.name
        type: string
        brief: Message destination name
        examples: ["orders", "user-events"]
        requirement_level: required
      - id: operation
        type: string
        brief: Type of operation (publish, receive, process)
        examples: ["publish", "receive", "process"]
        requirement_level: required
      - id: message.id
        type: string
        brief: Unique message identifier
        requirement_level: recommended
      - id: message.body.size
        type: int
        brief: Message body size in bytes
        requirement_level: recommended
      - id: batch.message_count
        type: int
        brief: Number of messages in a batch
        requirement_level:
          conditionally_required: If batch operation

  # gRPC Spans
  - id: rpc.grpc
    prefix: rpc
    type: span
    brief: Semantic conventions for gRPC
    attributes:
      - id: system
        type: string
        brief: RPC system (always "grpc" for gRPC)
        requirement_level: required
      - id: service
        type: string
        brief: Full gRPC service name
        examples: ["myservice.EchoService"]
        requirement_level: required
      - id: method
        type: string
        brief: gRPC method name
        examples: ["Echo", "Ping"]
        requirement_level: required
      - id: grpc.status_code
        type: int
        brief: gRPC status code
        examples: [0, 1, 2]
        requirement_level: required

  # Exception Events
  - id: exception
    prefix: exception
    type: event
    brief: Semantic conventions for exception events
    attributes:
      - id: type
        type: string
        brief: Exception type/class name
        examples: ["java.lang.NullPointerException", "ValueError"]
        requirement_level: recommended
      - id: message
        type: string
        brief: Exception message
        requirement_level: recommended
      - id: stacktrace
        type: string
        brief: Exception stacktrace
        requirement_level: recommended
      - id: escaped
        type: boolean
        brief: Whether exception escaped the span scope
        requirement_level: recommended

  # Resource Attributes
  - id: service
    prefix: service
    type: resource
    brief: Service resource attributes
    attributes:
      - id: name
        type: string
        brief: Logical service name
        examples: ["api-gateway", "user-service"]
        requirement_level: required
      - id: version
        type: string
        brief: Service version
        examples: ["1.0.0", "2.3.4-beta"]
        requirement_level: recommended
      - id: namespace
        type: string
        brief: Service namespace
        examples: ["production", "staging"]
        requirement_level: recommended
      - id: instance.id
        type: string
        brief: Unique instance identifier
        requirement_level: recommended

  # Process Attributes
  - id: process
    prefix: process
    type: resource
    brief: Process resource attributes
    attributes:
      - id: pid
        type: int
        brief: Process ID
        requirement_level: recommended
      - id: executable.name
        type: string
        brief: Process executable name
        examples: ["node", "python3"]
        requirement_level: recommended
      - id: command
        type: string
        brief: Full command used to start process
        requirement_level: recommended
      - id: runtime.name
        type: string
        brief: Runtime environment name
        examples: ["OpenJDK Runtime Environment", "Node.js"]
        requirement_level: recommended
      - id: runtime.version
        type: string
        brief: Runtime version
        examples: ["17.0.1", "18.16.0"]
        requirement_level: recommended

  # Host Attributes
  - id: host
    prefix: host
    type: resource
    brief: Host resource attributes
    attributes:
      - id: name
        type: string
        brief: Hostname
        examples: ["web-server-1", "prod-api-01"]
        requirement_level: recommended
      - id: id
        type: string
        brief: Unique host identifier
        requirement_level: recommended
      - id: type
        type: string
        brief: Host type (cloud instance type, etc.)
        examples: ["n1-standard-1", "t2.micro"]
        requirement_level: recommended
      - id: arch
        type: string
        brief: CPU architecture
        examples: ["amd64", "arm64"]
        requirement_level: recommended

  # Cloud Attributes
  - id: cloud
    prefix: cloud
    type: resource
    brief: Cloud resource attributes
    attributes:
      - id: provider
        type: string
        brief: Cloud provider name
        examples: ["aws", "gcp", "azure"]
        requirement_level: recommended
      - id: region
        type: string
        brief: Cloud region
        examples: ["us-east-1", "europe-west1"]
        requirement_level: recommended
      - id: availability_zone
        type: string
        brief: Cloud availability zone
        examples: ["us-east-1a", "europe-west1-b"]
        requirement_level: recommended
      - id: account.id
        type: string
        brief: Cloud account ID
        requirement_level: recommended
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = OpenTelemetryProvider::new();
        assert_eq!(provider.name(), "OpenTelemetryProvider");
    }

    #[test]
    fn test_resolve_embedded() {
        let provider = OpenTelemetryProvider::new();
        let params = ProviderParams::default();

        let schema = provider.resolve_schema("embedded", &params).unwrap();
        match schema {
            Schema::Custom(content) => {
                assert!(content.contains("http.client"));
                assert!(content.contains("db"));
            }
            _ => panic!("Expected Custom schema"),
        }
    }

    #[test]
    fn test_generate_types() {
        let provider = OpenTelemetryProvider::new();
        let params = ProviderParams::default();

        let schema = provider.resolve_schema("embedded", &params).unwrap();
        let types = provider.generate_types(&schema, "OTel").unwrap();

        // Should have modules for http, db, messaging, etc.
        assert!(!types.modules.is_empty());

        // Find HTTP module
        let http_module = types.modules.iter()
            .find(|m| m.path.contains(&"http".to_string()) || m.path.contains(&"Http".to_string()));
        assert!(http_module.is_some(), "Should have HTTP module");
    }

    #[test]
    fn test_field_naming() {
        let provider = OpenTelemetryProvider::new();
        let params = ProviderParams::default();

        let schema = provider.resolve_schema("embedded", &params).unwrap();
        let types = provider.generate_types(&schema, "OTel").unwrap();

        // Find a type and check field naming
        for module in &types.modules {
            for type_def in &module.types {
                if let TypeDefinition::Record(record) = type_def {
                    // Fields should be camelCase (e.g., requestMethod, responseStatusCode)
                    for (field_name, _) in &record.fields {
                        // First char should be lowercase (camelCase)
                        let first_char = field_name.chars().next().unwrap();
                        assert!(
                            first_char.is_lowercase() || first_char.is_numeric(),
                            "Field '{}' should be camelCase",
                            field_name
                        );
                    }
                }
            }
        }
    }
}
