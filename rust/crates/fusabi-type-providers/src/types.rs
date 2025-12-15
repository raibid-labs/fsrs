//! Core type definitions for type providers
//!
//! This module contains the type definitions that are shared between
//! type providers and the rest of the compiler.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Type expressions for field type annotations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeExpr {
    /// Named type (e.g., int, bool, string, UserType)
    Named(String),
    /// Tuple type (e.g., int * string)
    Tuple(Vec<TypeExpr>),
    /// Function type (e.g., int -> string)
    Function(Box<TypeExpr>, Box<TypeExpr>),
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Named(name) => write!(f, "{}", name),
            TypeExpr::Tuple(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
            TypeExpr::Function(arg, ret) => write!(f, "{} -> {}", arg, ret),
        }
    }
}

/// Record type definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDef {
    /// Name of the record type
    pub name: String,
    /// Field definitions: (field_name, field_type)
    pub fields: Vec<(String, TypeExpr)>,
}

impl fmt::Display for RecordDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {{ ", self.name)?;
        for (i, (field_name, field_type)) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}: {}", field_name, field_type)?;
        }
        write!(f, " }}")
    }
}

/// Variant definition in a discriminated union.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantDef {
    /// Name of the variant (e.g., "Some", "None", "Circle")
    pub name: String,
    /// Field types for this variant (empty for simple enums)
    pub fields: Vec<TypeExpr>,
}

impl fmt::Display for VariantDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.fields.is_empty() {
            write!(f, " of ")?;
            for (i, field) in self.fields.iter().enumerate() {
                if i > 0 {
                    write!(f, " * ")?;
                }
                write!(f, "{}", field)?;
            }
        }
        Ok(())
    }
}

impl VariantDef {
    /// Create a new variant with no fields
    pub fn new_simple(name: String) -> Self {
        VariantDef {
            name,
            fields: vec![],
        }
    }

    /// Create a new variant with fields
    pub fn new(name: String, fields: Vec<TypeExpr>) -> Self {
        VariantDef { name, fields }
    }

    /// Returns true if this variant has no fields (simple enum case)
    pub fn is_simple(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the number of fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

/// Discriminated union type definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DuDef {
    /// Name of the DU type
    pub name: String,
    /// Variants/cases of this DU
    pub variants: Vec<VariantDef>,
}

impl fmt::Display for DuDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = ", self.name)?;
        for (i, variant) in self.variants.iter().enumerate() {
            if i > 0 {
                write!(f, " | ")?;
            }
            write!(f, "{}", variant)?;
        }
        Ok(())
    }
}

impl DuDef {
    /// Get all variant names
    pub fn variant_names(&self) -> Vec<&str> {
        self.variants.iter().map(|v| v.name.as_str()).collect()
    }

    /// Find a variant by name
    pub fn find_variant(&self, name: &str) -> Option<&VariantDef> {
        self.variants.iter().find(|v| v.name == name)
    }

    /// Returns true if this is a simple enumeration (all variants have no fields)
    pub fn is_simple_enum(&self) -> bool {
        self.variants.iter().all(|v| v.is_simple())
    }

    /// Returns the number of variants
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

/// Type definition variants (Records or Discriminated Unions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeDefinition {
    /// Record type definition
    Record(RecordDef),
    /// Discriminated union type definition
    Du(DuDef),
}

impl fmt::Display for TypeDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeDefinition::Record(r) => write!(f, "{}", r),
            TypeDefinition::Du(du) => write!(f, "{}", du),
        }
    }
}

/// Field definition - single field in a record type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: TypeExpr,
}

impl FieldDef {
    pub fn new(name: String, field_type: TypeExpr) -> Self {
        Self { name, field_type }
    }
}

/// Result of validating fields against a type definition
#[derive(Debug, Clone)]
pub struct FieldValidationResult {
    /// Fields that are in the type but not provided
    pub missing_fields: Vec<String>,
    /// Fields that are provided but not in the type
    pub extra_fields: Vec<String>,
    /// Whether validation passed (no missing required fields, no extra fields)
    pub is_valid: bool,
}

impl TypeDefinition {
    /// Validate provided fields against this type definition.
    ///
    /// Returns a result indicating which fields are missing or extra.
    /// For record types, validates that all provided fields exist in the type.
    /// For DU types, returns invalid (DUs don't have field constructors this way).
    pub fn validate_fields(&self, provided_fields: &[String]) -> FieldValidationResult {
        match self {
            TypeDefinition::Record(record) => {
                let expected_fields: std::collections::HashSet<&str> =
                    record.fields.iter().map(|(name, _)| name.as_str()).collect();
                let provided_set: std::collections::HashSet<&str> =
                    provided_fields.iter().map(|s| s.as_str()).collect();

                let missing: Vec<String> = expected_fields
                    .difference(&provided_set)
                    .map(|s| s.to_string())
                    .collect();

                let extra: Vec<String> = provided_set
                    .difference(&expected_fields)
                    .map(|s| s.to_string())
                    .collect();

                // For now, we allow missing fields (they might be optional)
                // But we don't allow extra fields
                let is_valid = extra.is_empty();

                FieldValidationResult {
                    missing_fields: missing,
                    extra_fields: extra,
                    is_valid,
                }
            }
            TypeDefinition::Du(_) => {
                // DU types don't support field-based construction
                FieldValidationResult {
                    missing_fields: vec![],
                    extra_fields: provided_fields.to_vec(),
                    is_valid: false,
                }
            }
        }
    }

    /// Get the expected field names for this type (if it's a record)
    pub fn field_names(&self) -> Vec<String> {
        match self {
            TypeDefinition::Record(record) => {
                record.fields.iter().map(|(name, _)| name.clone()).collect()
            }
            TypeDefinition::Du(_) => vec![],
        }
    }
}
