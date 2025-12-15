//! Type Provider Resolution
//!
//! This module bridges the type provider infrastructure with the Fusabi frontend.
//! It resolves type provider declarations and converts generated types to the
//! frontend's type representation.
//!
//! # Resolution Flow
//!
//! 1. Parse `type Alias = Provider<"source">` declarations in the AST
//! 2. Look up the provider in the `ProviderRegistry`
//! 3. Call `resolve_schema` and `generate_types` on the provider
//! 4. Convert generated types to frontend `Type` and `TypeDefinition`
//! 5. Inject resolved types into the `TypeEnv` for type inference

use crate::ast::{
    DuTypeDef, RecordTypeDef, TypeDefinition as AstTypeDef, TypeExpr as AstTypeExpr,
    TypeProviderDecl, VariantDef as AstVariantDef,
};
use crate::types::{Type, TypeEnv, TypeScheme};
use fusabi_type_providers::{
    GeneratedTypes, ProviderParams, ProviderRegistry, TypeDefinition as ProviderTypeDef,
    TypeExpr as ProviderTypeExpr, TypeProvider,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Error type for provider resolution failures
#[derive(Debug, Clone)]
pub enum ResolverError {
    /// Provider not found in registry
    ProviderNotFound(String),
    /// Schema resolution failed
    SchemaError(String),
    /// Type generation failed
    GenerationError(String),
    /// Type conversion failed
    ConversionError(String),
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverError::ProviderNotFound(name) => {
                write!(f, "Type provider not found: {}", name)
            }
            ResolverError::SchemaError(msg) => write!(f, "Schema resolution error: {}", msg),
            ResolverError::GenerationError(msg) => write!(f, "Type generation error: {}", msg),
            ResolverError::ConversionError(msg) => write!(f, "Type conversion error: {}", msg),
        }
    }
}

impl std::error::Error for ResolverError {}

/// Result type for provider resolution operations
pub type ResolverResult<T> = Result<T, ResolverError>;

/// Resolved types from a type provider
#[derive(Debug, Clone)]
pub struct ResolvedTypes {
    /// The alias name (from the type declaration)
    pub alias: String,
    /// Generated type definitions
    pub types: Vec<AstTypeDef>,
    /// Type schemes for type environment injection
    pub type_schemes: HashMap<String, TypeScheme>,
}

/// Resolves type provider declarations to concrete types
#[derive(Default)]
pub struct ProviderResolver {
    /// Registry of available type providers
    registry: ProviderRegistry,
}

impl ProviderResolver {
    /// Create a new provider resolver with an empty registry
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
        }
    }

    /// Create a resolver with a pre-configured registry
    pub fn with_registry(registry: ProviderRegistry) -> Self {
        Self { registry }
    }

    /// Register a type provider
    pub fn register<P: TypeProvider + 'static>(&mut self, provider: P) {
        self.registry.register(Arc::new(provider));
    }

    /// List available providers
    pub fn list_providers(&self) -> Vec<&str> {
        self.registry.list_providers()
    }

    /// Resolve a type provider declaration to concrete types
    pub fn resolve(&self, decl: &TypeProviderDecl) -> ResolverResult<ResolvedTypes> {
        // Get provider from registry
        let provider = self
            .registry
            .get(&decl.provider)
            .ok_or_else(|| ResolverError::ProviderNotFound(decl.provider.clone()))?;

        // Build provider params
        let params = ProviderParams {
            cache_ttl_secs: None,
            custom: decl
                .params
                .iter()
                .cloned()
                .collect::<HashMap<String, String>>(),
        };

        // Resolve schema from source
        let schema = provider
            .resolve_schema(&decl.source, &params)
            .map_err(|e| ResolverError::SchemaError(e.to_string()))?;

        // Generate types from schema
        let generated = provider
            .generate_types(&schema, &decl.name)
            .map_err(|e| ResolverError::GenerationError(e.to_string()))?;

        // Convert to AST types
        let types = self.convert_generated_types(&generated)?;

        // Generate type schemes for TypeEnv
        let type_schemes = self.generate_type_schemes(&types);

        Ok(ResolvedTypes {
            alias: decl.name.clone(),
            types,
            type_schemes,
        })
    }

    /// Convert provider-generated types to AST types
    fn convert_generated_types(&self, generated: &GeneratedTypes) -> ResolverResult<Vec<AstTypeDef>> {
        let mut result = Vec::new();

        // Convert root types
        for type_def in &generated.root_types {
            result.push(self.convert_type_def(type_def)?);
        }

        // Convert module types
        for module in &generated.modules {
            for type_def in &module.types {
                // Prefix type name with module path
                let converted = self.convert_type_def(type_def)?;
                result.push(converted);
            }
        }

        Ok(result)
    }

    /// Convert a single type definition from provider format to AST format
    fn convert_type_def(&self, type_def: &ProviderTypeDef) -> ResolverResult<AstTypeDef> {
        match type_def {
            ProviderTypeDef::Record(record) => {
                let fields = record
                    .fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.convert_type_expr(ty)))
                    .collect();

                Ok(AstTypeDef::Record(RecordTypeDef {
                    name: record.name.clone(),
                    fields,
                }))
            }
            ProviderTypeDef::Du(du) => {
                let variants = du
                    .variants
                    .iter()
                    .map(|v| AstVariantDef {
                        name: v.name.clone(),
                        fields: v.fields.iter().map(|ty| self.convert_type_expr(ty)).collect(),
                    })
                    .collect();

                Ok(AstTypeDef::Du(DuTypeDef {
                    name: du.name.clone(),
                    variants,
                }))
            }
        }
    }

    /// Convert a type expression from provider format to AST format
    fn convert_type_expr(&self, ty: &ProviderTypeExpr) -> AstTypeExpr {
        match ty {
            ProviderTypeExpr::Named(name) => AstTypeExpr::Named(name.clone()),
            ProviderTypeExpr::Tuple(types) => {
                AstTypeExpr::Tuple(types.iter().map(|t| self.convert_type_expr(t)).collect())
            }
            ProviderTypeExpr::Function(param, ret) => AstTypeExpr::Function(
                Box::new(self.convert_type_expr(param)),
                Box::new(self.convert_type_expr(ret)),
            ),
        }
    }

    /// Generate TypeSchemes from AST type definitions for TypeEnv injection
    fn generate_type_schemes(&self, types: &[AstTypeDef]) -> HashMap<String, TypeScheme> {
        let mut schemes = HashMap::new();

        for type_def in types {
            match type_def {
                AstTypeDef::Record(record) => {
                    // Record type is represented as a HashMap of field types
                    let fields: HashMap<String, Type> = record
                        .fields
                        .iter()
                        .map(|(name, ty)| (name.clone(), self.ast_type_to_type(ty)))
                        .collect();
                    let scheme = TypeScheme::mono(Type::Record(fields));
                    schemes.insert(record.name.clone(), scheme);
                }
                AstTypeDef::Du(du) => {
                    // DU type - register the type as a Variant with empty type params
                    let scheme = TypeScheme::mono(Type::Variant(du.name.clone(), vec![]));
                    schemes.insert(du.name.clone(), scheme);

                    // Register variant constructors
                    for variant in &du.variants {
                        let du_type = Type::Variant(du.name.clone(), vec![]);
                        let variant_type = if variant.fields.is_empty() {
                            du_type
                        } else if variant.fields.len() == 1 {
                            Type::Function(
                                Box::new(self.ast_type_to_type(&variant.fields[0])),
                                Box::new(du_type),
                            )
                        } else {
                            // Multi-field variant: (t1 * t2 * ...) -> Type
                            let tuple_fields: Vec<Type> = variant
                                .fields
                                .iter()
                                .map(|f| self.ast_type_to_type(f))
                                .collect();
                            Type::Function(
                                Box::new(Type::Tuple(tuple_fields)),
                                Box::new(du_type),
                            )
                        };
                        schemes.insert(variant.name.clone(), TypeScheme::mono(variant_type));
                    }
                }
                AstTypeDef::Provider(_) => {
                    // Provider declarations should already be resolved
                }
            }
        }

        schemes
    }

    /// Convert AST type expression to inference Type
    fn ast_type_to_type(&self, ty: &AstTypeExpr) -> Type {
        match ty {
            AstTypeExpr::Named(name) => match name.as_str() {
                "int" => Type::Int,
                "float" => Type::Float,
                "bool" => Type::Bool,
                "string" => Type::String,
                "unit" => Type::Unit,
                // Custom types are represented as Variant with empty type params
                _ => Type::Variant(name.clone(), vec![]),
            },
            AstTypeExpr::Tuple(types) => {
                Type::Tuple(types.iter().map(|t| self.ast_type_to_type(t)).collect())
            }
            AstTypeExpr::Function(param, ret) => Type::Function(
                Box::new(self.ast_type_to_type(param)),
                Box::new(self.ast_type_to_type(ret)),
            ),
        }
    }

    /// Inject resolved types into a TypeEnv
    pub fn inject_into_env(&self, resolved: &ResolvedTypes, env: &mut TypeEnv) {
        for (name, scheme) in &resolved.type_schemes {
            env.insert(name.clone(), scheme.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = ProviderResolver::new();
        assert!(resolver.list_providers().is_empty());
    }

    #[test]
    fn test_provider_not_found() {
        let resolver = ProviderResolver::new();
        let decl = TypeProviderDecl::new(
            "MyTypes".to_string(),
            "UnknownProvider".to_string(),
            "source".to_string(),
        );
        let result = resolver.resolve(&decl);
        assert!(matches!(result, Err(ResolverError::ProviderNotFound(_))));
    }

    #[test]
    fn test_type_conversion() {
        let resolver = ProviderResolver::new();

        // Test Named type conversion
        let named = ProviderTypeExpr::Named("int".to_string());
        let converted = resolver.convert_type_expr(&named);
        assert!(matches!(converted, AstTypeExpr::Named(n) if n == "int"));

        // Test Tuple type conversion
        let tuple = ProviderTypeExpr::Tuple(vec![
            ProviderTypeExpr::Named("int".to_string()),
            ProviderTypeExpr::Named("string".to_string()),
        ]);
        let converted = resolver.convert_type_expr(&tuple);
        assert!(matches!(converted, AstTypeExpr::Tuple(_)));
    }

    #[test]
    fn test_ast_to_type_primitives() {
        let resolver = ProviderResolver::new();

        assert!(matches!(
            resolver.ast_type_to_type(&AstTypeExpr::Named("int".to_string())),
            Type::Int
        ));
        assert!(matches!(
            resolver.ast_type_to_type(&AstTypeExpr::Named("string".to_string())),
            Type::String
        ));
        assert!(matches!(
            resolver.ast_type_to_type(&AstTypeExpr::Named("bool".to_string())),
            Type::Bool
        ));
    }

    #[test]
    fn test_ast_to_type_custom() {
        let resolver = ProviderResolver::new();

        // Custom types become Variant types
        let ty = resolver.ast_type_to_type(&AstTypeExpr::Named("Person".to_string()));
        assert!(matches!(ty, Type::Variant(name, params) if name == "Person" && params.is_empty()));
    }
}
