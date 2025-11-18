// FSRS VM Interpreter
// Implements the bytecode interpreter loop with stack-based execution

use crate::chunk::Chunk;
use crate::instruction::Instruction;
use crate::value::Value;
use std::fmt;

/// Runtime error that can occur during VM execution
#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    /// Stack underflow - attempted to pop from empty stack
    StackUnderflow,
    /// Type mismatch - operation expected different type
    TypeMismatch {
        expected: &'static str,
        got: &'static str,
    },
    /// Division by zero
    DivisionByZero,
    /// Invalid constant index
    InvalidConstantIndex(u16),
    /// Invalid local index
    InvalidLocalIndex(u8),
    /// Invalid instruction pointer
    InvalidInstructionPointer(usize),
    /// Call stack overflow
    CallStackOverflow,
    /// No active frame
    NoActiveFrame,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow => write!(f, "Stack underflow"),
            VmError::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            }
            VmError::DivisionByZero => write!(f, "Division by zero"),
            VmError::InvalidConstantIndex(idx) => write!(f, "Invalid constant index: {}", idx),
            VmError::InvalidLocalIndex(idx) => write!(f, "Invalid local index: {}", idx),
            VmError::InvalidInstructionPointer(ip) => {
                write!(f, "Invalid instruction pointer: {}", ip)
            }
            VmError::CallStackOverflow => write!(f, "Call stack overflow"),
            VmError::NoActiveFrame => write!(f, "No active frame"),
        }
    }
}

impl std::error::Error for VmError {}

/// Call frame - represents an active function call
#[derive(Debug, Clone)]
pub struct Frame {
    /// The chunk being executed
    pub chunk: Chunk,
    /// Instruction pointer - index into chunk.instructions
    pub ip: usize,
    /// Base pointer - index into VM stack where this frame's locals start
    pub base: usize,
}

impl Frame {
    /// Create a new frame for executing a chunk
    pub fn new(chunk: Chunk, base: usize) -> Self {
        Frame { chunk, ip: 0, base }
    }

    /// Fetch the next instruction and advance IP
    pub fn fetch_instruction(&mut self) -> Result<Instruction, VmError> {
        if self.ip >= self.chunk.instructions.len() {
            return Err(VmError::InvalidInstructionPointer(self.ip));
        }
        let instr = self.chunk.instructions[self.ip].clone();
        self.ip += 1;
        Ok(instr)
    }

    /// Get a constant from the chunk
    pub fn get_constant(&self, idx: u16) -> Result<Value, VmError> {
        self.chunk
            .constant_at(idx)
            .cloned()
            .ok_or(VmError::InvalidConstantIndex(idx))
    }
}

/// Maximum call stack depth
/// The virtual machine - bytecode interpreter
#[derive(Debug)]
pub struct Vm {
    /// Value stack for operands and intermediate results
    stack: Vec<Value>,
    /// Call frame stack
    frames: Vec<Frame>,
}

impl Vm {
    /// Create a new VM
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Execute a chunk of bytecode
    pub fn execute(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        // Push initial frame
        let frame = Frame::new(chunk, 0);
        self.frames.push(frame);

        // Main interpreter loop
        loop {
            // Get current frame
            let frame = self.current_frame_mut()?;

            // Fetch next instruction
            let instruction = frame.fetch_instruction()?;

            // Execute instruction
            match instruction {
                Instruction::LoadConst(idx) => {
                    let constant = frame.get_constant(idx)?;
                    self.push(constant);
                }

                Instruction::LoadLocal(idx) => {
                    let value = self.get_local(idx)?;
                    self.push(value);
                }

                Instruction::StoreLocal(idx) => {
                    let value = self.pop()?;
                    self.set_local(idx, value)?;
                }

                Instruction::Pop => {
                    self.pop()?;
                }

                // Arithmetic operations
                Instruction::Add => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a + b));
                }

                Instruction::Sub => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a - b));
                }

                Instruction::Mul => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a * b));
                }

                Instruction::Div => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    if b == 0 {
                        return Err(VmError::DivisionByZero);
                    }
                    self.push(Value::Int(a / b));
                }

                // Comparison operations
                Instruction::Eq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }

                Instruction::Neq => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a != b));
                }

                Instruction::Lt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Bool(a < b));
                }

                Instruction::Lte => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Bool(a <= b));
                }

                Instruction::Gt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Bool(a > b));
                }

                Instruction::Gte => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Bool(a >= b));
                }

                // Logical operations
                Instruction::And => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.push(Value::Bool(a && b));
                }

                Instruction::Or => {
                    let b = self.pop_bool()?;
                    let a = self.pop_bool()?;
                    self.push(Value::Bool(a || b));
                }

                Instruction::Not => {
                    let a = self.pop_bool()?;
                    self.push(Value::Bool(!a));
                }

                // Control flow
                Instruction::Jump(offset) => {
                    self.jump(offset)?;
                }

                Instruction::JumpIfFalse(offset) => {
                    let condition = self.pop()?;
                    if !condition.is_truthy() {
                        self.jump(offset)?;
                    }
                }

                Instruction::Return => {
                    // Pop the frame
                    self.frames.pop();

                    // If no more frames, we're done
                    if self.frames.is_empty() {
                        // Return the top of stack or Unit if empty
                        return Ok(self.stack.pop().unwrap_or(Value::Unit));
                    }
                }

                // Unimplemented instructions for Phase 1
                Instruction::LoadUpvalue(_)
                | Instruction::StoreUpvalue(_)
                | Instruction::Call(_)
                | Instruction::TailCall(_) => {
                    unimplemented!("Instruction not implemented in Phase 1: {:?}", instruction)
                }
            }
        }
    }

    /// Push a value onto the stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop a value from the stack
    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Pop an integer from the stack
    fn pop_int(&mut self) -> Result<i64, VmError> {
        let value = self.pop()?;
        value.as_int().ok_or(VmError::TypeMismatch {
            expected: "int",
            got: value.type_name(),
        })
    }

    /// Pop a boolean from the stack
    fn pop_bool(&mut self) -> Result<bool, VmError> {
        let value = self.pop()?;
        value.as_bool().ok_or(VmError::TypeMismatch {
            expected: "bool",
            got: value.type_name(),
        })
    }

    /// Get the current frame (mutable)
    fn current_frame_mut(&mut self) -> Result<&mut Frame, VmError> {
        self.frames.last_mut().ok_or(VmError::NoActiveFrame)
    }

    /// Get a local variable
    fn get_local(&self, idx: u8) -> Result<Value, VmError> {
        let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
        let local_idx = frame.base + idx as usize;
        self.stack
            .get(local_idx)
            .cloned()
            .ok_or(VmError::InvalidLocalIndex(idx))
    }

    /// Set a local variable
    fn set_local(&mut self, idx: u8, value: Value) -> Result<(), VmError> {
        let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
        let local_idx = frame.base + idx as usize;

        // Extend stack if necessary
        while self.stack.len() <= local_idx {
            self.stack.push(Value::Unit);
        }

        self.stack[local_idx] = value;
        Ok(())
    }

    /// Jump by a signed offset relative to the current IP
    /// The offset is relative to the instruction AFTER the jump instruction
    fn jump(&mut self, offset: i16) -> Result<(), VmError> {
        let frame = self.current_frame_mut()?;

        // Calculate new IP. Note that frame.ip has already been incremented by fetch_instruction
        // so it points to the next instruction after the jump
        let new_ip = if offset >= 0 {
            frame.ip.wrapping_add(offset as usize)
        } else {
            frame.ip.wrapping_sub((-offset) as usize)
        };

        // Validate the new IP
        if new_ip > frame.chunk.instructions.len() {
            return Err(VmError::InvalidInstructionPointer(new_ip));
        }

        frame.ip = new_ip;
        Ok(())
    }

    /// Get the current stack size (for debugging)
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    /// Get the current frame count (for debugging)
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkBuilder;

    #[test]
    fn test_vm_load_const() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_add() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(10))
            .constant(Value::Int(32))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_sub() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(50))
            .constant(Value::Int(8))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Sub)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_mul() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(6))
            .constant(Value::Int(7))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Mul)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_div() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(84))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Div)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_div_by_zero() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Int(0))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Div)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::DivisionByZero)));
    }

    #[test]
    fn test_vm_eq() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Eq)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_lt() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(10))
            .constant(Value::Int(20))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Lt)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_and() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Bool(true))
            .constant(Value::Bool(false))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::And)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_not() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Not)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_locals() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::StoreLocal(0))
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_jump() {
        let mut vm = Vm::new();
        let mut chunk = Chunk::new();
        chunk.add_constant(Value::Int(1));
        chunk.add_constant(Value::Int(2));
        chunk.emit(Instruction::LoadConst(0));
        chunk.emit(Instruction::Jump(1));
        chunk.emit(Instruction::LoadConst(1));
        chunk.emit(Instruction::Return);
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_vm_jump_if_false_taken() {
        let mut vm = Vm::new();
        let mut chunk = Chunk::new();
        chunk.add_constant(Value::Bool(false));
        chunk.add_constant(Value::Int(1));
        chunk.add_constant(Value::Int(2));
        chunk.emit(Instruction::LoadConst(0));
        chunk.emit(Instruction::JumpIfFalse(1));
        chunk.emit(Instruction::LoadConst(1));
        chunk.emit(Instruction::LoadConst(2));
        chunk.emit(Instruction::Return);
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_vm_if_then_else() {
        let mut vm = Vm::new();
        let mut chunk = Chunk::new();
        let cond = chunk.add_constant(Value::Bool(true));
        let then_val = chunk.add_constant(Value::Int(42));
        let else_val = chunk.add_constant(Value::Int(99));
        chunk.emit(Instruction::LoadConst(cond));
        chunk.emit(Instruction::JumpIfFalse(2));
        chunk.emit(Instruction::LoadConst(then_val));
        chunk.emit(Instruction::Jump(1));
        chunk.emit(Instruction::LoadConst(else_val));
        chunk.emit(Instruction::Return);
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_fibonacci() {
        let mut vm = Vm::new();
        let mut chunk = Chunk::new();
        let zero = chunk.add_constant(Value::Int(0));
        let one = chunk.add_constant(Value::Int(1));
        let five = chunk.add_constant(Value::Int(5));
        chunk.emit(Instruction::LoadConst(zero));
        chunk.emit(Instruction::StoreLocal(0));
        chunk.emit(Instruction::LoadConst(one));
        chunk.emit(Instruction::StoreLocal(1));
        chunk.emit(Instruction::LoadConst(five));
        chunk.emit(Instruction::StoreLocal(2));
        chunk.emit(Instruction::LoadLocal(2));
        chunk.emit(Instruction::JumpIfFalse(11));
        chunk.emit(Instruction::LoadLocal(0));
        chunk.emit(Instruction::LoadLocal(1));
        chunk.emit(Instruction::Add);
        chunk.emit(Instruction::LoadLocal(1));
        chunk.emit(Instruction::StoreLocal(0));
        chunk.emit(Instruction::StoreLocal(1));
        chunk.emit(Instruction::LoadLocal(2));
        chunk.emit(Instruction::LoadConst(one));
        chunk.emit(Instruction::Sub);
        chunk.emit(Instruction::StoreLocal(2));
        chunk.emit(Instruction::Jump(-13));
        chunk.emit(Instruction::LoadLocal(0));
        chunk.emit(Instruction::Return);
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_vm_stack_underflow() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::StackUnderflow)));
    }

    #[test]
    fn test_vm_type_mismatch() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "int",
                got: "bool"
            })
        ));
    }
}
