// Fusabi VM Bytecode Chunk
// A chunk represents a compiled function with instructions and constants

use crate::instruction::Instruction;
use crate::value::Value;
use std::fmt;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// A chunk of bytecode representing a compiled function
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub name: Option<String>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: Vec::new(),
            name: None,
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        Chunk {
            instructions: Vec::new(),
            constants: Vec::new(),
            name: Some(name.into()),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }

    pub fn emit(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn emit_all(&mut self, instructions: impl IntoIterator<Item = Instruction>) {
        self.instructions.extend(instructions);
    }

    pub fn current_offset(&self) -> usize {
        self.instructions.len()
    }

    pub fn instruction_at(&self, offset: usize) -> Option<&Instruction> {
        self.instructions.get(offset)
    }

    pub fn constant_at(&self, index: u16) -> Option<&Value> {
        self.constants.get(index as usize)
    }

    pub fn disassemble(&self) {
        let name = self.name.as_deref().unwrap_or("<unnamed>");
        println!("== {} ==", name);
        for (offset, instr) in self.instructions.iter().enumerate() {
            self.disassemble_instruction(offset, instr);
        }
    }

    fn disassemble_instruction(&self, offset: usize, instr: &Instruction) {
        print!("{:04} ", offset);
        match instr {
            Instruction::LoadConst(idx) => {
                if let Some(val) = self.constant_at(*idx) {
                    println!("LOAD_CONST {} ({})", idx, val);
                } else {
                    println!("LOAD_CONST {} (invalid index)", idx);
                }
            }
            _ => println!("{}", instr),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("<unnamed>");
        writeln!(f, "== {} ==", name)?;
        for (offset, instr) in self.instructions.iter().enumerate() {
            write!(f, "{:04} ", offset)?;
            match instr {
                Instruction::LoadConst(idx) => {
                    if let Some(val) = self.constant_at(*idx) {
                        writeln!(f, "LOAD_CONST {} ({})", idx, val)?;
                    } else {
                        writeln!(f, "LOAD_CONST {} (invalid)", idx)?;
                    }
                }
                _ => writeln!(f, "{}", instr)?,
            }
        }
        Ok(())
    }
}

pub struct ChunkBuilder {
    chunk: Chunk,
}

impl ChunkBuilder {
    pub fn new() -> Self {
        ChunkBuilder {
            chunk: Chunk::new(),
        }
    }

    pub fn with_name(name: impl Into<String>) -> Self {
        ChunkBuilder {
            chunk: Chunk::with_name(name),
        }
    }

    pub fn constant(mut self, value: Value) -> Self {
        self.chunk.add_constant(value);
        self
    }

    pub fn instruction(mut self, instr: Instruction) -> Self {
        self.chunk.emit(instr);
        self
    }

    pub fn instructions(mut self, instrs: impl IntoIterator<Item = Instruction>) -> Self {
        self.chunk.emit_all(instrs);
        self
    }

    pub fn build(self) -> Chunk {
        self.chunk
    }
}

impl Default for ChunkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_basic() {
        let mut chunk = Chunk::new();
        let idx = chunk.add_constant(Value::Int(42));
        chunk.emit(Instruction::LoadConst(idx));
        chunk.emit(Instruction::Return);
        assert_eq!(chunk.instructions.len(), 2);
        assert_eq!(chunk.constants.len(), 1);
    }
}
