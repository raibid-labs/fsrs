// Integration tests for list functionality in fsrs-demo
// These tests verify end-to-end list operations through the full pipeline

use fsrs_demo::run_source;
use fsrs_vm::Value;

#[cfg(test)]
mod list_literal_tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let source = "[]";
        let result = run_source(source).expect("Failed to execute empty list");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_single_element_list() {
        let source = "[42]";
        let result = run_source(source).expect("Failed to execute single element list");
        assert_eq!(result, Value::vec_to_cons(vec![Value::Int(42)]));
    }

    #[test]
    fn test_multiple_element_list() {
        let source = "[1; 2; 3; 4; 5]";
        let result = run_source(source).expect("Failed to execute multiple element list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
            ])
        );
    }

    #[test]
    fn test_list_with_strings() {
        let source = r#"["hello"; "world"]"#;
        let result = run_source(source).expect("Failed to execute string list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Str("hello".to_string()),
                Value::Str("world".to_string()),
            ])
        );
    }

    #[test]
    fn test_list_with_booleans() {
        let source = "[true; false; true]";
        let result = run_source(source).expect("Failed to execute boolean list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ])
        );
    }
}

#[cfg(test)]
mod cons_operator_tests {
    use super::*;

    #[test]
    fn test_cons_simple() {
        let source = "1 :: []";
        let result = run_source(source).expect("Failed to execute cons");
        assert_eq!(result, Value::vec_to_cons(vec![Value::Int(1)]));
    }

    #[test]
    fn test_cons_chain() {
        let source = "1 :: 2 :: 3 :: []";
        let result = run_source(source).expect("Failed to execute cons chain");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
        );
    }

    #[test]
    fn test_cons_prepend_to_list() {
        let source = "let numbers = [2; 3; 4] in 1 :: numbers";
        let result = run_source(source).expect("Failed to execute cons prepend");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ])
        );
    }

    #[test]
    fn test_cons_with_expressions() {
        let source = "let x = 1 + 2 in x :: [4; 5]";
        let result = run_source(source).expect("Failed to execute cons with expression");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(3), Value::Int(4), Value::Int(5),])
        );
    }

    #[test]
    fn test_multiple_cons_operations() {
        let source = "let a = 1 :: [] in let b = 2 :: a in 3 :: b";
        let result = run_source(source).expect("Failed to execute multiple cons");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(3), Value::Int(2), Value::Int(1),])
        );
    }
}

#[cfg(test)]
mod nested_list_tests {
    use super::*;

    #[test]
    fn test_list_of_lists() {
        let source = "[[1; 2]; [3; 4]]";
        let result = run_source(source).expect("Failed to execute list of lists");

        let inner1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let inner2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let expected = Value::vec_to_cons(vec![inner1, inner2]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_matrix_structure() {
        let source = "[[1; 2]; [3; 4]; [5; 6]]";
        let result = run_source(source).expect("Failed to execute matrix");

        let row1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let row2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let row3 = Value::vec_to_cons(vec![Value::Int(5), Value::Int(6)]);
        let expected = Value::vec_to_cons(vec![row1, row2, row3]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_of_empty_lists() {
        let source = "[[]; []; []]";
        let result = run_source(source).expect("Failed to execute list of empty lists");

        let expected = Value::vec_to_cons(vec![Value::Nil, Value::Nil, Value::Nil]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_deeply_nested_lists() {
        let source = "[[[1]]]";
        let result = run_source(source).expect("Failed to execute deeply nested list");

        let innermost = Value::vec_to_cons(vec![Value::Int(1)]);
        let middle = Value::vec_to_cons(vec![innermost]);
        let expected = Value::vec_to_cons(vec![middle]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_cons_nested_lists() {
        let source = "[1; 2] :: [[3; 4]] :: []";
        let result = run_source(source).expect("Failed to execute cons with nested lists");

        let list1 = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2_inner = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let list2 = Value::vec_to_cons(vec![list2_inner]);
        let expected = Value::vec_to_cons(vec![list1, list2]);

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod list_with_bindings_tests {
    use super::*;

    #[test]
    fn test_list_in_let_binding() {
        let source = "let numbers = [1; 2; 3] in numbers";
        let result = run_source(source).expect("Failed to execute list in let binding");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
        );
    }

    #[test]
    fn test_multiple_list_bindings() {
        let source = "let a = [1; 2] in let b = [3; 4] in a";
        let result = run_source(source).expect("Failed to execute multiple list bindings");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2),])
        );
    }

    #[test]
    fn test_list_binding_with_cons() {
        let source = "let base = [2; 3; 4] in let extended = 1 :: base in extended";
        let result = run_source(source).expect("Failed to execute list binding with cons");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
            ])
        );
    }

    #[test]
    fn test_nested_list_bindings() {
        let source = "let inner = [1; 2] in let outer = [inner; [3; 4]] in outer";
        let result = run_source(source).expect("Failed to execute nested list bindings");

        let inner = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2)]);
        let list2 = Value::vec_to_cons(vec![Value::Int(3), Value::Int(4)]);
        let expected = Value::vec_to_cons(vec![inner, list2]);

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod list_conditionals_tests {
    use super::*;

    #[test]
    fn test_conditional_list_selection() {
        let source = "if true then [1; 2] else [3; 4]";
        let result = run_source(source).expect("Failed to execute conditional list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2),])
        );
    }

    #[test]
    fn test_conditional_empty_list() {
        let source = "if false then [1; 2] else []";
        let result = run_source(source).expect("Failed to execute conditional empty list");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_nested_conditional_with_lists() {
        let source = "let x = 5 in if x > 3 then (if x > 10 then [100] else [1; 2; 3]) else []";
        let result = run_source(source).expect("Failed to execute nested conditional with lists");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
        );
    }
}

#[cfg(test)]
mod list_equality_tests {
    use super::*;

    #[test]
    fn test_empty_list_equality() {
        let source = "[] = []";
        let result = run_source(source).expect("Failed to execute empty list equality");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_list_equality_same() {
        let source = "[1; 2; 3] = [1; 2; 3]";
        let result = run_source(source).expect("Failed to execute list equality");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_list_equality_different() {
        let source = "[1; 2] = [1; 3]";
        let result = run_source(source).expect("Failed to execute list inequality");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_list_inequality_operator() {
        let source = "[1; 2] <> [3; 4]";
        let result = run_source(source).expect("Failed to execute list inequality operator");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_nested_list_equality() {
        let source = "[[1; 2]; [3; 4]] = [[1; 2]; [3; 4]]";
        let result = run_source(source).expect("Failed to execute nested list equality");
        assert_eq!(result, Value::Bool(true));
    }
}

#[cfg(test)]
mod list_edge_cases {
    use super::*;

    #[test]
    fn test_single_cons_to_empty() {
        let source = "42 :: []";
        let result = run_source(source).expect("Failed to execute single cons");
        assert_eq!(result, Value::vec_to_cons(vec![Value::Int(42)]));
    }

    #[test]
    fn test_long_list() {
        let source = "[1; 2; 3; 4; 5; 6; 7; 8; 9; 10]";
        let result = run_source(source).expect("Failed to execute long list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Int(1),
                Value::Int(2),
                Value::Int(3),
                Value::Int(4),
                Value::Int(5),
                Value::Int(6),
                Value::Int(7),
                Value::Int(8),
                Value::Int(9),
                Value::Int(10),
            ])
        );
    }

    #[test]
    fn test_cons_associativity() {
        // :: is right-associative: 1 :: 2 :: 3 :: [] = 1 :: (2 :: (3 :: []))
        let source1 = "1 :: 2 :: 3 :: []";
        let source2 = "1 :: (2 :: (3 :: []))";

        let result1 = run_source(source1).expect("Failed to execute source1");
        let result2 = run_source(source2).expect("Failed to execute source2");

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_mixed_type_list() {
        // Lists can contain different types (though not type-safe in real F#)
        let source = r#"[1; "hello"; true]"#;
        let result = run_source(source).expect("Failed to execute mixed type list");
        assert_eq!(
            result,
            Value::vec_to_cons(vec![
                Value::Int(1),
                Value::Str("hello".to_string()),
                Value::Bool(true),
            ])
        );
    }
}
