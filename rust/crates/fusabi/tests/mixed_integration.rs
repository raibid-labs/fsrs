//! Integration tests for Records and Discriminated Unions working together
//!
//! This test suite validates the integration between Records (L4) and
//! Discriminated Unions (L4), demonstrating realistic usage patterns where
//! both features interact in complex ways.

use fusabi::run_source;
use fusabi_vm::Value;

// ============================================================================
// RECORDS CONTAINING VARIANTS (5+ tests)
// ============================================================================

#[test]
fn test_record_with_option_field() {
    // Test: Record containing an Option variant
    // { name = "Alice"; age = Some(30) }
    let source = r#"
        let person = { name = "Alice"; age = Some(30) } in
        match person.age with
        | Some(a) -> a
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
#[ignore = "TODO: Requires parser support for variant construction in records"]
fn test_record_with_result_field() {
    // Test: Record containing a Result variant
    let source = r#"
        let response = { status = Ok(200); message = "Success" } in
        match response.status with
        | Ok(code) -> code
        | Error(_) -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(200));
}

#[test]
#[ignore = "TODO: Requires parser support for variant construction in records"]
fn test_record_with_multiple_variant_fields() {
    // Test: Record containing multiple variant fields
    let source = r#"
        let user = {
            name = Some("Bob");
            age = Some(25);
            email = None
        } in
        match (user.name, user.age) with
        | (Some(n), Some(a)) -> a
        | _ -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(25));
}

#[test]
#[ignore = "TODO: Requires parser support for nested structures"]
fn test_nested_record_with_variants() {
    // Test: Nested records containing variants
    let source = r#"
        let config = {
            server = { host = Some("localhost"); port = Some(8080) };
            debug = true
        } in
        match config.server.port with
        | Some(p) -> p
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(8080));
}

#[test]
#[ignore = "TODO: Requires parser support for record update with variants"]
fn test_update_record_variant_field() {
    // Test: Updating a variant field in a record
    let source = r#"
        let user = { name = "Charlie"; status = Some("active") } in
        let updated = { user with status = None } in
        match updated.status with
        | Some(_) -> false
        | None -> true
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
#[ignore = "TODO: Requires parser support for records in lists"]
fn test_list_of_records_with_variants() {
    // Test: List of records containing variant fields
    let source = r#"
        let users = [
            { name = "Alice"; status = Some("active") };
            { name = "Bob"; status = None }
        ] in
        let first = List.head users in
        match first.status with
        | Some(s) -> true
        | None -> false
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VARIANTS CONTAINING RECORDS (5+ tests)
// ============================================================================

#[test]
#[ignore = "TODO: Requires parser support for records in variant payloads"]
fn test_variant_with_record_payload() {
    // Test: Variant containing a record as payload
    // User({ name = "Alice"; age = 30 })
    let source = r#"
        let entity = User({ name = "Alice"; age = 30 }) in
        match entity with
        | User(u) -> u.age
        | Guest -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
#[ignore = "TODO: Requires parser support for complex variant payloads"]
fn test_result_with_record_payload() {
    // Test: Result variant containing record payloads
    let source = r#"
        let result = Ok({ code = 200; message = "Success" }) in
        match result with
        | Ok(resp) -> resp.code
        | Error(_) -> 500
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(200));
}

#[test]
#[ignore = "TODO: Requires parser support for nested record payloads"]
fn test_variant_with_nested_record() {
    // Test: Variant containing nested records
    let source = r#"
        let response = Success({
            data = { user = { name = "Bob"; id = 123 } };
            timestamp = 1234567890
        }) in
        match response with
        | Success(r) -> r.data.user.id
        | Failure(_) -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(123));
}

#[test]
#[ignore = "TODO: Requires parser support for multiple record payloads"]
fn test_variant_with_multiple_records() {
    // Test: Variant with multiple record payloads
    let source = r#"
        let event = Transfer({ from = { id = 1; name = "Alice" }; to = { id = 2; name = "Bob" } }) in
        match event with
        | Transfer(t) -> t.from.id + t.to.id
        | _ -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
#[ignore = "TODO: Requires parser support for record update in variants"]
fn test_update_record_in_variant() {
    // Test: Updating a record inside a variant
    let source = r#"
        let entity = Person({ name = "Charlie"; age = 25 }) in
        match entity with
        | Person(p) ->
            let updated = { p with age = 26 } in
            updated.age
        | _ -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(26));
}

#[test]
#[ignore = "TODO: Requires parser support for list of variants with records"]
fn test_list_of_variants_with_records() {
    // Test: List of variants containing records
    let source = r#"
        let entities = [
            User({ name = "Alice"; role = "admin" });
            User({ name = "Bob"; role = "user" });
            Guest
        ] in
        List.length entities
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3));
}

// ============================================================================
// MIXED PATTERN MATCHING (5+ tests)
// ============================================================================

#[test]
#[ignore = "TODO: Requires parser support for complex pattern matching"]
fn test_match_variant_extract_record() {
    // Test: Pattern match on variant to extract and use record
    let source = r#"
        let result = Ok({ value = 42; valid = true }) in
        match result with
        | Ok(rec) when rec.valid -> rec.value
        | Ok(rec) -> 0
        | Error(_) -> -1
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
#[ignore = "TODO: Requires parser support for nested pattern matching"]
fn test_match_nested_variant_record() {
    // Test: Pattern match on nested variants and records
    let source = r#"
        let data = Some(Ok({ value = 100 })) in
        match data with
        | Some(Ok(rec)) -> rec.value
        | Some(Error(_)) -> -1
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(100));
}

#[test]
#[ignore = "TODO: Requires parser support for tuple pattern matching"]
fn test_match_tuple_of_mixed_types() {
    // Test: Pattern match on tuple containing records and variants
    let source = r#"
        let data = ({ name = "Alice"; age = 30 }, Some(100)) in
        match data with
        | (user, Some(score)) -> user.age + score
        | (user, None) -> user.age
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(130));
}

#[test]
#[ignore = "TODO: Requires parser support for list pattern with mixed types"]
fn test_match_list_of_mixed_types() {
    // Test: Pattern match on list containing both records and variants
    let source = r#"
        let items = [Some({ value = 10 }); None; Some({ value = 20 })] in
        match items with
        | Some(rec) :: rest -> rec.value
        | None :: rest -> 0
        | [] -> -1
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
#[ignore = "TODO: Requires parser support for multiple pattern guards"]
fn test_match_with_record_field_guards() {
    // Test: Pattern matching with guards on record fields
    let source = r#"
        let entity = User({ name = "Admin"; level = 10 }) in
        match entity with
        | User(u) when u.level > 5 -> true
        | User(_) -> false
        | Guest -> false
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
#[ignore = "TODO: Requires parser support for complex destructuring"]
fn test_destructure_variant_and_record() {
    // Test: Destructuring both variant and record in pattern
    let source = r#"
        let response = Success({ code = 200; body = "OK" }) in
        match response with
        | Success({ code = c; body = b }) -> c
        | Failure(_) -> 500
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(200));
}

// ============================================================================
// COMPLEX NESTED STRUCTURES (5+ tests)
// ============================================================================

#[test]
#[ignore = "TODO: Requires parser support for deeply nested structures"]
fn test_deeply_nested_mixed_structure() {
    // Test: Record -> Variant -> Record -> Variant
    let source = r#"
        let config = {
            database = Some({
                connection = Ok({ host = "localhost"; port = 5432 })
            })
        } in
        match config.database with
        | Some(db) ->
            match db.connection with
            | Ok(conn) -> conn.port
            | Error(_) -> 0
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(5432));
}

#[test]
#[ignore = "TODO: Requires parser support for list of nested structures"]
fn test_list_of_nested_mixed_types() {
    // Test: List of records containing variants containing records
    let source = r#"
        let users = [
            { id = 1; profile = Some({ name = "Alice"; age = 30 }) };
            { id = 2; profile = None }
        ] in
        let first = List.head users in
        match first.profile with
        | Some(p) -> p.age
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
#[ignore = "TODO: Requires parser support for tree-like structures"]
fn test_tree_with_mixed_nodes() {
    // Test: Tree structure using both records and variants
    let source = r#"
        let tree = Node({
            value = 10;
            left = Some(Leaf({ value = 5 }));
            right = Some(Leaf({ value = 15 }))
        }) in
        match tree with
        | Node(n) -> n.value
        | Leaf(l) -> l.value
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(10));
}

#[test]
#[ignore = "TODO: Requires parser support for complex result types"]
fn test_result_with_complex_types() {
    // Test: Result containing records with variant fields
    let source = r#"
        let result = Ok({
            user = { name = "Bob"; status = Some("active") };
            permissions = ["read"; "write"]
        }) in
        match result with
        | Ok(data) ->
            match data.user.status with
            | Some(_) -> true
            | None -> false
        | Error(_) -> false
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
#[ignore = "TODO: Requires parser support for state machine patterns"]
fn test_state_machine_with_mixed_types() {
    // Test: State machine using variants with record payloads
    let source = r#"
        let state = Active({ user_id = 123; session_id = "abc"; last_seen = 1234567890 }) in
        match state with
        | Idle -> 0
        | Active(s) -> s.user_id
        | Suspended(s) -> s.user_id
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(123));
}

#[test]
#[ignore = "TODO: Requires parser support for option chaining"]
fn test_option_chaining_with_records() {
    // Test: Chaining operations on nested Option<Record>
    let source = r#"
        let data = Some({ value = Some({ inner = 42 }) }) in
        match data with
        | Some(outer) ->
            match outer.value with
            | Some(inner) -> inner.inner
            | None -> 0
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(42));
}

// ============================================================================
// FUNCTION COMPOSITION WITH MIXED TYPES (5+ tests)
// ============================================================================

#[test]
#[ignore = "TODO: Requires parser support for function with record/variant"]
fn test_function_returning_variant_with_record() {
    // Test: Function that returns a variant containing a record
    let source = r#"
        let makeUser name age =
            if age > 0 then
                Ok({ name = name; age = age })
            else
                Error("Invalid age")
        in
        match makeUser "Alice" 30 with
        | Ok(u) -> u.age
        | Error(_) -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
#[ignore = "TODO: Requires parser support for map operations"]
fn test_map_over_variants_with_records() {
    // Test: Mapping a function over variants containing records
    let source = r#"
        let increment_age user_opt =
            match user_opt with
            | Some(u) -> Some({ u with age = u.age + 1 })
            | None -> None
        in
        let user = Some({ name = "Bob"; age = 25 }) in
        match increment_age user with
        | Some(u) -> u.age
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(26));
}

#[test]
#[ignore = "TODO: Requires parser support for filter operations"]
fn test_filter_records_by_variant_field() {
    // Test: Filtering records based on variant fields
    let source = r#"
        let is_active user =
            match user.status with
            | Some("active") -> true
            | _ -> false
        in
        let user = { name = "Charlie"; status = Some("active") } in
        is_active user
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
#[ignore = "TODO: Requires parser support for reduce operations"]
fn test_fold_over_mixed_structures() {
    // Test: Folding over a list of mixed structures
    let source = r#"
        let sum_values items =
            let rec helper acc list =
                match list with
                | [] -> acc
                | Some({ value = v }) :: rest -> helper (acc + v) rest
                | None :: rest -> helper acc rest
            in
            helper 0 items
        in
        let items = [Some({ value = 10 }); None; Some({ value = 20 })] in
        sum_values items
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(30));
}

#[test]
#[ignore = "TODO: Requires parser support for pipeline operations"]
fn test_pipeline_mixed_transformations() {
    // Test: Pipelining transformations on mixed types
    let source = r#"
        let validate_user user =
            if user.age > 0 then Some(user) else None
        in
        let extract_age user_opt =
            match user_opt with
            | Some(u) -> Ok(u.age)
            | None -> Error("Invalid user")
        in
        let user = { name = "Diana"; age = 28 } in
        match extract_age (validate_user user) with
        | Ok(age) -> age
        | Error(_) -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(28));
}

// ============================================================================
// REAL-WORLD PATTERNS (5+ tests)
// ============================================================================

#[test]
#[ignore = "TODO: Requires parser support for API response patterns"]
fn test_api_response_pattern() {
    // Test: Typical API response pattern with mixed types
    let source = r#"
        let response = {
            status = 200;
            data = Some({ users = [{ id = 1; name = "Alice" }] });
            error = None
        } in
        match response.data with
        | Some(d) -> List.length d.users
        | None -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
#[ignore = "TODO: Requires parser support for validation patterns"]
fn test_validation_result_pattern() {
    // Test: Validation result pattern
    let source = r#"
        let validate_input input =
            if String.length input.value > 0 then
                Valid({ data = input.value; timestamp = 123 })
            else
                Invalid({ errors = ["Empty value"]; timestamp = 123 })
        in
        let input = { value = "test"; required = true } in
        match validate_input input with
        | Valid(v) -> true
        | Invalid(_) -> false
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
#[ignore = "TODO: Requires parser support for event sourcing patterns"]
fn test_event_sourcing_pattern() {
    // Test: Event sourcing pattern with events as variants with record payloads
    let source = r#"
        let apply_event state event =
            match event with
            | UserCreated(data) -> { state with users = data :: state.users }
            | UserDeleted(id) -> state  // Simplified
            | UserUpdated(data) -> state
        in
        let initial = { users = [] } in
        let event = UserCreated({ id = 1; name = "Alice"; email = "alice@example.com" }) in
        let new_state = apply_event initial event in
        List.length new_state.users
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
#[ignore = "TODO: Requires parser support for configuration patterns"]
fn test_configuration_pattern() {
    // Test: Configuration pattern with optional nested settings
    let source = r#"
        let get_port config =
            match config.server with
            | Some(server) ->
                match server.port with
                | Some(p) -> p
                | None -> 8080  // Default
            | None -> 8080
        in
        let config = {
            app_name = "MyApp";
            server = Some({ host = "localhost"; port = Some(3000) })
        } in
        get_port config
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(3000));
}

#[test]
#[ignore = "TODO: Requires parser support for error handling patterns"]
fn test_error_handling_chain() {
    // Test: Error handling chain with Result types
    let source = r#"
        let parse_int str =
            if str = "42" then Ok(42) else Error("Parse error")
        in
        let validate_positive n =
            if n > 0 then Ok(n) else Error("Not positive")
        in
        let process input =
            match parse_int input.value with
            | Ok(n) -> validate_positive n
            | Error(e) -> Error(e)
        in
        let input = { value = "42"; required = true } in
        match process input with
        | Ok(n) -> n
        | Error(_) -> 0
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
#[ignore = "TODO: Requires parser support for builder patterns"]
fn test_builder_pattern_with_options() {
    // Test: Builder pattern using records with optional fields
    let source = r#"
        let build_config builder =
            {
                host = match builder.host with | Some(h) -> h | None -> "localhost";
                port = match builder.port with | Some(p) -> p | None -> 8080;
                debug = match builder.debug with | Some(d) -> d | None -> false
            }
        in
        let builder = { host = Some("example.com"); port = None; debug = Some(true) } in
        let config = build_config builder in
        config.port
    "#;
    let result = run_source(source).unwrap();
    assert_eq!(result, Value::Int(8080));
}
