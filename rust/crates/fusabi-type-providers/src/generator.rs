//! Type generation utilities for type providers

use crate::types::{RecordDef, DuDef, VariantDef, TypeExpr, TypeDefinition};

/// Generated types from a provider
#[derive(Debug, Clone)]
pub struct GeneratedTypes {
    /// Nested modules with types
    pub modules: Vec<GeneratedModule>,
    /// Root-level types
    pub root_types: Vec<TypeDefinition>,
}

impl GeneratedTypes {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            root_types: Vec::new(),
        }
    }

    pub fn with_module(mut self, module: GeneratedModule) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_type(mut self, ty: TypeDefinition) -> Self {
        self.root_types.push(ty);
        self
    }
}

impl Default for GeneratedTypes {
    fn default() -> Self {
        Self::new()
    }
}

/// A generated module containing types
#[derive(Debug, Clone)]
pub struct GeneratedModule {
    /// Module path (e.g., ["Core", "V1"])
    pub path: Vec<String>,
    /// Types in this module
    pub types: Vec<TypeDefinition>,
}

impl GeneratedModule {
    pub fn new(path: Vec<String>) -> Self {
        Self {
            path,
            types: Vec::new(),
        }
    }

    pub fn with_type(mut self, ty: TypeDefinition) -> Self {
        self.types.push(ty);
        self
    }
}

/// Naming strategy for generated types
#[derive(Debug, Clone, Copy, Default)]
pub enum NamingStrategy {
    #[default]
    PascalCase,
    CamelCase,
    SnakeCase,
    PreserveOriginal,
}

impl NamingStrategy {
    pub fn apply(&self, name: &str) -> String {
        match self {
            Self::PascalCase => to_pascal_case(name),
            Self::CamelCase => to_camel_case(name),
            Self::SnakeCase => to_snake_case(name),
            Self::PreserveOriginal => name.to_string(),
        }
    }
}

/// Type generator with configurable naming
pub struct TypeGenerator {
    pub naming: NamingStrategy,
}

impl TypeGenerator {
    pub fn new(naming: NamingStrategy) -> Self {
        Self { naming }
    }

    /// Create a record type definition
    pub fn make_record(&self, name: &str, fields: Vec<(String, TypeExpr)>) -> RecordDef {
        RecordDef {
            name: self.naming.apply(name),
            fields: fields.into_iter()
                .map(|(n, t)| (self.naming.apply(&n), t))
                .collect(),
        }
    }

    /// Create a discriminated union type definition
    pub fn make_du(&self, name: &str, variants: Vec<VariantDef>) -> DuDef {
        DuDef {
            name: self.naming.apply(name),
            variants,
        }
    }

    /// Map JSON type name to Fusabi TypeExpr
    pub fn json_type_to_fusabi(&self, json_type: &str) -> TypeExpr {
        match json_type {
            "string" => TypeExpr::Named("string".to_string()),
            "integer" | "int" | "int64" | "int32" => TypeExpr::Named("int".to_string()),
            "number" | "float" | "double" => TypeExpr::Named("float".to_string()),
            "boolean" | "bool" => TypeExpr::Named("bool".to_string()),
            "null" => TypeExpr::Named("unit".to_string()),
            other => TypeExpr::Named(self.naming.apply(other)),
        }
    }
}

impl Default for TypeGenerator {
    fn default() -> Self {
        Self::new(NamingStrategy::PascalCase)
    }
}

// Helper functions for case conversion
fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == '.')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().chain(chars).collect(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("foo_bar"), "FooBar");
        assert_eq!(to_pascal_case("foo-bar"), "FooBar");
        assert_eq!(to_pascal_case("fooBar"), "FooBar");
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(to_snake_case("FooBar"), "foo_bar");
        assert_eq!(to_snake_case("fooBar"), "foo_bar");
    }
}
