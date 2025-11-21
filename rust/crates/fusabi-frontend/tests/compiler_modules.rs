//! Integration tests for module-aware compilation
//!
//! This test suite validates that the compiler can correctly handle programs with modules,
//! including module definitions, imports, qualified names, and nested modules.
//!
//! NOTE: Phase 1 limitation - Lambda compilation is not yet fully implemented,
//! so these tests focus on module structure and simple expressions.

use fusabi_frontend::ast::{BinOp, Expr, Import, Literal, ModuleDef, ModuleItem, Program};
use fusabi_frontend::compiler::Compiler;

/// Helper to create a simple module with a single let binding
fn make_simple_module(name: &str, binding_name: &str, expr: Expr) -> ModuleDef {
    ModuleDef {
        name: name.to_string(),
        items: vec![ModuleItem::Let(binding_name.to_string(), expr)],
    }
}

/// Helper to create an import statement
fn make_import(module_name: &str) -> Import {
    Import {
        module_path: vec![module_name.to_string()],
        is_qualified: false,
    }
}

#[test]
fn test_compile_empty_program() {
    let program = Program {
        modules: vec![],
        imports: vec![],
        main_expr: None,
    };

    let chunk = Compiler::compile_program(&program).unwrap();

    // Should have at least LoadConst(Unit) and Return
    assert!(chunk.instructions.len() >= 2);
}

#[test]
fn test_compile_program_with_simple_module_constant() {
    // module Math =
    //     let pi = 3
    let math_module = make_simple_module("Math", "pi", Expr::Lit(Literal::Int(3)));

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();

    // Verify chunk has instructions
    assert!(!chunk.instructions.is_empty());
    assert!(!chunk.constants.is_empty());
}

#[test]
fn test_compile_program_with_import() {
    // module Constants =
    //     let value = 10
    //
    // open Constants
    // 42
    let constants_module = make_simple_module("Constants", "value", Expr::Lit(Literal::Int(10)));

    let program = Program {
        modules: vec![constants_module],
        imports: vec![make_import("Constants")],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_qualified_name() {
    // module Math =
    //     let value = 100
    //
    // Math.value
    let math_module = make_simple_module("Math", "value", Expr::Lit(Literal::Int(100)));

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.value".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    // Should contain the constant 100
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(100)));
}

#[test]
fn test_compile_imported_binding() {
    // module Constants =
    //     let answer = 42
    //
    // open Constants
    // answer
    let constants_module = make_simple_module("Constants", "answer", Expr::Lit(Literal::Int(42)));

    let program = Program {
        modules: vec![constants_module],
        imports: vec![make_import("Constants")],
        main_expr: Some(Expr::Var("answer".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(42)));
}

#[test]
fn test_compile_multiple_modules() {
    // module Math =
    //     let pi = 3
    //
    // module Physics =
    //     let c = 300000000
    //
    // 42
    let math_module = make_simple_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let physics_module = make_simple_module("Physics", "c", Expr::Lit(Literal::Int(300000000)));

    let program = Program {
        modules: vec![math_module, physics_module],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_multiple_imports() {
    // module Math =
    //     let pi = 3
    //
    // module Physics =
    //     let c = 300000000
    //
    // open Math
    // open Physics
    // 42
    let math_module = make_simple_module("Math", "pi", Expr::Lit(Literal::Int(3)));
    let physics_module = make_simple_module("Physics", "c", Expr::Lit(Literal::Int(300000000)));

    let program = Program {
        modules: vec![math_module, physics_module],
        imports: vec![make_import("Math"), make_import("Physics")],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_module_with_multiple_bindings() {
    // module Math =
    //     let pi = 3
    //     let e = 2
    //
    // 42
    let math_module = ModuleDef {
        name: "Math".to_string(),
        items: vec![
            ModuleItem::Let("pi".to_string(), Expr::Lit(Literal::Int(3))),
            ModuleItem::Let("e".to_string(), Expr::Lit(Literal::Int(2))),
        ],
    };

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_nested_modules() {
    // module Outer =
    //     module Inner =
    //         let value = 100
    //
    // 42
    let inner_module = make_simple_module("Inner", "value", Expr::Lit(Literal::Int(100)));

    let outer_module = ModuleDef {
        name: "Outer".to_string(),
        items: vec![ModuleItem::Module(Box::new(inner_module))],
    };

    let program = Program {
        modules: vec![outer_module],
        imports: vec![],
        main_expr: Some(Expr::Lit(Literal::Int(42))),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_compile_module_with_constant() {
    // module Constants =
    //     let pi = 3
    //     let e = 2
    //
    // Constants.pi
    let constants_module = ModuleDef {
        name: "Constants".to_string(),
        items: vec![
            ModuleItem::Let("pi".to_string(), Expr::Lit(Literal::Int(3))),
            ModuleItem::Let("e".to_string(), Expr::Lit(Literal::Int(2))),
        ],
    };

    let program = Program {
        modules: vec![constants_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Constants.pi".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    // Should have constant for pi (3)
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(3)));
}

#[test]
fn test_compile_program_using_imported_constant() {
    // module Constants =
    //     let value = 99
    //
    // open Constants
    // value
    let constants_module = make_simple_module("Constants", "value", Expr::Lit(Literal::Int(99)));

    let program = Program {
        modules: vec![constants_module],
        imports: vec![make_import("Constants")],
        main_expr: Some(Expr::Var("value".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(99)));
}

#[test]
fn test_compile_error_undefined_module() {
    // Reference a module that doesn't exist
    let program = Program {
        modules: vec![],
        imports: vec![],
        main_expr: Some(Expr::Var("NonExistent.func".to_string())),
    };

    let result = Compiler::compile_program(&program);

    // Should fail because module doesn't exist
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            fusabi_frontend::compiler::CompileError::UndefinedVariable(_)
        ));
    }
}

#[test]
fn test_compile_error_undefined_binding_in_module() {
    // module Math = let value = 10
    // Math.nonexistent
    let math_module = make_simple_module("Math", "value", Expr::Lit(Literal::Int(10)));

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.nonexistent".to_string())),
    };

    let result = Compiler::compile_program(&program);

    // Should fail because binding doesn't exist in Math module
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(
            e,
            fusabi_frontend::compiler::CompileError::UndefinedVariable(_)
        ));
    }
}

#[test]
fn test_compile_module_with_expression() {
    // module Math =
    //     let sum = 3 + 4
    //
    // Math.sum
    let math_module = make_simple_module(
        "Math",
        "sum",
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(3))),
            right: Box::new(Expr::Lit(Literal::Int(4))),
        },
    );

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Math.sum".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(3)));
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(4)));
}

#[test]
fn test_compile_program_with_expression_using_module() {
    // module Math =
    //     let value = 10
    //
    // Math.value + 5
    let math_module = make_simple_module("Math", "value", Expr::Lit(Literal::Int(10)));

    let program = Program {
        modules: vec![math_module],
        imports: vec![],
        main_expr: Some(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("Math.value".to_string())),
            right: Box::new(Expr::Lit(Literal::Int(5))),
        }),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(10)));
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(5)));
}

#[test]
fn test_compile_imported_expression() {
    // module Math =
    //     let doubled = 21 + 21
    //
    // open Math
    // doubled
    let math_module = make_simple_module(
        "Math",
        "doubled",
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(21))),
            right: Box::new(Expr::Lit(Literal::Int(21))),
        },
    );

    let program = Program {
        modules: vec![math_module],
        imports: vec![make_import("Math")],
        main_expr: Some(Expr::Var("doubled".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk.constants.contains(&fusabi_vm::value::Value::Int(21)));
}

#[test]
fn test_compile_module_with_bool() {
    // module Flags =
    //     let enabled = true
    //
    // Flags.enabled
    let flags_module = make_simple_module("Flags", "enabled", Expr::Lit(Literal::Bool(true)));

    let program = Program {
        modules: vec![flags_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Flags.enabled".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk
        .constants
        .contains(&fusabi_vm::value::Value::Bool(true)));
}

#[test]
fn test_compile_module_with_string() {
    // module Messages =
    //     let greeting = "hello"
    //
    // Messages.greeting
    let messages_module = make_simple_module(
        "Messages",
        "greeting",
        Expr::Lit(Literal::Str("hello".to_string())),
    );

    let program = Program {
        modules: vec![messages_module],
        imports: vec![],
        main_expr: Some(Expr::Var("Messages.greeting".to_string())),
    };

    let chunk = Compiler::compile_program(&program).unwrap();
    assert!(!chunk.instructions.is_empty());
    assert!(chunk
        .constants
        .contains(&fusabi_vm::value::Value::Str("hello".to_string())));
}
