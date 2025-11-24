// Integration tests for array features in Fusabi
// Tests comprehensive array functionality including literals, indexing, updates, and operations

use fusabi::run_source;
use fusabi_vm::Value;

// ============================================================================
// Basic Operations (6 tests)
// ============================================================================

#[test]
fn test_empty_array_literal() {
    let source = "[||]";
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(arr) = result {
        assert_eq!(arr.borrow().len(), 0);
    }
}

#[test]
fn test_single_element_array() {
    let source = "[|42|]";
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 1);
        assert_eq!(borrowed[0], Value::Int(42));
    }
}

#[test]
fn test_multiple_element_array() {
    let source = "[|1; 2; 3; 4; 5|]";
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 5);
        assert_eq!(borrowed[0], Value::Int(1));
        assert_eq!(borrowed[4], Value::Int(5));
    }
}

#[test]
fn test_array_indexing() {
    let source = r#"
        let arr = [|10; 20; 30; 40; 50|] in
        arr.[2]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_array_equality() {
    let source = r#"
        let arr1 = [|1; 2; 3|] in
        let arr2 = [|1; 2; 3|] in
        arr1 = arr2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_array_with_different_types() {
    let source = r#"
        [|1; true; "hello"|]
    "#;
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Value::Int(1));
        assert_eq!(borrowed[1], Value::Bool(true));
        assert!(matches!(borrowed[2], Value::Str(_)));
    }
}

// ============================================================================
// Array Updates (6 tests)
// ============================================================================

#[test]
fn test_simple_update() {
    let source = r#"
        let arr = [|1; 2; 3|] in
        arr.[1] <- 99
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Int(1));
        assert_eq!(borrowed[1], Value::Int(99));
        assert_eq!(borrowed[2], Value::Int(3));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_multiple_updates() {
    let source = r#"
        let arr = [|1; 2; 3; 4; 5|] in
        let arr2 = arr.[0] <- 10 in
        let arr3 = arr2.[2] <- 30 in
        arr3
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Int(10));
        assert_eq!(borrowed[2], Value::Int(30));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_chained_updates() {
    let source = r#"
        let arr = [|1; 2; 3; 4; 5|] in
        ((arr.[0] <- 10).[1] <- 20).[2] <- 30
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Int(10));
        assert_eq!(borrowed[1], Value::Int(20));
        assert_eq!(borrowed[2], Value::Int(30));
        assert_eq!(borrowed[3], Value::Int(4));
        assert_eq!(borrowed[4], Value::Int(5));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_update_preserves_original() {
    let source = r#"
        let arr = [|1; 2; 3|] in
        let arr2 = arr.[1] <- 99 in
        arr.[1]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(2)); // Original unchanged
}

#[test]
fn test_update_with_expressions() {
    let source = r#"
        let base = [|10; 20; 30|] in
        let value = 100 in
        base.[1] <- value
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[1], Value::Int(100));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_update_out_of_bounds() {
    let source = r#"
        let arr = [|1; 2; 3|] in
        arr.[5] <- 99
    "#;
    let result = run_source(source);
    assert!(result.is_err()); // Should error on out of bounds
}

// ============================================================================
// Nested Arrays (5 tests)
// ============================================================================

#[test]
fn test_array_of_arrays() {
    let source = r#"
        [|[|1; 2|]; [|3; 4|]; [|5; 6|]|]
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(outer) = result {
        let borrowed = outer.borrow();
        assert_eq!(borrowed.len(), 3);

        if let Value::Array(first_row) = &borrowed[0] {
            let first_borrowed = first_row.borrow();
            assert_eq!(first_borrowed[0], Value::Int(1));
            assert_eq!(first_borrowed[1], Value::Int(2));
        } else {
            panic!("Expected nested array");
        }
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_nested_indexing() {
    let source = r#"
        let matrix = [|[|1; 2|]; [|3; 4|]; [|5; 6|]|] in
        matrix.[1].[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_update_nested_array() {
    let source = r#"
        let matrix = [|[|1; 2|]; [|3; 4|]|] in
        matrix.[0] <- [|99; 88|]
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(outer) = result {
        let borrowed = outer.borrow();
        if let Value::Array(first_row) = &borrowed[0] {
            let first_borrowed = first_row.borrow();
            assert_eq!(first_borrowed[0], Value::Int(99));
            assert_eq!(first_borrowed[1], Value::Int(88));
        } else {
            panic!("Expected nested array");
        }
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_mixed_nesting_depth() {
    let source = r#"
        let mixed = [|[|1; 2; 3|]; [|4; 5|]; [|6|]|] in
        mixed.[2].[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(6));
}

#[test]
fn test_empty_nested_arrays() {
    let source = r#"
        [|[||]; [||]; [||]|]
    "#;
    let result = run_source(source).unwrap();
    if let Value::Array(outer) = result {
        let borrowed = outer.borrow();
        assert_eq!(borrowed.len(), 3);

        for elem in borrowed.iter() {
            if let Value::Array(inner) = elem {
                assert_eq!(inner.borrow().len(), 0);
            } else {
                panic!("Expected nested array");
            }
        }
    } else {
        panic!("Expected array");
    }
}

// ============================================================================
// Array.length (4 tests)
// ============================================================================

#[test]
fn test_length_empty_array() {
    let source = r#"
        Array.length [||]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(0));
}

#[test]
fn test_length_non_empty_array() {
    let source = r#"
        let arr = [|1; 2; 3; 4; 5|] in
        Array.length arr
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_length_after_updates() {
    let source = r#"
        let arr = [|10; 20; 30|] in
        let updated = arr.[1] <- 99 in
        Array.length updated
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_length_of_nested_arrays() {
    let source = r#"
        let matrix = [|[|1; 2; 3|]; [|4; 5|]; [|6|]|] in
        let outer_len = Array.length matrix in
        let first_len = Array.length matrix.[0] in
        let second_len = Array.length matrix.[1] in
        first_len + second_len
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(5)); // 3 + 2
}

// ============================================================================
// Bindings (4 tests)
// ============================================================================

#[test]
fn test_arrays_in_let_bindings() {
    let source = r#"
        let arr1 = [|1; 2; 3|] in
        let arr2 = [|4; 5; 6|] in
        arr1.[0] + arr2.[1]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(6)); // 1 + 5
}

#[test]
fn test_multiple_array_bindings() {
    let source = r#"
        let a = [|10|] in
        let b = [|20|] in
        let c = [|30|] in
        a.[0] + b.[0] + c.[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(60));
}

#[test]
fn test_arrays_with_variables_as_elements() {
    let source = r#"
        let x = 10 in
        let y = 20 in
        let z = 30 in
        let arr = [|x; y; z|] in
        arr.[1]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_array_updates_in_bindings() {
    let source = r#"
        let original = [|1; 2; 3|] in
        let updated1 = original.[0] <- 10 in
        let updated2 = updated1.[1] <- 20 in
        let updated3 = updated2.[2] <- 30 in
        updated3.[2]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

// ============================================================================
// Conditionals (3 tests)
// ============================================================================

#[test]
fn test_conditional_array_selection() {
    let source = r#"
        let arr1 = [|1; 2; 3|] in
        let arr2 = [|4; 5; 6|] in
        let result = if true then arr1 else arr2 in
        result.[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_arrays_in_if_then_else() {
    let source = r#"
        let condition = false in
        let result = if condition then [|10|] else [|20|] in
        result.[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(20));
}

#[test]
fn test_nested_conditionals_with_arrays() {
    let source = r#"
        let x = 5 in
        let arr = if x > 10 then [|100|] else if x > 0 then [|50|] else [|0|] in
        arr.[0]
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(50));
}

// ============================================================================
// Edge Cases (4 tests)
// ============================================================================

#[test]
fn test_single_element_operations() {
    let source = r#"
        let arr = [|42|] in
        let updated = arr.[0] <- 100 in
        let len = Array.length updated in
        updated.[0] + len
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(101)); // 100 + 1
}

#[test]
fn test_large_array() {
    let source = r#"
        let arr = [|1; 2; 3; 4; 5; 6; 7; 8; 9; 10; 11; 12; 13; 14; 15|] in
        Array.length arr
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_index_bounds_checking() {
    let source = r#"
        let arr = [|1; 2; 3|] in
        arr.[10]
    "#;
    let result = run_source(source);
    assert!(result.is_err()); // Should error on out of bounds
}

#[test]
fn test_negative_index() {
    let source = r#"
        let arr = [|1; 2; 3|] in
        arr.[-1]
    "#;
    let result = run_source(source);
    // This should error - negative indices not supported
    assert!(result.is_err());
}
