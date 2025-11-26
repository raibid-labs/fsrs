// Fusabi Standard Library
// Provides built-in functions for List, String, Map, and Option operations

pub mod list;
pub mod map;
pub mod option;
pub mod string;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "osc")]
pub mod net;

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Register all standard library functions into the VM
pub fn register_stdlib(vm: &mut Vm) {
    // 1. Register functions in HostRegistry
    {
        let mut registry = vm.host_registry.borrow_mut();

        // List functions
        registry.register("List.length", |_vm, args| {
            wrap_unary(args, list::list_length)
        });
        registry.register("List.head", |_vm, args| wrap_unary(args, list::list_head));
        registry.register("List.tail", |_vm, args| wrap_unary(args, list::list_tail));
        registry.register("List.reverse", |_vm, args| {
            wrap_unary(args, list::list_reverse)
        });
        registry.register("List.isEmpty", |_vm, args| {
            wrap_unary(args, list::list_is_empty)
        });
        registry.register("List.append", |_vm, args| {
            wrap_binary(args, list::list_append)
        });
        registry.register("List.concat", |_vm, args| {
            wrap_unary(args, list::list_concat)
        });
        registry.register("List.map", list::list_map);

        // String functions
        registry.register("String.length", |_vm, args| {
            wrap_unary(args, string::string_length)
        });
        registry.register("String.trim", |_vm, args| {
            wrap_unary(args, string::string_trim)
        });
        registry.register("String.toLower", |_vm, args| {
            wrap_unary(args, string::string_to_lower)
        });
        registry.register("String.toUpper", |_vm, args| {
            wrap_unary(args, string::string_to_upper)
        });
        registry.register("String.split", |_vm, args| {
            wrap_binary(args, string::string_split)
        });
        registry.register("String.concat", |_vm, args| {
            wrap_unary(args, string::string_concat)
        });
        registry.register("String.contains", |_vm, args| {
            wrap_binary(args, string::string_contains)
        });
        registry.register("String.startsWith", |_vm, args| {
            wrap_binary(args, string::string_starts_with)
        });
        registry.register("String.endsWith", |_vm, args| {
            wrap_binary(args, string::string_ends_with)
        });

        // Map functions
        registry.register("Map.empty", |_vm, args| {
            wrap_unary(args, map::map_empty)
        });
        registry.register("Map.add", |_vm, args| {
            wrap_ternary(args, map::map_add)
        });
        registry.register("Map.remove", |_vm, args| {
            wrap_binary(args, map::map_remove)
        });
        registry.register("Map.find", |_vm, args| {
            wrap_binary(args, map::map_find)
        });
        registry.register("Map.tryFind", |_vm, args| {
            wrap_binary(args, map::map_try_find)
        });
        registry.register("Map.containsKey", |_vm, args| {
            wrap_binary(args, map::map_contains_key)
        });
        registry.register("Map.isEmpty", |_vm, args| {
            wrap_unary(args, map::map_is_empty)
        });
        registry.register("Map.count", |_vm, args| {
            wrap_unary(args, map::map_count)
        });
        registry.register("Map.ofList", |_vm, args| {
            wrap_unary(args, map::map_of_list)
        });
        registry.register("Map.toList", |_vm, args| {
            wrap_unary(args, map::map_to_list)
        });

        // Option functions
        registry.register("Option.isSome", |_vm, args| {
            wrap_unary(args, option::option_is_some)
        });
        registry.register("Option.isNone", |_vm, args| {
            wrap_unary(args, option::option_is_none)
        });
        registry.register("Option.defaultValue", |_vm, args| {
            wrap_binary(args, option::option_default_value)
        });
        registry.register("Option.defaultWith", option::option_default_with);
        registry.register("Option.map", option::option_map);
        registry.register("Option.bind", option::option_bind);
        registry.register("Option.iter", option::option_iter);
        registry.register("Option.map2", option::option_map2);
        registry.register("Option.orElse", |_vm, args| {
            wrap_binary(args, option::option_or_else)
        });

        // Option constructors - Some and None
        registry.register("Some", |_vm, args| {
            if args.len() != 1 {
                return Err(VmError::Runtime(format!(
                    "Some expects 1 argument, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![args[0].clone()],
            })
        });
        registry.register("None", |_vm, args| {
            if !args.is_empty() {
                return Err(VmError::Runtime(format!(
                    "None expects 0 arguments, got {}",
                    args.len()
                )));
            }
            Ok(Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            })
        });

        // Json functions (if json feature is enabled)
        #[cfg(feature = "json")]
        {
            registry.register("Json.parse", |_vm, args| {
                wrap_unary(args, json::json_parse)
            });
            registry.register("Json.stringify", |_vm, args| {
                wrap_unary(args, json::json_stringify)
            });
            registry.register("Json.stringifyPretty", |_vm, args| {
                wrap_unary(args, json::json_stringify_pretty)
            });
        }

        // Net.Osc functions (if osc feature is enabled)
        #[cfg(feature = "osc")]
        {
            registry.register("Osc.client", net::osc::osc_client);
            registry.register("Osc.send", net::osc::osc_send);
        }
    }

    // 2. Populate Globals with Module Records

    // Helper to create NativeFn value
    let native = |name: &str, arity: u8| Value::NativeFn {
        name: name.to_string(),
        arity,
        args: vec![],
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
    vm.globals.insert(
        "List".to_string(),
        Value::Record(Rc::new(RefCell::new(list_fields))),
    );

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
    vm.globals.insert(
        "String".to_string(),
        Value::Record(Rc::new(RefCell::new(string_fields))),
    );

    // Map Module
    let mut map_fields = HashMap::new();
    map_fields.insert("empty".to_string(), native("Map.empty", 1));
    map_fields.insert("add".to_string(), native("Map.add", 3));
    map_fields.insert("remove".to_string(), native("Map.remove", 2));
    map_fields.insert("find".to_string(), native("Map.find", 2));
    map_fields.insert("tryFind".to_string(), native("Map.tryFind", 2));
    map_fields.insert("containsKey".to_string(), native("Map.containsKey", 2));
    map_fields.insert("isEmpty".to_string(), native("Map.isEmpty", 1));
    map_fields.insert("count".to_string(), native("Map.count", 1));
    map_fields.insert("ofList".to_string(), native("Map.ofList", 1));
    map_fields.insert("toList".to_string(), native("Map.toList", 1));
    vm.globals.insert(
        "Map".to_string(),
        Value::Record(Rc::new(RefCell::new(map_fields))),
    );

    // Option Module
    let mut option_fields = HashMap::new();
    option_fields.insert("isSome".to_string(), native("Option.isSome", 1));
    option_fields.insert("isNone".to_string(), native("Option.isNone", 1));
    option_fields.insert("defaultValue".to_string(), native("Option.defaultValue", 2));
    option_fields.insert("defaultWith".to_string(), native("Option.defaultWith", 2));
    option_fields.insert("map".to_string(), native("Option.map", 2));
    option_fields.insert("bind".to_string(), native("Option.bind", 2));
    option_fields.insert("iter".to_string(), native("Option.iter", 2));
    option_fields.insert("map2".to_string(), native("Option.map2", 3));
    option_fields.insert("orElse".to_string(), native("Option.orElse", 2));
    vm.globals.insert(
        "Option".to_string(),
        Value::Record(Rc::new(RefCell::new(option_fields))),
    );

    // Register Option constructors as globals
    vm.globals.insert("Some".to_string(), native("Some", 1));
    vm.globals.insert("None".to_string(), native("None", 0));

    // Json Module (if json feature is enabled)
    #[cfg(feature = "json")]
    {
        let mut json_fields = HashMap::new();
        json_fields.insert("parse".to_string(), native("Json.parse", 1));
        json_fields.insert("stringify".to_string(), native("Json.stringify", 1));
        json_fields.insert(
            "stringifyPretty".to_string(),
            native("Json.stringifyPretty", 1),
        );
        vm.globals.insert(
            "Json".to_string(),
            Value::Record(Rc::new(RefCell::new(json_fields))),
        );
    }

    // Osc Module (if osc feature is enabled)
    #[cfg(feature = "osc")]
    {
        let mut osc_fields = HashMap::new();
        osc_fields.insert("client".to_string(), native("Osc.client", 2));
        osc_fields.insert("send".to_string(), native("Osc.send", 3));
        vm.globals.insert(
            "Osc".to_string(),
            Value::Record(Rc::new(RefCell::new(osc_fields))),
        );
    }
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

fn wrap_ternary<F>(args: &[Value], f: F) -> Result<Value, VmError>
where
    F: Fn(&Value, &Value, &Value) -> Result<Value, VmError>,
{
    if args.len() != 3 {
        return Err(VmError::Runtime(format!(
            "Expected 3 arguments, got {}",
            args.len()
        )));
    }
    f(&args[0], &args[1], &args[2])
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

    #[test]
    fn test_register_option_functions() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);

        // Check all Option functions are registered
        assert!(vm.host_registry.borrow().has_function("Option.isSome"));
        assert!(vm.host_registry.borrow().has_function("Option.isNone"));
        assert!(vm.host_registry.borrow().has_function("Option.defaultValue"));
        assert!(vm.host_registry.borrow().has_function("Option.defaultWith"));
        assert!(vm.host_registry.borrow().has_function("Option.map"));
        assert!(vm.host_registry.borrow().has_function("Option.bind"));
        assert!(vm.host_registry.borrow().has_function("Option.iter"));
        assert!(vm.host_registry.borrow().has_function("Option.map2"));
        assert!(vm.host_registry.borrow().has_function("Option.orElse"));

        // Check constructors
        assert!(vm.host_registry.borrow().has_function("Some"));
        assert!(vm.host_registry.borrow().has_function("None"));
    }

    #[test]
    fn test_register_option_globals() {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);

        // Check Option module global
        assert!(vm.globals.contains_key("Option"));
        if let Some(Value::Record(r)) = vm.globals.get("Option") {
            let borrowed = r.borrow();
            assert!(borrowed.contains_key("isSome"));
            assert!(borrowed.contains_key("isNone"));
            assert!(borrowed.contains_key("defaultValue"));
            assert!(borrowed.contains_key("defaultWith"));
            assert!(borrowed.contains_key("map"));
            assert!(borrowed.contains_key("bind"));
            assert!(borrowed.contains_key("iter"));
            assert!(borrowed.contains_key("map2"));
            assert!(borrowed.contains_key("orElse"));
        } else {
            panic!("Option global is not a record");
        }

        // Check constructor globals
        assert!(vm.globals.contains_key("Some"));
        assert!(vm.globals.contains_key("None"));
    }
}
