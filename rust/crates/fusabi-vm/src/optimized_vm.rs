// Fusabi Fast VM - Optimized Bytecode Interpreter
// Implements performance optimizations for the dispatch loop
//
// NOTE: This VM is optimized for pure bytecode execution without native function calls.
// For workloads that require host/native functions, use the standard Vm.

use crate::chunk::Chunk;
use crate::closure::{Closure, Upvalue};
use crate::gc::GcHeap;
use crate::instruction::Instruction;
use crate::value::Value;
use crate::vm::{Frame, VmError};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// Default stack capacity for pre-allocation
const DEFAULT_STACK_CAPACITY: usize = 256;
/// Default frame stack capacity
const DEFAULT_FRAME_CAPACITY: usize = 64;

/// Fast VM - optimized bytecode interpreter
///
/// Optimizations over the base Vm:
/// - Pre-allocated stack with capacity
/// - Inlined hot path functions
/// - Unchecked access in inner loop with debug assertions
/// - Reduced instruction cloning via references
///
/// Limitations:
/// - Does not support native/host function calls (use standard Vm for that)
/// - Optimized for pure bytecode workloads
#[derive(Debug)]
pub struct FastVm {
    /// Value stack for operands and intermediate results (pre-allocated)
    stack: Vec<Value>,
    /// Call frame stack (pre-allocated)
    frames: Vec<Frame>,
    /// Global variables
    pub globals: HashMap<String, Value>,
    /// Garbage collector heap
    pub gc_heap: GcHeap,
}

impl FastVm {
    /// Create a new FastVm with pre-allocated capacity
    pub fn new() -> Self {
        FastVm {
            stack: Vec::with_capacity(DEFAULT_STACK_CAPACITY),
            frames: Vec::with_capacity(DEFAULT_FRAME_CAPACITY),
            globals: HashMap::new(),
            gc_heap: GcHeap::new(),
        }
    }

    /// Create a FastVm with custom stack capacity
    pub fn with_capacity(stack_capacity: usize, frame_capacity: usize) -> Self {
        FastVm {
            stack: Vec::with_capacity(stack_capacity),
            frames: Vec::with_capacity(frame_capacity),
            globals: HashMap::new(),
            gc_heap: GcHeap::new(),
        }
    }

    /// Create a FastVm with a custom GC threshold
    pub fn with_gc_threshold(threshold: usize) -> Self {
        FastVm {
            stack: Vec::with_capacity(DEFAULT_STACK_CAPACITY),
            frames: Vec::with_capacity(DEFAULT_FRAME_CAPACITY),
            globals: HashMap::new(),
            gc_heap: GcHeap::with_threshold(threshold),
        }
    }

    /// Collect garbage
    pub fn collect_garbage(&mut self) {
        let mut roots = Vec::new();
        roots.extend(self.stack.iter().cloned());
        roots.extend(self.globals.values().cloned());
        for frame in &self.frames {
            roots.push(Value::Closure(frame.closure.clone()));
            for upvalue in &frame.closure.upvalues {
                let upvalue = upvalue.lock().unwrap();
                if let Upvalue::Closed(value) = &*upvalue {
                    roots.push(value.clone());
                }
            }
        }
        self.gc_heap.collect(&roots);
    }

    /// Check if GC should run and trigger collection if needed
    pub fn maybe_collect_garbage(&mut self) {
        if self.gc_heap.should_collect() {
            self.collect_garbage();
        }
    }

    /// Get GC statistics
    pub fn gc_stats(&self) -> &crate::gc::GcStats {
        &self.gc_heap.stats
    }

    /// Execute a chunk of bytecode
    pub fn execute(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        let closure = Arc::new(Closure::new(chunk));
        let frame = Frame::new(closure, 0);
        self.frames.push(frame);
        self.run()
    }

    /// Optimized interpreter loop
    pub fn run(&mut self) -> Result<Value, VmError> {
        let start_depth = self.frames.len();

        loop {
            // Get instruction pointer and instructions reference
            let (ip, _instructions, closure) = {
                let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
                (
                    frame.ip,
                    &frame.closure.chunk.instructions,
                    frame.closure.clone(),
                )
            };

            // Bounds check with early return
            if ip >= closure.chunk.instructions.len() {
                return Err(VmError::InvalidInstructionPointer(ip));
            }

            // SAFETY: We just checked bounds above
            let instruction = unsafe { closure.chunk.instructions.get_unchecked(ip) };

            // Advance IP
            self.frames.last_mut().unwrap().ip += 1;

            // Dispatch on instruction reference (no clone for simple instructions)
            match instruction {
                Instruction::LoadConst(idx) => {
                    let constant = self.get_constant(*idx)?;
                    self.push_fast(constant);
                }

                Instruction::LoadLocal(idx) => {
                    let value = self.get_local_fast(*idx)?;
                    self.push_fast(value);
                }

                Instruction::StoreLocal(idx) => {
                    let value = self.pop_fast()?;
                    self.set_local_fast(*idx, value)?;
                }

                Instruction::LoadUpvalue(idx) => {
                    let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
                    let upvalue = frame
                        .closure
                        .get_upvalue(*idx as usize)
                        .ok_or(VmError::Runtime(format!("Invalid upvalue index: {}", idx)))?;
                    let value = match &*upvalue.lock().unwrap() {
                        Upvalue::Closed(v) => v.clone(),
                        Upvalue::Open(stack_idx) => self.stack[*stack_idx].clone(),
                    };
                    self.push_fast(value);
                }

                Instruction::StoreUpvalue(idx) => {
                    let value = self.pop_fast()?;
                    let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
                    let upvalue = frame
                        .closure
                        .get_upvalue(*idx as usize)
                        .ok_or(VmError::Runtime(format!("Invalid upvalue index: {}", idx)))?;
                    match &mut *upvalue.lock().unwrap() {
                        Upvalue::Closed(v) => *v = value,
                        Upvalue::Open(stack_idx) => self.stack[*stack_idx] = value,
                    };
                }

                Instruction::LoadGlobal(idx) => {
                    let name_val = self.get_constant(*idx)?;
                    let name = match name_val {
                        Value::Str(s) => s,
                        _ => {
                            return Err(VmError::TypeMismatch {
                                expected: "string (global name)",
                                got: name_val.type_name(),
                            })
                        }
                    };
                    let value =
                        self.globals.get(&name).cloned().ok_or_else(|| {
                            VmError::Runtime(format!("Undefined global: {}", name))
                        })?;
                    self.push_fast(value);
                }

                Instruction::Pop => {
                    self.pop_fast()?;
                }

                Instruction::Dup => {
                    let value = self.peek_fast()?.clone();
                    self.push_fast(value);
                }

                Instruction::CheckInt(expected) => {
                    let value = self.peek_fast()?;
                    let matches = matches!(value, Value::Int(n) if *n == *expected);
                    self.push_fast(Value::Bool(matches));
                }

                Instruction::CheckBool(expected) => {
                    let value = self.peek_fast()?;
                    let matches = matches!(value, Value::Bool(b) if *b == *expected);
                    self.push_fast(Value::Bool(matches));
                }

                Instruction::CheckString(expected) => {
                    let value = self.peek_fast()?;
                    let matches = matches!(value, Value::Str(s) if s == expected);
                    self.push_fast(Value::Bool(matches));
                }

                Instruction::CheckTupleLen(expected) => {
                    let value = self.peek_fast()?;
                    let matches = if let Value::Tuple(elements) = value {
                        elements.len() == *expected as usize
                    } else {
                        false
                    };
                    self.push_fast(Value::Bool(matches));
                }

                Instruction::GetTupleElem(index) => {
                    let value = self.peek_fast()?;
                    if let Value::Tuple(elements) = value {
                        if (*index as usize) < elements.len() {
                            self.push_fast(elements[*index as usize].clone());
                        } else {
                            return Err(VmError::Runtime("Tuple index out of bounds".into()));
                        }
                    } else {
                        return Err(VmError::Runtime("Not a tuple".into()));
                    }
                }

                Instruction::Add => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_add(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Sub => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_sub(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Mul => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_mul(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Div => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_div(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Concat => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    match (a, b) {
                        (Value::Str(a), Value::Str(b)) => {
                            let mut result = a;
                            result.push_str(&b);
                            self.push_fast(Value::Str(result))
                        }
                        (a, b) => {
                            return Err(VmError::Runtime(format!(
                                "Type mismatch in concatenation: {} ++ {}",
                                a.type_name(),
                                b.type_name()
                            )))
                        }
                    }
                }

                Instruction::Eq => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    self.push_fast(Value::Bool(a == b));
                }

                Instruction::Neq => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    self.push_fast(Value::Bool(a != b));
                }

                Instruction::Lt => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_cmp_lt(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Lte => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_cmp_lte(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Gt => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_cmp_gt(a, b)?;
                    self.push_fast(result);
                }

                Instruction::Gte => {
                    let b = self.pop_fast()?;
                    let a = self.pop_fast()?;
                    let result = self.binary_cmp_gte(a, b)?;
                    self.push_fast(result);
                }

                Instruction::And => {
                    let b = self.pop_bool_fast()?;
                    let a = self.pop_bool_fast()?;
                    self.push_fast(Value::Bool(a && b));
                }

                Instruction::Or => {
                    let b = self.pop_bool_fast()?;
                    let a = self.pop_bool_fast()?;
                    self.push_fast(Value::Bool(a || b));
                }

                Instruction::Not => {
                    let a = self.pop_bool_fast()?;
                    self.push_fast(Value::Bool(!a));
                }

                Instruction::Jump(offset) => {
                    self.jump_fast(*offset)?;
                }

                Instruction::JumpIfFalse(offset) => {
                    let condition = self.pop_fast()?;
                    if !condition.is_truthy() {
                        self.jump_fast(*offset)?;
                    }
                }

                Instruction::MakeTuple(n) => {
                    let n = *n as usize;
                    let mut elements = Vec::with_capacity(n);
                    for _ in 0..n {
                        elements.push(self.pop_fast()?);
                    }
                    elements.reverse();
                    self.push_fast(Value::Tuple(elements));
                }

                Instruction::GetTupleField(idx) => {
                    let value = self.pop_fast()?;
                    match value.as_tuple() {
                        Some(elements) => {
                            let index = *idx as usize;
                            if index >= elements.len() {
                                return Err(VmError::InvalidTupleFieldIndex {
                                    index: *idx,
                                    tuple_size: elements.len(),
                                });
                            }
                            self.push_fast(elements[index].clone());
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
                    let constant = self.get_constant(*idx)?;
                    let prototype = match constant.as_closure() {
                        Some(c) => c,
                        None => {
                            return Err(VmError::TypeMismatch {
                                expected: "closure",
                                got: constant.type_name(),
                            })
                        }
                    };

                    let mut closure = Closure::new(prototype.chunk.clone());
                    closure.arity = prototype.arity;
                    closure.name = prototype.name.clone();

                    for _ in 0..*upvalue_count {
                        self.pop_fast()?;
                    }

                    self.push_fast(Value::Closure(Arc::new(closure)));
                }

                Instruction::Call(argc) => {
                    self.execute_call(*argc)?;
                }

                Instruction::CallMethod(method_name_idx, argc) => {
                    self.execute_call_method(*method_name_idx, *argc)?;
                }

                Instruction::TailCall(_argc) => {
                    // TODO: Implement tail call optimization
                    unimplemented!("TailCall not yet implemented in FastVm")
                }

                Instruction::CloseUpvalue(_) => {
                    // Placeholder
                }

                Instruction::Return => {
                    self.frames.pop();
                    if self.frames.len() < start_depth {
                        return Ok(self.stack.pop().unwrap_or(Value::Unit));
                    }
                }

                Instruction::MakeList(n) => {
                    let n = *n as usize;
                    let mut elements = Vec::with_capacity(n);
                    for _ in 0..n {
                        elements.push(self.pop_fast()?);
                    }
                    elements.reverse();
                    let list = Value::vec_to_cons(elements);
                    self.push_fast(list);
                }

                Instruction::Cons => {
                    let tail = self.pop_fast()?;
                    let head = self.pop_fast()?;
                    self.push_fast(Value::Cons {
                        head: Box::new(head),
                        tail: Box::new(tail),
                    });
                }

                Instruction::ListHead => {
                    let value = self.pop_fast()?;
                    match value {
                        Value::Cons { head, .. } => self.push_fast(*head),
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
                    let value = self.pop_fast()?;
                    match value {
                        Value::Cons { tail, .. } => self.push_fast(*tail),
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
                    let value = self.pop_fast()?;
                    self.push_fast(Value::Bool(value.is_nil()));
                }

                Instruction::MakeArray(n) => {
                    let n = *n as usize;
                    let mut elements = Vec::with_capacity(n);
                    for _ in 0..n {
                        elements.push(self.pop_fast()?);
                    }
                    elements.reverse();
                    let array = Value::Array(Arc::new(Mutex::new(elements)));
                    self.push_fast(array);
                }

                Instruction::ArrayGet => {
                    let index = self.pop_fast()?;
                    let array = self.pop_fast()?;
                    let idx = self.extract_index(&index)?;
                    let value = array.array_get(idx).map_err(VmError::Runtime)?;
                    self.push_fast(value);
                }

                Instruction::ArraySet => {
                    let value = self.pop_fast()?;
                    let index = self.pop_fast()?;
                    let array = self.pop_fast()?;
                    let idx = self.extract_index(&index)?;
                    array.array_set(idx, value).map_err(VmError::Runtime)?;
                    self.push_fast(Value::Unit);
                }

                Instruction::ArrayLength => {
                    let array = self.pop_fast()?;
                    let len = array.array_length().map_err(VmError::Runtime)?;
                    self.push_fast(Value::Int(len));
                }

                Instruction::ArrayUpdate => {
                    let value = self.pop_fast()?;
                    let index = self.pop_fast()?;
                    let array = self.pop_fast()?;
                    let idx = self.extract_index(&index)?;

                    let new_arr = if let Value::Array(arr) = &array {
                        let mut new_elements = arr.lock().unwrap().clone();
                        if idx >= new_elements.len() {
                            return Err(VmError::Runtime(format!("Index {} out of bounds", idx)));
                        }
                        new_elements[idx] = value;
                        Value::Array(Arc::new(Mutex::new(new_elements)))
                    } else {
                        return Err(VmError::TypeMismatch {
                            expected: "array",
                            got: array.type_name(),
                        });
                    };
                    self.push_fast(new_arr);
                }

                Instruction::MakeRecord(n) => {
                    let n = *n as usize;
                    let mut fields = HashMap::new();
                    for _ in 0..n {
                        let field_value = self.pop_fast()?;
                        let field_name = self.pop_fast()?;
                        let field_name_str =
                            field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                                expected: "string",
                                got: field_name.type_name(),
                            })?;
                        fields.insert(field_name_str.to_string(), field_value);
                    }
                    let record = Value::Record(Arc::new(Mutex::new(fields)));
                    self.push_fast(record);
                }

                Instruction::GetRecordField => {
                    let field_name = self.pop_fast()?;
                    let record = self.pop_fast()?;
                    let field_name_str =
                        field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                            expected: "string",
                            got: field_name.type_name(),
                        })?;
                    let field_value = record
                        .record_get(field_name_str)
                        .map_err(VmError::Runtime)?;
                    self.push_fast(field_value);
                }

                Instruction::UpdateRecord(n) => {
                    let n = *n as usize;
                    let mut updates = HashMap::new();
                    for _ in 0..n {
                        let field_value = self.pop_fast()?;
                        let field_name = self.pop_fast()?;
                        let field_name_str =
                            field_name.as_str().ok_or_else(|| VmError::TypeMismatch {
                                expected: "string",
                                got: field_name.type_name(),
                            })?;
                        updates.insert(field_name_str.to_string(), field_value);
                    }
                    let record = self.pop_fast()?;
                    let new_record = record.record_update(updates).map_err(VmError::Runtime)?;
                    self.push_fast(new_record);
                }

                Instruction::MakeVariant(n) => {
                    let n = *n as usize;
                    let mut fields = Vec::with_capacity(n);
                    for _ in 0..n {
                        fields.push(self.pop_fast()?);
                    }
                    fields.reverse();

                    let variant_name = self.pop_fast()?;
                    let type_name = self.pop_fast()?;

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

                    let variant = Value::Variant {
                        type_name: type_name_str.to_string(),
                        variant_name: variant_name_str.to_string(),
                        fields,
                    };
                    self.push_fast(variant);
                }

                Instruction::CheckVariantTag(tag) => {
                    let variant = self.pop_fast()?;
                    let matches = variant.is_variant_named(tag);
                    self.push_fast(Value::Bool(matches));
                }

                Instruction::GetVariantField(idx) => {
                    let variant = self.pop_fast()?;
                    let field_value = variant
                        .variant_get_field(*idx as usize)
                        .map_err(VmError::Runtime)?;
                    self.push_fast(field_value);
                }
            }
        }
    }

    // ========== Inlined Stack Operations ==========

    #[inline(always)]
    fn push_fast(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline(always)]
    fn pop_fast(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    #[inline(always)]
    fn peek_fast(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackUnderflow)
    }

    #[inline(always)]
    fn pop_bool_fast(&mut self) -> Result<bool, VmError> {
        let value = self.pop_fast()?;
        value.as_bool().ok_or(VmError::TypeMismatch {
            expected: "bool",
            got: value.type_name(),
        })
    }

    // ========== Inlined Local Variable Access ==========

    #[inline(always)]
    fn get_local_fast(&self, idx: u8) -> Result<Value, VmError> {
        let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
        let local_idx = frame.base + idx as usize;

        debug_assert!(local_idx < self.stack.len(), "Local index out of bounds");

        if local_idx < self.stack.len() {
            // SAFETY: Bounds checked in debug, and we return error in release if invalid
            Ok(unsafe { self.stack.get_unchecked(local_idx).clone() })
        } else {
            Err(VmError::InvalidLocalIndex(idx))
        }
    }

    #[inline(always)]
    fn set_local_fast(&mut self, idx: u8, value: Value) -> Result<(), VmError> {
        let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
        let local_idx = frame.base + idx as usize;

        while self.stack.len() <= local_idx {
            self.stack.push(Value::Unit);
        }

        debug_assert!(local_idx < self.stack.len(), "Local index out of bounds");
        self.stack[local_idx] = value;
        Ok(())
    }

    // ========== Inlined Constant Access ==========

    #[inline(always)]
    fn get_constant(&self, idx: u16) -> Result<Value, VmError> {
        let frame = self.frames.last().ok_or(VmError::NoActiveFrame)?;
        frame
            .closure
            .chunk
            .constant_at(idx)
            .cloned()
            .ok_or(VmError::InvalidConstantIndex(idx))
    }

    // ========== Inlined Jump ==========

    #[inline(always)]
    fn jump_fast(&mut self, offset: i16) -> Result<(), VmError> {
        let frame = self.frames.last_mut().ok_or(VmError::NoActiveFrame)?;
        let new_ip = if offset >= 0 {
            frame.ip.wrapping_add(offset as usize)
        } else {
            frame.ip.wrapping_sub((-offset) as usize)
        };

        if new_ip > frame.closure.chunk.instructions.len() {
            return Err(VmError::InvalidInstructionPointer(new_ip));
        }

        frame.ip = new_ip;
        Ok(())
    }

    // ========== Inlined Arithmetic Operations ==========

    #[inline(always)]
    fn binary_add(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in addition: {} + {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_sub(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in subtraction: {} - {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_mul(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in multiplication: {} * {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_div(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in division: {} / {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    // ========== Inlined Comparison Operations ==========

    #[inline(always)]
    fn binary_cmp_lt(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in comparison: {} < {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_cmp_lte(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in comparison: {} <= {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_cmp_gt(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in comparison: {} > {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    #[inline(always)]
    fn binary_cmp_gte(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (a, b) => Err(VmError::Runtime(format!(
                "Type mismatch in comparison: {} >= {}",
                a.type_name(),
                b.type_name()
            ))),
        }
    }

    // ========== Helper Methods ==========

    #[inline(always)]
    fn extract_index(&self, value: &Value) -> Result<usize, VmError> {
        match value {
            Value::Int(i) => {
                if *i < 0 {
                    Err(VmError::TypeMismatch {
                        expected: "non-negative int",
                        got: "negative int",
                    })
                } else {
                    Ok(*i as usize)
                }
            }
            _ => Err(VmError::TypeMismatch {
                expected: "int",
                got: value.type_name(),
            }),
        }
    }

    fn execute_call(&mut self, argc: u8) -> Result<(), VmError> {
        let func_idx = self
            .stack
            .len()
            .checked_sub(1 + argc as usize)
            .ok_or(VmError::StackUnderflow)?;
        let func = self.stack[func_idx].clone();

        match func {
            Value::Closure(closure) => {
                if closure.arity != argc {
                    return Err(VmError::Runtime(format!(
                        "Arity mismatch: expected {}, got {}",
                        closure.arity, argc
                    )));
                }
                let base = func_idx + 1;
                let frame = Frame::new(closure, base);
                self.frames.push(frame);
                Ok(())
            }
            Value::NativeFn { name, .. } => {
                // FastVm does not support native function calls
                // Use the standard Vm for workloads requiring host functions
                Err(VmError::Runtime(format!(
                    "FastVm does not support native function calls: '{}'. Use standard Vm instead.",
                    name
                )))
            }
            _ => Err(VmError::TypeMismatch {
                expected: "function",
                got: func.type_name(),
            }),
        }
    }

    fn execute_call_method(&mut self, method_name_idx: u16, _argc: u8) -> Result<(), VmError> {
        let method_name_val = self.get_constant(method_name_idx)?;
        let method_name = match method_name_val {
            Value::Str(s) => s,
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "string (method name)",
                    got: method_name_val.type_name(),
                })
            }
        };

        // FastVm does not support method calls on host data
        // Use the standard Vm for workloads requiring host functions
        Err(VmError::Runtime(format!(
            "FastVm does not support method calls: '{}'. Use standard Vm instead.",
            method_name
        )))
    }

    /// Call a closure from Rust code (re-entrant)
    pub fn call_closure(
        &mut self,
        closure: Arc<Closure>,
        args: &[Value],
    ) -> Result<Value, VmError> {
        if closure.arity as usize != args.len() {
            return Err(VmError::Runtime(format!(
                "Arity mismatch: expected {}, got {}",
                closure.arity,
                args.len()
            )));
        }

        for arg in args {
            self.push_fast(arg.clone());
        }

        let base = self.stack.len() - args.len();
        let frame = Frame::new(closure, base);
        self.frames.push(frame);
        self.run()
    }

    /// Call any callable value from Rust code
    pub fn call_value(&mut self, func: Value, args: &[Value]) -> Result<Value, VmError> {
        match func {
            Value::Closure(closure) => self.call_closure(closure, args),
            Value::NativeFn { name, .. } => Err(VmError::Runtime(format!(
                "FastVm does not support native function calls: '{}'. Use standard Vm instead.",
                name
            ))),
            _ => Err(VmError::TypeMismatch {
                expected: "function",
                got: func.type_name(),
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

impl Default for FastVm {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkBuilder;

    #[test]
    fn test_fast_vm_basic_arithmetic() {
        let mut vm = FastVm::new();
        let chunk = ChunkBuilder::new()
            .constant(Value::Int(10))
            .constant(Value::Int(20))
            .instruction(Instruction::LoadConst(0))
            .instruction(Instruction::LoadConst(1))
            .instruction(Instruction::Add)
            .instruction(Instruction::Return)
            .build();
        let result = vm.execute(chunk).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_fast_vm_locals() {
        let mut vm = FastVm::new();
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
    fn test_fast_vm_comparison() {
        let mut vm = FastVm::new();
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
    fn test_fast_vm_tuple() {
        let mut vm = FastVm::new();
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
    fn test_fast_vm_pre_allocated_capacity() {
        let vm = FastVm::with_capacity(512, 128);
        assert!(vm.stack.capacity() >= 512);
        assert!(vm.frames.capacity() >= 128);
    }
}
