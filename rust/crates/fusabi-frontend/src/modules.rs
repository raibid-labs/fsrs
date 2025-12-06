//! Module System for Fusabi Mini-F#
//!
//! This module implements the module system for organizing code in Fusabi,
//! supporting module definitions, imports (open statements), and qualified names.
//!
//! # Features
//!
//! - Module definitions with nested modules
//! - Open imports for bringing module bindings into scope
//! - Qualified name resolution (e.g., Math.add)
//! - Name conflict detection
//! - Type environment tracking per module
//!
//! # Example
//!
//! ```fsharp
//! module Math =
//!     let add x y = x + y
//!     let multiply x y = x * y
//!
//! open Math
//!
//! let result = add 5 10  // Uses Math.add via open
//! let result2 = Math.multiply 3 4  // Qualified access
//! ```

use crate::ast::{DuTypeDef, Expr, RecordTypeDef};
use crate::types::TypeEnv;
use std::collections::HashMap;

/// Module registry for name resolution
///
/// Maintains a registry of all modules and their exported bindings,
/// enabling both qualified and unqualified (via open) name lookups.
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    /// Map from module name to Module
    modules: HashMap<String, Module>,
}

/// A compiled module with its bindings and type environment
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name
    pub name: String,
    /// Value bindings (functions and constants)
    pub bindings: HashMap<String, Expr>,
    /// Type definitions (records and discriminated unions)
    pub types: HashMap<String, TypeDefinition>,
    /// Type environment for this module
    pub type_env: TypeEnv,
}

/// Type definition exported by a module
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    /// Record type definition
    Record(RecordTypeDef),
    /// Discriminated union type definition
    Du(DuTypeDef),
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        ModuleRegistry {
            modules: HashMap::new(),
        }
    }

    /// Create a new module registry with standard library modules pre-registered
    pub fn with_stdlib() -> Self {
        let mut registry = Self::new();
        registry.register_stdlib_modules();
        registry
    }

    /// Register standard library modules
    ///
    /// This registers the standard library modules (List, String, Map, Option, etc.)
    /// as placeholder modules. The actual implementations are provided by the VM
    /// at runtime, but this allows the compiler to resolve qualified names and
    /// imports without errors.
    fn register_stdlib_modules(&mut self) {
        // Helper function to create a placeholder variable that will be resolved at runtime
        let make_global_ref =
            |qualified_name: &str| -> Expr { Expr::Var(qualified_name.to_string()) };

        // List module
        let mut list_bindings = HashMap::new();
        list_bindings.insert("length".to_string(), make_global_ref("List.length"));
        list_bindings.insert("head".to_string(), make_global_ref("List.head"));
        list_bindings.insert("tail".to_string(), make_global_ref("List.tail"));
        list_bindings.insert("reverse".to_string(), make_global_ref("List.reverse"));
        list_bindings.insert("isEmpty".to_string(), make_global_ref("List.isEmpty"));
        list_bindings.insert("append".to_string(), make_global_ref("List.append"));
        list_bindings.insert("concat".to_string(), make_global_ref("List.concat"));
        list_bindings.insert("map".to_string(), make_global_ref("List.map"));
        self.register_module("List".to_string(), list_bindings, HashMap::new());

        // String module
        let mut string_bindings = HashMap::new();
        string_bindings.insert("length".to_string(), make_global_ref("String.length"));
        string_bindings.insert("trim".to_string(), make_global_ref("String.trim"));
        string_bindings.insert("toLower".to_string(), make_global_ref("String.toLower"));
        string_bindings.insert("toUpper".to_string(), make_global_ref("String.toUpper"));
        string_bindings.insert("split".to_string(), make_global_ref("String.split"));
        string_bindings.insert("concat".to_string(), make_global_ref("String.concat"));
        string_bindings.insert("contains".to_string(), make_global_ref("String.contains"));
        string_bindings.insert(
            "startsWith".to_string(),
            make_global_ref("String.startsWith"),
        );
        string_bindings.insert("endsWith".to_string(), make_global_ref("String.endsWith"));
        self.register_module("String".to_string(), string_bindings, HashMap::new());

        // Map module
        let mut map_bindings = HashMap::new();
        map_bindings.insert("empty".to_string(), make_global_ref("Map.empty"));
        map_bindings.insert("add".to_string(), make_global_ref("Map.add"));
        map_bindings.insert("remove".to_string(), make_global_ref("Map.remove"));
        map_bindings.insert("find".to_string(), make_global_ref("Map.find"));
        map_bindings.insert("tryFind".to_string(), make_global_ref("Map.tryFind"));
        map_bindings.insert(
            "containsKey".to_string(),
            make_global_ref("Map.containsKey"),
        );
        map_bindings.insert("isEmpty".to_string(), make_global_ref("Map.isEmpty"));
        map_bindings.insert("count".to_string(), make_global_ref("Map.count"));
        map_bindings.insert("ofList".to_string(), make_global_ref("Map.ofList"));
        map_bindings.insert("toList".to_string(), make_global_ref("Map.toList"));
        self.register_module("Map".to_string(), map_bindings, HashMap::new());

        // Option module
        let mut option_bindings = HashMap::new();
        option_bindings.insert("isSome".to_string(), make_global_ref("Option.isSome"));
        option_bindings.insert("isNone".to_string(), make_global_ref("Option.isNone"));
        option_bindings.insert(
            "defaultValue".to_string(),
            make_global_ref("Option.defaultValue"),
        );
        option_bindings.insert(
            "defaultWith".to_string(),
            make_global_ref("Option.defaultWith"),
        );
        option_bindings.insert("map".to_string(), make_global_ref("Option.map"));
        option_bindings.insert("bind".to_string(), make_global_ref("Option.bind"));
        option_bindings.insert("iter".to_string(), make_global_ref("Option.iter"));
        option_bindings.insert("map2".to_string(), make_global_ref("Option.map2"));
        option_bindings.insert("orElse".to_string(), make_global_ref("Option.orElse"));
        self.register_module("Option".to_string(), option_bindings, HashMap::new());

        // System.Collections.Generic module (for compatibility)
        // This is a common .NET namespace that F# developers might use
        let mut system_collections_generic_bindings = HashMap::new();
        // Map common .NET collection types to Fusabi equivalents
        system_collections_generic_bindings.insert("List".to_string(), make_global_ref("List"));
        system_collections_generic_bindings
            .insert("Dictionary".to_string(), make_global_ref("Map"));
        self.register_module(
            "System.Collections.Generic".to_string(),
            system_collections_generic_bindings,
            HashMap::new(),
        );

        // Support for System.Collections as well
        let mut system_collections_bindings = HashMap::new();
        system_collections_bindings.insert(
            "Generic".to_string(),
            make_global_ref("System.Collections.Generic"),
        );
        self.register_module(
            "System.Collections".to_string(),
            system_collections_bindings,
            HashMap::new(),
        );

        // System module (parent of System.Collections)
        let mut system_bindings = HashMap::new();
        system_bindings.insert(
            "Collections".to_string(),
            make_global_ref("System.Collections"),
        );
        self.register_module("System".to_string(), system_bindings, HashMap::new());
    }

    /// Register a module with its bindings and types
    ///
    /// # Arguments
    ///
    /// * `name` - Module name
    /// * `bindings` - Map of value bindings (name -> expression)
    /// * `types` - Map of type definitions (type name -> definition)
    pub fn register_module(
        &mut self,
        name: String,
        bindings: HashMap<String, Expr>,
        types: HashMap<String, TypeDefinition>,
    ) {
        let module = Module {
            name: name.clone(),
            bindings,
            types,
            type_env: TypeEnv::new(),
        };
        self.modules.insert(name, module);
    }

    /// Resolve a qualified name (e.g., "Math.add")
    ///
    /// # Returns
    ///
    /// The expression bound to the qualified name, or None if not found.
    pub fn resolve_qualified(&self, module_name: &str, binding_name: &str) -> Option<&Expr> {
        self.modules
            .get(module_name)
            .and_then(|m| m.bindings.get(binding_name))
    }

    /// Get all bindings from a module (for "open" imports)
    ///
    /// # Returns
    ///
    /// A reference to all bindings in the module, or None if module not found.
    pub fn get_module_bindings(&self, module_name: &str) -> Option<&HashMap<String, Expr>> {
        self.modules.get(module_name).map(|m| &m.bindings)
    }

    /// Get all type definitions from a module
    ///
    /// # Returns
    ///
    /// A reference to all type definitions in the module, or None if module not found.
    pub fn get_module_types(&self, module_name: &str) -> Option<&HashMap<String, TypeDefinition>> {
        self.modules.get(module_name).map(|m| &m.types)
    }

    /// Check if a module exists
    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// List all registered module names
    pub fn module_names(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Module path for nested modules
///
/// Represents a path like ["Geometry", "Point"] for accessing nested modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    /// Path components (e.g., ["Math", "Geometry"])
    pub components: Vec<String>,
}

impl ModulePath {
    /// Create a new module path from components
    pub fn new(components: Vec<String>) -> Self {
        ModulePath { components }
    }

    /// Create a single-component path
    pub fn single(name: String) -> Self {
        ModulePath {
            components: vec![name],
        }
    }

    /// Get the full qualified name (e.g., "Math.Geometry")
    pub fn qualified_name(&self) -> String {
        self.components.join(".")
    }

    /// Get the last component (e.g., "Geometry" from "Math.Geometry")
    pub fn last(&self) -> Option<&str> {
        self.components.last().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal, TypeExpr};

    #[test]
    fn test_module_registry_new() {
        let registry = ModuleRegistry::new();
        assert_eq!(registry.module_names().len(), 0);
    }

    #[test]
    fn test_module_registry_with_stdlib() {
        let registry = ModuleRegistry::with_stdlib();

        // Should have standard library modules
        assert!(registry.has_module("List"));
        assert!(registry.has_module("String"));
        assert!(registry.has_module("Map"));
        assert!(registry.has_module("Option"));
        assert!(registry.has_module("System"));
        assert!(registry.has_module("System.Collections"));
        assert!(registry.has_module("System.Collections.Generic"));

        // Should be able to resolve stdlib functions
        assert!(registry.resolve_qualified("List", "length").is_some());
        assert!(registry.resolve_qualified("String", "trim").is_some());
        assert!(registry.resolve_qualified("Map", "empty").is_some());
        assert!(registry.resolve_qualified("Option", "isSome").is_some());
    }

    #[test]
    fn test_register_and_resolve_module() {
        let mut registry = ModuleRegistry::new();

        // Create a simple Math module
        let mut bindings = HashMap::new();
        bindings.insert(
            "add".to_string(),
            Expr::Lambda {
                param: "x".to_string(),
                body: Box::new(Expr::Lambda {
                    param: "y".to_string(),
                    body: Box::new(Expr::BinOp {
                        op: crate::ast::BinOp::Add,
                        left: Box::new(Expr::Var("x".to_string())),
                        right: Box::new(Expr::Var("y".to_string())),
                    }),
                }),
            },
        );

        registry.register_module("Math".to_string(), bindings, HashMap::new());

        // Verify module exists
        assert!(registry.has_module("Math"));
        assert_eq!(registry.module_names().len(), 1);

        // Resolve qualified name
        let expr = registry.resolve_qualified("Math", "add");
        assert!(expr.is_some());
        assert!(expr.unwrap().is_lambda());
    }

    #[test]
    fn test_get_module_bindings() {
        let mut registry = ModuleRegistry::new();

        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Expr::Lit(Literal::Int(42)));
        bindings.insert("y".to_string(), Expr::Lit(Literal::Int(100)));

        registry.register_module("Test".to_string(), bindings, HashMap::new());

        let module_bindings = registry.get_module_bindings("Test");
        assert!(module_bindings.is_some());
        assert_eq!(module_bindings.unwrap().len(), 2);
    }

    #[test]
    fn test_resolve_nonexistent_module() {
        let registry = ModuleRegistry::new();
        assert!(!registry.has_module("Nonexistent"));
        assert!(registry.resolve_qualified("Nonexistent", "add").is_none());
    }

    #[test]
    fn test_module_path() {
        let path = ModulePath::new(vec!["Math".to_string(), "Geometry".to_string()]);
        assert_eq!(path.qualified_name(), "Math.Geometry");
        assert_eq!(path.last(), Some("Geometry"));

        let single = ModulePath::single("Math".to_string());
        assert_eq!(single.qualified_name(), "Math");
        assert_eq!(single.last(), Some("Math"));
    }

    #[test]
    fn test_module_with_types() {
        let mut registry = ModuleRegistry::new();

        let mut types = HashMap::new();
        types.insert(
            "Person".to_string(),
            TypeDefinition::Record(RecordTypeDef {
                name: "Person".to_string(),
                fields: vec![
                    ("name".to_string(), TypeExpr::Named("string".to_string())),
                    ("age".to_string(), TypeExpr::Named("int".to_string())),
                ],
            }),
        );

        registry.register_module("Data".to_string(), HashMap::new(), types);

        let module_types = registry.get_module_types("Data");
        assert!(module_types.is_some());
        assert_eq!(module_types.unwrap().len(), 1);
        assert!(module_types.unwrap().contains_key("Person"));
    }
}
