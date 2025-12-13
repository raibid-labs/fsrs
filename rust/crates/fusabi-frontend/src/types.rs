//! Type System Infrastructure for Fusabi
//!
//! This module implements the foundational type system for Hindley-Milner type inference.
//! It provides the core type representation, type variables, type schemes, substitutions,
//! and type environments needed for type checking F# expressions.
//!
//! # Architecture
//!
//! The type system follows the Hindley-Milner type inference algorithm with:
//! - **Type**: Core type representation (primitives, functions, type variables)
//! - **TypeVar**: Type variables ('a, 'b, etc.) for polymorphism
//! - **TypeScheme**: Type schemes for let-polymorphism (∀a. τ)
//! - **Substitution**: Type variable substitutions [α ↦ τ]
//! - **TypeEnv**: Type environment (Γ) mapping names to type schemes
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::types::{Type, TypeVar, TypeEnv, Substitution};
//!
//! // Create a type variable 'a
//! let var_a = TypeVar::new(0, "a");
//! let ty = Type::Var(var_a.clone());
//!
//! // Create a substitution ['a -> int]
//! let subst = Substitution::singleton(var_a, Type::Int);
//!
//! // Apply substitution
//! let result = subst.apply_type(&ty);
//! assert_eq!(result, Type::Int);
//! ```

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

/// Type variable identifier.
///
/// Type variables represent unknown types during type inference.
/// Examples: 'a, 'b, 't1, 't2
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    /// Unique identifier for this type variable
    pub id: usize,
    /// Human-readable name (e.g., "a", "b", "t1")
    pub name: String,
}

impl TypeVar {
    /// Create a new type variable with the given id and name.
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        TypeVar {
            id,
            name: name.into(),
        }
    }

    /// Create a fresh type variable with a generated name.
    pub fn fresh(id: usize) -> Self {
        TypeVar {
            id,
            name: format!("t{}", id),
        }
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}", self.name)
    }
}

/// Core type representation.
///
/// Represents all types in the Fusabi type system, including:
/// - Primitive types (int, bool, string, unit)
/// - Composite types (tuples, lists, arrays, records)
/// - Function types
/// - Type variables (for inference)
/// - Discriminated unions
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Type variable (e.g., 'a, 'b)
    Var(TypeVar),

    /// Integer type
    Int,

    /// Boolean type
    Bool,

    /// String type
    String,

    /// Unit type ()
    Unit,

    /// Float type
    Float,

    /// Tuple type (e.g., int * string * bool)
    Tuple(Vec<Type>),

    /// List type (e.g., int list, 'a list)
    List(Box<Type>),

    /// Array type (e.g., int[], 'a[])
    Array(Box<Type>),

    /// Function type (e.g., int -> int, 'a -> 'b)
    Function(Box<Type>, Box<Type>),

    /// Record type with named fields
    Record(HashMap<String, Type>),

    /// Discriminated union variant (type name, type parameters)
    Variant(String, Vec<Type>),
}

impl Type {
    /// Get all free type variables in this type.
    ///
    /// A free type variable is one that is not bound by any quantifier.
    /// Used for generalization and substitution.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        match self {
            Type::Var(v) => {
                let mut set = HashSet::new();
                set.insert(v.clone());
                set
            }
            Type::Int | Type::Bool | Type::String | Type::Unit | Type::Float => HashSet::new(),
            Type::Tuple(types) => types.iter().flat_map(|t| t.free_vars()).collect(),
            Type::List(t) | Type::Array(t) => t.free_vars(),
            Type::Function(arg, ret) => {
                let mut set = arg.free_vars();
                set.extend(ret.free_vars());
                set
            }
            Type::Record(fields) => fields.values().flat_map(|t| t.free_vars()).collect(),
            Type::Variant(_, params) => params.iter().flat_map(|t| t.free_vars()).collect(),
        }
    }

    /// Apply a substitution to this type.
    ///
    /// Replaces all occurrences of type variables according to the substitution.
    pub fn apply(&self, subst: &Substitution) -> Type {
        match self {
            Type::Var(v) => subst.lookup(v).unwrap_or_else(|| self.clone()),
            Type::Int | Type::Bool | Type::String | Type::Unit | Type::Float => self.clone(),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| t.apply(subst)).collect()),
            Type::List(t) => Type::List(Box::new(t.apply(subst))),
            Type::Array(t) => Type::Array(Box::new(t.apply(subst))),
            Type::Function(arg, ret) => {
                Type::Function(Box::new(arg.apply(subst)), Box::new(ret.apply(subst)))
            }
            Type::Record(fields) => Type::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.apply(subst)))
                    .collect(),
            ),
            Type::Variant(name, params) => Type::Variant(
                name.clone(),
                params.iter().map(|t| t.apply(subst)).collect(),
            ),
        }
    }

    /// Occurs check: does this type variable occur in this type?
    ///
    /// Used to prevent infinite types during unification.
    /// Returns true if the variable appears anywhere in the type structure.
    pub fn occurs_check(&self, var: &TypeVar) -> bool {
        match self {
            Type::Var(v) => v == var,
            Type::Int | Type::Bool | Type::String | Type::Unit | Type::Float => false,
            Type::Tuple(types) => types.iter().any(|t| t.occurs_check(var)),
            Type::List(t) | Type::Array(t) => t.occurs_check(var),
            Type::Function(arg, ret) => arg.occurs_check(var) || ret.occurs_check(var),
            Type::Record(fields) => fields.values().any(|t| t.occurs_check(var)),
            Type::Variant(_, params) => params.iter().any(|t| t.occurs_check(var)),
        }
    }

    /// Helper to create a function type with multiple arguments.
    ///
    /// Creates a right-associative chain of function types.
    /// Example: `function_multi(&[int, bool], string)` creates `int -> bool -> string`
    pub fn function_multi(args: &[Type], ret: Type) -> Type {
        args.iter().rev().fold(ret, |acc, arg| {
            Type::Function(Box::new(arg.clone()), Box::new(acc))
        })
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Var(v) => write!(f, "{}", v),
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Unit => write!(f, "unit"),
            Type::Float => write!(f, "float"),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::List(t) => write!(f, "{} list", t),
            Type::Array(t) => write!(f, "{}[]", t),
            Type::Function(arg, ret) => {
                // Add parentheses for nested function types
                match **arg {
                    Type::Function(_, _) => write!(f, "({}) -> {}", arg, ret),
                    _ => write!(f, "{} -> {}", arg, ret),
                }
            }
            Type::Record(fields) => {
                write!(f, "{{")?;
                let mut first = true;
                for (name, ty) in fields {
                    if !first {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", name, ty)?;
                    first = false;
                }
                write!(f, "}}")
            }
            Type::Variant(name, params) if params.is_empty() => write!(f, "{}", name),
            Type::Variant(name, params) => {
                write!(f, "{}<", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ">")
            }
        }
    }
}

/// Type substitution mapping type variables to types.
///
/// Represents the substitution [α ↦ τ] in the type inference algorithm.
/// Substitutions are composed during unification.
#[derive(Debug, Clone, PartialEq)]
pub struct Substitution {
    /// Map from type variables to their substituted types
    mappings: HashMap<TypeVar, Type>,
}

impl Substitution {
    /// Create an empty substitution (identity).
    pub fn empty() -> Self {
        Substitution {
            mappings: HashMap::new(),
        }
    }

    /// Create a substitution with a single mapping [var ↦ ty].
    pub fn singleton(var: TypeVar, ty: Type) -> Self {
        let mut mappings = HashMap::new();
        mappings.insert(var, ty);
        Substitution { mappings }
    }

    /// Lookup a type variable in the substitution.
    ///
    /// Returns the substituted type if present, None otherwise.
    pub fn lookup(&self, var: &TypeVar) -> Option<Type> {
        self.mappings.get(var).cloned()
    }

    /// Insert or update a mapping in the substitution.
    pub fn insert(&mut self, var: TypeVar, ty: Type) {
        self.mappings.insert(var, ty);
    }

    /// Compose two substitutions: s1 ∘ s2
    ///
    /// The result applies s2 first, then s1.
    /// For all types t: (s1 ∘ s2)(t) = s1(s2(t))
    pub fn compose(s1: &Self, s2: &Self) -> Self {
        let mut mappings = HashMap::new();

        // Apply s1 to all values in s2
        for (var, ty) in &s2.mappings {
            mappings.insert(var.clone(), ty.apply(s1));
        }

        // Add mappings from s1 that aren't in s2
        for (var, ty) in &s1.mappings {
            if !mappings.contains_key(var) {
                mappings.insert(var.clone(), ty.clone());
            }
        }

        Substitution { mappings }
    }

    /// Apply this substitution to a type.
    pub fn apply_type(&self, ty: &Type) -> Type {
        ty.apply(self)
    }

    /// Apply this substitution to a type scheme.
    pub fn apply_scheme(&self, scheme: &TypeScheme) -> TypeScheme {
        // Remove quantified variables from substitution before applying
        let mut subst = self.clone();
        for var in &scheme.vars {
            subst.mappings.remove(var);
        }
        TypeScheme {
            vars: scheme.vars.clone(),
            ty: scheme.ty.apply(&subst),
        }
    }

    /// Check if this substitution is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Get the number of mappings in this substitution.
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// Iterate over the mappings in this substitution.
    pub fn iter(&self) -> impl Iterator<Item = (&TypeVar, &Type)> {
        self.mappings.iter()
    }
}

impl fmt::Display for Substitution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let mut first = true;
        for (var, ty) in &self.mappings {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{} ↦ {}", var, ty)?;
            first = false;
        }
        write!(f, "]")
    }
}

/// Type scheme for let-polymorphism.
///
/// Represents a polymorphic type ∀a b c. τ
/// Used to type let-bound variables with generalization.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeScheme {
    /// Quantified type variables
    pub vars: Vec<TypeVar>,
    /// The body type
    pub ty: Type,
}

impl TypeScheme {
    /// Create a monomorphic type scheme (no quantified variables).
    pub fn mono(ty: Type) -> Self {
        TypeScheme {
            vars: Vec::new(),
            ty,
        }
    }

    /// Create a polymorphic type scheme.
    pub fn poly(vars: Vec<TypeVar>, ty: Type) -> Self {
        TypeScheme { vars, ty }
    }

    /// Get free type variables in this scheme.
    ///
    /// Returns variables that appear in the type but aren't quantified.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut free = self.ty.free_vars();
        for var in &self.vars {
            free.remove(var);
        }
        free
    }

    /// Apply a substitution to this type scheme.
    ///
    /// Quantified variables are protected from substitution.
    pub fn apply(&self, subst: &Substitution) -> TypeScheme {
        subst.apply_scheme(self)
    }

    /// Check if this scheme is monomorphic (no quantified variables).
    pub fn is_mono(&self) -> bool {
        self.vars.is_empty()
    }

    /// Get the underlying type (ignoring quantifiers).
    pub fn inner_type(&self) -> &Type {
        &self.ty
    }
}

impl fmt::Display for TypeScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.vars.is_empty() {
            write!(f, "{}", self.ty)
        } else {
            write!(f, "∀")?;
            for (i, var) in self.vars.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{}", var)?;
            }
            write!(f, ". {}", self.ty)
        }
    }
}

/// Type environment (Γ) mapping names to type schemes.
///
/// Represents the typing context during type inference.
/// Supports scoping through parent environments.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeEnv {
    /// Bindings in this environment
    bindings: HashMap<String, TypeScheme>,
    /// Parent environment for scoping
    parent: Option<Rc<TypeEnv>>,
}

impl TypeEnv {
    /// Create an empty type environment.
    pub fn new() -> Self {
        TypeEnv {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    /// Create a type environment with a parent.
    pub fn with_parent(parent: Rc<TypeEnv>) -> Self {
        TypeEnv {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    /// Extend the environment with a new binding.
    ///
    /// Returns a new environment with the binding added.
    /// The new environment has this environment as its parent.
    pub fn extend(&self, name: String, scheme: TypeScheme) -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(name, scheme);
        TypeEnv {
            bindings,
            parent: Some(Rc::new(self.clone())),
        }
    }

    /// Add a binding to this environment (mutating).
    pub fn insert(&mut self, name: String, scheme: TypeScheme) {
        self.bindings.insert(name, scheme);
    }

    /// Lookup a name in the environment.
    ///
    /// Searches this environment and all parent environments.
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.lookup(name)))
    }

    /// Get all free type variables in the environment.
    ///
    /// Returns the union of free variables in all type schemes.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut free: HashSet<TypeVar> = self
            .bindings
            .values()
            .flat_map(|scheme| scheme.free_vars())
            .collect();

        if let Some(parent) = &self.parent {
            free.extend(parent.free_vars());
        }

        free
    }

    /// Apply a substitution to all bindings in the environment.
    pub fn apply(&self, subst: &Substitution) -> TypeEnv {
        TypeEnv {
            bindings: self
                .bindings
                .iter()
                .map(|(name, scheme)| (name.clone(), scheme.apply(subst)))
                .collect(),
            parent: self.parent.as_ref().map(|p| Rc::new(p.apply(subst))),
        }
    }

    /// Generalize a type to a type scheme.
    ///
    /// Quantifies over all type variables free in the type
    /// but not free in the environment (closure).
    pub fn generalize(&self, ty: &Type) -> TypeScheme {
        let env_vars = self.free_vars();
        let type_vars = ty.free_vars();

        let quantified: Vec<TypeVar> = type_vars
            .into_iter()
            .filter(|v| !env_vars.contains(v))
            .collect();

        TypeScheme::poly(quantified, ty.clone())
    }

    /// Instantiate a type scheme with fresh type variables.
    ///
    /// Replaces all quantified variables with fresh type variables.
    /// This is the opposite of generalization.
    pub fn instantiate(
        &self,
        scheme: &TypeScheme,
        fresh_var: &mut impl FnMut() -> TypeVar,
    ) -> Type {
        if scheme.vars.is_empty() {
            return scheme.ty.clone();
        }

        let mut subst = Substitution::empty();
        for var in &scheme.vars {
            let fresh = fresh_var();
            subst.insert(var.clone(), Type::Var(fresh));
        }

        scheme.ty.apply(&subst)
    }

    /// Check if the environment is empty.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.parent.is_none()
    }

    /// Get the number of bindings in this environment (not including parent).
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Get all bindings in this environment (not including parent).
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &TypeScheme)> {
        self.bindings.iter()
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TypeEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (name, scheme) in &self.bindings {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", name, scheme)?;
            first = false;
        }
        if let Some(parent) = &self.parent {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "parent: {}", parent)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TypeVar Tests
    // ========================================================================

    #[test]
    fn test_typevar_new() {
        let var = TypeVar::new(0, "a");
        assert_eq!(var.id, 0);
        assert_eq!(var.name, "a");
    }

    #[test]
    fn test_typevar_fresh() {
        let var = TypeVar::fresh(42);
        assert_eq!(var.id, 42);
        assert_eq!(var.name, "t42");
    }

    #[test]
    fn test_typevar_display() {
        let var = TypeVar::new(0, "a");
        assert_eq!(format!("{}", var), "'a");
    }

    #[test]
    fn test_typevar_equality() {
        let var1 = TypeVar::new(0, "a");
        let var2 = TypeVar::new(0, "a");
        let var3 = TypeVar::new(1, "a");
        assert_eq!(var1, var2);
        assert_ne!(var1, var3);
    }

    #[test]
    fn test_typevar_hash() {
        let mut set = HashSet::new();
        let var1 = TypeVar::new(0, "a");
        let var2 = TypeVar::new(0, "a");
        set.insert(var1.clone());
        set.insert(var2);
        assert_eq!(set.len(), 1);
    }

    // ========================================================================
    // Type Tests
    // ========================================================================

    #[test]
    fn test_type_primitives() {
        assert_eq!(format!("{}", Type::Int), "int");
        assert_eq!(format!("{}", Type::Bool), "bool");
        assert_eq!(format!("{}", Type::String), "string");
        assert_eq!(format!("{}", Type::Unit), "unit");
        assert_eq!(format!("{}", Type::Float), "float");
    }

    #[test]
    fn test_type_var() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Var(var);
        assert_eq!(format!("{}", ty), "'a");
    }

    #[test]
    fn test_type_function() {
        let ty = Type::Function(Box::new(Type::Int), Box::new(Type::Bool));
        assert_eq!(format!("{}", ty), "int -> bool");
    }

    #[test]
    fn test_type_function_nested() {
        let ty = Type::Function(
            Box::new(Type::Function(Box::new(Type::Int), Box::new(Type::Bool))),
            Box::new(Type::String),
        );
        assert_eq!(format!("{}", ty), "(int -> bool) -> string");
    }

    #[test]
    fn test_type_tuple() {
        let ty = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
        assert_eq!(format!("{}", ty), "(int * bool * string)");
    }

    #[test]
    fn test_type_list() {
        let ty = Type::List(Box::new(Type::Int));
        assert_eq!(format!("{}", ty), "int list");
    }

    #[test]
    fn test_type_array() {
        let ty = Type::Array(Box::new(Type::Bool));
        assert_eq!(format!("{}", ty), "bool[]");
    }

    #[test]
    fn test_type_record() {
        let mut fields = HashMap::new();
        fields.insert("x".to_string(), Type::Int);
        fields.insert("y".to_string(), Type::Int);
        let ty = Type::Record(fields);
        let display = format!("{}", ty);
        assert!(display.contains("x: int"));
        assert!(display.contains("y: int"));
    }

    #[test]
    fn test_type_variant_simple() {
        let ty = Type::Variant("Option".to_string(), vec![]);
        assert_eq!(format!("{}", ty), "Option");
    }

    #[test]
    fn test_type_variant_with_params() {
        let ty = Type::Variant("Option".to_string(), vec![Type::Int]);
        assert_eq!(format!("{}", ty), "Option<int>");
    }

    #[test]
    fn test_type_function_multi() {
        let ty = Type::function_multi(&[Type::Int, Type::Bool], Type::String);
        assert_eq!(format!("{}", ty), "int -> bool -> string");
    }

    // ========================================================================
    // Type Free Variables Tests
    // ========================================================================

    #[test]
    fn test_free_vars_primitives() {
        assert!(Type::Int.free_vars().is_empty());
        assert!(Type::Bool.free_vars().is_empty());
        assert!(Type::String.free_vars().is_empty());
        assert!(Type::Unit.free_vars().is_empty());
    }

    #[test]
    fn test_free_vars_var() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Var(var.clone());
        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));
    }

    #[test]
    fn test_free_vars_function() {
        let var_a = TypeVar::new(0, "a");
        let var_b = TypeVar::new(1, "b");
        let ty = Type::Function(
            Box::new(Type::Var(var_a.clone())),
            Box::new(Type::Var(var_b.clone())),
        );
        let free = ty.free_vars();
        assert_eq!(free.len(), 2);
        assert!(free.contains(&var_a));
        assert!(free.contains(&var_b));
    }

    #[test]
    fn test_free_vars_tuple() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Tuple(vec![Type::Int, Type::Var(var.clone()), Type::Bool]);
        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));
    }

    #[test]
    fn test_free_vars_list() {
        let var = TypeVar::new(0, "a");
        let ty = Type::List(Box::new(Type::Var(var.clone())));
        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));
    }

    // ========================================================================
    // Occurs Check Tests
    // ========================================================================

    #[test]
    fn test_occurs_check_var_in_var() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Var(var.clone());
        assert!(ty.occurs_check(&var));
    }

    #[test]
    fn test_occurs_check_var_not_in_primitive() {
        let var = TypeVar::new(0, "a");
        assert!(!Type::Int.occurs_check(&var));
        assert!(!Type::Bool.occurs_check(&var));
    }

    #[test]
    fn test_occurs_check_var_in_function() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Function(Box::new(Type::Var(var.clone())), Box::new(Type::Int));
        assert!(ty.occurs_check(&var));
    }

    #[test]
    fn test_occurs_check_var_not_in_function() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Function(Box::new(Type::Int), Box::new(Type::Bool));
        assert!(!ty.occurs_check(&var));
    }

    #[test]
    fn test_occurs_check_var_in_tuple() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Tuple(vec![Type::Int, Type::Var(var.clone())]);
        assert!(ty.occurs_check(&var));
    }

    #[test]
    fn test_occurs_check_var_in_list() {
        let var = TypeVar::new(0, "a");
        let ty = Type::List(Box::new(Type::Var(var.clone())));
        assert!(ty.occurs_check(&var));
    }

    // ========================================================================
    // Substitution Tests
    // ========================================================================

    #[test]
    fn test_substitution_empty() {
        let subst = Substitution::empty();
        assert!(subst.is_empty());
        assert_eq!(subst.len(), 0);
    }

    #[test]
    fn test_substitution_singleton() {
        let var = TypeVar::new(0, "a");
        let subst = Substitution::singleton(var.clone(), Type::Int);
        assert_eq!(subst.len(), 1);
        assert_eq!(subst.lookup(&var), Some(Type::Int));
    }

    #[test]
    fn test_substitution_lookup() {
        let var = TypeVar::new(0, "a");
        let subst = Substitution::singleton(var.clone(), Type::Int);
        assert_eq!(subst.lookup(&var), Some(Type::Int));

        let other_var = TypeVar::new(1, "b");
        assert_eq!(subst.lookup(&other_var), None);
    }

    #[test]
    fn test_substitution_apply_var() {
        let var = TypeVar::new(0, "a");
        let subst = Substitution::singleton(var.clone(), Type::Int);
        let ty = Type::Var(var);
        assert_eq!(subst.apply_type(&ty), Type::Int);
    }

    #[test]
    fn test_substitution_apply_function() {
        let var_a = TypeVar::new(0, "a");
        let var_b = TypeVar::new(1, "b");
        let mut subst = Substitution::empty();
        subst.insert(var_a.clone(), Type::Int);
        subst.insert(var_b.clone(), Type::Bool);

        let ty = Type::Function(Box::new(Type::Var(var_a)), Box::new(Type::Var(var_b)));
        let result = subst.apply_type(&ty);
        assert_eq!(
            result,
            Type::Function(Box::new(Type::Int), Box::new(Type::Bool))
        );
    }

    #[test]
    fn test_substitution_compose_empty() {
        let s1 = Substitution::empty();
        let s2 = Substitution::empty();
        let result = Substitution::compose(&s1, &s2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_substitution_compose_basic() {
        let var_a = TypeVar::new(0, "a");
        let var_b = TypeVar::new(1, "b");

        let s1 = Substitution::singleton(var_a.clone(), Type::Int);
        let s2 = Substitution::singleton(var_b.clone(), Type::Var(var_a.clone()));

        let result = Substitution::compose(&s1, &s2);

        // s2[b -> a], then s1[a -> int], so b -> int
        assert_eq!(result.lookup(&var_b), Some(Type::Int));
    }

    #[test]
    fn test_substitution_display() {
        let var = TypeVar::new(0, "a");
        let subst = Substitution::singleton(var, Type::Int);
        let display = format!("{}", subst);
        assert!(display.contains("'a"));
        assert!(display.contains("int"));
    }

    // ========================================================================
    // TypeScheme Tests
    // ========================================================================

    #[test]
    fn test_typescheme_mono() {
        let scheme = TypeScheme::mono(Type::Int);
        assert!(scheme.is_mono());
        assert_eq!(scheme.inner_type(), &Type::Int);
        assert_eq!(format!("{}", scheme), "int");
    }

    #[test]
    fn test_typescheme_poly() {
        let var = TypeVar::new(0, "a");
        let scheme = TypeScheme::poly(vec![var.clone()], Type::Var(var));
        assert!(!scheme.is_mono());
        assert_eq!(format!("{}", scheme), "∀'a. 'a");
    }

    #[test]
    fn test_typescheme_free_vars_mono() {
        let var = TypeVar::new(0, "a");
        let scheme = TypeScheme::mono(Type::Var(var.clone()));
        let free = scheme.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));
    }

    #[test]
    fn test_typescheme_free_vars_poly() {
        let var = TypeVar::new(0, "a");
        let scheme = TypeScheme::poly(vec![var.clone()], Type::Var(var.clone()));
        let free = scheme.free_vars();
        assert!(free.is_empty()); // 'a is quantified
    }

    #[test]
    fn test_typescheme_apply() {
        let var_a = TypeVar::new(0, "a");
        let var_b = TypeVar::new(1, "b");
        let scheme = TypeScheme::poly(
            vec![var_a.clone()],
            Type::Function(
                Box::new(Type::Var(var_a.clone())),
                Box::new(Type::Var(var_b.clone())),
            ),
        );

        let subst = Substitution::singleton(var_b.clone(), Type::Int);
        let result = scheme.apply(&subst);

        // 'a should not be substituted (quantified), but 'b should
        assert_eq!(
            result.ty,
            Type::Function(Box::new(Type::Var(var_a)), Box::new(Type::Int))
        );
    }

    // ========================================================================
    // TypeEnv Tests
    // ========================================================================

    #[test]
    fn test_typeenv_new() {
        let env = TypeEnv::new();
        assert!(env.is_empty());
        assert_eq!(env.len(), 0);
    }

    #[test]
    fn test_typeenv_insert() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Int));
        assert_eq!(env.len(), 1);
        assert!(env.lookup("x").is_some());
    }

    #[test]
    fn test_typeenv_lookup() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Int));
        assert_eq!(env.lookup("x").unwrap().inner_type(), &Type::Int);
        assert!(env.lookup("y").is_none());
    }

    #[test]
    fn test_typeenv_extend() {
        let env1 = TypeEnv::new();
        let env2 = env1.extend("x".to_string(), TypeScheme::mono(Type::Int));
        assert!(env1.lookup("x").is_none());
        assert!(env2.lookup("x").is_some());
    }

    #[test]
    fn test_typeenv_lookup_parent() {
        let mut env1 = TypeEnv::new();
        env1.insert("x".to_string(), TypeScheme::mono(Type::Int));
        let env2 = TypeEnv::with_parent(Rc::new(env1));
        assert!(env2.lookup("x").is_some());
    }

    #[test]
    fn test_typeenv_free_vars() {
        let var = TypeVar::new(0, "a");
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Var(var.clone())));
        let free = env.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));
    }

    #[test]
    fn test_typeenv_generalize() {
        let var = TypeVar::new(0, "a");
        let env = TypeEnv::new();
        let scheme = env.generalize(&Type::Var(var.clone()));
        assert_eq!(scheme.vars.len(), 1);
        assert_eq!(scheme.vars[0], var);
    }

    #[test]
    fn test_typeenv_generalize_with_env_vars() {
        let var_a = TypeVar::new(0, "a");
        let var_b = TypeVar::new(1, "b");

        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Var(var_a.clone())));

        // Type contains both 'a and 'b, but 'a is in environment
        let ty = Type::Function(
            Box::new(Type::Var(var_a.clone())),
            Box::new(Type::Var(var_b.clone())),
        );
        let scheme = env.generalize(&ty);

        // Only 'b should be quantified
        assert_eq!(scheme.vars.len(), 1);
        assert_eq!(scheme.vars[0], var_b);
    }

    #[test]
    fn test_typeenv_instantiate() {
        let var = TypeVar::new(0, "a");
        let scheme = TypeScheme::poly(vec![var.clone()], Type::Var(var));

        let env = TypeEnv::new();
        let mut counter = 100;
        let result = env.instantiate(&scheme, &mut || {
            counter += 1;
            TypeVar::fresh(counter)
        });

        // Should get a fresh type variable
        match result {
            Type::Var(v) => {
                assert_eq!(v.id, 101);
                assert_eq!(v.name, "t101");
            }
            _ => panic!("Expected type variable"),
        }
    }

    #[test]
    fn test_typeenv_apply() {
        let var = TypeVar::new(0, "a");
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Var(var.clone())));

        let subst = Substitution::singleton(var, Type::Int);
        let new_env = env.apply(&subst);

        assert_eq!(new_env.lookup("x").unwrap().inner_type(), &Type::Int);
    }

    #[test]
    fn test_typeenv_bindings() {
        let mut env = TypeEnv::new();
        env.insert("x".to_string(), TypeScheme::mono(Type::Int));
        env.insert("y".to_string(), TypeScheme::mono(Type::Bool));

        let bindings: Vec<_> = env.bindings().collect();
        assert_eq!(bindings.len(), 2);
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_integration_simple_substitution() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Function(
            Box::new(Type::Var(var.clone())),
            Box::new(Type::Var(var.clone())),
        );
        let subst = Substitution::singleton(var, Type::Int);
        let result = ty.apply(&subst);
        assert_eq!(
            result,
            Type::Function(Box::new(Type::Int), Box::new(Type::Int))
        );
    }

    #[test]
    fn test_integration_generalize_instantiate() {
        let var = TypeVar::new(0, "a");
        let ty = Type::Var(var.clone());

        let env = TypeEnv::new();
        let scheme = env.generalize(&ty);

        let mut counter = 0;
        let result = env.instantiate(&scheme, &mut || {
            counter += 1;
            TypeVar::fresh(counter)
        });

        // Should get a fresh variable different from the original
        match result {
            Type::Var(v) => assert_ne!(v.id, var.id),
            _ => panic!("Expected type variable"),
        }
    }

    #[test]
    fn test_integration_complex_type() {
        // (int -> 'a) -> 'a list
        let var = TypeVar::new(0, "a");
        let ty = Type::Function(
            Box::new(Type::Function(
                Box::new(Type::Int),
                Box::new(Type::Var(var.clone())),
            )),
            Box::new(Type::List(Box::new(Type::Var(var.clone())))),
        );

        let free = ty.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&var));

        let subst = Substitution::singleton(var, Type::Bool);
        let result = ty.apply(&subst);

        assert_eq!(
            result,
            Type::Function(
                Box::new(Type::Function(Box::new(Type::Int), Box::new(Type::Bool))),
                Box::new(Type::List(Box::new(Type::Bool))),
            )
        );
    }
}
