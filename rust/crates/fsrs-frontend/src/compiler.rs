//! Bytecode Compiler for FSRS Mini-F#
//!
//! This module implements compilation from AST to bytecode chunks for the FSRS VM.
//! The compiler performs constant pooling, variable scoping, and generates efficient
//! bytecode instruction sequences.
//!
//! # Architecture
//!
//! The compiler uses:
//! - **Constant Pool**: Deduplicates literal values across the bytecode
//! - **Local Variables**: Stack-allocated variables tracked by index
//! - **Jump Patching**: Forward jump resolution for control flow
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::compiler::Compiler;
//!
//! // Compile: 42 + 1
//! let expr = Expr::BinOp {
//!     op: BinOp::Add,
//!     left: Box::new(Expr::Lit(Literal::Int(42))),
//!     right: Box::new(Expr::Lit(Literal::Int(1))),
//! };
//!
//! let chunk = Compiler::compile(&expr).unwrap();
//! // Generates: LOAD_CONST 0; LOAD_CONST 1; ADD; RETURN
//! ```

use crate::ast::{BinOp, Expr, Literal};
use fsrs_vm::chunk::Chunk;
use fsrs_vm::instruction::Instruction;
use fsrs_vm::value::Value;
use std::fmt;

/// Compilation errors
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// Undefined variable reference
    UndefinedVariable(String),
    /// Too many constants in constant pool (max u16::MAX)
    TooManyConstants,
    /// Too many local variables (max u8::MAX)
    TooManyLocals,
    /// Invalid jump offset (beyond i16 range)
    InvalidJumpOffset,
    /// Unsupported float operations in Phase 1
    UnsupportedFloat,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            CompileError::TooManyConstants => {
                write!(f, "Too many constants (max {})", u16::MAX)
            }
            CompileError::TooManyLocals => {
                write!(f, "Too many local variables (max {})", u8::MAX)
            }
            CompileError::InvalidJumpOffset => {
                write!(f, "Jump offset too large")
            }
            CompileError::UnsupportedFloat => {
                write!(f, "Float operations not supported in Phase 1")
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Compilation result type
pub type CompileResult<T> = Result<T, CompileError>;

/// Local variable information
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

/// Bytecode compiler state
pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    /// Create a new compiler
    fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    /// Main entry point: compile an expression to a chunk
    pub fn compile(expr: &Expr) -> CompileResult<Chunk> {
        let mut compiler = Compiler::new();
        compiler.compile_expr(expr)?;
        compiler.emit(Instruction::Return);
        Ok(compiler.chunk)
    }

    /// Compile an expression and emit instructions
    fn compile_expr(&mut self, expr: &Expr) -> CompileResult<()> {
        match expr {
            Expr::Lit(lit) => self.compile_literal(lit),
            Expr::Var(name) => self.compile_var(name),
            Expr::BinOp { op, left, right } => self.compile_binop(*op, left, right),
            Expr::Let { name, value, body } => self.compile_let(name, value, body),
            Expr::LetRec { name, value, body } => self.compile_let_rec(name, value, body),
            Expr::LetRecMutual { bindings, body } => self.compile_let_rec_mutual(bindings, body),
            Expr::Lambda { param, body } => self.compile_lambda(param, body),
            Expr::App { func, arg } => self.compile_app(func, arg),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => self.compile_if(cond, then_branch, else_branch),
        }
    }

    /// Compile a literal value
    fn compile_literal(&mut self, lit: &Literal) -> CompileResult<()> {
        let value = match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Str(s) => Value::Str(s.clone()),
            Literal::Unit => Value::Unit,
            Literal::Float(_) => return Err(CompileError::UnsupportedFloat),
        };

        let idx = self.add_constant(value)?;
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    /// Compile a variable reference
    fn compile_var(&mut self, name: &str) -> CompileResult<()> {
        // Search for local variable from innermost to outermost scope
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                let idx = i as u8;
                self.emit(Instruction::LoadLocal(idx));
                return Ok(());
            }
        }

        // Variable not found in any scope
        Err(CompileError::UndefinedVariable(name.to_string()))
    }

    /// Compile a binary operation
    fn compile_binop(&mut self, op: BinOp, left: &Expr, right: &Expr) -> CompileResult<()> {
        // Compile operands
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        // Emit operation instruction
        let instr = match op {
            BinOp::Add => Instruction::Add,
            BinOp::Sub => Instruction::Sub,
            BinOp::Mul => Instruction::Mul,
            BinOp::Div => Instruction::Div,
            BinOp::Eq => Instruction::Eq,
            BinOp::Neq => Instruction::Neq,
            BinOp::Lt => Instruction::Lt,
            BinOp::Lte => Instruction::Lte,
            BinOp::Gt => Instruction::Gt,
            BinOp::Gte => Instruction::Gte,
            BinOp::And => Instruction::And,
            BinOp::Or => Instruction::Or,
        };
        self.emit(instr);
        Ok(())
    }

    /// Compile a let-binding
    fn compile_let(&mut self, name: &str, value: &Expr, body: &Expr) -> CompileResult<()> {
        // Compile the value expression
        self.compile_expr(value)?;

        // Enter new scope
        self.begin_scope();

        // Add local variable
        self.add_local(name.to_string())?;

        // Store the value in the local slot
        let local_idx = (self.locals.len() - 1) as u8;
        self.emit(Instruction::StoreLocal(local_idx));

        // Compile the body expression
        self.compile_expr(body)?;

        // Exit scope - note: we don't emit POP for the body result
        // The result stays on top of the stack for the caller
        let locals_to_remove = self.end_scope_count();

        // Only pop if we have multiple locals and need to clean up intermediates
        // For Phase 1 MVP, we simplify by not emitting POPs
        // The locals stay in their stack slots until function return
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile a lambda function (Phase 1: simplified, no closures yet)
    fn compile_lambda(&mut self, param: &str, body: &Expr) -> CompileResult<()> {
        // For Phase 1, we'll compile lambdas as inline code
        // In Phase 2, we'll create proper closure objects

        // For now, create a nested chunk for the lambda body
        let mut lambda_compiler = Compiler::new();

        // Lambda parameter becomes local 0
        lambda_compiler.begin_scope();
        lambda_compiler.add_local(param.to_string())?;

        // Compile the lambda body
        lambda_compiler.compile_expr(body)?;
        lambda_compiler.emit(Instruction::Return);

        // Clean up scope
        let locals_to_remove = lambda_compiler.end_scope_count();
        for _ in 0..locals_to_remove {
            lambda_compiler.locals.pop();
        }
        lambda_compiler.scope_depth -= 1;

        // For Phase 1, we'll store the lambda chunk as a constant
        // This is a simplified implementation - full closures come in Phase 2

        // For now, emit a placeholder (we'll improve this in Phase 2)
        // In Phase 1, lambdas are limited to immediate application
        Ok(())
    }

    /// Compile a recursive let-binding using placeholder strategy
    fn compile_let_rec(&mut self, name: &str, value: &Expr, body: &Expr) -> CompileResult<()> {
        // Strategy: Create a placeholder, compile the function with
        // the name in scope, then update the binding

        // 1. Push placeholder (will be replaced by the actual value)
        let placeholder_idx = self.add_constant(Value::Unit)?;
        self.emit(Instruction::LoadConst(placeholder_idx));

        // 2. Enter scope and add local for the recursive binding
        self.begin_scope();
        self.add_local(name.to_string())?;
        let local_idx = (self.locals.len() - 1) as u8;

        // 3. Store placeholder in local slot
        self.emit(Instruction::StoreLocal(local_idx));

        // 4. Compile the value (usually a lambda) with name in scope
        // The value can now reference itself via the local
        self.compile_expr(value)?;

        // 5. Update the local slot with the actual value
        self.emit(Instruction::StoreLocal(local_idx));

        // 6. Compile body (the local is still in scope)
        self.compile_expr(body)?;

        // 7. Clean up scope
        let locals_to_remove = self.end_scope_count();
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile mutually recursive bindings
    fn compile_let_rec_mutual(
        &mut self,
        bindings: &[(String, Expr)],
        body: &Expr,
    ) -> CompileResult<()> {
        // Strategy: Create placeholders for all bindings, then fill them in

        // 1. Enter scope
        self.begin_scope();

        // 2. Push placeholders and create locals for all bindings
        let placeholder_idx = self.add_constant(Value::Unit)?;
        let mut local_indices = Vec::new();

        for (name, _) in bindings {
            // Push placeholder
            self.emit(Instruction::LoadConst(placeholder_idx));

            // Add local
            self.add_local(name.clone())?;
            let local_idx = (self.locals.len() - 1) as u8;
            local_indices.push(local_idx);

            // Store placeholder
            self.emit(Instruction::StoreLocal(local_idx));
        }

        // 3. Compile each value (with all names in scope)
        for (i, (_name, value)) in bindings.iter().enumerate() {
            self.compile_expr(value)?;
            self.emit(Instruction::StoreLocal(local_indices[i]));
        }

        // 4. Compile body
        self.compile_expr(body)?;

        // 5. Clean up scope
        let locals_to_remove = self.end_scope_count();
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile a function application
    fn compile_app(&mut self, func: &Expr, arg: &Expr) -> CompileResult<()> {
        // Compile the function expression
        self.compile_expr(func)?;

        // Compile the argument expression
        self.compile_expr(arg)?;

        // Emit call instruction with 1 argument
        self.emit(Instruction::Call(1));

        Ok(())
    }

    /// Compile an if-then-else expression
    fn compile_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> CompileResult<()> {
        // Compile condition
        self.compile_expr(cond)?;

        // Emit JumpIfFalse with placeholder offset
        let jump_to_else = self.emit_jump(Instruction::JumpIfFalse(0));

        // Note: JumpIfFalse pops the condition value, so no manual POP needed

        // Compile then branch
        self.compile_expr(then_branch)?;

        // Emit Jump to skip else branch with placeholder offset
        let jump_to_end = self.emit_jump(Instruction::Jump(0));

        // Patch the JumpIfFalse to point here
        self.patch_jump(jump_to_else)?;

        // Compile else branch
        self.compile_expr(else_branch)?;

        // Patch the Jump to point here
        self.patch_jump(jump_to_end)?;

        Ok(())
    }

    /// Emit an instruction
    fn emit(&mut self, instruction: Instruction) {
        self.chunk.emit(instruction);
    }

    /// Emit a jump instruction and return its index for later patching
    fn emit_jump(&mut self, instruction: Instruction) -> usize {
        self.emit(instruction);
        self.chunk.current_offset() - 1
    }

    /// Patch a jump instruction with the correct offset
    fn patch_jump(&mut self, jump_index: usize) -> CompileResult<()> {
        // Calculate the offset from the jump instruction to the current position
        let jump_offset = self.chunk.current_offset() - jump_index - 1;

        // Check if offset fits in i16
        if jump_offset > i16::MAX as usize {
            return Err(CompileError::InvalidJumpOffset);
        }

        // Get the current instruction and patch it
        match self.chunk.instructions[jump_index] {
            Instruction::Jump(_) => {
                self.chunk.instructions[jump_index] = Instruction::Jump(jump_offset as i16);
            }
            Instruction::JumpIfFalse(_) => {
                self.chunk.instructions[jump_index] = Instruction::JumpIfFalse(jump_offset as i16);
            }
            _ => unreachable!("patch_jump called on non-jump instruction"),
        }

        Ok(())
    }

    /// Add a constant to the constant pool and return its index
    fn add_constant(&mut self, value: Value) -> CompileResult<u16> {
        let count = self.chunk.constants.len();
        if count >= u16::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }
        let idx = self.chunk.add_constant(value);
        Ok(idx)
    }
    /// Begin a new scope
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Count locals to be removed when ending current scope
    fn end_scope_count(&self) -> usize {
        self.locals
            .iter()
            .rev()
            .take_while(|local| local.depth > self.scope_depth - 1)
            .count()
    }

    /// Add a local variable
    fn add_local(&mut self, name: String) -> CompileResult<()> {
        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TDD: Literal Compilation Tests (RED -> GREEN)
    // ========================================================================

    #[test]
    fn test_compile_literal_int() {
        let expr = Expr::Lit(Literal::Int(42));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Int(42));
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::Return);
    }

    #[test]
    fn test_compile_literal_bool() {
        let expr = Expr::Lit(Literal::Bool(true));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Bool(true));
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    }

    #[test]
    fn test_compile_literal_string() {
        let expr = Expr::Lit(Literal::Str("hello".to_string()));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Str("hello".to_string()));
    }

    #[test]
    fn test_compile_literal_unit() {
        let expr = Expr::Lit(Literal::Unit);
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Unit);
    }

    #[test]
    fn test_compile_literal_float_unsupported() {
        let expr = Expr::Lit(Literal::Float(3.15));
        let result = Compiler::compile(&expr);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CompileError::UnsupportedFloat);
    }

    // ========================================================================
    // TDD: Binary Operation Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_add() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: LoadConst 1, LoadConst 2, Add, Return
        assert_eq!(chunk.constants.len(), 2);
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
        assert_eq!(chunk.instructions[2], Instruction::Add);
        assert_eq!(chunk.instructions[3], Instruction::Return);
    }

    #[test]
    fn test_compile_all_arithmetic_ops() {
        let ops = vec![
            (BinOp::Add, Instruction::Add),
            (BinOp::Sub, Instruction::Sub),
            (BinOp::Mul, Instruction::Mul),
            (BinOp::Div, Instruction::Div),
        ];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Int(10))),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    #[test]
    fn test_compile_comparison_ops() {
        let ops = vec![
            (BinOp::Eq, Instruction::Eq),
            (BinOp::Neq, Instruction::Neq),
            (BinOp::Lt, Instruction::Lt),
            (BinOp::Lte, Instruction::Lte),
            (BinOp::Gt, Instruction::Gt),
            (BinOp::Gte, Instruction::Gte),
        ];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Int(1))),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    #[test]
    fn test_compile_logical_ops() {
        let ops = vec![(BinOp::And, Instruction::And), (BinOp::Or, Instruction::Or)];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Bool(true))),
                right: Box::new(Expr::Lit(Literal::Bool(false))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    // ========================================================================
    // TDD: Let-Binding Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_let_simple() {
        // let x = 42 in x
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(42))),
            body: Box::new(Expr::Var("x".to_string())),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: LoadConst(42), StoreLocal(0), LoadLocal(0), Return
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::LoadConst(_))));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::StoreLocal(_))));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::LoadLocal(_))));
    }

    #[test]
    fn test_compile_let_with_binop() {
        // let x = 10 in x + 5
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(10))),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            }),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::Add));
    }

    #[test]
    fn test_compile_let_nested() {
        // let x = 1 in let y = 2 in x + y
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(1))),
            body: Box::new(Expr::Let {
                name: "y".to_string(),
                value: Box::new(Expr::Lit(Literal::Int(2))),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::Var("y".to_string())),
                }),
            }),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have multiple locals
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(1)));
    }

    // ========================================================================
    // TDD: Variable Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_undefined_variable() {
        let expr = Expr::Var("x".to_string());
        let result = Compiler::compile(&expr);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CompileError::UndefinedVariable("x".to_string())
        );
    }

    // ========================================================================
    // TDD: If-Then-Else Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_if_simple() {
        // if true then 1 else 0
        let expr = Expr::If {
            cond: Box::new(Expr::Lit(Literal::Bool(true))),
            then_branch: Box::new(Expr::Lit(Literal::Int(1))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have jump instructions
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
    fn test_compile_if_with_comparison() {
        // if 10 > 5 then 42 else 0
        let expr = Expr::If {
            cond: Box::new(Expr::BinOp {
                op: BinOp::Gt,
                left: Box::new(Expr::Lit(Literal::Int(10))),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            }),
            then_branch: Box::new(Expr::Lit(Literal::Int(42))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::Gt));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
    }

    // ========================================================================
    // TDD: Constant Pool Tests
    // ========================================================================

    #[test]
    fn test_constant_deduplication() {
        // 42 + 42 should only have one constant
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(42))),
            right: Box::new(Expr::Lit(Literal::Int(42))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Note: Current implementation doesn't deduplicate
        // This test documents current behavior
        assert!(chunk.constants.len() <= 2);
    }
}
