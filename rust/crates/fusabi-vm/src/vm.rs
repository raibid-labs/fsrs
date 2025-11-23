// Fusabi VM Interpreter
// Implements the bytecode interpreter loop with stack-based execution

use crate::chunk::Chunk;
use crate::closure::{Closure, Upvalue};
use crate::instruction::Instruction;
use crate::value::Value;
use crate::host::HostRegistry;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

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
    /// The closure being executed
    pub closure: Rc<Closure>,
    /// Instruction pointer - index into closure.chunk.instructions
    pub ip: usize,
    /// Base pointer - index into VM stack where this frame's locals start
    pub base: usize,
}

impl Frame {
    /// Create a new frame for executing a closure
    pub fn new(closure: Rc<Closure>, base: usize) -> Self {
        Frame {
            closure,
            ip: 0,
            base,
        }
    }

    /// Fetch the next instruction and advance IP
    pub fn fetch_instruction(&mut self) -> Result<Instruction, VmError> {
        if self.ip >= self.closure.chunk.instructions.len() {
            return Err(VmError::InvalidInstructionPointer(self.ip));
        }
        let instr = self.closure.chunk.instructions[self.ip].clone();
        self.ip += 1;
        Ok(instr)
    }

    /// Get a constant from the chunk
    pub fn get_constant(&self, idx: u16) -> Result<Value, VmError> {
        self.closure
            .chunk
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
    /// Global variables
    pub globals: HashMap<String, Value>,
    /// Host function registry
    pub host_registry: Rc<RefCell<HostRegistry>>,
}

impl Vm {
    /// Create a new VM
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
            host_registry: Rc::new(RefCell::new(HostRegistry::new())),
        }
    }

    /// Execute a chunk of bytecode
    pub fn execute(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        // Wrap the top-level chunk in a closure
        let closure = Rc::new(Closure::new(chunk));
        
        // Push initial frame
        let frame = Frame::new(closure, 0);
        self.frames.push(frame);
        
        self.run()
    }

    /// Run the interpreter loop
    pub fn run(&mut self) -> Result<Value, VmError> {
        let start_depth = self.frames.len();

        // Main interpreter loop
        loop {
            // Fetch next instruction in a separate scope to release mutable borrow on self
            let instruction = {
                let frame = self.current_frame_mut()?;
                frame.fetch_instruction()?
            };

            // Execute instruction
            match instruction {
                Instruction::LoadConst(idx) => {
                    let constant = self.current_frame()?.get_constant(idx)?;
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

                Instruction::LoadUpvalue(idx) => {
                    let frame = self.current_frame()?;
                    let upvalue = frame.closure.get_upvalue(idx as usize).ok_or(VmError::Runtime(format!("Invalid upvalue index: {}", idx)))?;
                    let value = match &*upvalue.borrow() {
                        Upvalue::Closed(v) => v.clone(),
                        Upvalue::Open(stack_idx) => self.stack[*stack_idx].clone(),
                    };
                    self.push(value);
                }

                Instruction::StoreUpvalue(idx) => {
                    let value = self.pop()?;
                    let frame = self.current_frame()?;
                    let upvalue = frame.closure.get_upvalue(idx as usize).ok_or(VmError::Runtime(format!("Invalid upvalue index: {}", idx)))?;
                    
                    {
                        match &mut *upvalue.borrow_mut() {
                            Upvalue::Closed(v) => *v = value,
                            Upvalue::Open(stack_idx) => self.stack[*stack_idx] = value,
                        }
                    };
                }

                Instruction::LoadGlobal(idx) => {
                    let name_val = self.current_frame()?.get_constant(idx)?;
                    let name = match name_val {
                        Value::Str(s) => s,
                        _ => return Err(VmError::TypeMismatch { expected: "string (global name)", got: name_val.type_name() }),
                    };
                    
                    let value = self.globals.get(&name).cloned().ok_or_else(|| VmError::Runtime(format!("Undefined global: {}", name)))?;
                    self.push(value);
                }

                Instruction::Pop => {
                    self.pop()?;
                }

                Instruction::Dup => {
                    let value = self.peek()?;
                    self.push(value.clone());
                }

                Instruction::CheckInt(expected) => {
                    let value = self.peek()?;
                    let matches = matches!(value, Value::Int(n) if *n == expected);
                    self.push(Value::Bool(matches));
                }

                Instruction::CheckBool(expected) => {
                    let value = self.peek()?;
                    let matches = matches!(value, Value::Bool(b) if *b == expected);
                    self.push(Value::Bool(matches));
                }

                Instruction::CheckString(expected) => {
                    let value = self.peek()?;
                    let matches = matches!(value, Value::Str(s) if *s == expected);
                    self.push(Value::Bool(matches));
                }

                Instruction::CheckTupleLen(expected) => {
                    let value = self.peek()?;
                    let matches = if let Value::Tuple(elements) = value {
                        elements.len() == expected as usize
                    } else {
                        false
                    };
                    self.push(Value::Bool(matches));
                }

                Instruction::GetTupleElem(index) => {
                    let value = self.peek()?;
                    if let Value::Tuple(elements) = value {
                        if (index as usize) < elements.len() {
                            self.push(elements[index as usize].clone());
                        } else {
                            return Err(VmError::Runtime("Tuple index out of bounds".into()));
                        }
                    } else {
                        return Err(VmError::Runtime("Not a tuple".into()));
                    }
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

                Instruction::MakeClosure(idx, upvalue_count) => {
                    let constant = self.current_frame()?.get_constant(idx)?;
                    let prototype = match constant.as_closure() {
                        Some(c) => c,
                        None => return Err(VmError::TypeMismatch {
                            expected: "closure",
                            got: constant.type_name(),
                        }),
                    };

                    let mut closure = Closure::new(prototype.chunk.clone());
                    closure.arity = prototype.arity;
                    closure.name = prototype.name.clone();

                    // Pop upvalues (placeholder logic for now)
                    for _ in 0..upvalue_count {
                        self.pop()?;
                    }

                    self.push(Value::Closure(Rc::new(closure)));
                }

                Instruction::Call(argc) => {
                    let func_idx = self.stack.len().checked_sub(1 + argc as usize).ok_or(VmError::StackUnderflow)?;
                    let func = self.stack[func_idx].clone();

                    match func {
                        Value::Closure(closure) => {
                            if closure.arity != argc {
                                return Err(VmError::Runtime(format!(
                                    "Arity mismatch: expected {}, got {}",
                                    closure.arity, argc
                                )));
                            }
                            
                            // Locals start at first argument
                            let base = func_idx + 1;
                            let frame = Frame::new(closure.clone(), base);
                            self.frames.push(frame);
                        }
                        Value::NativeFn { name, arity, args: applied_args } => {
                            // Pop new arguments from stack
                            let mut new_args = Vec::with_capacity(argc as usize);
                            for _ in 0..argc {
                                new_args.push(self.pop()?);
                            }
                            new_args.reverse(); // Arguments are pushed left-to-right, so stack has last arg on top.
                            
                            // Combine with already applied arguments
                            let mut all_args = applied_args.clone();
                            all_args.extend(new_args);
                            
                            let total_args = all_args.len();
                            let arity_usize = arity as usize;
                            
                            if total_args < arity_usize {
                                // Partial application: return new NativeFn with accumulated args
                                self.push(Value::NativeFn {
                                    name: name.clone(),
                                    arity: arity,
                                    args: all_args,
                                });
                            } else if total_args == arity_usize {
                                // Exact match: execute
                                // Call via registry
                                let host_fn = {
                                    let registry = self.host_registry.borrow();
                                    registry.get(&name)
                                }; // Drop borrow

                                if let Some(f) = host_fn {
                                    let result = f(self, &all_args)?;
                                    self.push(result);
                                } else {
                                    return Err(VmError::Runtime(format!("Undefined host function: {}", name)));
                                }
                            } else {
                                // Over-application: execute with first 'arity' args, then call result with rest
                                // For Phase 1/2/3, let's just error or handle simple case
                                // To handle this properly, we'd need to recurse or push result and call again.
                                // Simpler: Error for now.
                                return Err(VmError::Runtime(format!(
                                    "Native function '{}' expects {} arguments, got {}", 
                                    name, arity, total_args
                                )));
                            }
                        }
                        _ => return Err(VmError::TypeMismatch {
                            expected: "function",
                            got: func.type_name(),
                        }),
                    }
                }

                Instruction::CloseUpvalue(_) => {
                    // Placeholder
                }

                Instruction::Return => {
                    // Pop the frame
                    self.frames.pop();

                    // If we've dropped below the starting depth, we're done with this run() call
                    if self.frames.len() < start_depth {
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

                // Record operations
                Instruction::MakeRecord(n) => {
                    // Pop N field name/value pairs from stack in reverse order
                    use std::cell::RefCell;
                    use std::collections::HashMap;
                    use std::rc::Rc;

                    let mut fields = HashMap::new();
                    for _ in 0..n {
                        // Stack: [..., value, field_name] (top)
                        let field_value = self.pop()?;
                        let field_name = self.pop()?;

                        // Field name must be a string
                        let field_name_str =
                            field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                                expected: "string",
                                got: field_name.type_name(),
                            })?;

                        fields.insert(field_name_str.to_string(), field_value);
                    }

                    let record = Value::Record(Rc::new(RefCell::new(fields)));
                    self.push(record);
                }

                Instruction::GetRecordField => {
                    // Stack: [..., field_name, record] (top)
                    let field_name = self.pop()?;
                    let record = self.pop()?;

                    // Field name must be a string
                    let field_name_str =
                        field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                            expected: "string",
                            got: field_name.type_name(),
                        })?;

                    // Get the field value
                    let field_value = record
                        .record_get(field_name_str)
                        .map_err(VmError::Runtime)?;

                    self.push(field_value);
                }

                Instruction::UpdateRecord(n) => {
                    // Pop N field name/value pairs, then the record
                    use std::collections::HashMap;

                    let mut updates = HashMap::new();
                    for _ in 0..n {
                        // Stack: [..., value, field_name] (top)
                        let field_value = self.pop()?;
                        let field_name = self.pop()?;

                        // Field name must be a string
                        let field_name_str =
                            field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                                expected: "string",
                                got: field_name.type_name(),
                            })?;

                        updates.insert(field_name_str.to_string(), field_value);
                    }

                    let record = self.pop()?;

                    // Create updated record (immutable)
                    let new_record = record.record_update(updates).map_err(VmError::Runtime)?;

                    self.push(new_record);
                }

                // Discriminated union operations
                Instruction::MakeVariant(n) => {
                    // Pop N fields, then variant_name, then type_name from stack
                    // Stack: [..., type_name, variant_name, field_0, ..., field_N-1] (top)

                    // Collect fields in reverse order (stack is LIFO)
                    let mut fields = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        fields.push(self.pop()?);
                    }
                    fields.reverse(); // Restore correct field order

                    // Pop variant_name and type_name
                    let variant_name = self.pop()?;
                    let type_name = self.pop()?;

                    // Type name and variant name must be strings
                    let type_name_str =
                        type_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                            expected: "string",
                            got: type_name.type_name(),
                        })?;

                    let variant_name_str =
                        variant_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                            expected: "string",
                            got: variant_name.type_name(),
                        })?;

                    // Create variant value
                    let variant = Value::Variant {
                        type_name: type_name_str.to_string(),
                        variant_name: variant_name_str.to_string(),
                        fields,
                    };

                    self.push(variant);
                }

                Instruction::CheckVariantTag(ref tag) => {
                    // Pop variant from stack, push bool indicating if tag matches
                    let variant = self.pop()?;

                    // Check if value is a variant with the specified tag
                    let matches = variant.is_variant_named(tag);

                    self.push(Value::Bool(matches));
                }

                Instruction::GetVariantField(idx) => {
                    // Pop variant from stack, push field at index
                    let variant = self.pop()?;

                    // Get the field value
                    let field_value = variant
                        .variant_get_field(idx as usize)
                        .map_err(VmError::Runtime)?;

                    self.push(field_value);
                }

                _ => {
                    unimplemented!("Instruction not implemented in Phase 1: {:?}", instruction)
                }
            }
        }
    }

    /// Peek at the top of the stack without removing it
    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackUnderflow)
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

    /// Get the current frame (immutable)
    fn current_frame(&self) -> Result<&Frame, VmError> {
        self.frames.last().ok_or(VmError::NoActiveFrame)
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
        if new_ip > frame.closure.chunk.instructions.len() {
            return Err(VmError::InvalidInstructionPointer(new_ip));
        }

        frame.ip = new_ip;
        Ok(())
    }

    /// Call a closure from Rust code (re-entrant)
    pub fn call_closure(&mut self, closure: Rc<Closure>, args: &[Value]) -> Result<Value, VmError> {
        if closure.arity as usize != args.len() {
             return Err(VmError::Runtime(format!(
                 "Arity mismatch: expected {}, got {}", 
                 closure.arity, args.len()
             )));
        }
        
        // Push args to stack
        for arg in args {
            self.push(arg.clone());
        }
        
        // Calculate base pointer (locals start at first argument)
        let base = self.stack.len() - args.len();
        
        // Push frame
        let frame = Frame::new(closure, base);
        self.frames.push(frame);
        
        // Run the VM loop until this frame returns
        self.run()
    }

    /// Call any callable value (Closure or NativeFn) from Rust code
    pub fn call_value(&mut self, func: Value, args: &[Value]) -> Result<Value, VmError> {
        match func {
            Value::Closure(closure) => self.call_closure(closure, args),
            Value::NativeFn { name, arity, args: applied_args } => {
                let mut all_args = applied_args.clone();
                all_args.extend_from_slice(args);
                
                let total_args = all_args.len();
                let arity_usize = arity as usize;
                
                if total_args < arity_usize {
                    Ok(Value::NativeFn {
                        name,
                        arity,
                        args: all_args,
                    })
                } else if total_args == arity_usize {
                    let host_fn = {
                        let registry = self.host_registry.borrow();
                        registry.get(&name)
                    };
                    if let Some(f) = host_fn {
                        f(self, &all_args)
                    } else {
                        Err(VmError::Runtime(format!("Undefined host function: {}", name)))
                    }
                } else {
                    Err(VmError::Runtime(format!(
                        "Native function '{}' expects {} arguments, got {}", 
                        name, arity, total_args
                    )))
                }
            }
            _ => Err(VmError::TypeMismatch { 
                expected: "function", 
                got: func.type_name() 
            }),
        }
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

    // ========== Variant (DU) VM Tests ==========

    #[test]
    fn test_vm_make_variant_simple() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Direction".to_string()))
            .constant(Value::Str("Left".to_string()))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::MakeVariant(0)) // 0 fields
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_variant());
        assert_eq!(result.variant_name(), Ok("Left"));
        assert_eq!(result.variant_type_name(), Ok("Direction"));
        assert_eq!(format!("{}", result), "Left");
    }

    #[test]
    fn test_vm_make_variant_with_field() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Option".to_string()))
            .constant(Value::Str("Some".to_string()))
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field value
            .instruction(Instruction::MakeVariant(1)) // 1 field
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_variant());
        assert_eq!(result.variant_name(), Ok("Some"));
        assert_eq!(result.variant_get_field(0), Ok(Value::Int(42)));
        assert_eq!(format!("{}", result), "Some(42)");
    }

    #[test]
    fn test_vm_make_variant_with_multiple_fields() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Shape".to_string()))
            .constant(Value::Str("Rectangle".to_string()))
            .constant(Value::Int(10))
            .constant(Value::Int(20))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field 0
            .instruction(Instruction::LoadConst(3)) // field 1
            .instruction(Instruction::MakeVariant(2)) // 2 fields
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_variant());
        assert_eq!(result.variant_name(), Ok("Rectangle"));
        assert_eq!(result.variant_get_field(0), Ok(Value::Int(10)));
        assert_eq!(result.variant_get_field(1), Ok(Value::Int(20)));
        assert_eq!(format!("{}", result), "Rectangle(10, 20)");
    }

    #[test]
    fn test_vm_check_variant_tag_true() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Option".to_string()))
            .constant(Value::Str("Some".to_string()))
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field
            .instruction(Instruction::MakeVariant(1)) // Create Some(42)
            .instruction(Instruction::CheckVariantTag("Some".to_string()))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_check_variant_tag_false() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Option".to_string()))
            .constant(Value::Str("Some".to_string()))
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field
            .instruction(Instruction::MakeVariant(1)) // Create Some(42)
            .instruction(Instruction::CheckVariantTag("None".to_string()))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_check_variant_tag_not_variant() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::CheckVariantTag("Some".to_string()))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_get_variant_field() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Shape".to_string()))
            .constant(Value::Str("Rectangle".to_string()))
            .constant(Value::Int(10))
            .constant(Value::Int(20))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field 0
            .instruction(Instruction::LoadConst(3)) // field 1
            .instruction(Instruction::MakeVariant(2)) // Create Rectangle(10, 20)
            .instruction(Instruction::GetVariantField(1)) // Get field 1
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_vm_variant_nested() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Option".to_string()))
            .constant(Value::Str("Some".to_string()))
            .constant(Value::Int(42))
            .constant(Value::Str("Result".to_string()))
            .constant(Value::Str("Ok".to_string()))
            // Create Some(42) and store it
            .instruction(Instruction::LoadConst(0)) // Option
            .instruction(Instruction::LoadConst(1)) // Some
            .instruction(Instruction::LoadConst(2)) // 42
            .instruction(Instruction::MakeVariant(1)) // Stack: [Some(42)]
            .instruction(Instruction::StoreLocal(0)) // Store Some(42) in local 0
            // Create Ok(Some(42))
            .instruction(Instruction::LoadConst(3)) // Result
            .instruction(Instruction::LoadConst(4)) // Ok
            .instruction(Instruction::LoadLocal(0)) // Load Some(42)
            .instruction(Instruction::MakeVariant(1)) // Ok(Some(42))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_variant());
        assert_eq!(result.variant_name(), Ok("Ok"));
        let inner = result.variant_get_field(0).unwrap();
        assert!(inner.is_variant());
        assert_eq!(inner.variant_name(), Ok("Some"));
        assert_eq!(format!("{}", result), "Ok(Some(42))");
    }

    #[test]
    fn test_vm_variant_in_local() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Option".to_string()))
            .constant(Value::Str("Some".to_string()))
            .constant(Value::Int(42))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // field
            .instruction(Instruction::MakeVariant(1))
            .instruction(Instruction::StoreLocal(0))
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result.variant_name(), Ok("Some"));
        assert_eq!(result.variant_get_field(0), Ok(Value::Int(42)));
    }

    #[test]
    fn test_vm_variant_with_tuple_field() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Point".to_string()))
            .constant(Value::Str("Coordinate".to_string()))
            .constant(Value::Int(1))
            .constant(Value::Int(2))
            .instruction(Instruction::LoadConst(2)) // 1
            .instruction(Instruction::LoadConst(3)) // 2
            .instruction(Instruction::MakeTuple(2)) // (1, 2)
            .instruction(Instruction::StoreLocal(0)) // Store tuple in local 0
            .instruction(Instruction::LoadConst(0)) // Point
            .instruction(Instruction::LoadConst(1)) // Coordinate
            .instruction(Instruction::LoadLocal(0)) // Load tuple
            .instruction(Instruction::MakeVariant(1)) // Coordinate((1, 2))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert!(result.is_variant());
        let field = result.variant_get_field(0).unwrap();
        assert!(field.is_tuple());
        assert_eq!(format!("{}", result), "Coordinate((1, 2))");
    }

    #[test]
    fn test_vm_variant_mixed_field_types() {
        let mut vm = Vm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Str("Mixed".to_string()))
            .constant(Value::Str("Data".to_string()))
            .constant(Value::Int(42))
            .constant(Value::Str("hello".to_string()))
            .constant(Value::Bool(true))
            .instruction(Instruction::LoadConst(0)) // type_name
            .instruction(Instruction::LoadConst(1)) // variant_name
            .instruction(Instruction::LoadConst(2)) // 42
            .instruction(Instruction::LoadConst(3)) // "hello"
            .instruction(Instruction::LoadConst(4)) // true
            .instruction(Instruction::MakeVariant(3))
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result.variant_get_field(0), Ok(Value::Int(42)));
        assert_eq!(
            result.variant_get_field(1),
            Ok(Value::Str("hello".to_string()))
        );
        assert_eq!(result.variant_get_field(2), Ok(Value::Bool(true)));
        assert_eq!(format!("{}", result), "Data(42, hello, true)");
    }
}
