//! Comprehensive tests for the type system infrastructure.
//!
//! This test suite covers all aspects of the Hindley-Milner type system foundation:
//! - Type variables and equality
//! - Type construction and display
//! - Free variables computation
//! - Occurs check for recursive types
//! - Substitution operations
//! - Type schemes (generalization and instantiation)
//! - Type environment operations
//! - Integration tests combining multiple components

use fsrs_frontend::types::{Substitution, Type, TypeEnv, TypeScheme, TypeVar};
use std::collections::HashSet;

// ========================================================================
// TypeVar Tests (6 tests)
// ========================================================================

#[test]
fn test_typevar_creation() {
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

    let fresh = TypeVar::fresh(10);
    assert_eq!(format!("{}", fresh), "'t10");
}

#[test]
fn test_typevar_equality() {
    let var1 = TypeVar::new(0, "a");
    let var2 = TypeVar::new(0, "a");
    let var3 = TypeVar::new(1, "a");
    let var4 = TypeVar::new(0, "b");

    assert_eq!(var1, var2);
    assert_ne!(var1, var3);
    assert_ne!(var1, var4);
}

#[test]
fn test_typevar_hash_uniqueness() {
    let mut set = HashSet::new();
    let var1 = TypeVar::new(0, "a");
    let var2 = TypeVar::new(0, "a");
    let var3 = TypeVar::new(1, "b");

    set.insert(var1);
    set.insert(var2);
    set.insert(var3);

    assert_eq!(set.len(), 2); // var1 and var2 are the same
}

#[test]
fn test_typevar_clone() {
    let var1 = TypeVar::new(0, "a");
    let var2 = var1.clone();
    assert_eq!(var1, var2);
    assert_eq!(var1.id, var2.id);
    assert_eq!(var1.name, var2.name);
}

// ========================================================================
// Type Construction and Display Tests (8 tests)
// ========================================================================

#[test]
fn test_type_primitives_display() {
    assert_eq!(format!("{}", Type::Int), "int");
    assert_eq!(format!("{}", Type::Bool), "bool");
    assert_eq!(format!("{}", Type::String), "string");
    assert_eq!(format!("{}", Type::Unit), "unit");
    assert_eq!(format!("{}", Type::Float), "float");
}

#[test]
fn test_type_function_display() {
    let ty = Type::Function(Box::new(Type::Int), Box::new(Type::Bool));
    assert_eq!(format!("{}", ty), "int -> bool");

    let ty2 = Type::Function(
        Box::new(Type::Int),
        Box::new(Type::Function(Box::new(Type::Bool), Box::new(Type::String))),
    );
    assert_eq!(format!("{}", ty2), "int -> bool -> string");
}

#[test]
fn test_type_function_nested_display() {
    let ty = Type::Function(
        Box::new(Type::Function(Box::new(Type::Int), Box::new(Type::Bool))),
        Box::new(Type::String),
    );
    assert_eq!(format!("{}", ty), "(int -> bool) -> string");
}

#[test]
fn test_type_tuple_display() {
    let ty = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
    assert_eq!(format!("{}", ty), "(int * bool * string)");

    let empty = Type::Tuple(vec![]);
    assert_eq!(format!("{}", empty), "()");
}

#[test]
fn test_type_list_display() {
    let ty = Type::List(Box::new(Type::Int));
    assert_eq!(format!("{}", ty), "int list");

    let ty2 = Type::List(Box::new(Type::Var(TypeVar::new(0, "a"))));
    assert_eq!(format!("{}", ty2), "'a list");
}

#[test]
fn test_type_array_display() {
    let ty = Type::Array(Box::new(Type::Bool));
    assert_eq!(format!("{}", ty), "bool[]");

    let ty2 = Type::Array(Box::new(Type::Var(TypeVar::new(0, "t"))));
    assert_eq!(format!("{}", ty2), "'t[]");
}

#[test]
fn test_type_function_multi() {
    let ty = Type::function_multi(&[Type::Int, Type::Bool, Type::String], Type::Unit);
    assert_eq!(format!("{}", ty), "int -> bool -> string -> unit");
}

#[test]
fn test_type_complex_nested() {
    // (int -> 'a) -> 'a list
    let var = TypeVar::new(0, "a");
    let ty = Type::Function(
        Box::new(Type::Function(
            Box::new(Type::Int),
            Box::new(Type::Var(var.clone())),
        )),
        Box::new(Type::List(Box::new(Type::Var(var)))),
    );
    assert_eq!(format!("{}", ty), "(int -> 'a) -> 'a list");
}

// ========================================================================
// Free Variables Tests (6 tests)
// ========================================================================

#[test]
fn test_free_vars_primitives() {
    assert!(Type::Int.free_vars().is_empty());
    assert!(Type::Bool.free_vars().is_empty());
    assert!(Type::String.free_vars().is_empty());
    assert!(Type::Unit.free_vars().is_empty());
    assert!(Type::Float.free_vars().is_empty());
}

#[test]
fn test_free_vars_single_var() {
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
fn test_free_vars_list_and_array() {
    let var = TypeVar::new(0, "a");

    let list = Type::List(Box::new(Type::Var(var.clone())));
    let list_free = list.free_vars();
    assert_eq!(list_free.len(), 1);
    assert!(list_free.contains(&var));

    let array = Type::Array(Box::new(Type::Var(var.clone())));
    let array_free = array.free_vars();
    assert_eq!(array_free.len(), 1);
    assert!(array_free.contains(&var));
}

#[test]
fn test_free_vars_nested_complex() {
    // ('a -> 'b) -> ('b -> 'c) -> ('a -> 'c)
    let var_a = TypeVar::new(0, "a");
    let var_b = TypeVar::new(1, "b");
    let var_c = TypeVar::new(2, "c");

    let ty = Type::function_multi(
        &[
            Type::Function(
                Box::new(Type::Var(var_a.clone())),
                Box::new(Type::Var(var_b.clone())),
            ),
            Type::Function(
                Box::new(Type::Var(var_b.clone())),
                Box::new(Type::Var(var_c.clone())),
            ),
        ],
        Type::Function(
            Box::new(Type::Var(var_a.clone())),
            Box::new(Type::Var(var_c.clone())),
        ),
    );

    let free = ty.free_vars();
    assert_eq!(free.len(), 3);
    assert!(free.contains(&var_a));
    assert!(free.contains(&var_b));
    assert!(free.contains(&var_c));
}

// ========================================================================
// Occurs Check Tests (5 tests)
// ========================================================================

#[test]
fn test_occurs_check_in_var() {
    let var = TypeVar::new(0, "a");
    let ty = Type::Var(var.clone());
    assert!(ty.occurs_check(&var));

    let other_var = TypeVar::new(1, "b");
    assert!(!ty.occurs_check(&other_var));
}

#[test]
fn test_occurs_check_in_primitives() {
    let var = TypeVar::new(0, "a");
    assert!(!Type::Int.occurs_check(&var));
    assert!(!Type::Bool.occurs_check(&var));
    assert!(!Type::String.occurs_check(&var));
    assert!(!Type::Unit.occurs_check(&var));
}

#[test]
fn test_occurs_check_in_function() {
    let var = TypeVar::new(0, "a");
    let ty1 = Type::Function(Box::new(Type::Var(var.clone())), Box::new(Type::Int));
    assert!(ty1.occurs_check(&var));

    let ty2 = Type::Function(Box::new(Type::Int), Box::new(Type::Var(var.clone())));
    assert!(ty2.occurs_check(&var));

    let ty3 = Type::Function(Box::new(Type::Int), Box::new(Type::Bool));
    assert!(!ty3.occurs_check(&var));
}

#[test]
fn test_occurs_check_in_tuple() {
    let var = TypeVar::new(0, "a");
    let ty1 = Type::Tuple(vec![Type::Int, Type::Var(var.clone()), Type::Bool]);
    assert!(ty1.occurs_check(&var));

    let ty2 = Type::Tuple(vec![Type::Int, Type::Bool]);
    assert!(!ty2.occurs_check(&var));
}

#[test]
fn test_occurs_check_in_nested() {
    let var = TypeVar::new(0, "a");
    let ty = Type::List(Box::new(Type::Tuple(vec![
        Type::Int,
        Type::Var(var.clone()),
    ])));
    assert!(ty.occurs_check(&var));

    let ty2 = Type::Array(Box::new(Type::Function(
        Box::new(Type::Int),
        Box::new(Type::Var(var.clone())),
    )));
    assert!(ty2.occurs_check(&var));
}

// ========================================================================
// Substitution Tests (8 tests)
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
fn test_substitution_apply_nested() {
    let var = TypeVar::new(0, "a");
    let subst = Substitution::singleton(var.clone(), Type::Int);

    let ty = Type::List(Box::new(Type::Tuple(vec![
        Type::Var(var.clone()),
        Type::Bool,
    ])));
    let result = subst.apply_type(&ty);
    let expected = Type::List(Box::new(Type::Tuple(vec![Type::Int, Type::Bool])));
    assert_eq!(result, expected);
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

    // s2: b -> a, then s1: a -> int, so b -> int
    assert_eq!(result.lookup(&var_b), Some(Type::Int));
    assert_eq!(result.lookup(&var_a), Some(Type::Int));
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
// TypeScheme Tests (7 tests)
// ========================================================================

#[test]
fn test_typescheme_mono() {
    let scheme = TypeScheme::mono(Type::Int);
    assert!(scheme.is_mono());
    assert_eq!(scheme.inner_type(), &Type::Int);
    assert_eq!(format!("{}", scheme), "int");
}

#[test]
fn test_typescheme_poly_simple() {
    let var = TypeVar::new(0, "a");
    let scheme = TypeScheme::poly(vec![var.clone()], Type::Var(var));
    assert!(!scheme.is_mono());
    assert_eq!(format!("{}", scheme), "∀'a. 'a");
}

#[test]
fn test_typescheme_poly_function() {
    let var = TypeVar::new(0, "a");
    let ty = Type::Function(
        Box::new(Type::Var(var.clone())),
        Box::new(Type::Var(var.clone())),
    );
    let scheme = TypeScheme::poly(vec![var], ty);
    assert_eq!(format!("{}", scheme), "∀'a. 'a -> 'a");
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
fn test_typescheme_free_vars_partial() {
    let var_a = TypeVar::new(0, "a");
    let var_b = TypeVar::new(1, "b");
    let ty = Type::Function(
        Box::new(Type::Var(var_a.clone())),
        Box::new(Type::Var(var_b.clone())),
    );
    let scheme = TypeScheme::poly(vec![var_a.clone()], ty);

    let free = scheme.free_vars();
    assert_eq!(free.len(), 1);
    assert!(!free.contains(&var_a)); // quantified
    assert!(free.contains(&var_b)); // free
}

#[test]
fn test_typescheme_apply_substitution() {
    let var_a = TypeVar::new(0, "a");
    let var_b = TypeVar::new(1, "b");
    let ty = Type::Function(
        Box::new(Type::Var(var_a.clone())),
        Box::new(Type::Var(var_b.clone())),
    );
    let scheme = TypeScheme::poly(vec![var_a.clone()], ty);

    let subst = Substitution::singleton(var_b.clone(), Type::Int);
    let result = scheme.apply(&subst);

    // 'a should not be substituted (quantified), but 'b should
    assert_eq!(
        result.ty,
        Type::Function(Box::new(Type::Var(var_a)), Box::new(Type::Int))
    );
}

// ========================================================================
// TypeEnv Tests (10 tests)
// ========================================================================

#[test]
fn test_typeenv_new() {
    let env = TypeEnv::new();
    assert!(env.is_empty());
    assert_eq!(env.len(), 0);
}

#[test]
fn test_typeenv_insert_lookup() {
    let mut env = TypeEnv::new();
    env.insert("x".to_string(), TypeScheme::mono(Type::Int));
    assert_eq!(env.len(), 1);

    let scheme = env.lookup("x").unwrap();
    assert_eq!(scheme.inner_type(), &Type::Int);
}

#[test]
fn test_typeenv_lookup_not_found() {
    let env = TypeEnv::new();
    assert!(env.lookup("x").is_none());
}

#[test]
fn test_typeenv_extend() {
    let env1 = TypeEnv::new();
    let env2 = env1.extend("x".to_string(), TypeScheme::mono(Type::Int));

    assert!(env1.lookup("x").is_none());
    assert!(env2.lookup("x").is_some());
    assert_eq!(env2.lookup("x").unwrap().inner_type(), &Type::Int);
}

#[test]
fn test_typeenv_extend_shadowing() {
    let mut env1 = TypeEnv::new();
    env1.insert("x".to_string(), TypeScheme::mono(Type::Int));

    let env2 = env1.extend("x".to_string(), TypeScheme::mono(Type::Bool));

    // env2 should see Bool, env1 should still see Int
    assert_eq!(env1.lookup("x").unwrap().inner_type(), &Type::Int);
    assert_eq!(env2.lookup("x").unwrap().inner_type(), &Type::Bool);
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
fn test_typeenv_generalize_empty_env() {
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
fn test_typeenv_apply_substitution() {
    let var = TypeVar::new(0, "a");
    let mut env = TypeEnv::new();
    env.insert("x".to_string(), TypeScheme::mono(Type::Var(var.clone())));

    let subst = Substitution::singleton(var, Type::Int);
    let new_env = env.apply(&subst);

    assert_eq!(new_env.lookup("x").unwrap().inner_type(), &Type::Int);
}

// ========================================================================
// Integration Tests (5 tests)
// ========================================================================

#[test]
fn test_integration_identity_function() {
    // ∀a. a -> a
    let var = TypeVar::new(0, "a");
    let ty = Type::Function(
        Box::new(Type::Var(var.clone())),
        Box::new(Type::Var(var.clone())),
    );
    let scheme = TypeScheme::poly(vec![var], ty);

    let env = TypeEnv::new();
    let mut counter = 0;
    let inst1 = env.instantiate(&scheme, &mut || {
        counter += 1;
        TypeVar::fresh(counter)
    });
    let inst2 = env.instantiate(&scheme, &mut || {
        counter += 1;
        TypeVar::fresh(counter)
    });

    // Should get different fresh variables each time
    assert_ne!(inst1, inst2);
}

#[test]
fn test_integration_map_function() {
    // ∀a b. (a -> b) -> a list -> b list
    let var_a = TypeVar::new(0, "a");
    let var_b = TypeVar::new(1, "b");
    let ty = Type::function_multi(
        &[
            Type::Function(
                Box::new(Type::Var(var_a.clone())),
                Box::new(Type::Var(var_b.clone())),
            ),
            Type::List(Box::new(Type::Var(var_a.clone()))),
        ],
        Type::List(Box::new(Type::Var(var_b.clone()))),
    );
    let scheme = TypeScheme::poly(vec![var_a, var_b], ty);

    assert!(!scheme.is_mono());
    assert_eq!(scheme.vars.len(), 2);
}

#[test]
fn test_integration_complex_substitution() {
    // Start with: (a -> b) -> a -> b
    let var_a = TypeVar::new(0, "a");
    let var_b = TypeVar::new(1, "b");
    let ty = Type::function_multi(
        &[
            Type::Function(
                Box::new(Type::Var(var_a.clone())),
                Box::new(Type::Var(var_b.clone())),
            ),
            Type::Var(var_a.clone()),
        ],
        Type::Var(var_b.clone()),
    );

    // Substitute a -> int, b -> bool
    let mut subst = Substitution::empty();
    subst.insert(var_a, Type::Int);
    subst.insert(var_b, Type::Bool);

    let result = ty.apply(&subst);
    let expected = Type::function_multi(
        &[
            Type::Function(Box::new(Type::Int), Box::new(Type::Bool)),
            Type::Int,
        ],
        Type::Bool,
    );

    assert_eq!(result, expected);
}

#[test]
fn test_integration_generalize_instantiate_roundtrip() {
    let var = TypeVar::new(0, "a");
    let ty = Type::List(Box::new(Type::Var(var)));

    let env = TypeEnv::new();
    let scheme = env.generalize(&ty);

    assert_eq!(scheme.vars.len(), 1);

    let mut counter = 100;
    let instantiated = env.instantiate(&scheme, &mut || {
        counter += 1;
        TypeVar::fresh(counter)
    });

    // Should be a list with a fresh type variable
    match instantiated {
        Type::List(inner) => match *inner {
            Type::Var(v) => assert_eq!(v.id, 101),
            _ => panic!("Expected type variable"),
        },
        _ => panic!("Expected list type"),
    }
}

#[test]
fn test_integration_environment_scoping() {
    let mut env1 = TypeEnv::new();
    env1.insert("x".to_string(), TypeScheme::mono(Type::Int));

    let env2 = env1.extend("y".to_string(), TypeScheme::mono(Type::Bool));
    let env3 = env2.extend("z".to_string(), TypeScheme::mono(Type::String));

    // env3 should see all bindings
    assert!(env3.lookup("x").is_some());
    assert!(env3.lookup("y").is_some());
    assert!(env3.lookup("z").is_some());

    // env2 should only see x and y
    assert!(env2.lookup("x").is_some());
    assert!(env2.lookup("y").is_some());
    assert!(env2.lookup("z").is_none());

    // env1 should only see x
    assert!(env1.lookup("x").is_some());
    assert!(env1.lookup("y").is_none());
    assert!(env1.lookup("z").is_none());
}
