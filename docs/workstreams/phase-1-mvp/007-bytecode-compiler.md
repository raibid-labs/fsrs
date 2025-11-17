# Issue #007: Bytecode Compiler

## Overview
Implement the compiler that transforms AST expressions into bytecode chunks.

## Labels
- `feature`, `phase-1: mvp`, `priority: critical`, `component: frontend`, `effort: l` (4-5 days)

## Milestone
Phase 1.3: Integration (Week 3)

## Dependencies
- #001 (AST) - MUST BE COMPLETE
- #003 (Parser) - MUST BE COMPLETE
- #005 (Bytecode) - MUST BE COMPLETE

## Acceptance Criteria
- [ ] Compile let-bindings to bytecode
- [ ] Compile arithmetic expressions
- [ ] Compile if/then/else with jumps
- [ ] Compile function calls (basic)
- [ ] 40+ compiler tests

## Technical Specification

```rust
// rust/crates/fsrs-frontend/src/compiler.rs

use crate::ast::{Expr, Literal, BinOp};
use fsrs_vm::bytecode::{Chunk, Instruction};
use fsrs_vm::value::Value;

pub struct Compiler {
    chunk: Chunk,
}

pub enum CompileError {
    UnsupportedFeature(String),
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
        }
    }

    pub fn compile(expr: &Expr) -> Result<Chunk, CompileError> {
        let mut compiler = Compiler::new();
        compiler.compile_expr(expr)?;
        compiler.chunk.emit(Instruction::Return);
        Ok(compiler.chunk)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Lit(lit) => self.compile_literal(lit),
            Expr::BinOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.compile_binop(*op)?;
                Ok(())
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.compile_if(cond, then_branch, else_branch)
            }
            _ => Err(CompileError::UnsupportedFeature(format!("{:?}", expr))),
        }
    }

    fn compile_literal(&mut self, lit: &Literal) {
        let val = match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Str(s) => Value::Str(s.clone()),
            Literal::Unit => Value::Unit,
            _ => unimplemented!(),
        };
        let idx = self.chunk.add_constant(val);
        self.chunk.emit(Instruction::LoadConst(idx));
    }

    fn compile_binop(&mut self, op: BinOp) -> Result<(), CompileError> {
        let instr = match op {
            BinOp::Add => Instruction::Add,
            BinOp::Sub => Instruction::Sub,
            BinOp::Mul => Instruction::Mul,
            BinOp::Div => Instruction::Div,
            BinOp::Eq => Instruction::Eq,
            BinOp::Lt => Instruction::Lt,
            _ => return Err(CompileError::UnsupportedFeature(format!("{:?}", op))),
        };
        self.chunk.emit(instr);
        Ok(())
    }

    fn compile_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> Result<(), CompileError> {
        self.compile_expr(cond)?;

        // Jump to else if condition is false
        let else_jump = self.chunk.instructions.len();
        self.chunk.emit(Instruction::JumpIfFalse(0)); // Placeholder

        // Compile then branch
        self.compile_expr(then_branch)?;

        // Jump over else branch
        let end_jump = self.chunk.instructions.len();
        self.chunk.emit(Instruction::Jump(0)); // Placeholder

        // Patch else jump
        let else_offset = self.chunk.instructions.len() as i16 - else_jump as i16;
        self.chunk.instructions[else_jump] = Instruction::JumpIfFalse(else_offset);

        // Compile else branch
        self.compile_expr(else_branch)?;

        // Patch end jump
        let end_offset = self.chunk.instructions.len() as i16 - end_jump as i16;
        self.chunk.instructions[end_jump] = Instruction::Jump(end_offset);

        Ok(())
    }
}
```

## Estimated Effort
**4-5 days**

## Related Issues
- Depends on #001, #003, #005
- Used by #008 (Demo)