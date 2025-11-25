//! Integration tests for while loops with break and continue
//!
//! Tests the complete flow for while loops: source code → lexer → parser → compiler → bytecode

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
fn test_while_loop_basic() {
    // Basic while loop: while true do 1
    let chunk = compile_source("while true do 1").unwrap();

    // Should have: LoadConst(true), JumpIfFalse, LoadConst(1), Pop, Jump(back), LoadConst(unit)
    assert!(chunk.constants.contains(&Value::Bool(true)));
    assert!(chunk.constants.contains(&Value::Int(1)));
    assert!(chunk.constants.contains(&Value::Unit));

    // Check for JumpIfFalse instruction
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::JumpIfFalse(_))));
    // Check for backward Jump
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::Jump(offset) if *offset < 0)));
}

#[test]
fn test_while_loop_with_condition() {
    // While loop with a variable condition (requires let binding)
    let source = "let x = 5 in while x > 0 do x";
    let chunk = compile_source(source).unwrap();

    // Should have necessary instructions for condition evaluation
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::Gt)));
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::JumpIfFalse(_))));
}

#[test]
fn test_while_loop_returns_unit() {
    // While loops should return unit
    let chunk = compile_source("while false do 42").unwrap();

    // The last constant loaded should be Unit (return value of while)
    let last_load_const = chunk.instructions.iter()
        .filter_map(|i| match i {
            Instruction::LoadConst(idx) => Some(*idx),
            _ => None,
        })
        .last();

    if let Some(idx) = last_load_const {
        // Should be loading unit
        assert_eq!(chunk.constants[idx as usize], Value::Unit);
    }
}

#[test]
fn test_break_statement() {
    // While loop with break: while true do break
    let chunk = compile_source("while true do break").unwrap();

    // Should have two jumps: JumpIfFalse for condition and Jump for break
    let jump_count = chunk.instructions.iter()
        .filter(|i| matches!(i, Instruction::Jump(_) | Instruction::JumpIfFalse(_)))
        .count();

    // At least: JumpIfFalse (condition), Jump (break), Jump (loop back)
    assert!(jump_count >= 2);
}

#[test]
fn test_continue_statement() {
    // While loop with continue: while true do continue
    let chunk = compile_source("while true do continue").unwrap();

    // Should have jumps: JumpIfFalse for condition, Jump for continue, and backward Jump for loop
    let jump_count = chunk.instructions.iter()
        .filter(|i| matches!(i, Instruction::Jump(_) | Instruction::JumpIfFalse(_)))
        .count();

    assert!(jump_count >= 2);
}

#[test]
fn test_break_outside_loop_fails() {
    // Break outside of a loop should fail to compile
    let result = compile_source("break");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Break"));
}

#[test]
fn test_continue_outside_loop_fails() {
    // Continue outside of a loop should fail to compile
    let result = compile_source("continue");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Continue"));
}

#[test]
fn test_nested_while_loops() {
    // Nested while loops
    let source = "while true do while false do 1";
    let chunk = compile_source(source).unwrap();

    // Should have multiple JumpIfFalse instructions (one per loop)
    let jump_if_false_count = chunk.instructions.iter()
        .filter(|i| matches!(i, Instruction::JumpIfFalse(_)))
        .count();

    assert!(jump_if_false_count >= 2);

    // Should have backward jumps (one per loop)
    let backward_jump_count = chunk.instructions.iter()
        .filter(|i| matches!(i, Instruction::Jump(offset) if *offset < 0))
        .count();

    assert!(backward_jump_count >= 2);
}

#[test]
fn test_break_in_nested_loop() {
    // Break should only exit the innermost loop
    let source = "while true do while true do break";
    let chunk = compile_source(source).unwrap();

    // Compilation should succeed
    assert!(chunk.instructions.len() > 0);
}

#[test]
fn test_continue_in_nested_loop() {
    // Continue should only affect the innermost loop
    let source = "while true do while true do continue";
    let chunk = compile_source(source).unwrap();

    // Compilation should succeed
    assert!(chunk.instructions.len() > 0);
}

#[test]
fn test_while_with_if_and_break() {
    // More complex control flow
    let source = "let x = 10 in while x > 0 do if x == 5 then break else x";
    let chunk = compile_source(source).unwrap();

    // Should have both while and if control flow
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::JumpIfFalse(_))));
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::Jump(_))));
    assert!(chunk.instructions.iter().any(|i| matches!(i, Instruction::Eq)));
}

#[test]
fn test_parsing_while_syntax() {
    // Test that the parser correctly handles "while <cond> do <body>" syntax
    let mut lexer = Lexer::new("while x > 0 do x");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    assert!(ast.is_while());
}

#[test]
fn test_parsing_break_keyword() {
    // Test that break is parsed correctly
    let mut lexer = Lexer::new("break");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    assert!(ast.is_break());
}

#[test]
fn test_parsing_continue_keyword() {
    // Test that continue is parsed correctly
    let mut lexer = Lexer::new("continue");
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    assert!(ast.is_continue());
}
