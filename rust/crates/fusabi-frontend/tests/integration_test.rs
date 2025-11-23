//! Integration tests for the full Fusabi frontend pipeline
//!
//! Tests the complete flow: source code → lexer → parser → compiler → bytecode

use fusabi_frontend::compiler::Compiler;
use fusabi_frontend::lexer::Lexer;
use fusabi_frontend::parser::Parser;
use fusabi_vm::instruction::Instruction;
use fusabi_vm::value::Value;

/// Helper function to run the full pipeline
fn compile_source(source: &str) -> Result<fusabi_vm::chunk::Chunk, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lex error: {}", e))?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    let chunk = Compiler::compile(&ast).map_err(|e| format!("Compile error: {}", e))?;

    Ok(chunk)
}

#[test]
fn test_integration_simple_literal() {
    let chunk = compile_source("42").unwrap();

    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants[0], Value::Int(42));
    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::Return);
}

#[test]
fn test_integration_simple_addition() {
    let chunk = compile_source("1 + 2").unwrap();

    assert_eq!(chunk.constants.len(), 2);
    assert_eq!(chunk.constants[0], Value::Int(1));
    assert_eq!(chunk.constants[1], Value::Int(2));

    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
    assert_eq!(chunk.instructions[2], Instruction::Add);
    assert_eq!(chunk.instructions[3], Instruction::Return);
}

#[test]
fn test_integration_complex_arithmetic() {
    // (10 + 5) * 2
    let chunk = compile_source("(10 + 5) * 2").unwrap();

    assert_eq!(chunk.constants.len(), 3);

    // Should have: 10, 5, ADD, 2, MUL
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Add)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Mul)));
}

#[test]
fn test_integration_let_binding() {
    let chunk = compile_source("let x = 42 in x + 1").unwrap();

    // Should have constants for 42 and 1
    assert!(chunk.constants.contains(&Value::Int(42)));
    assert!(chunk.constants.contains(&Value::Int(1)));

    // Should have StoreLocal, LoadLocal, and Add instructions
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::StoreLocal(_))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadLocal(_))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Add)));
}

#[test]
fn test_integration_nested_let() {
    let chunk = compile_source("let x = 1 in let y = 2 in x + y").unwrap();

    // Should have both locals accessed
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadLocal(0))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadLocal(1))));
}

#[test]
fn test_integration_if_expression() {
    let chunk = compile_source("if true then 1 else 0").unwrap();

    // Should have constants for true, 1, and 0
    assert_eq!(chunk.constants.len(), 3);

    // Should have conditional jump instructions
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Jump(_))));
}

#[test]
fn test_integration_comparison_in_if() {
    let chunk = compile_source("let x = 10 in if x > 5 then 1 else 0").unwrap();

    // Should have Gt instruction
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Gt)));

    // Should have jump instructions
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
}

#[test]
fn test_integration_all_arithmetic_ops() {
    let test_cases = vec![
        ("10 + 5", Instruction::Add),
        ("10 - 5", Instruction::Sub),
        ("10 * 5", Instruction::Mul),
        ("10 / 5", Instruction::Div),
    ];

    for (source, expected_instr) in test_cases {
        let chunk = compile_source(source).unwrap();
        assert!(
            chunk.instructions.contains(&expected_instr),
            "Source '{}' should generate instruction {:?}",
            source,
            expected_instr
        );
    }
}

#[test]
fn test_integration_all_comparison_ops() {
    let test_cases = vec![
        ("1 = 2", Instruction::Eq),
        ("1 <> 2", Instruction::Neq),
        ("1 < 2", Instruction::Lt),
        ("1 <= 2", Instruction::Lte),
        ("1 > 2", Instruction::Gt),
        ("1 >= 2", Instruction::Gte),
    ];

    for (source, expected_instr) in test_cases {
        let chunk = compile_source(source).unwrap();
        assert!(
            chunk.instructions.contains(&expected_instr),
            "Source '{}' should generate instruction {:?}",
            source,
            expected_instr
        );
    }
}

#[test]
fn test_integration_logical_ops() {
    let test_cases = vec![
        ("true && false", Instruction::And),
        ("true || false", Instruction::Or),
    ];

    for (source, expected_instr) in test_cases {
        let chunk = compile_source(source).unwrap();
        assert!(
            chunk.instructions.contains(&expected_instr),
            "Source '{}' should generate instruction {:?}",
            source,
            expected_instr
        );
    }
}

#[test]
fn test_integration_string_literal() {
    let chunk = compile_source("\"hello world\"").unwrap();

    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants[0], Value::Str("hello world".to_string()));
}

#[test]
fn test_integration_unit_literal() {
    let chunk = compile_source("()").unwrap();

    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants[0], Value::Unit);
}

#[test]
fn test_integration_complex_nested_expression() {
    // let a = 5 in let b = 10 in if a < b then a + b else a * b
    let source = "let a = 5 in let b = 10 in if a < b then a + b else a * b";
    let chunk = compile_source(source).unwrap();

    // Should compile successfully
    assert!(!chunk.instructions.is_empty());

    // Should have comparison
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Lt)));

    // Should have both arithmetic operations
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Add)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Mul)));

    // Should have conditional jumps
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
}

#[test]
fn test_integration_lambda_application() {
    let source = "(fun x -> x + 1) 42";
    let result = compile_source(source);

    // Should compile (even if simplified in Phase 1)
    assert!(result.is_ok());
}

#[test]
fn test_integration_with_comments() {
    let source = r#"
        // This is a comment
        let x = 42 in // Another comment
        x + 1 // Final comment
    "#;

    let chunk = compile_source(source).unwrap();

    // Comments should be ignored, should compile to same as without comments
    assert!(chunk.constants.contains(&Value::Int(42)));
    assert!(chunk.constants.contains(&Value::Int(1)));
}

#[test]
fn test_integration_multiline_expression() {
    let source = r#"
        let x = 10 in
        let y = 20 in
        if x < y then
            x + y
        else
            x * y
    "#;

    let chunk = compile_source(source).unwrap();

    // Should compile successfully
    assert!(!chunk.instructions.is_empty());
}

#[test]
fn test_integration_error_undefined_variable() {
    // With global support, undefined variables compile to LoadGlobal
    let chunk = compile_source("x + 1").unwrap();

    // Check for LoadGlobal instruction for "x"
    // Note: We can't easily check the constant value here due to complexity of matching,
    // but checking for LoadGlobal is sufficient to prove it didn't error.
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::LoadGlobal(_))));
}

#[test]
fn test_integration_error_syntax() {
    let result = compile_source("let x = in x");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Parse error"));
}

#[test]
fn test_integration_deeply_nested_let() {
    let source = "let a = 1 in let b = 2 in let c = 3 in let d = 4 in a + b + c + d";
    let chunk = compile_source(source).unwrap();

    // Should successfully handle deep nesting
    assert!(!chunk.instructions.is_empty());
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Add)));
}

#[test]
fn test_integration_nested_if_expressions() {
    let source = "if true then (if false then 1 else 2) else 3";
    let chunk = compile_source(source).unwrap();

    // Should have multiple jump instructions for nested ifs
    let jump_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::JumpIfFalse(_) | Instruction::Jump(_)))
        .count();

    assert!(jump_count >= 4);
}

#[test]
fn test_integration_boolean_expressions() {
    let source = "let a = true in let b = false in a && b || a";
    let chunk = compile_source(source).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::And)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Or)));
}

#[test]
fn test_integration_all_literal_types() {
    let sources = vec![
        ("42", Value::Int(42)),
        ("true", Value::Bool(true)),
        ("false", Value::Bool(false)),
        ("\"test\"", Value::Str("test".to_string())),
        ("()", Value::Unit),
    ];

    for (source, expected_value) in sources {
        let chunk = compile_source(source).unwrap();
        assert!(
            chunk.constants.contains(&expected_value),
            "Source '{}' should have constant {:?}",
            source,
            expected_value
        );
    }
}

#[test]
fn test_integration_precedence_arithmetic() {
    // 1 + 2 * 3 should be 1 + (2 * 3), not (1 + 2) * 3
    let chunk = compile_source("1 + 2 * 3").unwrap();

    // Should have: 1, 2, 3, MUL (2*3 first), ADD (1 + result)
    // The order of LoadConst should reflect correct precedence
    assert_eq!(chunk.constants.len(), 3);

    // Find the sequence of operations
    let mul_pos = chunk
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::Mul))
        .unwrap();
    let add_pos = chunk
        .instructions
        .iter()
        .position(|i| matches!(i, Instruction::Add))
        .unwrap();

    // MUL should come before ADD
    assert!(mul_pos < add_pos);
}

#[test]
fn test_integration_real_world_factorial_setup() {
    // While we can't compile full recursive factorial yet,
    // we can test the building blocks
    let source = "let n = 5 in if n <= 1 then 1 else n * 2";
    let chunk = compile_source(source).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Lte)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Mul)));
}

#[test]
fn test_integration_real_world_max_function() {
    // max(a, b) = if a > b then a else b
    let source = "let a = 10 in let b = 20 in if a > b then a else b";
    let chunk = compile_source(source).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Gt)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
}

#[test]
fn test_integration_constant_pool_size() {
    // Test that constants are added correctly
    let source = "1 + 2 + 3 + 4 + 5";
    let chunk = compile_source(source).unwrap();

    assert_eq!(chunk.constants.len(), 5);

    // All integers should be in the constant pool
    for i in 1..=5 {
        assert!(chunk.constants.contains(&Value::Int(i)));
    }
}

#[test]
fn test_integration_instruction_count_simple() {
    let chunk = compile_source("42").unwrap();

    // Should be minimal: LoadConst + Return
    assert_eq!(chunk.instructions.len(), 2);
}

#[test]
fn test_integration_instruction_count_binop() {
    let chunk = compile_source("1 + 2").unwrap();

    // LoadConst, LoadConst, Add, Return
    assert_eq!(chunk.instructions.len(), 4);
}

#[test]
fn test_integration_end_to_end_complete_program() {
    // A complete mini program testing multiple features
    let source = r#"
        let x = 10 in
        let y = 20 in
        let sum = x + y in
        let product = x * y in
        if sum > 25 then
            product
        else
            sum
    "#;

    let chunk = compile_source(source).unwrap();

    // Verify it compiles successfully with all expected operations
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Add)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Mul)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::Gt)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::JumpIfFalse(_))));

    // Should have multiple local variables
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::StoreLocal(_))));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadLocal(_))));
}
