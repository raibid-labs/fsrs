// Fusabi Standard Library
// Provides built-in functions for List, String, and Option operations

pub mod list;
pub mod option;
pub mod string;

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Register all standard library functions into the VM
pub fn register_stdlib(vm: &mut Vm) {
    // 1. Register functions in HostRegistry
    {
        let mut registry = vm.host_registry.borrow_mut();

        // List functions
        registry.register("List.length", |_vm, args| wrap_unary(args, list::list_length));
        registry.register("List.head", |_vm, args| wrap_unary(args, list::list_head));
        registry.register("List.tail", |_vm, args| wrap_unary(args, list::list_tail));
        registry.register("List.reverse", |_vm, args| wrap_unary(args, list::list_reverse));
        registry.register("List.isEmpty", |_vm, args| wrap_unary(args, list::list_is_empty));
        registry.register("List.append", |_vm, args| wrap_binary(args, list::list_append));
        registry.register("List.concat", |_vm, args| wrap_unary(args, list::list_concat));
        registry.register("List.map", list::list_map);

        // String functions
        registry.register("String.length", |_vm, args| wrap_unary(args, string::string_length));
        registry.register("String.trim", |_vm, args| wrap_unary(args, string::string_trim));
        registry.register("String.toLower", |_vm, args| wrap_unary(args, string::string_to_lower));
        registry.register("String.toUpper", |_vm, args| wrap_unary(args, string::string_to_upper));
        registry.register("String.split", |_vm, args| wrap_binary(args, string::string_split));
        registry.register("String.concat", |_vm, args| wrap_unary(args, string::string_concat));
        registry.register("String.contains", |_vm, args| wrap_binary(args, string::string_contains));
        registry.register("String.startsWith", |_vm, args| wrap_binary(args, string::string_starts_with));
        registry.register("String.endsWith", |_vm, args| wrap_binary(args, string::string_ends_with));

        // Option functions
        registry.register("Option.isSome", |_vm, args| wrap_unary(args, option::option_is_some));
        registry.register("Option.isNone", |_vm, args| wrap_unary(args, option::option_is_none));
        registry.register("Option.defaultValue", |_vm, args| wrap_binary(args, option::option_default_value));
    }

    // 2. Populate Globals with Module Records
    
    // Helper to create NativeFn value
    let native = |name: &str, arity: u8| Value::NativeFn { 
        name: name.to_string(), 
        arity, 
        args: vec![] 
    };

    // List Module
    let mut list_fields = HashMap::new();
    list_fields.insert("length".to_string(), native("List.length", 1));
    list_fields.insert("head".to_string(), native("List.head", 1));
    list_fields.insert("tail".to_string(), native("List.tail", 1));
    list_fields.insert("reverse".to_string(), native("List.reverse", 1));
    list_fields.insert("isEmpty".to_string(), native("List.isEmpty", 1));
    list_fields.insert("append".to_string(), native("List.append", 2));
    list_fields.insert("concat".to_string(), native("List.concat", 1));
    list_fields.insert("map".to_string(), native("List.map", 2));
    vm.globals.insert("List".to_string(), Value::Record(Rc::new(RefCell::new(list_fields))));

    // String Module
    let mut string_fields = HashMap::new();
    string_fields.insert("length".to_string(), native("String.length", 1));
    string_fields.insert("trim".to_string(), native("String.trim", 1));
    string_fields.insert("toLower".to_string(), native("String.toLower", 1));
    string_fields.insert("toUpper".to_string(), native("String.toUpper", 1));
    string_fields.insert("split".to_string(), native("String.split", 2));
    string_fields.insert("concat".to_string(), native("String.concat", 1));
    string_fields.insert("contains".to_string(), native("String.contains", 2));
    string_fields.insert("startsWith".to_string(), native("String.startsWith", 2));
    string_fields.insert("endsWith".to_string(), native("String.endsWith", 2));
    vm.globals.insert("String".to_string(), Value::Record(Rc::new(RefCell::new(string_fields))));

    // Option Module
    let mut option_fields = HashMap::new();
    option_fields.insert("isSome".to_string(), native("Option.isSome", 1));
    option_fields.insert("isNone".to_string(), native("Option.isNone", 1));
    option_fields.insert("defaultValue".to_string(), native("Option.defaultValue", 2));
    vm.globals.insert("Option".to_string(), Value::Record(Rc::new(RefCell::new(option_fields))));
}

fn wrap_unary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value) -> Result<Value, VmError>,
{
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Expected 1 argument, got {}",
            args.len()
        )));
    }
    f(&args[0])
}

fn wrap_binary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value, &Value) -> Result<Value, VmError>,
{
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Expected 2 arguments, got {}",
            args.len()
        )));
    }
    f(&args[0], &args[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_stdlib() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);
        
        // Check HostRegistry
        assert!(vm.host_registry.borrow().has_function("List.length"));
        
        // Check Globals
        assert!(vm.globals.contains_key("List"));
        if let Some(Value::Record(r)) = vm.globals.get("List") {
            assert!(r.borrow().contains_key("length"));
        } else {
            panic!("List global is not a record");
        }
    }
}
