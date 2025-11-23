// Test for bytecode serialization and deserialization

use fusabi_vm::{
    chunk::Chunk,
    instruction::Instruction,
    value::Value,
    FZB_MAGIC, FZB_VERSION,
    serialize_chunk, deserialize_chunk,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use fusabi_vm::closure::{Closure, Upvalue};

#[test]
fn test_serialize_deserialize_chunk_simple() {
    let mut chunk = Chunk::new();
    let const_int = chunk.add_constant(Value::Int(42));
    let const_str = chunk.add_constant(Value::Str("hello".to_string()));
    chunk.emit(Instruction::LoadConst(const_int));
    chunk.emit(Instruction::LoadConst(const_str));
    chunk.emit(Instruction::Add); // This will cause type error, but valid bytecode structure
    chunk.emit(Instruction::Return);
    chunk.name = Some("test_chunk".to_string());

    let bytes = serialize_chunk(&chunk).unwrap();
    let restored_chunk = deserialize_chunk(&bytes).unwrap();

    assert_eq!(chunk, restored_chunk);
}

#[test]
fn test_serialize_deserialize_chunk_with_closure_prototype() {
    let mut inner_chunk = Chunk::new();
    inner_chunk.add_constant(Value::Int(1));
    inner_chunk.emit(Instruction::LoadConst(0));
    inner_chunk.emit(Instruction::Return);
    inner_chunk.name = Some("inner_func".to_string());

    let closure_prototype = Closure::with_arity(inner_chunk, 1);
    
    let mut chunk = Chunk::new();
    let const_closure_idx = chunk.add_constant(Value::Closure(Rc::new(closure_prototype)));
    chunk.emit(Instruction::MakeClosure(const_closure_idx, 0)); // 0 upvalues
    chunk.emit(Instruction::Return);

    let bytes = serialize_chunk(&chunk).unwrap();
    let restored_chunk = deserialize_chunk(&bytes).unwrap();

    assert_eq!(chunk, restored_chunk);

    // Verify the closure prototype inside constants
    if let Value::Closure(restored_closure) = &restored_chunk.constants[0] {
        assert_eq!(restored_closure.chunk.name, Some("inner_func".to_string()));
        assert_eq!(restored_closure.arity, 1);
    } else {
        panic!("Expected a Closure in constants");
    }
}

#[test]
fn test_serialize_deserialize_chunk_with_native_fn_prototype() {
    let native_fn_val = Value::NativeFn {
        name: "test_native_fn".to_string(),
        arity: 2,
        args: vec![Value::Int(10)], // Pre-applied args
    };

    let mut chunk = Chunk::new();
    let const_native_fn_idx = chunk.add_constant(native_fn_val.clone());
    chunk.emit(Instruction::LoadConst(const_native_fn_idx));
    chunk.emit(Instruction::Return);

    let bytes = serialize_chunk(&chunk).unwrap();
    let restored_chunk = deserialize_chunk(&bytes).unwrap();

    assert_eq!(chunk, restored_chunk);

    // Verify the native fn prototype inside constants
    if let Value::NativeFn { name, arity, args } = &restored_chunk.constants[0] {
        assert_eq!(name, "test_native_fn");
        assert_eq!(*arity, 2);
        assert_eq!(*args, vec![Value::Int(10)]);
    } else {
        panic!("Expected a NativeFn in constants");
    }
}

#[test]
fn test_serialize_deserialize_chunk_complex_value() {
    let list_val = Value::vec_to_cons(vec![Value::Int(1), Value::Str("a".to_string())]);
    let tuple_val = Value::Tuple(vec![Value::Bool(true), list_val.clone()]);

    let mut chunk = Chunk::new();
    chunk.add_constant(tuple_val.clone());
    chunk.emit(Instruction::Return);

    let bytes = serialize_chunk(&chunk).unwrap();
    let restored_chunk = deserialize_chunk(&bytes).unwrap();

    assert_eq!(chunk, restored_chunk);
    assert_eq!(restored_chunk.constants[0], tuple_val);
}

#[test]
fn test_magic_bytes_and_version() {
    let chunk = Chunk::new();
    let bytes = serialize_chunk(&chunk).unwrap();
    assert!(bytes.starts_with(FZB_MAGIC));
    assert_eq!(bytes[4], FZB_VERSION);
}

#[test]
fn test_deserialize_invalid_magic_bytes() {
    let mut bytes = vec![0xFF; 10]; // Invalid magic bytes
    bytes[0..4].copy_from_slice(b"BAD!");
    bytes[4] = FZB_VERSION;
    let result = deserialize_chunk(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid magic bytes"));
}

#[test]
fn test_deserialize_unsupported_version() {
    let mut bytes = vec![0; 10];
    bytes[0..4].copy_from_slice(FZB_MAGIC);
    bytes[4] = 99; // Unsupported version
    let result = deserialize_chunk(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported version"));
}

#[test]
fn test_deserialize_file_too_short() {
    let bytes = vec![0; 3]; // Too short for magic bytes + version
    let result = deserialize_chunk(&bytes);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("File too short"));
}
