//! Demonstration that comma-separated list syntax now works (Issue #125)

use fusabi_frontend::ast::Expr;
use fusabi_frontend::lexer::Lexer;
use fusabi_frontend::parser::Parser;

fn parse(source: &str) -> Result<Expr, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    parser.parse().map_err(|e| e.to_string())
}

#[test]
fn test_issue_125_comma_syntax() {
    // This is the exact syntax from issue #125 that was failing
    let source = "let list = [1, 2, 3] in list";

    match parse(source) {
        Ok(expr) => {
            // Verify it parses correctly
            match expr {
                Expr::Let { name, value, .. } => {
                    assert_eq!(name, "list");
                    match *value {
                        Expr::List(elements) => {
                            assert_eq!(elements.len(), 3);
                            println!(
                                "SUCCESS: Parsed list with {} elements using comma syntax",
                                elements.len()
                            );
                        }
                        _ => panic!("Expected List in value"),
                    }
                }
                _ => panic!("Expected Let expression"),
            }
        }
        Err(e) => {
            panic!("Failed to parse: {}", e);
        }
    }
}

#[test]
fn test_all_supported_list_syntaxes() {
    let test_cases = vec![
        ("[1, 2, 3]", 3, "comma-separated"),
        ("[1, 2, 3,]", 3, "comma with trailing"),
        ("[1; 2; 3]", 3, "semicolon-separated"),
        ("[1; 2; 3;]", 3, "semicolon with trailing"),
        ("[]", 0, "empty list"),
        ("[[1, 2], [3, 4]]", 2, "nested lists with commas"),
    ];

    for (source, expected_len, description) in test_cases {
        match parse(source) {
            Ok(Expr::List(elements)) => {
                assert_eq!(elements.len(), expected_len, "Failed for: {}", description);
                println!("PASS: {} - parsed {} elements", description, expected_len);
            }
            Ok(_) => panic!("Expected List for: {}", description),
            Err(e) => panic!("Failed to parse {}: {}", description, e),
        }
    }
}

#[test]
fn test_bytecode_api_syntax() {
    // This is the exact source from the bytecode API test
    let source = r#"
        let list = [1, 2, 3] in
        let doubled = List.map (fun x -> x * 2) list in
        List.head doubled
    "#;

    // This should parse without errors now
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    println!("SUCCESS: Bytecode API test syntax parses correctly");
}
