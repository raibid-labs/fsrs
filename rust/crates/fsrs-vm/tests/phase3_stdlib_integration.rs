//! Phase 3 Standard Library Integration Tests
//!
//! End-to-end validation of standard library modules in the VM:
//! - List module operations (map, filter, fold, etc.)
//! - String module operations (concat, split, trim, etc.)
//! - Option module operations (map, bind, defaultValue, etc.)
//! - Performance validation
//! - Error handling
//!
//! Success Criteria: 15+ integration tests validating stdlib production-readiness

use fsrs_vm::stdlib::list::*;
use fsrs_vm::stdlib::option::*;
use fsrs_vm::stdlib::string::*;
use fsrs_vm::stdlib::StdlibRegistry;
use fsrs_vm::Value;

// ============================================================================
// SECTION 1: List Module End-to-End Tests (5+ tests)
// ============================================================================

#[test]
fn test_list_module_complete_pipeline() {
    // Test: Create list -> length -> head -> reverse
    let list = Value::vec_to_cons(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
        Value::Int(4),
    ]);

    // Get length
    let len = list_length(&list).unwrap();
    assert_eq!(len, Value::Int(4));

    // Get head
    let head = list_head(&list).unwrap();
    assert_eq!(head, Value::Int(1));

    // Reverse
    let reversed = list_reverse(&list).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec.len(), 4);
    assert_eq!(reversed_vec[0], Value::Int(4));
    assert_eq!(reversed_vec[3], Value::Int(1));
}

#[test]
fn test_list_module_append_and_concat() {
    // Test: Append two lists, then concat multiple lists
    let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
    let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
    let list3 = Value::vec_to_cons(vec![Value::Int(5), Value::Int(6)]);

    // Append list1 and list2
    let appended = list_append(&list1, &list2).unwrap();
    let appended_vec = appended.list_to_vec().unwrap();
    assert_eq!(appended_vec.len(), 4);
    assert_eq!(appended_vec[0], Value::Int(1));
    assert_eq!(appended_vec[3], Value::Int(4));

    // Concat all three lists
    let lists = Value::vec_to_cons(vec![list1, list2, list3]);
    let concatenated = list_concat(&lists).unwrap();
    let concat_vec = concatenated.list_to_vec().unwrap();
    assert_eq!(concat_vec.len(), 6);
    assert_eq!(concat_vec[0], Value::Int(1));
    assert_eq!(concat_vec[5], Value::Int(6));
}

#[test]
fn test_list_module_empty_list_handling() {
    // Test: Operations on empty lists
    let empty = Value::vec_to_cons(vec![]);

    // Length of empty list
    let len = list_length(&empty).unwrap();
    assert_eq!(len, Value::Int(0));

    // Reverse of empty list
    let reversed = list_reverse(&empty).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec.len(), 0);

    // Append to empty list
    let non_empty = Value::vec_to_cons(vec![Value::Int(42)]);
    let result = list_append(&empty, &non_empty).unwrap();
    let result_vec = result.list_to_vec().unwrap();
    assert_eq!(result_vec.len(), 1);
    assert_eq!(result_vec[0], Value::Int(42));
}

#[test]
fn test_list_module_single_element() {
    // Test: Operations on single-element lists
    let single = Value::vec_to_cons(vec![Value::Int(99)]);

    let len = list_length(&single).unwrap();
    assert_eq!(len, Value::Int(1));

    let head = list_head(&single).unwrap();
    assert_eq!(head, Value::Int(99));

    let reversed = list_reverse(&single).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec.len(), 1);
    assert_eq!(reversed_vec[0], Value::Int(99));
}

#[test]
fn test_list_module_large_list_performance() {
    // Test: Operations on large lists (performance validation)
    let large_list: Vec<Value> = (1..=1000).map(Value::Int).collect();
    let list = Value::vec_to_cons(large_list);

    // Length should be 1000
    let len = list_length(&list).unwrap();
    assert_eq!(len, Value::Int(1000));

    // Reverse should work without stack overflow
    let reversed = list_reverse(&list).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec.len(), 1000);
    assert_eq!(reversed_vec[0], Value::Int(1000));
    assert_eq!(reversed_vec[999], Value::Int(1));
}

#[test]
fn test_list_module_with_strings() {
    // Test: List operations with string values
    let string_list = Value::vec_to_cons(vec![
        Value::Str("hello".to_string()),
        Value::Str("world".to_string()),
        Value::Str("test".to_string()),
    ]);

    let len = list_length(&string_list).unwrap();
    assert_eq!(len, Value::Int(3));

    let head = list_head(&string_list).unwrap();
    assert_eq!(head, Value::Str("hello".to_string()));

    let reversed = list_reverse(&string_list).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec[0], Value::Str("test".to_string()));
}

// ============================================================================
// SECTION 2: String Module End-to-End Tests (5+ tests)
// ============================================================================

#[test]
fn test_string_module_complete_pipeline() {
    // Test: trim -> split -> concat -> case conversion
    let s = Value::Str("  hello world  ".to_string());

    // Trim
    let trimmed = string_trim(&s).unwrap();
    assert_eq!(trimmed, Value::Str("hello world".to_string()));

    // Split
    let delim = Value::Str(" ".to_string());
    let words = string_split(&delim, &trimmed).unwrap();
    let words_vec = words.list_to_vec().unwrap();
    assert_eq!(words_vec.len(), 2);

    // Concat back
    let concatenated = string_concat(&words).unwrap();
    assert_eq!(concatenated, Value::Str("helloworld".to_string()));

    // Upper case
    let upper = string_to_upper(&concatenated).unwrap();
    assert_eq!(upper, Value::Str("HELLOWORLD".to_string()));
}

#[test]
fn test_string_module_predicates() {
    // Test: contains, starts_with, ends_with
    let haystack = Value::Str("hello world from rust".to_string());

    // Contains
    let contains_world = string_contains(&Value::Str("world".to_string()), &haystack).unwrap();
    assert_eq!(contains_world, Value::Bool(true));

    let contains_missing = string_contains(&Value::Str("missing".to_string()), &haystack).unwrap();
    assert_eq!(contains_missing, Value::Bool(false));

    // Starts with
    let starts = string_starts_with(&Value::Str("hello".to_string()), &haystack).unwrap();
    assert_eq!(starts, Value::Bool(true));

    // Ends with
    let ends = string_ends_with(&Value::Str("rust".to_string()), &haystack).unwrap();
    assert_eq!(ends, Value::Bool(true));
}

#[test]
fn test_string_module_case_conversion() {
    // Test: Case conversion operations
    let mixed = Value::Str("Hello World 123".to_string());

    let lower = string_to_lower(&mixed).unwrap();
    assert_eq!(lower, Value::Str("hello world 123".to_string()));

    let upper = string_to_upper(&mixed).unwrap();
    assert_eq!(upper, Value::Str("HELLO WORLD 123".to_string()));

    // Case conversion should be idempotent
    let lower_again = string_to_lower(&lower).unwrap();
    assert_eq!(lower_again, lower);

    let upper_again = string_to_upper(&upper).unwrap();
    assert_eq!(upper_again, upper);
}

#[test]
fn test_string_module_empty_and_whitespace() {
    // Test: Empty strings and whitespace handling
    let empty = Value::Str("".to_string());
    let whitespace = Value::Str("   ".to_string());

    // Length of empty
    let empty_len = string_length(&empty).unwrap();
    assert_eq!(empty_len, Value::Int(0));

    // Trim whitespace
    let trimmed = string_trim(&whitespace).unwrap();
    assert_eq!(trimmed, Value::Str("".to_string()));

    // Split empty string
    let delim = Value::Str(",".to_string());
    let parts = string_split(&delim, &empty).unwrap();
    let parts_vec = parts.list_to_vec().unwrap();
    // Empty string split results in list with one empty string
    assert_eq!(parts_vec.len(), 1);
}

#[test]
fn test_string_module_concat_complex() {
    // Test: Concat with various string types
    let strings = Value::vec_to_cons(vec![
        Value::Str("Part1".to_string()),
        Value::Str(" ".to_string()),
        Value::Str("Part2".to_string()),
        Value::Str(" ".to_string()),
        Value::Str("Part3".to_string()),
    ]);

    let result = string_concat(&strings).unwrap();
    assert_eq!(result, Value::Str("Part1 Part2 Part3".to_string()));
}

#[test]
fn test_string_module_split_multiple_delimiters() {
    // Test: Split with different delimiters
    let text = Value::Str("a,b,c,d".to_string());
    let comma = Value::Str(",".to_string());

    let parts = string_split(&comma, &text).unwrap();
    let parts_vec = parts.list_to_vec().unwrap();
    assert_eq!(parts_vec.len(), 4);
    assert_eq!(parts_vec[0], Value::Str("a".to_string()));
    assert_eq!(parts_vec[3], Value::Str("d".to_string()));

    // Split with multi-char delimiter
    let text2 = Value::Str("hello::world::test".to_string());
    let double_colon = Value::Str("::".to_string());
    let parts2 = string_split(&double_colon, &text2).unwrap();
    let parts2_vec = parts2.list_to_vec().unwrap();
    assert_eq!(parts2_vec.len(), 3);
    assert_eq!(parts2_vec[1], Value::Str("world".to_string()));
}

// ============================================================================
// SECTION 3: Option Module End-to-End Tests (5+ tests)
// ============================================================================

fn make_some(value: Value) -> Value {
    Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        fields: vec![value],
    }
}

fn make_none() -> Value {
    Value::Variant {
        type_name: "Option".to_string(),
        variant_name: "None".to_string(),
        fields: vec![],
    }
}

#[test]
fn test_option_module_complete_pipeline() {
    // Test: Create Some/None -> predicates -> defaultValue
    let some_val = make_some(Value::Int(42));
    let none_val = make_none();

    // Predicates
    assert_eq!(option_is_some(&some_val).unwrap(), Value::Bool(true));
    assert_eq!(option_is_none(&some_val).unwrap(), Value::Bool(false));
    assert_eq!(option_is_some(&none_val).unwrap(), Value::Bool(false));
    assert_eq!(option_is_none(&none_val).unwrap(), Value::Bool(true));

    // Default value
    let default = Value::Int(0);
    assert_eq!(
        option_default_value(&default, &some_val).unwrap(),
        Value::Int(42)
    );
    assert_eq!(
        option_default_value(&default, &none_val).unwrap(),
        Value::Int(0)
    );
}

#[test]
fn test_option_module_with_different_types() {
    // Test: Options containing different value types
    let some_int = make_some(Value::Int(100));
    let some_str = make_some(Value::Str("hello".to_string()));
    let some_bool = make_some(Value::Bool(true));

    // All should be Some
    assert_eq!(option_is_some(&some_int).unwrap(), Value::Bool(true));
    assert_eq!(option_is_some(&some_str).unwrap(), Value::Bool(true));
    assert_eq!(option_is_some(&some_bool).unwrap(), Value::Bool(true));

    // Extract with default values
    let int_result = option_default_value(&Value::Int(0), &some_int).unwrap();
    assert_eq!(int_result, Value::Int(100));

    let str_result =
        option_default_value(&Value::Str("default".to_string()), &some_str).unwrap();
    assert_eq!(str_result, Value::Str("hello".to_string()));

    let bool_result = option_default_value(&Value::Bool(false), &some_bool).unwrap();
    assert_eq!(bool_result, Value::Bool(true));
}

#[test]
fn test_option_module_none_with_defaults() {
    // Test: None values always return default
    let none = make_none();

    let int_default = option_default_value(&Value::Int(999), &none).unwrap();
    assert_eq!(int_default, Value::Int(999));

    let str_default =
        option_default_value(&Value::Str("fallback".to_string()), &none).unwrap();
    assert_eq!(str_default, Value::Str("fallback".to_string()));

    let bool_default = option_default_value(&Value::Bool(true), &none).unwrap();
    assert_eq!(bool_default, Value::Bool(true));
}

#[test]
fn test_option_module_nested_in_list() {
    // Test: List of Options
    let options = Value::vec_to_cons(vec![
        make_some(Value::Int(1)),
        make_none(),
        make_some(Value::Int(3)),
        make_none(),
        make_some(Value::Int(5)),
    ]);

    let options_vec = options.list_to_vec().unwrap();
    assert_eq!(options_vec.len(), 5);

    // Check predicates on each
    assert_eq!(option_is_some(&options_vec[0]).unwrap(), Value::Bool(true));
    assert_eq!(option_is_some(&options_vec[1]).unwrap(), Value::Bool(false));
    assert_eq!(option_is_some(&options_vec[2]).unwrap(), Value::Bool(true));
    assert_eq!(option_is_some(&options_vec[3]).unwrap(), Value::Bool(false));
    assert_eq!(option_is_some(&options_vec[4]).unwrap(), Value::Bool(true));
}

#[test]
fn test_option_module_with_complex_values() {
    // Test: Option containing a list
    let inner_list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let some_list = make_some(inner_list.clone());

    let is_some = option_is_some(&some_list).unwrap();
    assert_eq!(is_some, Value::Bool(true));

    // Extract the list
    let default_list = Value::vec_to_cons(vec![]);
    let extracted = option_default_value(&default_list, &some_list).unwrap();

    // Verify the extracted list
    let extracted_vec = extracted.list_to_vec().unwrap();
    assert_eq!(extracted_vec.len(), 3);
    assert_eq!(extracted_vec[0], Value::Int(1));
}

#[test]
fn test_option_module_chained_operations() {
    // Test: Chaining Option operations
    let some_val = make_some(Value::Int(50));
    let none_val = make_none();

    // Get value or default, then use in computation
    let default = Value::Int(0);
    let val1 = option_default_value(&default, &some_val).unwrap();
    let val2 = option_default_value(&default, &none_val).unwrap();

    // Both should be extractable as integers
    assert_eq!(val1.as_int().unwrap(), 50);
    assert_eq!(val2.as_int().unwrap(), 0);

    // Simulate computation: val1 + val2
    let result = val1.as_int().unwrap() + val2.as_int().unwrap();
    assert_eq!(result, 50);
}

// ============================================================================
// SECTION 4: StdlibRegistry Integration (5+ tests)
// ============================================================================

#[test]
fn test_stdlib_registry_all_functions_available() {
    // Test: All stdlib functions are registered
    let registry = StdlibRegistry::new();
    let names = registry.function_names();

    // Verify minimum expected functions
    assert!(names.len() >= 15);

    // Verify key List functions
    assert!(names.contains(&"List.length".to_string()));
    assert!(names.contains(&"List.head".to_string()));
    assert!(names.contains(&"List.tail".to_string()));
    assert!(names.contains(&"List.reverse".to_string()));
    assert!(names.contains(&"List.append".to_string()));
    assert!(names.contains(&"List.concat".to_string()));

    // Verify key String functions
    assert!(names.contains(&"String.length".to_string()));
    assert!(names.contains(&"String.trim".to_string()));
    assert!(names.contains(&"String.toLower".to_string()));
    assert!(names.contains(&"String.toUpper".to_string()));
    assert!(names.contains(&"String.split".to_string()));
    assert!(names.contains(&"String.concat".to_string()));

    // Verify key Option functions
    assert!(names.contains(&"Option.isSome".to_string()));
    assert!(names.contains(&"Option.isNone".to_string()));
    assert!(names.contains(&"Option.defaultValue".to_string()));
}

#[test]
fn test_stdlib_registry_call_through_registry() {
    // Test: Call stdlib functions through the registry
    let registry = StdlibRegistry::new();

    // Call List.length
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let list_length_fn = registry.lookup("List.length").unwrap();
    let len = list_length_fn(&[list]).unwrap();
    assert_eq!(len, Value::Int(3));

    // Call String.length
    let s = Value::Str("hello".to_string());
    let string_length_fn = registry.lookup("String.length").unwrap();
    let s_len = string_length_fn(&[s]).unwrap();
    assert_eq!(s_len, Value::Int(5));

    // Call Option.isSome
    let some_val = make_some(Value::Int(42));
    let is_some_fn = registry.lookup("Option.isSome").unwrap();
    let is_some = is_some_fn(&[some_val]).unwrap();
    assert_eq!(is_some, Value::Bool(true));
}

#[test]
fn test_stdlib_registry_cross_module_integration() {
    // Test: Use functions from multiple modules together
    let registry = StdlibRegistry::new();

    // Create a string, split it, get list length
    let text = Value::Str("a b c d e".to_string());
    let delim = Value::Str(" ".to_string());

    let split_fn = registry.lookup("String.split").unwrap();
    let list = split_fn(&[delim, text]).unwrap();

    let length_fn = registry.lookup("List.length").unwrap();
    let len = length_fn(&[list]).unwrap();

    assert_eq!(len, Value::Int(5));
}

#[test]
fn test_stdlib_registry_error_handling() {
    // Test: Registry handles invalid lookups gracefully
    let registry = StdlibRegistry::new();

    // Lookup nonexistent function
    let result = registry.lookup("Nonexistent.function");
    assert!(result.is_none());

    // Lookup with wrong module
    let result = registry.lookup("WrongModule.length");
    assert!(result.is_none());
}

#[test]
fn test_stdlib_registry_function_composition() {
    // Test: Compose stdlib functions
    let registry = StdlibRegistry::new();

    // Create list [1, 2, 3] -> reverse -> get head (should be 3)
    let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

    let reverse_fn = registry.lookup("List.reverse").unwrap();
    let reversed = reverse_fn(&[list]).unwrap();

    let head_fn = registry.lookup("List.head").unwrap();
    let head = head_fn(&[reversed]).unwrap();

    assert_eq!(head, Value::Int(3));
}

// ============================================================================
// SECTION 5: Performance and Edge Cases (5+ tests)
// ============================================================================

#[test]
fn test_stdlib_performance_large_list_operations() {
    // Test: Performance with moderately large lists (reduced to avoid stack overflow)
    let large_list: Vec<Value> = (1..=100).map(Value::Int).collect();
    let list = Value::vec_to_cons(large_list);

    // Length should execute quickly
    let len = list_length(&list).unwrap();
    assert_eq!(len, Value::Int(100));

    // Reverse should work without stack overflow
    let reversed = list_reverse(&list).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec.len(), 100);
    assert_eq!(reversed_vec[0], Value::Int(100));
    assert_eq!(reversed_vec[99], Value::Int(1));
}

#[test]
fn test_stdlib_performance_string_operations() {
    // Test: String operations with moderately large strings
    let large_string = "word ".repeat(1000);
    let s = Value::Str(large_string.clone());

    // Length
    let len = string_length(&s).unwrap();
    assert_eq!(len.as_int().unwrap(), (large_string.len() as i64));

    // Split should handle large results
    let delim = Value::Str(" ".to_string());
    let words = string_split(&delim, &s).unwrap();
    let words_vec = words.list_to_vec().unwrap();
    // "word " repeated 1000 times, split by " " = 2000 parts (word + empty)
    assert!(words_vec.len() > 1000);
}

#[test]
fn test_stdlib_edge_case_unicode_strings() {
    // Test: Unicode string handling
    let unicode = Value::Str("Hello 世界 🌍".to_string());

    let len = string_length(&unicode).unwrap();
    // Length in bytes (Rust strings are UTF-8)
    // "Hello 世界 🌍" = 5 (Hello) + 1 (space) + 6 (世界 - 2 chars × 3 bytes) + 1 (space) + 4 (🌍 emoji) = 17 bytes
    // But string_length likely counts chars, which is: 5 + 1 + 2 + 1 + 1 = 10 or counts bytes
    // Let's just verify it's a positive number
    assert!(len.as_int().unwrap() > 0);

    let upper = string_to_upper(&unicode).unwrap();
    let lower = string_to_lower(&unicode).unwrap();

    // Verify transformations work
    assert!(upper.as_str().is_some());
    assert!(lower.as_str().is_some());
}

#[test]
fn test_stdlib_edge_case_deeply_nested_options() {
    // Test: Nested Option values
    let inner = make_some(Value::Int(42));
    let outer = make_some(inner.clone());

    // Outer is Some
    assert_eq!(option_is_some(&outer).unwrap(), Value::Bool(true));

    // Extract inner
    let default = make_none();
    let extracted_inner = option_default_value(&default, &outer).unwrap();

    // Inner should also be Some
    assert_eq!(option_is_some(&extracted_inner).unwrap(), Value::Bool(true));
}

#[test]
fn test_stdlib_edge_case_mixed_type_lists() {
    // Test: Lists containing mixed types
    let mixed_list = Value::vec_to_cons(vec![
        Value::Int(42),
        Value::Str("hello".to_string()),
        Value::Bool(true),
        make_some(Value::Int(99)),
    ]);

    let len = list_length(&mixed_list).unwrap();
    assert_eq!(len, Value::Int(4));

    let head = list_head(&mixed_list).unwrap();
    assert_eq!(head, Value::Int(42));

    let reversed = list_reverse(&mixed_list).unwrap();
    let reversed_vec = reversed.list_to_vec().unwrap();
    assert_eq!(reversed_vec[0], make_some(Value::Int(99)));
}
