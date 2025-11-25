// Comprehensive tests for structural typing of anonymous records (Issue #114)

use fusabi::run_source;
use fusabi_vm::Value;

// ============================================================================
// STRUCTURAL TYPING TESTS
// ============================================================================

#[test]
fn test_structural_typing_same_fields_same_order() {
    let source = r#"
        let process r = r.x + r.y in
        let a = {| x = 5; y = 10 |} in
        let b = {| x = 3; y = 7 |} in
        let v1 = process a in
        let v2 = process b in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(25));
}

#[test]
fn test_structural_typing_same_fields_different_order() {
    let source = r#"
        let process r = r.x + r.y in
        let a = {| x = 5; y = 10 |} in
        let b = {| y = 7; x = 3 |} in
        let v1 = process a in
        let v2 = process b in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(25));
}

#[test]
fn test_structural_typing_multiple_calls() {
    let source = r#"
        let f r = r.width * r.height in
        let rect1 = {| width = 10; height = 20 |} in
        let rect2 = {| width = 5; height = 8 |} in
        let rect3 = {| height = 3; width = 4 |} in
        let a1 = f rect1 in
        let a2 = f rect2 in
        let a3 = f rect3 in
        a1 + a2 + a3
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(252)); // 200 + 40 + 12
}

#[test]
fn test_structural_typing_nested_records() {
    let source = r#"
        let getPort config = config.server.port in
        let cfg1 = {| server = {| host = "localhost"; port = 8080 |} |} in
        let cfg2 = {| server = {| port = 3000; host = "0.0.0.0" |} |} in
        let p1 = getPort cfg1 in
        let p2 = getPort cfg2 in
        p1 + p2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(11080));
}

#[test]
fn test_structural_typing_polymorphic_field_access() {
    let source = r#"
        let getName obj = obj.name in
        let person = {| name = "Alice"; age = 30 |} in
        let product = {| name = "Widget"; price = 19 |} in
        let company = {| price = 100; name = "Acme" |} in
        getName person
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Alice".to_string()));
}

#[test]
fn test_structural_typing_record_update_compatibility() {
    let source = r#"
        let incrementAge person = { person with age = person.age + 1 } in
        let alice = {| name = "Alice"; age = 30 |} in
        let bob = {| age = 25; name = "Bob" |} in
        let alice2 = incrementAge alice in
        let bob2 = incrementAge bob in
        alice2.age + bob2.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(57)); // 31 + 26
}

#[test]
fn test_structural_typing_three_fields() {
    let source = r#"
        let sum3 r = r.a + r.b + r.c in
        let x = {| a = 1; b = 2; c = 3 |} in
        let y = {| c = 30; a = 10; b = 20 |} in
        let s1 = sum3 x in
        let s2 = sum3 y in
        s1 + s2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(66)); // 6 + 60
}

#[test]
fn test_structural_typing_boolean_fields() {
    let source = r#"
        let bothActive r = r.active && r.verified in
        let user1 = {| active = true; verified = true |} in
        let user2 = {| verified = false; active = true |} in
        let check1 = bothActive user1 in
        let check2 = bothActive user2 in
        check1 && check2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(false)); // true && false
}

#[test]
fn test_structural_typing_mixed_types() {
    let source = r#"
        let describe r = if r.active then r.count else 0 in
        let obj1 = {| active = true; count = 10 |} in
        let obj2 = {| count = 20; active = false |} in
        let v1 = describe obj1 in
        let v2 = describe obj2 in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(10)); // 10 + 0
}

#[test]
fn test_structural_typing_single_field() {
    let source = r#"
        let getValue r = r.value in
        let a = {| value = 42 |} in
        let b = {| value = 99 |} in
        let v1 = getValue a in
        let v2 = getValue b in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(141));
}

#[test]
fn test_structural_typing_deeply_nested() {
    let source = r#"
        let getDebugPort r = r.config.app.settings.debug.port in
        let sys1 = {|
            config = {|
                app = {|
                    settings = {|
                        debug = {| port = 9000; enabled = true |}
                    |}
                |}
            |}
        |} in
        let sys2 = {|
            config = {|
                app = {|
                    settings = {|
                        debug = {| enabled = false; port = 5000 |}
                    |}
                |}
            |}
        |} in
        let p1 = getDebugPort sys1 in
        let p2 = getDebugPort sys2 in
        p1 + p2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(14000));
}

// ============================================================================
// FIELD ORDER IRRELEVANCE TESTS
// ============================================================================

#[test]
fn test_field_order_two_fields() {
    let source = r#"
        let a = {| x = 10; y = 20 |} in
        let b = {| y = 20; x = 10 |} in
        a.x = b.x && a.y = b.y
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_field_order_four_fields() {
    let source = r#"
        let process r = r.a + r.b + r.c + r.d in
        let x = {| a = 1; b = 2; c = 3; d = 4 |} in
        let y = {| d = 4; c = 3; b = 2; a = 1 |} in
        let z = {| b = 2; d = 4; a = 1; c = 3 |} in
        let s1 = process x in
        let s2 = process y in
        let s3 = process z in
        s1 = s2 && s2 = s3
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_field_order_nested_different_orders() {
    let source = r#"
        let getValue r = r.outer.inner.value in
        let a = {|
            outer = {|
                inner = {| value = 42; label = "test" |}
            |}
        |} in
        let b = {|
            outer = {|
                inner = {| label = "test"; value = 42 |}
            |}
        |} in
        let v1 = getValue a in
        let v2 = getValue b in
        v1 = v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// TYPE COMPATIBILITY TESTS
// ============================================================================

#[test]
fn test_compatible_records_same_structure() {
    let source = r#"
        let f r = r.x in
        let a = {| x = 5; y = 10 |} in
        let b = {| x = 3; y = 7 |} in
        let c = {| y = 1; x = 2 |} in
        let v1 = f a in
        let v2 = f b in
        let v3 = f c in
        v1 + v2 + v3
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(10)); // 5 + 3 + 2
}

#[test]
fn test_compatible_records_subset_access() {
    let source = r#"
        let getX r = r.x in
        let getY r = r.y in
        let point = {| x = 10; y = 20; z = 30 |} in
        let x_val = getX point in
        let y_val = getY point in
        x_val + y_val
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
fn test_compatible_records_in_sequence() {
    let source = r#"
        let double r = r.value * 2 in
        let r1 = {| value = 5 |} in
        let r2 = {| value = 10 |} in
        let r3 = {| value = 15 |} in
        let d1 = double r1 in
        let d2 = double r2 in
        let d3 = double r3 in
        d1 + d2 + d3
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(60)); // 10 + 20 + 30
}

// ============================================================================
// NESTED AND COMPLEX STRUCTURAL TYPING
// ============================================================================

#[test]
fn test_structural_typing_nested_update() {
    let source = r#"
        let cfg = {| server = {| host = "localhost"; port = 8080 |} |} in
        let oldServer = cfg.server in
        let newServer = { oldServer with port = 9000 } in
        let updated = { cfg with server = newServer } in
        updated.server.port
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(9000));
}

#[test]
fn test_structural_typing_with_arrays() {
    let source = r#"
        let getFirst r = r.items.[0] in
        let data1 = {| items = [|10; 20; 30|] |} in
        let data2 = {| items = [|5; 15; 25|] |} in
        let v1 = getFirst data1 in
        let v2 = getFirst data2 in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_structural_typing_complex_computation() {
    let source = r#"
        let area rect = rect.width * rect.height in
        let perimeter rect = 2 * (rect.width + rect.height) in
        let r1 = {| width = 10; height = 5 |} in
        let r2 = {| height = 8; width = 6 |} in
        let a1 = area r1 in
        let a2 = area r2 in
        let p1 = perimeter r1 in
        let p2 = perimeter r2 in
        let total_area = a1 + a2 in
        let total_perimeter = p1 + p2 in
        total_area + total_perimeter
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(156)); // (50 + 48) + (30 + 28)
}

#[test]
fn test_structural_typing_function_composition() {
    let source = r#"
        let doubleX r = r.x * 2 in
        
        
        let p1 = {| x = 5; y = 10 |} in
        let p2 = {| y = 20; x = 7 |} in
        let v1 = doubleX p1 in
        let v2 = doubleX p2 in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(24)); // (5*2) + (7*2) = 24
}

#[test]
fn test_structural_typing_conditional_access() {
    let source = r#"
        let getValue r = if r.enabled then r.value else 0 in
        let config1 = {| enabled = true; value = 100 |} in
        let config2 = {| value = 200; enabled = false |} in
        let v1 = getValue config1 in
        let v2 = getValue config2 in
        v1 + v2
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(100)); // 100 + 0
}
