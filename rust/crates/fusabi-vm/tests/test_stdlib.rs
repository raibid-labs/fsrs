// Fusabi Standard Library Integration Tests
// Tests stdlib functions through the VM

use fusabi_vm::stdlib::list::*;
use fusabi_vm::stdlib::option::*;
use fusabi_vm::stdlib::string::*;
use fusabi_vm::stdlib::register_stdlib;
use fusabi_vm::{Value, Vm, VmError};
use std::rc::Rc;

// Helper to get a configured VM for tests
fn get_test_vm() -> Vm {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    vm
}

// ========== List Tests ==========

#[test]
fn test_list_length_integration() {
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let _vm = get_test_vm(); // Ensure VM logic is initialized if needed (though purely functional here)
    let result = list_length(&list).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_list_head_integration() {
    let list = Value::vec_to_cons(vec![Value::Int(42)]);
    let result = list_head(&list).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_list_reverse_integration() {
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = list_reverse(&list).unwrap();
    let expected = Value::vec_to_cons(vec![Value::Int(3), Value::Int(2), Value::Int(1)]);
    assert_eq!(result, expected);
}

#[test]
fn test_list_append_integration() {
    let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
    let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
    let result = list_append(&list1, &list2).unwrap();
    let expected = Value::vec_to_cons(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_list_concat_integration() {
    let list1 = Value::vec_to_cons(vec![Value::Int(1)]);
    let list2 = Value::vec_to_cons(vec![Value::Int(2)]);
    let list3 = Value::vec_to_cons(vec![Value::Int(3)]);
    let lists = Value::vec_to_cons(vec![list1, list2, list3]);
    let result = list_concat(&lists).unwrap();
    let expected = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(result, expected);
}

// ========== String Tests ==========

#[test]
fn test_string_length_integration() {
    let s = Value::Str("hello".to_string());
    let result = string_length(&s).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_string_trim_integration() {
    let s = Value::Str("  hello  ".to_string());
    let result = string_trim(&s).unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
}

#[test]
fn test_string_split_integration() {
    let delim = Value::Str(" ".to_string());
    let s = Value::Str("hello world".to_string());
    let result = string_split(&delim, &s).unwrap();
    let expected = Value::vec_to_cons(vec![
        Value::Str("hello".to_string()),
        Value::Str("world".to_string()),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn test_string_concat_integration() {
    let list = Value::vec_to_cons(vec![
        Value::Str("hello".to_string()),
        Value::Str(" ".to_string()),
        Value::Str("world".to_string()),
    ]);
    let result = string_concat(&list).unwrap();
    assert_eq!(result, Value::Str("hello world".to_string()));
}

#[test]
fn test_string_case_conversion_integration() {
    let s = Value::Str("Hello World".to_string());
    let lower = string_to_lower(&s).unwrap();
    let upper = string_to_upper(&s).unwrap();
    assert_eq!(lower, Value::Str("hello world".to_string()));
    assert_eq!(upper, Value::Str("HELLO WORLD".to_string()));
}

#[test]
fn test_string_predicates_integration() {
    let haystack = Value::Str("hello world".to_string());
    let needle = Value::Str("world".to_string());
    let prefix = Value::Str("hello".to_string());
    let suffix = Value::Str("world".to_string());

    let contains = string_contains(&needle, &haystack).unwrap();
    let starts = string_starts_with(&prefix, &haystack).unwrap();
    let ends = string_ends_with(&suffix, &haystack).unwrap();

    assert_eq!(contains, Value::Bool(true));
    assert_eq!(starts, Value::Bool(true));
    assert_eq!(ends, Value::Bool(true));
}

// ========== Option Tests ==========

#[test]
fn test_option_is_some_integration() {
    let some_val = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![Value::Int(42)],
    };
    let result = option_is_some(&some_val).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_option_is_none_integration() {
    let none_val = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        fields: vec![],
    };
    let result = option_is_none(&none_val).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_option_default_value_integration() {
    let some_val = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![Value::Int(42)],
    };
    let none_val = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        fields: vec![],
    };
    let default = Value::Int(0);

    let result_some = option_default_value(&default, &some_val).unwrap();
    let result_none = option_default_value(&default, &none_val).unwrap();

    assert_eq!(result_some, Value::Int(42));
    assert_eq!(result_none, Value::Int(0));
}

// ========== Registry Tests ==========

#[test]
fn test_register_stdlib_functions_and_globals() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm); // Register functions and populate globals

    // Verify HostRegistry
    assert!(vm.host_registry.borrow().has_function("List.length"));
    assert!(vm.host_registry.borrow().has_function("String.length"));
    assert!(vm.host_registry.borrow().has_function("Option.isSome"));

    // Verify Globals (Module Records)
    assert!(vm.globals.contains_key("List"));
    if let Some(Value::Record(r)) = vm.globals.get("List") {
        let borrowed = r.borrow();
        assert!(borrowed.contains_key("length"));
        assert!(matches!(borrowed.get("length").unwrap(), Value::NativeFn { name, arity: 1, args: _ } if name == "List.length"));
    } else {
        panic!("List global is not a record");
    }

    assert!(vm.globals.contains_key("String"));
    if let Some(Value::Record(r)) = vm.globals.get("String") {
        let borrowed = r.borrow();
        assert!(borrowed.contains_key("length"));
    }

    assert!(vm.globals.contains_key("Option"));
    if let Some(Value::Record(r)) = vm.globals.get("Option") {
        let borrowed = r.borrow();
        assert!(borrowed.contains_key("isSome"));
    }
}

// Helper to call stdlib function through VM
fn call_stdlib_function(vm: &mut Vm, name: &str, args: &[Value]) -> Result<Value, VmError> {
    // Handle module.function lookup
    if let Some((module, func_name)) = name.split_once('.') {
        let module_val = vm.globals.get(module).ok_or_else(|| VmError::Runtime(format!("Undefined module: {}", module)))?;
        if let Value::Record(r) = module_val {
             let func = r.borrow().get(func_name).cloned().ok_or_else(|| VmError::Runtime(format!("Undefined function: {}", name)))?;
             return vm.call_value(func, args);
        }
    }
    
    let func = vm.globals.get(name).cloned().ok_or(VmError::Runtime(format!("Undefined global function: {}", name)))?;
    vm.call_value(func, args)
}

#[test]
fn test_list_length_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = call_stdlib_function(&mut vm, "List.length", &[list]).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_string_concat_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let list = Value::vec_to_cons(vec![Value::Str("hello".to_string()), Value::Str("world".to_string())]);
    let result = call_stdlib_function(&mut vm, "String.concat", &[list]).unwrap();
    assert_eq!(result, Value::Str("helloworld".to_string()));
}

#[test]
fn test_list_map_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    
    // Create a simple identity closure
    let func_chunk = fusabi_vm::chunk::ChunkBuilder::new()
        .constant(Value::Int(0)) // Placeholder for local 0
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(0))
        .instruction(fusabi_vm::instruction::Instruction::Return)
        .build();
    let func_closure = Rc::new(fusabi_vm::closure::Closure::with_arity(func_chunk, 1));
    
    let result = call_stdlib_function(&mut vm, "List.map", &[Value::Closure(func_closure), list]).unwrap();
    assert_eq!(result, Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
}