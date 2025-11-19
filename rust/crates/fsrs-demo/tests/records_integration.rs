// Integration tests for Records Layer 4: Compiler Integration
// Tests compilation and execution of record operations

use fsrs_demo::run_source;
use fsrs_vm::Value;

// ============================================================================
// BASIC RECORD LITERAL TESTS
// ============================================================================

#[test]
fn test_empty_record() {
    let source = r#"{}"#;
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Record(_)));
    if let Value::Record(rec) = result {
        assert_eq!(rec.borrow().len(), 0);
    }
}

#[test]
fn test_single_field_record() {
    let source = r#"{ name = "Alice" }"#;
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Record(_)));
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert_eq!(borrowed.len(), 1);
        assert_eq!(
            *borrowed.get("name").unwrap(),
            Value::Str("Alice".to_string())
        );
    }
}

#[test]
fn test_multi_field_record() {
    let source = r#"{ name = "Bob"; age = 30; active = true }"#;
    let result = run_source(source).unwrap();
    assert!(matches!(result, Value::Record(_)));
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(
            *borrowed.get("name").unwrap(),
            Value::Str("Bob".to_string())
        );
        assert_eq!(*borrowed.get("age").unwrap(), Value::Int(30));
        assert_eq!(*borrowed.get("active").unwrap(), Value::Bool(true));
    }
}

#[test]
fn test_record_with_computed_fields() {
    let source = r#"{ x = 5 + 3; y = 10 * 2; sum = (5 + 3) + (10 * 2) }"#;
    let result = run_source(source).unwrap();
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert_eq!(*borrowed.get("x").unwrap(), Value::Int(8));
        assert_eq!(*borrowed.get("y").unwrap(), Value::Int(20));
        assert_eq!(*borrowed.get("sum").unwrap(), Value::Int(28));
    }
}

// ============================================================================
// FIELD ACCESS TESTS
// ============================================================================

#[test]
fn test_simple_field_access() {
    let source = r#"
        let person = { name = "Charlie"; age = 25 } in
        person.name
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Charlie".to_string()));
}

#[test]
fn test_multiple_field_accesses() {
    let source = r#"
        let person = { name = "Diana"; age = 35; city = "NYC" } in
        person.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(35));
}

#[test]
fn test_chained_field_access() {
    let source = r#"
        let config = { server = { host = "localhost"; port = 8080 } } in
        config.server.host
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("localhost".to_string()));
}

// ============================================================================
// RECORD UPDATE TESTS
// ============================================================================

#[ignore]
#[test]
fn test_single_field_update() {
    let source = r#"
        let person = { name = "Eve"; age = 28 } in
        let updated = { person with age = 29 } in
        updated.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(29));
}

#[ignore]
#[test]
fn test_multi_field_update() {
    let source = r#"
        let person = { name = "Frank"; age = 40; city = "LA" } in
        let updated = { person with age = 41; city = "SF" } in
        updated.city
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("SF".to_string()));
}

#[ignore]
#[test]
fn test_update_preserves_original() {
    let source = r#"
        let person = { name = "Grace"; age = 30 } in
        let updated = { person with age = 31 } in
        person.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

// ============================================================================
// NESTED RECORD TESTS
// ============================================================================

#[test]
fn test_nested_record_literal() {
    let source = r#"
        {
            user = { name = "Henry"; id = 101 };
            metadata = { created = "2024-01-01"; version = 1 }
        }
    "#;
    let result = run_source(source).unwrap();
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert!(matches!(borrowed.get("user").unwrap(), Value::Record(_)));
        assert!(matches!(
            borrowed.get("metadata").unwrap(),
            Value::Record(_)
        ));
    }
}

#[test]
fn test_nested_record_access() {
    let source = r#"
        let data = {
            user = { name = "Iris"; profile = { age = 32; bio = "Developer" } }
        } in
        data.user.profile.bio
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Developer".to_string()));
}

#[ignore]
#[test]
fn test_nested_record_update() {
    let source = r#"
        let data = { user = { name = "Jack"; age = 25 } } in
        let updated = { data with user = { name = "Jack"; age = 26 } } in
        updated.user.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(26));
}

// ============================================================================
// RECORDS WITH FUNCTIONS
// ============================================================================

#[ignore]
#[test]
fn test_function_returning_record() {
    let source = r#"
        let makePerson name age = { name = name; age = age } in
        makePerson "Kelly" 27
    "#;
    let result = run_source(source).unwrap();
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert_eq!(
            *borrowed.get("name").unwrap(),
            Value::Str("Kelly".to_string())
        );
        assert_eq!(*borrowed.get("age").unwrap(), Value::Int(27));
    }
}

#[ignore]
#[test]
fn test_function_taking_record() {
    let source = r#"
        let getAge person = person.age in
        let person = { name = "Larry"; age = 45 } in
        getAge person
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(45));
}

#[ignore]
#[test]
fn test_function_updating_record() {
    let source = r#"
        let incrementAge person = { person with age = person.age + 1 } in
        let person = { name = "Mary"; age = 29 } in
        let older = incrementAge person in
        older.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

// ============================================================================
// RECORDS IN LET BINDINGS
// ============================================================================

#[test]
fn test_record_in_let_binding() {
    let source = r#"
        let person = { name = "Nancy"; age = 33 } in
        let name = person.name in
        name
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Nancy".to_string()));
}

#[test]
fn test_destructured_let_binding() {
    let source = r#"
        let person = { x = 10; y = 20 } in
        let sum = person.x + person.y in
        sum
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

// ============================================================================
// RECORDS IN DATA STRUCTURES
// ============================================================================

#[ignore]
#[test]
fn test_record_in_list() {
    let source = r#"
        let people = [
            { name = "Oscar"; age = 28 };
            { name = "Pam"; age = 32 }
        ] in
        let first = List.head people in
        first.name
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Oscar".to_string()));
}

#[ignore]
#[test]
fn test_list_of_records_map() {
    let source = r#"
        let people = [
            { name = "Quinn"; age = 25 };
            { name = "Rita"; age = 30 }
        ] in
        let ages = List.map (fun p -> p.age) people in
        List.head ages
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(25));
}

// ============================================================================
// RECORDS IN MATCH EXPRESSIONS
// ============================================================================

#[test]
fn test_record_in_match_value() {
    let source = r#"
        let person = { name = "Sam"; age = 40 } in
        match person.age with
        | 40 -> "forty"
        | _ -> "other"
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("forty".to_string()));
}

#[test]
fn test_match_on_record_field() {
    let source = r#"
        let status = { code = 200; message = "OK" } in
        match status.code with
        | 200 -> true
        | _ -> false
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// COMPLEX SCENARIOS
// ============================================================================

#[test]
fn test_record_with_array_field() {
    let source = r#"
        let data = { values = [|1; 2; 3|]; count = 3 } in
        data.count
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_record_with_list_field() {
    let source = r#"
        let data = { items = [1; 2; 3]; total = 3 } in
        data.total
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[ignore]
#[test]
fn test_multiple_record_updates() {
    let source = r#"
        let person = { name = "Tom"; age = 20; city = "NYC" } in
        let person2 = { person with age = 21 } in
        let person3 = { person2 with city = "LA" } in
        person3.city
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("LA".to_string()));
}

#[test]
fn test_record_equality_fields() {
    let source = r#"
        let p1 = { x = 5; y = 10 } in
        let p2 = { x = 5; y = 10 } in
        p1.x = p2.x && p1.y = p2.y
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_record_with_boolean_fields() {
    let source = r#"
        let flags = { active = true; verified = false; admin = true } in
        flags.active && flags.admin
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_record_field_arithmetic() {
    let source = r#"
        let rect = { width = 10; height = 20 } in
        rect.width * rect.height
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(200));
}

#[ignore]
#[test]
fn test_record_update_with_computation() {
    let source = r#"
        let counter = { count = 5 } in
        let updated = { counter with count = counter.count * 2 } in
        updated.count
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
fn test_deeply_nested_records() {
    let source = r#"
        let config = {
            app = {
                name = "MyApp";
                settings = {
                    debug = true;
                    port = 3000
                }
            }
        } in
        config.app.settings.port
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3000));
}

// ============================================================================
// EDGE CASES AND ERROR CONDITIONS
// ============================================================================

#[test]
fn test_record_with_string_field_names() {
    let source = r#"
        { firstName = "Uma"; lastName = "Volt" }
    "#;
    let result = run_source(source).unwrap();
    if let Value::Record(rec) = result {
        let borrowed = rec.borrow();
        assert_eq!(borrowed.len(), 2);
        assert!(borrowed.contains_key("firstName"));
        assert!(borrowed.contains_key("lastName"));
    }
}

#[test]
fn test_record_field_shadowing() {
    let source = r#"
        let name = "Wendy" in
        let person = { name = name; age = 24 } in
        person.name
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Str("Wendy".to_string()));
}
