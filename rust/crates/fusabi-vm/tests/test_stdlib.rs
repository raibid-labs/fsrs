// Fusabi Standard Library Integration Tests
// Tests stdlib functions through the VM

use fusabi_vm::stdlib::list::*;
use fusabi_vm::stdlib::option::*;
use fusabi_vm::stdlib::register_stdlib;
use fusabi_vm::stdlib::string::*;
use fusabi_vm::{Value, Vm, VmError};
use std::sync::Arc;

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

#[test]
fn test_string_format_integration() {
    let fmt = Value::Str("%s version %d.%d".to_string());
    let args = Value::vec_to_cons(vec![
        Value::Str("MyApp".to_string()),
        Value::Int(1),
        Value::Int(0),
    ]);
    let result = string_format(&fmt, &args).unwrap();
    assert_eq!(result, Value::Str("MyApp version 1.0".to_string()));
}

#[test]
fn test_string_format_precision_integration() {
    let fmt = Value::Str("Price: $%.2f".to_string());
    let args = Value::vec_to_cons(vec![Value::Float(19.99)]);
    let result = string_format(&fmt, &args).unwrap();
    assert_eq!(result, Value::Str("Price: $19.99".to_string()));
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
    assert!(vm.host_registry.lock().unwrap().has_function("List.length"));
    assert!(vm
        .host_registry
        .lock()
        .unwrap()
        .has_function("String.length"));
    assert!(vm
        .host_registry
        .lock()
        .unwrap()
        .has_function("Option.isSome"));

    // Verify Globals (Module Records)
    assert!(vm.globals.contains_key("List"));
    if let Some(Value::Record(r)) = vm.globals.get("List") {
        let borrowed = r.lock().unwrap();
        assert!(borrowed.contains_key("length"));
        assert!(
            matches!(borrowed.get("length").unwrap(), Value::NativeFn { name, arity: 1, args: _ } if name == "List.length")
        );
    } else {
        panic!("List global is not a record");
    }

    assert!(vm.globals.contains_key("String"));
    if let Some(Value::Record(r)) = vm.globals.get("String") {
        let borrowed = r.lock().unwrap();
        assert!(borrowed.contains_key("length"));
    }

    assert!(vm.globals.contains_key("Option"));
    if let Some(Value::Record(r)) = vm.globals.get("Option") {
        let borrowed = r.lock().unwrap();
        assert!(borrowed.contains_key("isSome"));
    }
}

// Helper to call stdlib function through VM
fn call_stdlib_function(vm: &mut Vm, name: &str, args: &[Value]) -> Result<Value, VmError> {
    // Handle module.function lookup
    if let Some((module, func_name)) = name.split_once('.') {
        let module_val = vm
            .globals
            .get(module)
            .ok_or_else(|| VmError::Runtime(format!("Undefined module: {}", module)))?;
        if let Value::Record(r) = module_val {
            let func = r
                .lock()
                .unwrap()
                .get(func_name)
                .cloned()
                .ok_or_else(|| VmError::Runtime(format!("Undefined function: {}", name)))?;
            return vm.call_value(func, args);
        }
    }

    let func = vm
        .globals
        .get(name)
        .cloned()
        .ok_or(VmError::Runtime(format!(
            "Undefined global function: {}",
            name
        )))?;
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
    let list = Value::vec_to_cons(vec![
        Value::Str("hello".to_string()),
        Value::Str("world".to_string()),
    ]);
    let result = call_stdlib_function(&mut vm, "String.concat", &[list]).unwrap();
    assert_eq!(result, Value::Str("helloworld".to_string()));
}

#[test]
fn test_string_format_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let fmt = Value::Str("%s version %d.%d".to_string());
    let args = Value::vec_to_cons(vec![
        Value::Str("MyApp".to_string()),
        Value::Int(1),
        Value::Int(0),
    ]);
    let result = call_stdlib_function(&mut vm, "String.format", &[fmt, args]).unwrap();
    assert_eq!(result, Value::Str("MyApp version 1.0".to_string()));
}

#[test]
fn test_sprintf_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let fmt = Value::Str("Hello, %s!".to_string());
    let args = Value::vec_to_cons(vec![Value::Str("World".to_string())]);
    let result = call_stdlib_function(&mut vm, "sprintf", &[fmt, args]).unwrap();
    assert_eq!(result, Value::Str("Hello, World!".to_string()));
}

#[test]
fn test_string_format_precision_through_vm() {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    let fmt = Value::Str("Price: $%.2f".to_string());
    let args = Value::vec_to_cons(vec![Value::Float(19.99)]);
    let result = call_stdlib_function(&mut vm, "String.format", &[fmt, args]).unwrap();
    assert_eq!(result, Value::Str("Price: $19.99".to_string()));
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
    let func_closure = Arc::new(fusabi_vm::closure::Closure::with_arity(func_chunk, 1));

    let result =
        call_stdlib_function(&mut vm, "List.map", &[Value::Closure(func_closure), list]).unwrap();
    assert_eq!(
        result,
        Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

// ========== Option Constructor Tests ==========

#[test]
fn test_some_constructor_through_vm() {
    let mut vm = get_test_vm();
    let result = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            assert_eq!(type_name, "Option");
            assert_eq!(variant_name, "Some");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Variant, got {:?}", result),
    }
}

#[test]
fn test_none_constructor_through_vm() {
    let mut vm = get_test_vm();
    let result = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    match result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            assert_eq!(type_name, "Option");
            assert_eq!(variant_name, "None");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected Variant, got {:?}", result),
    }
}

#[test]
fn test_option_map_through_vm() {
    let mut vm = get_test_vm();

    // Create Some(5)
    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(5)]).unwrap();

    // Create a function that doubles the value
    let double_chunk = fusabi_vm::chunk::ChunkBuilder::new()
        .constant(Value::Int(2))
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(0))
        .instruction(fusabi_vm::instruction::Instruction::LoadConst(0))
        .instruction(fusabi_vm::instruction::Instruction::Mul)
        .instruction(fusabi_vm::instruction::Instruction::Return)
        .build();
    let double_closure = Arc::new(fusabi_vm::closure::Closure::with_arity(double_chunk, 1));

    let result = call_stdlib_function(
        &mut vm,
        "Option.map",
        &[Value::Closure(double_closure), some_val],
    )
    .unwrap();

    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(10));
        }
        _ => panic!("Expected Some(10), got {:?}", result),
    }
}

#[test]
fn test_option_default_value_through_vm() {
    let mut vm = get_test_vm();

    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();
    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    let result_some =
        call_stdlib_function(&mut vm, "Option.defaultValue", &[Value::Int(0), some_val]).unwrap();
    let result_none =
        call_stdlib_function(&mut vm, "Option.defaultValue", &[Value::Int(0), none_val]).unwrap();

    assert_eq!(result_some, Value::Int(42));
    assert_eq!(result_none, Value::Int(0));
}

#[test]
fn test_option_is_some_through_vm() {
    let mut vm = get_test_vm();

    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();
    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    let result_some = call_stdlib_function(&mut vm, "Option.isSome", &[some_val]).unwrap();
    let result_none = call_stdlib_function(&mut vm, "Option.isSome", &[none_val]).unwrap();

    assert_eq!(result_some, Value::Bool(true));
    assert_eq!(result_none, Value::Bool(false));
}

#[test]
fn test_option_is_none_through_vm() {
    let mut vm = get_test_vm();

    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();
    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    let result_some = call_stdlib_function(&mut vm, "Option.isNone", &[some_val]).unwrap();
    let result_none = call_stdlib_function(&mut vm, "Option.isNone", &[none_val]).unwrap();

    assert_eq!(result_some, Value::Bool(false));
    assert_eq!(result_none, Value::Bool(true));
}

#[test]
fn test_option_map_none_through_vm() {
    let mut vm = get_test_vm();

    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    // Create any function (it shouldn't be called)
    let func_chunk = fusabi_vm::chunk::ChunkBuilder::new()
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(0))
        .instruction(fusabi_vm::instruction::Instruction::Return)
        .build();
    let func_closure = Arc::new(fusabi_vm::closure::Closure::with_arity(func_chunk, 1));

    let result = call_stdlib_function(
        &mut vm,
        "Option.map",
        &[Value::Closure(func_closure), none_val],
    )
    .unwrap();

    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "None");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected None, got {:?}", result),
    }
}

#[test]
fn test_option_or_else_through_vm() {
    let mut vm = get_test_vm();

    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();
    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();
    let backup_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(99)]).unwrap();

    // Some orElse backup = Some
    let result1 =
        call_stdlib_function(&mut vm, "Option.orElse", &[some_val, backup_val.clone()]).unwrap();
    match result1 {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Some(42), got {:?}", result1),
    }

    // None orElse backup = backup
    let result2 = call_stdlib_function(&mut vm, "Option.orElse", &[none_val, backup_val]).unwrap();
    match result2 {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(99));
        }
        _ => panic!("Expected Some(99), got {:?}", result2),
    }
}

#[test]
fn test_option_map2_through_vm() {
    let mut vm = get_test_vm();

    let some1 = call_stdlib_function(&mut vm, "Some", &[Value::Int(3)]).unwrap();
    let some2 = call_stdlib_function(&mut vm, "Some", &[Value::Int(4)]).unwrap();
    let none_val = call_stdlib_function(&mut vm, "None", &[]).unwrap();

    // Create an add function
    let add_chunk = fusabi_vm::chunk::ChunkBuilder::new()
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(0))
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(1))
        .instruction(fusabi_vm::instruction::Instruction::Add)
        .instruction(fusabi_vm::instruction::Instruction::Return)
        .build();
    let add_closure = Arc::new(fusabi_vm::closure::Closure::with_arity(add_chunk, 2));

    // Some(3) + Some(4) = Some(7)
    let result1 = call_stdlib_function(
        &mut vm,
        "Option.map2",
        &[Value::Closure(add_closure.clone()), some1, some2],
    )
    .unwrap();
    match result1 {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(7));
        }
        _ => panic!("Expected Some(7), got {:?}", result1),
    }

    // Some(3) + None = None
    let some3 = call_stdlib_function(&mut vm, "Some", &[Value::Int(3)]).unwrap();
    let result2 = call_stdlib_function(
        &mut vm,
        "Option.map2",
        &[Value::Closure(add_closure), some3, none_val],
    )
    .unwrap();
    match result2 {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "None");
            assert_eq!(fields.len(), 0);
        }
        _ => panic!("Expected None, got {:?}", result2),
    }
}

#[test]
fn test_option_iter_through_vm() {
    let mut vm = get_test_vm();

    let some_val = call_stdlib_function(&mut vm, "Some", &[Value::Int(42)]).unwrap();

    // Create a simple function that just returns unit (for iter side effect)
    let func_chunk = fusabi_vm::chunk::ChunkBuilder::new()
        .instruction(fusabi_vm::instruction::Instruction::LoadLocal(0))
        .instruction(fusabi_vm::instruction::Instruction::Pop)
        .instruction(fusabi_vm::instruction::Instruction::Return)
        .build();
    let func_closure = Arc::new(fusabi_vm::closure::Closure::with_arity(func_chunk, 1));

    let result = call_stdlib_function(
        &mut vm,
        "Option.iter",
        &[Value::Closure(func_closure), some_val],
    )
    .unwrap();
    assert_eq!(result, Value::Unit);
}

// ========== Print Function Tests ==========

#[test]
fn test_print_function_exists() {
    let vm = get_test_vm();
    // Check print is registered as a global
    assert!(vm.globals.contains_key("print"));
    assert!(vm.globals.contains_key("printfn"));
}

#[test]
fn test_print_returns_unit() {
    let mut vm = get_test_vm();
    // Both print and printfn should return Unit
    let result_print = call_stdlib_function(&mut vm, "print", &[Value::Int(42)]).unwrap();
    assert_eq!(result_print, Value::Unit);

    let result_printfn =
        call_stdlib_function(&mut vm, "printfn", &[Value::Str("test".to_string())]).unwrap();
    assert_eq!(result_printfn, Value::Unit);
}

#[test]
fn test_print_various_types() {
    let mut vm = get_test_vm();

    // Test with different types - all should return Unit without error
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[Value::Int(42)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[Value::Float(3.14)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[Value::Bool(true)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[Value::Str("hello".to_string())]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[Value::Unit]).unwrap(),
        Value::Unit
    );

    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[list]).unwrap(),
        Value::Unit
    );

    let tuple = Value::Tuple(vec![Value::Int(1), Value::Str("test".to_string())]);
    assert_eq!(
        call_stdlib_function(&mut vm, "print", &[tuple]).unwrap(),
        Value::Unit
    );
}

#[test]
fn test_printfn_various_types() {
    let mut vm = get_test_vm();

    // Test with different types - all should return Unit without error
    assert_eq!(
        call_stdlib_function(&mut vm, "printfn", &[Value::Int(42)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "printfn", &[Value::Float(3.14)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "printfn", &[Value::Bool(false)]).unwrap(),
        Value::Unit
    );
    assert_eq!(
        call_stdlib_function(&mut vm, "printfn", &[Value::Str("world".to_string())]).unwrap(),
        Value::Unit
    );

    let list = Value::vec_to_cons(vec![Value::Int(10), Value::Int(20)]);
    assert_eq!(
        call_stdlib_function(&mut vm, "printfn", &[list]).unwrap(),
        Value::Unit
    );
}
