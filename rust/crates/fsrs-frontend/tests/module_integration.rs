//! End-to-end integration tests for module system
//!
//! Tests the complete pipeline:
//! Source → Lexer → Parser → Program → ModuleRegistry

use fsrs_frontend::{ast::ModuleItem, lexer::Lexer, modules::ModuleRegistry, parser::Parser};
use std::collections::HashMap;

#[test]
fn test_e2e_module_parsing_and_registration() {
    // Source with modules
    let source = r#"
        module Math =
            let add x y = x + y
            let multiply x y = x * y

        module Utils =
            let identity x = x
    "#;

    // Parse to Program
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    // Verify structure
    assert_eq!(program.modules.len(), 2);
    assert_eq!(program.imports.len(), 0);
    assert!(program.main_expr.is_none());

    // Register modules
    let mut registry = ModuleRegistry::new();

    for module in &program.modules {
        let mut bindings = HashMap::new();

        for item in &module.items {
            match item {
                ModuleItem::Let(name, expr) => {
                    bindings.insert(name.clone(), expr.clone());
                }
                ModuleItem::LetRec(bindings_vec) => {
                    for (name, expr) in bindings_vec {
                        bindings.insert(name.clone(), expr.clone());
                    }
                }
                ModuleItem::Module(_) => {
                    // Nested modules would be handled here
                }
                ModuleItem::TypeDef(_) => {
                    // Type definitions would be handled here
                }
            }
        }

        registry.register_module(module.name.clone(), bindings, HashMap::new());
    }

    // Verify modules are registered
    assert!(registry.has_module("Math"));
    assert!(registry.has_module("Utils"));

    // Verify we can resolve bindings
    assert!(registry.resolve_qualified("Math", "add").is_some());
    assert!(registry.resolve_qualified("Math", "multiply").is_some());
    assert!(registry.resolve_qualified("Utils", "identity").is_some());

    // Verify nonexistent bindings return None
    assert!(registry.resolve_qualified("Math", "nonexistent").is_none());
    assert!(registry
        .resolve_qualified("NonexistentModule", "add")
        .is_none());
}

#[test]
fn test_e2e_with_imports() {
    let source = r#"
        module Math =
            let add x y = x + y

        module Utils =
            let double x = x + x
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.modules.len(), 2);
    assert_eq!(program.modules[0].name, "Math");
    assert_eq!(program.modules[1].name, "Utils");
}

#[test]
fn test_e2e_module_with_functions() {
    let source = r#"
        module Functions =
            let id x = x
            let compose f g x = f (g x)
            let apply f x = f x
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].items.len(), 3);

    // Verify all items are Let bindings
    for item in &program.modules[0].items {
        assert!(matches!(item, ModuleItem::Let(_, _)));
    }
}

#[test]
fn test_e2e_empty_module() {
    let source = "module Empty =";

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].name, "Empty");
    assert_eq!(program.modules[0].items.len(), 0);
}

#[test]
fn test_e2e_import_parsing() {
    let source = r#"
        open Math
        open Utils.Helpers

        42
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.imports.len(), 2);
    assert_eq!(program.imports[0].module_path, vec!["Math"]);
    assert_eq!(program.imports[1].module_path, vec!["Utils", "Helpers"]);
    assert!(program.main_expr.is_some());
}
