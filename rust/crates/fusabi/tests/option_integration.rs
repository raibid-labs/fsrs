//! Integration tests for Option type support
//!
//! Tests the full pipeline from Fusabi source code through compilation to execution,
//! verifying that Option type works correctly with all its operations.

use fusabi::run_source;
use fusabi_vm::Value;

// ========== Constructor Tests ==========

#[test]
fn test_some_constructor() {
    let source = r#"
        let x = Some 42 in
        x
    "#;
    let result = run_source(source).expect("Should compile and run");
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Some variant, got {:?}", result),
    }
}

#[test]
fn test_none_constructor() {
    let source = "None";
    let result = run_source(source).expect("Should compile and run");
    match result {
        Value::Variant { variant_name, .. } => {
            assert_eq!(variant_name, "None");
        }
        _ => panic!("Expected None variant, got {:?}", result),
    }
}

#[test]
fn test_some_with_string() {
    let source = r#"Some "hello""#;
    let result = run_source(source).expect("Should compile and run");
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Str("hello".to_string()));
        }
        _ => panic!("Expected Some variant with string"),
    }
}

// ========== Pattern Matching Tests ==========

#[test]
fn test_pattern_match_some() {
    let source = r#"
        let opt = Some 42 in
        match opt with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_pattern_match_none() {
    let source = r#"
        let opt = None in
        match opt with
        | Some x -> x
        | None -> 99
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_nested_pattern_match() {
    let source = r#"
        let opt = Some (Some 42) in
        match opt with
        | Some (Some x) -> x
        | Some None -> 0
        | None -> -1
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

// ========== Option.isSome and Option.isNone Tests ==========

#[test]
fn test_is_some_true() {
    let source = r#"
        let opt = Some 42 in
        Option.isSome opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_is_some_false() {
    let source = r#"
        let opt = None in
        Option.isSome opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_is_none_true() {
    let source = r#"
        let opt = None in
        Option.isNone opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_is_none_false() {
    let source = r#"
        let opt = Some 42 in
        Option.isNone opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(false));
}

// ========== Option.defaultValue Tests ==========

#[test]
fn test_default_value_some() {
    let source = r#"
        let opt = Some 42 in
        Option.defaultValue 0 opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_default_value_none() {
    let source = r#"
        let opt = None in
        Option.defaultValue 99 opt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(99));
}

// ========== Option.map Tests ==========

#[test]
fn test_option_map_some() {
    let source = r#"
        let opt = Some 5 in
        let double = fun x -> x * 2 in
        let result = Option.map double opt in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_option_map_none() {
    let source = r#"
        let opt = None in
        let double = fun x -> x * 2 in
        let result = Option.map double opt in
        Option.isNone result
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_option_map_chain() {
    let source = r#"
        let opt = Some 3 in
        let add1 = fun x -> x + 1 in
        let double = fun x -> x * 2 in
        let result = Option.map double (Option.map add1 opt) in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(8)); // (3 + 1) * 2 = 8
}

// ========== Option.bind Tests ==========

#[test]
fn test_option_bind_some() {
    let source = r#"
        let opt = Some 5 in
        let safeDivide = fun x -> if x > 0 then Some (10 / x) else None in
        let result = Option.bind safeDivide opt in
        match result with
        | Some x -> x
        | None -> -1
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(2)); // 10 / 5 = 2
}

#[test]
fn test_option_bind_none_input() {
    let source = r#"
        let opt = None in
        let safeDivide = fun x -> Some (10 / x) in
        let result = Option.bind safeDivide opt in
        Option.isNone result
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

// ========== Option.orElse Tests ==========

#[test]
fn test_option_or_else_some() {
    let source = r#"
        let opt1 = Some 42 in
        let opt2 = Some 99 in
        let result = Option.orElse opt1 opt2 in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_option_or_else_none() {
    let source = r#"
        let opt1 = None in
        let opt2 = Some 99 in
        let result = Option.orElse opt1 opt2 in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_option_or_else_both_none() {
    let source = r#"
        let opt1 = None in
        let opt2 = None in
        let result = Option.orElse opt1 opt2 in
        Option.isNone result
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

// ========== Option.map2 Tests ==========

#[test]
fn test_option_map2_both_some() {
    let source = r#"
        let opt1 = Some 3 in
        let opt2 = Some 4 in
        let add = fun x y -> x + y in
        let result = Option.map2 add opt1 opt2 in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_option_map2_first_none() {
    let source = r#"
        let opt1 = None in
        let opt2 = Some 4 in
        let add = fun x y -> x + y in
        let result = Option.map2 add opt1 opt2 in
        Option.isNone result
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_option_map2_second_none() {
    let source = r#"
        let opt1 = Some 3 in
        let opt2 = None in
        let add = fun x y -> x + y in
        let result = Option.map2 add opt1 opt2 in
        Option.isNone result
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Bool(true));
}

// ========== Complex Integration Tests ==========

#[test]
fn test_option_list_integration() {
    let source = r#"
        let opts = [Some 1; Some 2; None; Some 3] in
        let len = List.length opts in
        len
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(4));
}

#[test]
fn test_option_in_let_binding() {
    let source = r#"
        let getValue = fun opt ->
            match opt with
            | Some x -> x
            | None -> 0
            end
        in
        let x = getValue (Some 42) in
        let y = getValue None in
        x + y
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_option_higher_order_function() {
    let source = r#"
        let tryApply = fun f opt ->
            Option.map f opt
        in
        let double = fun x -> x * 2 in
        let result = tryApply double (Some 21) in
        match result with
        | Some x -> x
        | None -> 0
        end
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_option_composition() {
    let source = r#"
        let safeDivide = fun x y ->
            if y == 0 then None else Some (x / y)
        in
        let result1 = safeDivide 10 2 in
        let result2 = safeDivide 10 0 in
        let v1 = Option.defaultValue 0 result1 in
        let v2 = Option.defaultValue 0 result2 in
        v1 + v2
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(5)); // 5 + 0 = 5
}

#[test]
fn test_option_with_records() {
    let source = r#"
        let user = { name = "Alice"; age = Some 30 } in
        let ageOpt = user.age in
        Option.defaultValue 0 ageOpt
    "#;
    let result = run_source(source).expect("Should compile and run");
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_option_without_parens() {
    let source = "Some 42";
    let result = run_source(source).expect("Should compile and run");
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Some variant"),
    }
}

#[test]
fn test_option_with_parens() {
    let source = "Some(42)";
    let result = run_source(source).expect("Should compile and run");
    match result {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            assert_eq!(variant_name, "Some");
            assert_eq!(fields[0], Value::Int(42));
        }
        _ => panic!("Expected Some variant"),
    }
}
