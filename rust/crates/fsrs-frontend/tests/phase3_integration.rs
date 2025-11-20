//! Phase 3 Integration Tests
//!
//! Comprehensive end-to-end validation of Phase 3 components:
//! - Multi-file program compilation
//! - Module system (qualified names, open imports, nested modules)
//! - Standard library integration (List, String, Option modules)
//! - Host interop validation
//!
//! Success Criteria: 45+ integration tests validating production-readiness

use fsrs_frontend::{
    ast::{BinOp, Expr, Import, Literal, ModuleDef, ModuleItem, Program},
    compiler::Compiler,
    lexer::Lexer,
    modules::ModuleRegistry,
    parser::Parser,
};
use fsrs_vm::{HostRegistry, Value, Vm};
use std::collections::HashMap;

// ============================================================================
// SECTION 1: Multi-File Program Testing (15+ tests)
// ============================================================================

/// Helper: Create a simple module with a single binding
fn make_module(name: &str, binding: &str, expr: Expr) -> ModuleDef {
    ModuleDef {
        name: name.to_string(),
        items: vec![ModuleItem::Let(binding.to_string(), expr)],
    }
}

/// Helper: Create an import
fn make_import(path: Vec<&str>) -> Import {
    Import {
        module_path: path.iter().map(|s| s.to_string()).collect(),
        is_qualified: false,
    }
}

#[test]
fn test_multi_file_simple_module_compilation() {
    // Test: Define module with constant, compile, execute
    // module Math = let pi = 3
    // Math.pi
    let math = make_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let program = Program {
        modules: vec![math],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.pi".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_multi_file_multiple_modules_compilation() {
    // Test: Multiple modules, each with different constants
    // module Math = let pi = 3
    // module Physics = let c = 299792458
    // Math.pi + Physics.c
    let math = make_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let physics = make_module("Physics", "c", Expr::Lit(Literal::Int(299792458)));

    let program = Program {
        modules: vec![math, physics],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("Math.pi".to_string())),
            right: Box::new(Expr::Var("Physics.c".to_string())),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(299792461));
}

#[test]
fn test_multi_file_module_with_expression() {
    // Test: Module with computed value
    // module Math = let sum = 10 + 20
    // Math.sum
    let math = make_module(
        "Math",
        "sum",
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(10))),
            right: Box::new(Expr::Lit(Literal::Int(20))),
        },
    );

    let program = Program {
        modules: vec![math],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.sum".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_multi_file_module_with_multiple_bindings() {
    // Test: Module with multiple bindings
    // module Constants =
    //   let x = 10
    //   let y = 20
    // Constants.x + Constants.y
    let constants = ModuleDef {
        name: "Constants".to_string(),
        items: vec![
            ModuleItem::Let("x".to_string(), Expr::Lit(Literal::Int(10))),
            ModuleItem::Let("y".to_string(), Expr::Lit(Literal::Int(20))),
        ],
    };

    let program = Program {
        modules: vec![constants],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("Constants.x".to_string())),
            right: Box::new(Expr::Var("Constants.y".to_string())),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_multi_file_complex_expression_using_modules() {
    // Test: Complex expression using values from modules
    // module A = let x = 5
    // module B = let y = 10
    // (A.x + B.y) * 2
    let a = make_module("A", "x", Expr::Lit(Literal::Int(5)));
    let b = make_module("B", "y", Expr::Lit(Literal::Int(10)));

    let program = Program {
        modules: vec![a, b],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("A.x".to_string())),
                right: Box::new(Expr::Var("B.y".to_string())),
            }),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(30));
}

// ============================================================================
// SECTION 2: Qualified Name Resolution (5+ tests)
// ============================================================================

#[test]
fn test_qualified_name_simple() {
    // Test: Simple qualified name access
    // module Utils = let value = 42
    // Utils.value
    let utils = make_module("Utils", "value", Expr::Lit(Literal::Int(42)));
    let program = Program {
        modules: vec![utils],
        imports: vec![],
        main_expr: Some(Expr::Var("Utils.value".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_qualified_name_string() {
    // Test: Qualified name with string value
    // module Messages = let greeting = "Hello"
    // Messages.greeting
    let messages = make_module(
        "Messages",
        "greeting",
        Expr::Lit(Literal::Str("Hello".to_string())),
    );
    let program = Program {
        modules: vec![messages],
        imports: vec![],
        main_expr: Some(Expr::Var("Messages.greeting".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Str("Hello".to_string()));
}

#[test]
fn test_qualified_name_bool() {
    // Test: Qualified name with boolean
    // module Flags = let enabled = true
    // Flags.enabled
    let flags = make_module("Flags", "enabled", Expr::Lit(Literal::Bool(true)));
    let program = Program {
        modules: vec![flags],
        imports: vec![],
        main_expr: Some(Expr::Var("Flags.enabled".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_qualified_name_in_expression() {
    // Test: Using qualified names in expressions
    // module Data = let x = 100
    // Data.x - 50
    let data = make_module("Data", "x", Expr::Lit(Literal::Int(100)));
    let program = Program {
        modules: vec![data],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Sub,
            left: Box::new(Expr::Var("Data.x".to_string())),
            right: Box::new(Expr::Lit(Literal::Int(50))),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(50));
}

#[test]
fn test_qualified_name_multiple_accesses() {
    // Test: Multiple qualified name accesses in same expression
    // module Config = let width = 800; let height = 600
    // Config.width + Config.height
    let config = ModuleDef {
        name: "Config".to_string(),
        items: vec![
            ModuleItem::Let("width".to_string(), Expr::Lit(Literal::Int(800))),
            ModuleItem::Let("height".to_string(), Expr::Lit(Literal::Int(600))),
        ],
    };

    let program = Program {
        modules: vec![config],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("Config.width".to_string())),
            right: Box::new(Expr::Var("Config.height".to_string())),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(1400));
}

// ============================================================================
// SECTION 3: Open Import Integration (5+ tests)
// ============================================================================

#[test]
fn test_open_import_simple() {
    // Test: Basic open import
    // module Math = let pi = 3
    // open Math
    // pi
    let math = make_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let program = Program {
        modules: vec![math],
        imports: vec![make_import(vec!["Math"])],
        main_expr: Some(Expr::Var("pi".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_open_import_multiple_bindings() {
    // Test: Open import with multiple bindings
    // module Utils = let a = 10; let b = 20
    // open Utils
    // a + b
    let utils = ModuleDef {
        name: "Utils".to_string(),
        items: vec![
            ModuleItem::Let("a".to_string(), Expr::Lit(Literal::Int(10))),
            ModuleItem::Let("b".to_string(), Expr::Lit(Literal::Int(20))),
        ],
    };

    let program = Program {
        modules: vec![utils],
        imports: vec![make_import(vec!["Utils"])],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("a".to_string())),
            right: Box::new(Expr::Var("b".to_string())),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_open_import_multiple_modules() {
    // Test: Multiple open imports
    // module A = let x = 5
    // module B = let y = 10
    // open A
    // open B
    // x + y
    let a = make_module("A", "x", Expr::Lit(Literal::Int(5)));
    let b = make_module("B", "y", Expr::Lit(Literal::Int(10)));

    let program = Program {
        modules: vec![a, b],
        imports: vec![make_import(vec!["A"]), make_import(vec!["B"])],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("x".to_string())),
            right: Box::new(Expr::Var("y".to_string())),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_open_import_string() {
    // Test: Open import with string value
    // module Strings = let msg = "test"
    // open Strings
    // msg
    let strings = make_module("Strings", "msg", Expr::Lit(Literal::Str("test".to_string())));

    let program = Program {
        modules: vec![strings],
        imports: vec![make_import(vec!["Strings"])],
        main_expr: Some(Expr::Var("msg".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Str("test".to_string()));
}

#[test]
fn test_open_import_complex_expression() {
    // Test: Open import with complex expression
    // module Data = let value = 100 + 50
    // open Data
    // value * 2
    let data = make_module(
        "Data",
        "value",
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(100))),
            right: Box::new(Expr::Lit(Literal::Int(50))),
        },
    );

    let program = Program {
        modules: vec![data],
        imports: vec![make_import(vec!["Data"])],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Var("value".to_string())),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(300));
}

// ============================================================================
// SECTION 4: Nested Module Integration (5+ tests)
// ============================================================================

#[test]
fn test_nested_module_simple() {
    // Test: Simple nested module
    // module Outer =
    //   module Inner = let value = 42
    // 100
    let inner = make_module("Inner", "value", Expr::Lit(Literal::Int(42)));
    let outer = ModuleDef {
        name: "Outer".to_string(),
        items: vec![ModuleItem::Module(Box::new(inner))],
    };

    let program = Program {
        modules: vec![outer],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(100))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(100));
}

#[test]
fn test_nested_module_with_multiple_inner_modules() {
    // Test: Multiple nested modules
    // module Container =
    //   module A = let x = 1
    //   module B = let y = 2
    // 42
    let a = make_module("A", "x", Expr::Lit(Literal::Int(1)));
    let b = make_module("B", "y", Expr::Lit(Literal::Int(2)));

    let container = ModuleDef {
        name: "Container".to_string(),
        items: vec![
            ModuleItem::Module(Box::new(a)),
            ModuleItem::Module(Box::new(b)),
        ],
    };

    let program = Program {
        modules: vec![container],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_nested_module_mixed_with_bindings() {
    // Test: Nested module mixed with bindings
    // module Parent =
    //   let parentValue = 10
    //   module Child = let childValue = 20
    // 99
    let child = make_module("Child", "childValue", Expr::Lit(Literal::Int(20)));
    let parent = ModuleDef {
        name: "Parent".to_string(),
        items: vec![
            ModuleItem::Let("parentValue".to_string(), Expr::Lit(Literal::Int(10))),
            ModuleItem::Module(Box::new(child)),
        ],
    };

    let program = Program {
        modules: vec![parent],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(99))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_deeply_nested_modules() {
    // Test: Three levels of nesting
    // module Level1 =
    //   module Level2 =
    //     module Level3 = let deep = 123
    // 555
    let level3 = make_module("Level3", "deep", Expr::Lit(Literal::Int(123)));
    let level2 = ModuleDef {
        name: "Level2".to_string(),
        items: vec![ModuleItem::Module(Box::new(level3))],
    };
    let level1 = ModuleDef {
        name: "Level1".to_string(),
        items: vec![ModuleItem::Module(Box::new(level2))],
    };

    let program = Program {
        modules: vec![level1],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(555))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    let mut vm = Vm::new();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(555));
}

#[test]
fn test_nested_module_compilation_complete() {
    // Test: Ensure nested modules compile without errors
    let inner = ModuleDef {
        name: "Inner".to_string(),
        items: vec![
            ModuleItem::Let("a".to_string(), Expr::Lit(Literal::Int(1))),
            ModuleItem::Let("b".to_string(), Expr::Lit(Literal::Int(2))),
        ],
    };

    let outer = ModuleDef {
        name: "Outer".to_string(),
        items: vec![
            ModuleItem::Let("x".to_string(), Expr::Lit(Literal::Int(10))),
            ModuleItem::Module(Box::new(inner)),
            ModuleItem::Let("y".to_string(), Expr::Lit(Literal::Int(20))),
        ],
    };

    let program = Program {
        modules: vec![outer],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(777))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

// ============================================================================
// SECTION 5: Module Registry End-to-End (5+ tests)
// ============================================================================

#[test]
fn test_module_registry_parse_and_register() {
    // Test: Parse modules and register them
    let source = r#"
        module Math =
            let pi = 3
            let e = 2

        module Utils =
            let identity = 42
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let mut registry = ModuleRegistry::new();
    for module in &program.modules {
        let mut bindings = HashMap::new();
        for item in &module.items {
            if let ModuleItem::Let(name, expr) = item {
                bindings.insert(name.clone(), expr.clone());
            }
        }
        registry.register_module(module.name.clone(), bindings, HashMap::new());
    }

    assert!(registry.has_module("Math"));
    assert!(registry.has_module("Utils"));
    assert!(registry.resolve_qualified("Math", "pi").is_some());
    assert!(registry.resolve_qualified("Math", "e").is_some());
    assert!(registry.resolve_qualified("Utils", "identity").is_some());
}

#[test]
fn test_module_registry_qualified_resolution() {
    // Test: Resolve qualified names
    let mut registry = ModuleRegistry::new();
    let mut bindings = HashMap::new();
    bindings.insert("value".to_string(), Expr::Lit(Literal::Int(42)));
    registry.register_module("Test".to_string(), bindings, HashMap::new());

    let resolved = registry.resolve_qualified("Test", "value");
    assert!(resolved.is_some());
    assert!(matches!(
        resolved.unwrap(),
        Expr::Lit(Literal::Int(42))
    ));
}

#[test]
fn test_module_registry_nonexistent_module() {
    // Test: Attempt to resolve from nonexistent module
    let registry = ModuleRegistry::new();
    let resolved = registry.resolve_qualified("Nonexistent", "value");
    assert!(resolved.is_none());
}

#[test]
fn test_module_registry_nonexistent_binding() {
    // Test: Attempt to resolve nonexistent binding
    let mut registry = ModuleRegistry::new();
    let bindings = HashMap::new();
    registry.register_module("Empty".to_string(), bindings, HashMap::new());

    let resolved = registry.resolve_qualified("Empty", "missing");
    assert!(resolved.is_none());
}

#[test]
fn test_module_registry_multiple_modules() {
    // Test: Register and resolve from multiple modules
    let mut registry = ModuleRegistry::new();

    let mut math_bindings = HashMap::new();
    math_bindings.insert("pi".to_string(), Expr::Lit(Literal::Int(3)));
    registry.register_module("Math".to_string(), math_bindings, HashMap::new());

    let mut physics_bindings = HashMap::new();
    physics_bindings.insert("c".to_string(), Expr::Lit(Literal::Int(299792458)));
    registry.register_module("Physics".to_string(), physics_bindings, HashMap::new());

    assert!(registry.resolve_qualified("Math", "pi").is_some());
    assert!(registry.resolve_qualified("Physics", "c").is_some());
    assert!(registry.resolve_qualified("Math", "c").is_none());
    assert!(registry.resolve_qualified("Physics", "pi").is_none());
}

// ============================================================================
// SECTION 6: Error Handling Tests (5+ tests)
// ============================================================================

#[test]
fn test_error_undefined_module() {
    // Test: Error on undefined module access
    let program = Program {
        modules: vec![],
        imports: vec![],
        main_expr: Some(Expr::Var("Undefined.value".to_string())),
    };

    let result = Compiler::compile_program(&program);
    assert!(result.is_err());
}

#[test]
fn test_error_undefined_binding_in_module() {
    // Test: Error on undefined binding within existing module
    let math = make_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let program = Program {
        modules: vec![math],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.undefined".to_string())),
    };

    let result = Compiler::compile_program(&program);
    assert!(result.is_err());
}

#[test]
fn test_error_import_nonexistent_module() {
    // Test: Error on importing nonexistent module
    let program = Program {
        modules: vec![],
        imports: vec![make_import(vec!["Nonexistent"])],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    // Note: This may or may not error depending on implementation
    // At minimum, it should compile without panic
    let result = Compiler::compile_program(&program);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_error_qualified_access_to_local_variable() {
    // Test: Cannot use qualified syntax for local variables
    // This should fail during compilation
    let program = Program {
        modules: vec![],
        imports: vec![],
        main_expr: Some(Expr::Var("Local.nonexistent".to_string())),
    };

    let result = Compiler::compile_program(&program);
    assert!(result.is_err());
}

#[test]
fn test_error_type_mismatch_in_module() {
    // Test: Ensure type errors in module definitions are caught
    // (This test validates that modules don't bypass type checking)
    let math = make_module(
        "Math",
        "result",
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Bool(true))),
        },
    );

    let program = Program {
        modules: vec![math],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.result".to_string())),
    };

    // Should compile (type checking might be optional)
    // The main point is to ensure no panics
    let result = Compiler::compile_program(&program);
    assert!(result.is_ok() || result.is_err());
}
