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
    /// Invalid tuple field index
    InvalidTupleFieldIndex {
        index: u8,
        tuple_size: usize,
    },
    /// Attempted to access head/tail of empty list
    /// Runtime error with message
    Runtime(String),
    EmptyList,
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
            VmError::InvalidTupleFieldIndex { index, tuple_size } => {
                write!(
                    f,
                    "Invalid tuple field index: {} (tuple size: {})",
                    index, tuple_size
                )
            }
            VmError::EmptyList => write!(f, "Cannot access head/tail of empty list"),
            VmError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
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

                // Tuple operations
                Instruction::MakeTuple(n) => {
                    // Pop N values from stack in reverse order (last pushed is last in tuple)
                    let mut elements = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        elements.push(self.pop()?);
                    }
                    // Reverse to maintain left-to-right order
                    elements.reverse();
                    self.push(Value::Tuple(elements));
                }

                Instruction::GetTupleField(idx) => {
                    let value = self.pop()?;
                    match value.as_tuple() {
                        Some(elements) => {
                            let index = idx as usize;
                            if index >= elements.len() {
                                return Err(VmError::InvalidTupleFieldIndex {
                                    index: idx,
                                    tuple_size: elements.len(),
                                });
                            }
                            self.push(elements[index].clone());
                        }
                        None => {
                            return Err(VmError::TypeMismatch {
                                expected: "tuple",
                                got: value.type_name(),
                            });
                        }
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

                // List operations
                Instruction::MakeList(n) => {
                    // Pop N values from stack in reverse order
                    let mut elements = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        elements.push(self.pop()?);
                    }
                    // Reverse to maintain left-to-right order
                    elements.reverse();
                    // Build cons list from elements
                    let list = Value::vec_to_cons(elements);
                    self.push(list);
                }

                Instruction::Cons => {
                    let tail = self.pop()?;
                    let head = self.pop()?;
                    self.push(Value::Cons {
                        head: Box::new(head),
                        tail: Box::new(tail),
                    });
                }

                Instruction::ListHead => {
                    let value = self.pop()?;
                    match value {
                        Value::Cons { head, .. } => self.push(*head),
                        Value::Nil => return Err(VmError::EmptyList),
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "list",
                                got: value.type_name(),
                            })
                        }
                    }
                }

                Instruction::ListTail => {
                    let value = self.pop()?;
                    match value {
                        Value::Cons { tail, .. } => self.push(*tail),
                        Value::Nil => return Err(VmError::EmptyList),
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "list",
                                got: value.type_name(),
                            })
                        }
                    }
                }

                Instruction::IsNil => {
                    let value = self.pop()?;
                    self.push(Value::Bool(value.is_nil()));
                }

                // Array operations
                Instruction::MakeArray(n) => {
                    // Pop N values from stack in reverse order
                    let mut elements = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        elements.push(self.pop()?);
                    }
                    // Reverse to maintain left-to-right order
                    elements.reverse();
                    // Build array from elements
                    use std::cell::RefCell;
                    use std::rc::Rc;
                    let array = Value::Array(Rc::new(RefCell::new(elements)));
                    self.push(array);
                }

                Instruction::ArrayGet => {
                    let index = self.pop()?;
                    let array = self.pop()?;

                    let idx = match index {
                        Value::Int(i) => {
                            if i < 0 {
                                return Err(VmError::TypeMismatch {
                                    expected: "non-negative int",
                                    got: "negative int",
                                });
                            }
                            i as usize
                        }
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "int",
                                got: index.type_name(),
                            })
                        }
                    };

                    let value = array.array_get(idx).map_err(VmError::Runtime)?;

                    self.push(value);
                }

                Instruction::ArraySet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let array = self.pop()?;

                    let idx = match index {
                        Value::Int(i) => {
                            if i < 0 {
                                return Err(VmError::TypeMismatch {
                                    expected: "non-negative int",
                                    got: "negative int",
                                });
                            }
                            i as usize
                        }
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "int",
                                got: index.type_name(),
                            })
                        }
                    };

                    array.array_set(idx, value).map_err(VmError::Runtime)?;

                    // Push unit to indicate completion
                    self.push(Value::Unit);
                }

                Instruction::ArrayLength => {
                    let array = self.pop()?;
                    let len = array.array_length().map_err(VmError::Runtime)?;
                    self.push(Value::Int(len));
                }

                Instruction::ArrayUpdate => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let array = self.pop()?;

                    let idx = match index {
                        Value::Int(i) => {
                            if i < 0 {
                                return Err(VmError::TypeMismatch {
                                    expected: "non-negative int",
                                    got: "negative int",
                                });
                            }
                            i as usize
                        }
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "int",
                                got: index.type_name(),
                            })
                        }
                    };

                    // Clone the array for immutable update
                    use std::cell::RefCell;
                    use std::rc::Rc;
                    let new_arr = if let Value::Array(arr) = &array {
                        let mut new_elements = arr.borrow().clone();
                        if idx >= new_elements.len() {
                            return Err(VmError::Runtime(format!("Index {} out of bounds", idx)));
                        }
                        new_elements[idx] = value;
                        Value::Array(Rc::new(RefCell::new(new_elements)))
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "array",
                            got: array.type_name(),
                        });
                    };

                    self.push(new_arr);
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

    // ========== Tuple Tests ==========

    #[test]
    fn test_vm_make_tuple_empty() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .instruction(Instruction::MakeTuple(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Tuple(vec![]));
    }

    #[test]
    fn test_vm_make_tuple_pair() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    }

    #[test]
    fn test_vm_make_tuple_triple() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeTuple(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::Tuple(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_vm_make_tuple_mixed_types() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Str("hello".to_string()))
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeTuple(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::Tuple(vec![
                Value::Int(42),
                Value::Str("hello".to_string()),
                Value::Bool(true)
            ])
        );
    }

    #[test]
    fn test_vm_make_tuple_nested() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::Tuple(vec![
                Value::Int(1),
                Value::Tuple(vec![Value::Int(2), Value::Int(3)])
            ])
        );
    }

    #[test]
    fn test_vm_get_tuple_field() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::GetTupleField(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_vm_get_tuple_field_second() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::GetTupleField(1))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_vm_get_tuple_field_invalid_index() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::GetTupleField(5))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(
            result,
            Err(VmError::InvalidTupleFieldIndex {
                index: 5,
                tuple_size: 2
            })
        ));
    }

    #[test]
    fn test_vm_get_tuple_field_type_mismatch() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::GetTupleField(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "tuple",
                got: "int"
            })
        ));
    }

    #[test]
    fn test_vm_tuple_eq() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::Eq)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_tuple_neq() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::Neq)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_tuple_in_local() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::StoreLocal(0))
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Tuple(vec![Value::Int(1), Value::Int(2)]));
    }

    #[test]
    fn test_vm_tuple_large() {
        let mut vm = Vm::new();
        let mut builder = ChunkBuilder::new();
        for i in 1..=8 {
            builder = builder.constant(Value::Int(i));
        }
        for i in 0..8 {
            builder = builder.instruction(Instruction::LoadConst(i));
        }
        let chunk = builder
            .instruction(Instruction::MakeTuple(8))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        let expected = Value::Tuple(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
            Value::Int(6),
            Value::Int(7),
            Value::Int(8),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_vm_tuple_field_extraction_chain() {
        // Create ((1, 2), 3), extract outer[0], then inner[1]
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeTuple(2))
            .instruction(Instruction::GetTupleField(0))
            .instruction(Instruction::GetTupleField(1))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    // ========== List Operation Tests (Layer 3) ==========

    #[test]
    fn test_vm_make_list_empty() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .instruction(Instruction::MakeList(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_vm_make_list_single() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::MakeList(1))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::Cons {
                head: Box::new(Value::Int(42)),
                tail: Box::new(Value::Nil),
            }
        );
    }

    #[test]
    fn test_vm_make_list_multiple() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeList(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_vm_cons() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Nil)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Cons)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::Cons {
                head: Box::new(Value::Int(1)),
                tail: Box::new(Value::Nil),
            }
        );
    }

    #[test]
    fn test_vm_cons_nested() {
        // Build [2] first, then cons 1 onto it
        let mut vm = Vm::new();
        let inner_list = Value::vec_to_cons(vec![Value::Int(2)]);
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(inner_list)
            .instruction(Instruction::LoadConst(0)) // Load 1
            .instruction(Instruction::LoadConst(1)) // Load [2]
            .instruction(Instruction::Cons) // 1 :: [2]
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)])
        );
    }

    #[test]
    fn test_vm_list_head() {
        let mut vm = Vm::new();
        let list = Value::vec_to_cons(vec![Value::Int(42), Value::Int(100)]);
        let chunk = ChunkBuilder::new()
            .constant(list)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ListHead)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_vm_list_head_empty_error() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Nil)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ListHead)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::EmptyList)));
    }

    #[test]
    fn test_vm_list_head_type_error() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ListHead)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "list",
                got: "int"
            })
        ));
    }

    #[test]
    fn test_vm_list_tail() {
        let mut vm = Vm::new();
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let chunk = ChunkBuilder::new()
            .constant(list)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ListTail)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_vm_list_tail_empty_error() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Nil)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ListTail)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::EmptyList)));
    }

    #[test]
    fn test_vm_is_nil_true() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Nil)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::IsNil)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_is_nil_false() {
        let mut vm = Vm::new();
        let list = Value::vec_to_cons(vec![Value::Int(1)]);
        let chunk = ChunkBuilder::new()
            .constant(list)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::IsNil)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_list_display() {
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(format!("{}", list), "[1; 2; 3]");
    }

    #[test]
    fn test_vm_list_equality() {
        let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list3 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(3)]);
        assert_eq!(list1, list2);
        assert_ne!(list1, list3);
    }

    #[test]
    fn test_vm_nested_list() {
        let inner1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let inner2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let outer = Value::vec_to_cons(vec![inner1, inner2]);
        assert_eq!(format!("{}", outer), "[[1; 2]; [3; 4]]");
    }

    #[test]
    fn test_vm_list_in_local() {
        let mut vm = Vm::new();
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let chunk = ChunkBuilder::new()
            .constant(list.clone())
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::StoreLocal(0))
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, list);
    }

    // ========== Array Operation Tests ==========

    #[test]
    fn test_vm_make_array_empty() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .instruction(Instruction::MakeArray(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_array());
        assert_eq!(result.array_length(), Ok(0));
    }

    #[test]
    fn test_vm_make_array_single() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::MakeArray(1))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result.array_get(0), Ok(Value::Int(42)));
    }

    #[test]
    fn test_vm_make_array_multiple() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .constant(Value::Int(3))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeArray(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result.array_get(0), Ok(Value::Int(1)));
        assert_eq!(result.array_get(1), Ok(Value::Int(2)));
        assert_eq!(result.array_get(2), Ok(Value::Int(3)));
    }

    #[test]
    fn test_vm_array_get() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
        ])));
        let chunk = ChunkBuilder::new()
            .constant(arr)
            .constant(Value::Int(1))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_vm_array_get_bounds_error() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        let chunk = ChunkBuilder::new()
            .constant(arr)
            .constant(Value::Int(5))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_vm_array_set() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        let chunk = ChunkBuilder::new()
            .constant(arr.clone())
            .constant(Value::Int(1))
            .constant(Value::Int(99))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::ArraySet)
            .instruction(Instruction::Pop) // Pop unit result
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_vm_array_length() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        let chunk = ChunkBuilder::new()
            .constant(arr)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::ArrayLength)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_vm_array_update() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])));
        let chunk = ChunkBuilder::new()
            .constant(arr.clone())
            .constant(Value::Int(1))
            .constant(Value::Int(99))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::ArrayUpdate)
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(99));
        // Verify original array unchanged
        assert_eq!(arr.array_get(1), Ok(Value::Int(2)));
    }

    #[test]
    fn test_vm_array_negative_index_error() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])));
        let chunk = ChunkBuilder::new()
            .constant(arr)
            .constant(Value::Int(-1))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_vm_array_type_error() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Int(0))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::ArrayGet)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk);
        assert!(matches!(result, Err(VmError::Runtime(_))));
    }

    #[test]
    fn test_vm_array_mixed_types() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .constant(Value::Str("hello".to_string()))
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::LoadConst(2))
            .instruction(Instruction::MakeArray(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result.array_get(0), Ok(Value::Int(42)));
        assert_eq!(result.array_get(1), Ok(Value::Str("hello".to_string())));
        assert_eq!(result.array_get(2), Ok(Value::Bool(true)));
    }

    #[test]
    fn test_vm_array_in_local() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let arr = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let chunk = ChunkBuilder::new()
            .constant(arr.clone())
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::StoreLocal(0))
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, arr);
    }

    #[test]
    fn test_vm_array_nested() {
        let mut vm = Vm::new();
        use std::cell::RefCell;
        use std::rc::Rc;
        let inner = Value::Array(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
        let chunk = ChunkBuilder::new()
            .constant(inner)
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::MakeArray(1))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        let outer_elem = result.array_get(0).unwrap();
        assert_eq!(outer_elem.array_get(0), Ok(Value::Int(1)));
        assert_eq!(outer_elem.array_get(1), Ok(Value::Int(2)));
    }
}
