//! Integration tests for record literal and field access parsing (Issue #15 Layer 2 Phases 2-3)

use fsrs_frontend::ast::Expr;
use fsrs_frontend::lexer::Lexer;
use fsrs_frontend::parser::Parser;

// Helper function to parse a string
fn parse(input: &str) -> Result<Expr, fsrs_frontend::parser::ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ========================================================================
// Record Literal Tests (Phase 2)
// ========================================================================

#[test]
fn test_parse_record_literal_empty() {
    let expr = parse("{}").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 0);
}

#[test]
fn test_parse_record_literal_single_field() {
    let expr = parse("{ name = \"John\" }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "name");
}

#[test]
fn test_parse_record_literal_multiple_fields() {
    let expr = parse("{ name = \"John\"; age = 30 }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, "name");
    assert_eq!(fields[1].0, "age");
}

#[test]
fn test_parse_record_literal_trailing_semicolon() {
    let expr = parse("{ name = \"John\"; age = 30; }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 2);
}

#[test]
fn test_parse_record_literal_with_expressions() {
    let expr = parse("{ x = 1 + 2; y = 3 * 4 }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 2);
    assert!(fields[0].1.is_binop());
    assert!(fields[1].1.is_binop());
}

#[test]
fn test_parse_record_literal_nested() {
    let expr = parse("{ outer = { inner = 42 } }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 1);
    assert!(fields[0].1.is_record_literal());
}

#[test]
fn test_parse_record_literal_three_fields() {
    let expr = parse("{ name = \"John\"; age = 30; active = true }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_parse_record_literal_in_let() {
    let expr = parse("let person = { name = \"John\"; age = 30 } in person").unwrap();
    assert!(expr.is_let());
    if let Expr::Let { value, .. } = &expr {
        assert!(value.is_record_literal());
    }
}

#[test]
fn test_parse_record_literal_in_tuple() {
    let expr = parse("({ name = \"John\" }, { age = 30 })").unwrap();
    assert!(expr.is_tuple());
    if let Expr::Tuple(elements) = &expr {
        assert_eq!(elements.len(), 2);
        assert!(elements[0].is_record_literal());
        assert!(elements[1].is_record_literal());
    }
}

#[test]
fn test_parse_record_literal_as_function_arg() {
    let expr = parse("process { name = \"John\"; age = 30 }").unwrap();
    assert!(expr.is_app());
    if let Expr::App { arg, .. } = &expr {
        assert!(arg.is_record_literal());
    }
}

#[test]
fn test_parse_record_literal_missing_equals() {
    let expr = parse("{ name \"John\" }");
    assert!(expr.is_err());
}

#[test]
fn test_parse_record_literal_with_variables() {
    let expr = parse("{ name = n; age = a }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 2);
    assert!(fields[0].1.is_var());
    assert!(fields[1].1.is_var());
}

#[test]
fn test_parse_record_literal_complex_expressions() {
    let expr = parse("{ sum = x + y; product = x * y; avg = (x + y) / 2 }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 3);
    assert!(fields[0].1.is_binop());
    assert!(fields[1].1.is_binop());
    assert!(fields[2].1.is_binop());
}

#[test]
fn test_parse_record_literal_with_if_expression() {
    let expr = parse("{ result = if x > 0 then 1 else 0 }").unwrap();
    assert!(expr.is_record_literal());
    let (_, fields) = expr.as_record_literal().unwrap();
    assert_eq!(fields.len(), 1);
    assert!(fields[0].1.is_if());
}

// ========================================================================
// Record Field Access Tests (Phase 3)
// ========================================================================

#[test]
fn test_parse_record_access_simple() {
    let expr = parse("person.name").unwrap();
    assert!(expr.is_record_access());
}

#[test]
fn test_parse_record_access_nested() {
    let expr = parse("company.employee.name").unwrap();
    assert!(expr.is_record_access());
    if let Expr::RecordAccess { record, .. } = &expr {
        assert!(record.is_record_access());
    }
}

#[test]
fn test_parse_record_access_in_expression() {
    let expr = parse("person.age + 1").unwrap();
    assert!(expr.is_binop());
    if let Expr::BinOp { left, .. } = &expr {
        assert!(left.is_record_access());
    }
}

#[test]
fn test_parse_record_access_in_if() {
    let expr = parse("if person.age > 18 then \"adult\" else \"minor\"").unwrap();
    assert!(expr.is_if());
}

#[test]
fn test_parse_record_access_as_function_arg() {
    let expr = parse("print person.name").unwrap();
    assert!(expr.is_app());
    if let Expr::App { arg, .. } = &expr {
        assert!(arg.is_record_access());
    }
}

#[test]
fn test_parse_record_access_chained() {
    let expr = parse("a.b.c.d").unwrap();
    assert!(expr.is_record_access());
}

#[test]
fn test_parse_record_access_in_tuple() {
    let expr = parse("(person.name, person.age)").unwrap();
    assert!(expr.is_tuple());
    if let Expr::Tuple(elements) = &expr {
        assert!(elements[0].is_record_access());
        assert!(elements[1].is_record_access());
    }
}

#[test]
fn test_parse_record_access_in_list() {
    let expr = parse("[person.name; person.age]").unwrap();
    assert!(expr.is_list());
    if let Expr::List(elements) = &expr {
        assert!(elements[0].is_record_access());
        assert!(elements[1].is_record_access());
    }
}

#[test]
fn test_parse_record_access_comparison() {
    let expr = parse("person.age = 30").unwrap();
    assert!(expr.is_binop());
}

#[test]
fn test_parse_record_access_in_let() {
    let expr = parse("let n = person.name in n").unwrap();
    assert!(expr.is_let());
    if let Expr::Let { value, .. } = &expr {
        assert!(value.is_record_access());
    }
}
