# Issue #006: VM Interpreter Loop

## Overview
Implement the bytecode VM interpreter that executes instructions on a stack-based machine.

## Labels
- `feature`, `phase-1: mvp`, `priority: critical`, `foundational`, `component: vm`, `effort: m` (3-4 days)

## Milestone
Phase 1.2: VM Foundation (Week 2)

## Dependencies
- #004 (Value) - MUST BE COMPLETE
- #005 (Bytecode) - MUST BE COMPLETE

## Acceptance Criteria
- [ ] `Vm` struct with execution state
- [ ] Stack-based execution
- [ ] Implement all Phase 1 instructions
- [ ] Error handling for runtime errors
- [ ] 30+ interpreter tests

## Technical Specification

```rust
// rust/crates/fsrs-vm/src/vm.rs

use crate::value::Value;
use crate::bytecode::{Chunk, Instruction};

pub struct Vm {
    stack: Vec<Value>,
    ip: usize,
}

#[derive(Debug)]
pub enum VmError {
    StackUnderflow,
    TypeError { expected: String, found: String },
    DivisionByZero,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            ip: 0,
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<Value, VmError> {
        self.ip = 0;

        loop {
            if self.ip >= chunk.instructions.len() {
                break;
            }

            let instr = &chunk.instructions[self.ip];
            self.ip += 1;

            match instr {
                Instruction::LoadConst(idx) => {
                    let val = chunk.constants[*idx as usize].clone();
                    self.stack.push(val);
                }
                Instruction::Pop => {
                    self.stack.pop().ok_or(VmError::StackUnderflow)?;
                }
                Instruction::Add => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a + b));
                }
                Instruction::Sub => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a - b));
                }
                Instruction::Mul => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a * b));
                }
                Instruction::Div => {
                    let b = self.pop_int()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    let a = self.pop_int()?;
                    self.stack.push(Value::Int(a / b));
                }
                Instruction::Return => {
                    return self.stack.pop().ok_or(VmError::StackUnderflow);
                }
                _ => unimplemented!("Instruction: {:?}", instr),
            }
        }

        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn pop_int(&mut self) -> Result<i64, VmError> {
        let val = self.stack.pop().ok_or(VmError::StackUnderflow)?;
        val.as_int().ok_or_else(|| VmError::TypeError {
            expected: "int".to_string(),
            found: val.type_name().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let mut chunk = Chunk::new();
        let idx1 = chunk.add_constant(Value::Int(1));
        let idx2 = chunk.add_constant(Value::Int(2));
        chunk.emit(Instruction::LoadConst(idx1));
        chunk.emit(Instruction::LoadConst(idx2));
        chunk.emit(Instruction::Add);
        chunk.emit(Instruction::Return);

        let mut vm = Vm::new();
        let result = vm.run(&chunk).unwrap();
        assert_eq!(result, Value::Int(3));
    }
}
```

## Estimated Effort
**3-4 days**

## Related Issues
- Depends on #004, #005
- Used by #008 (Demo)