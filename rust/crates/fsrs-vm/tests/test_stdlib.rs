// FSRS Standard Library Integration Tests
// Tests stdlib functions through the VM

use fsrs_vm::stdlib::list::*;
use fsrs_vm::stdlib::option::*;
use fsrs_vm::stdlib::string::*;
use fsrs_vm::stdlib::StdlibRegistry;
use fsrs_vm::Value;

// ========== List Tests ==========

#[test]
fn test_list_length_integration() {
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
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
fn test_registry_lookup_integration() {
    let registry = StdlibRegistry::new();

    // Test that all major functions are registered
    assert!(registry.lookup("List.length").is_some());
    assert!(registry.lookup("List.head").is_some());
    assert!(registry.lookup("List.tail").is_some());
    assert!(registry.lookup("List.reverse").is_some());
    assert!(registry.lookup("List.append").is_some());
    assert!(registry.lookup("List.concat").is_some());

    assert!(registry.lookup("String.length").is_some());
    assert!(registry.lookup("String.trim").is_some());
    assert!(registry.lookup("String.toLower").is_some());
    assert!(registry.lookup("String.toUpper").is_some());
    assert!(registry.lookup("String.split").is_some());
    assert!(registry.lookup("String.concat").is_some());

    assert!(registry.lookup("Option.isSome").is_some());
    assert!(registry.lookup("Option.isNone").is_some());
    assert!(registry.lookup("Option.defaultValue").is_some());
}

#[test]
fn test_registry_function_call_integration() {
    let registry = StdlibRegistry::new();

    // Test calling a function through the registry
    let list_length_fn = registry.lookup("List.length").unwrap();
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let result = list_length_fn(&[list]).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_registry_all_functions_integration() {
    let registry = StdlibRegistry::new();
    let names = registry.function_names();

    // Verify we have at least the expected number of functions
    assert!(names.len() >= 15);

    // Verify each function can be looked up
    for name in names {
        assert!(registry.lookup(&name).is_some());
    }
}

// ========== Complex Integration Tests ==========

#[test]
fn test_list_string_combo_integration() {
    // Split a string, reverse the words, then concat
    let delim = Value::Str(" ".to_string());
    let s = Value::Str("hello world foo".to_string());
    let words = string_split(&delim, &s).unwrap();
    let reversed = list_reverse(&words).unwrap();
    let _joined = string_concat(&reversed).unwrap();

    // Should be "fooworldhello" (reversed order, no spaces)
    let expected_words = vec![
        Value::Str("foo".to_string()),
        Value::Str("world".to_string()),
        Value::Str("hello".to_string()),
    ];
    let expected_list = Value::vec_to_cons(expected_words);
    assert_eq!(reversed, expected_list);
}

#[test]
fn test_nested_list_operations_integration() {
    // Create multiple lists, concat them, then reverse
    let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
    let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
    let lists = Value::vec_to_cons(vec![list1, list2]);

    let concatenated = list_concat(&lists).unwrap();
    let reversed = list_reverse(&concatenated).unwrap();

    let expected = Value::vec_to_cons(vec![
        Value::Int(4),
        Value::Int(3),
        Value::Int(2),
        Value::Int(1),
    ]);
    assert_eq!(reversed, expected);
}

#[test]
fn test_option_with_string_integration() {
    let some_str = Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![Value::Str("hello".to_string())],
    };

    let default = Value::Str("default".to_string());
    let result = option_default_value(&default, &some_str).unwrap();

    // Should extract "hello" and we can use string operations on it
    let upper = string_to_upper(&result).unwrap();
    assert_eq!(upper, Value::Str("HELLO".to_string()));
}
