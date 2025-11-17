# Issue #005: Bytecode Instructions and Chunks

## Overview
Define the bytecode instruction set and chunk representation for the FSRS VM.

## Labels
- `feature`, `phase-1: mvp`, `priority: high`, `foundational`, `parallel-safe`, `component: vm`, `effort: s` (1-2 days)

## Milestone
Phase 1.2: VM Foundation (Week 2)

## Dependencies
None - Can work in parallel

## Acceptance Criteria
- [ ] `Instruction` enum with Phase 1 opcodes
- [ ] `Chunk` struct for bytecode + constants
- [ ] Constants pool management
- [ ] Disassembler for debugging
- [ ] Unit tests for bytecode construction

## Technical Specification

```rust
// rust/crates/fsrs-vm/src/bytecode.rs

use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Stack operations
    LoadConst(u16),   // Push constants[idx] onto stack
    Pop,              // Pop top of stack

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Comparison
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,

    // Logical
    And,
    Or,
    Not,

    // Control flow
    Jump(i16),         // Unconditional jump by offset
    JumpIfFalse(i16),  // Jump if top of stack is false
    Return,            // Return from function
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    // Disassemble for debugging
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);
        for (offset, instr) in self.instructions.iter().enumerate() {
            self.disassemble_instruction(offset, instr);
        }
    }

    fn disassemble_instruction(&self, offset: usize, instr: &Instruction) {
        print!("{:04} ", offset);
        match instr {
            Instruction::LoadConst(idx) => {
                let val = &self.constants[*idx as usize];
                println!("LOAD_CONST {} ({})", idx, val);
            }
            Instruction::Add => println!("ADD"),
            Instruction::Sub => println!("SUB"),
            Instruction::Return => println!("RETURN"),
            _ => println!("{:?}", instr),
        }
    }
}
```

## Estimated Effort
**1-2 days**

## Related Issues
- Used by #006 (VM) and #007 (Compiler)