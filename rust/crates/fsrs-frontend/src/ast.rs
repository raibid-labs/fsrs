//! Core AST (Abstract Syntax Tree) definitions for FSRS Mini-F#.
//!
//! This module defines the foundational types for representing F# expressions
//! in the FSRS compiler frontend. The AST serves as the intermediate representation
//! between parsing and bytecode compilation.
//!
//! # Phase 1 MVP Features
//!
//! The AST supports:
//! - Literals: integers, floats, booleans, strings, unit
//! - Variables and let-bindings
//! - Lambda functions and function application
//! - Binary operations (arithmetic, comparison, logical)
//! - Conditional expressions (if-then-else)
//! - Tuples (e.g., (1, 2), (x, y, z))
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//!
//! // Construct: let x = 42 in x + 1
//! let expr = Expr::Let {
//!     name: "x".to_string(),
//!     value: Box::new(Expr::Lit(Literal::Int(42))),
//!     body: Box::new(Expr::BinOp {
//!         op: BinOp::Add,
//!         left: Box::new(Expr::Var("x".to_string())),
//!         right: Box::new(Expr::Lit(Literal::Int(1))),
//!     }),
//! };
//! ```

use std::fmt;

/// Literal values in the AST.
///
/// Represents constant values that can appear in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal (e.g., 42, -10)
    Int(i64),
    /// Floating-point literal (e.g., 2.5, -0.5)
    Float(f64),
    /// Boolean literal (true or false)
    Bool(bool),
    /// String literal (e.g., "hello")
    Str(String),
    /// Unit value (equivalent to () in F#)
    Unit,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Unit => write!(f, "()"),
        }
    }
}

/// Binary operators supported in expressions.
///
/// Includes arithmetic, comparison, and logical operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic operators
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*)
    Mul,
    /// Division (/)
    Div,

    // Comparison operators
    /// Equality (=)
    Eq,
    /// Inequality (<>)
    Neq,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,

    // Logical operators
    /// Logical AND (&&)
    And,
    /// Logical OR (||)
    Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "=",
            BinOp::Neq => "<>",
            BinOp::Lt => "<",
            BinOp::Lte => "<=",
            BinOp::Gt => ">",
            BinOp::Gte => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        };
        write!(f, "{}", s)
    }
}

impl BinOp {
    /// Returns true if this is an arithmetic operator.
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div)
    }

    /// Returns true if this is a comparison operator.
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte
        )
    }

    /// Returns true if this is a logical operator.
    pub fn is_logical(&self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }
}

/// Core expression types in the AST.
///
/// Represents all expression forms supported in Phase 1 of FSRS.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference (e.g., x, foo)
    Var(String),

    /// Literal value
    Lit(Literal),

    /// Binary operation (e.g., x + 1, a && b)
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Let-binding (e.g., let x = 5 in x + 1)
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Recursive let-binding (e.g., let rec fact n = if n <= 1 then 1 else n * fact (n - 1))
    LetRec {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Mutually recursive let-bindings (e.g., let rec even n = ... and odd n = ...)
    LetRecMutual {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },

    /// Lambda function (e.g., fun x -> x + 1)
    Lambda { param: String, body: Box<Expr> },

    /// Function application (e.g., f x, add 1 2)
    App { func: Box<Expr>, arg: Box<Expr> },

    /// Conditional expression (e.g., if x > 0 then 1 else -1)
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    /// Tuple expression (e.g., (1, 2), (x, y, z))
    /// Empty tuple () is represented as Lit(Literal::Unit)
    Tuple(Vec<Expr>),
}

impl Expr {
    /// Returns true if this expression is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Lit(_))
    }

    /// Returns true if this expression is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self, Expr::Var(_))
    }

    /// Returns true if this expression is a binary operation.
    pub fn is_binop(&self) -> bool {
        matches!(self, Expr::BinOp { .. })
    }

    /// Returns true if this expression is a let-binding.
    pub fn is_let(&self) -> bool {
        matches!(self, Expr::Let { .. })
    }

    /// Returns true if this expression is a recursive let-binding.
    pub fn is_let_rec(&self) -> bool {
        matches!(self, Expr::LetRec { .. })
    }

    /// Returns true if this expression is a mutually recursive let-binding.
    pub fn is_let_rec_mutual(&self) -> bool {
        matches!(self, Expr::LetRecMutual { .. })
    }

    /// Returns true if this expression is a lambda.
    pub fn is_lambda(&self) -> bool {
        matches!(self, Expr::Lambda { .. })
    }

    /// Returns true if this expression is a function application.
    pub fn is_app(&self) -> bool {
        matches!(self, Expr::App { .. })
    }

    /// Returns true if this expression is a conditional.
    pub fn is_if(&self) -> bool {
        matches!(self, Expr::If { .. })
    }

    /// Returns true if this expression is a tuple.
    pub fn is_tuple(&self) -> bool {
        matches!(self, Expr::Tuple(_))
    }

    /// Returns the variable name if this is a Var, otherwise None.
    pub fn as_var(&self) -> Option<&str> {
        match self {
            Expr::Var(name) => Some(name),
            _ => None,
        }
    }

    /// Returns the literal value if this is a Lit, otherwise None.
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            Expr::Lit(lit) => Some(lit),
            _ => None,
        }
    }

    /// Returns the tuple elements if this is a Tuple, otherwise None.
    pub fn as_tuple(&self) -> Option<&Vec<Expr>> {
        match self {
            Expr::Tuple(elements) => Some(elements),
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Lit(lit) => write!(f, "{}", lit),
            Expr::BinOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::Let { name, value, body } => {
                write!(f, "(let {} = {} in {})", name, value, body)
            }
            Expr::LetRec { name, value, body } => {
                write!(f, "(let rec {} = {} in {})", name, value, body)
            }
            Expr::LetRecMutual { bindings, body } => {
                write!(f, "(let rec ")?;
                for (i, (name, value)) in bindings.iter().enumerate() {
                    if i > 0 {
                        write!(f, " and ")?;
                    }
                    write!(f, "{} = {}", name, value)?;
                }
                write!(f, " in {})", body)
            }
            Expr::Lambda { param, body } => {
                write!(f, "(fun {} -> {})", param, body)
            }
            Expr::App { func, arg } => {
                write!(f, "({} {})", func, arg)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                write!(f, "(if {} then {} else {})", cond, then_branch, else_branch)
            }
            Expr::Tuple(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Literal Tests
    // ========================================================================

    #[test]
    fn test_literal_int() {
        let lit = Literal::Int(42);
        assert_eq!(lit, Literal::Int(42));
        assert_eq!(format!("{}", lit), "42");
    }

    #[test]
    fn test_literal_float() {
        let lit = Literal::Float(2.5);
        assert_eq!(lit, Literal::Float(2.5));
        assert_eq!(format!("{}", lit), "2.5");
    }

    #[test]
    fn test_literal_bool() {
        let lit_true = Literal::Bool(true);
        let lit_false = Literal::Bool(false);
        assert_eq!(lit_true, Literal::Bool(true));
        assert_eq!(lit_false, Literal::Bool(false));
        assert_eq!(format!("{}", lit_true), "true");
        assert_eq!(format!("{}", lit_false), "false");
    }

    #[test]
    fn test_literal_str() {
        let lit = Literal::Str("hello".to_string());
        assert_eq!(lit, Literal::Str("hello".to_string()));
        assert_eq!(format!("{}", lit), "\"hello\"");
    }

    #[test]
    fn test_literal_unit() {
        let lit = Literal::Unit;
        assert_eq!(lit, Literal::Unit);
        assert_eq!(format!("{}", lit), "()");
    }

    #[test]
    fn test_literal_clone() {
        let lit = Literal::Int(42);
        let cloned = lit.clone();
        assert_eq!(lit, cloned);
    }

    // ========================================================================
    // BinOp Tests
    // ========================================================================

    #[test]
    fn test_binop_arithmetic() {
        assert!(BinOp::Add.is_arithmetic());
        assert!(BinOp::Sub.is_arithmetic());
        assert!(BinOp::Mul.is_arithmetic());
        assert!(BinOp::Div.is_arithmetic());
        assert!(!BinOp::Eq.is_arithmetic());
        assert!(!BinOp::And.is_arithmetic());
    }

    #[test]
    fn test_binop_comparison() {
        assert!(BinOp::Eq.is_comparison());
        assert!(BinOp::Neq.is_comparison());
        assert!(BinOp::Lt.is_comparison());
        assert!(BinOp::Lte.is_comparison());
        assert!(BinOp::Gt.is_comparison());
        assert!(BinOp::Gte.is_comparison());
        assert!(!BinOp::Add.is_comparison());
        assert!(!BinOp::And.is_comparison());
    }

    #[test]
    fn test_binop_logical() {
        assert!(BinOp::And.is_logical());
        assert!(BinOp::Or.is_logical());
        assert!(!BinOp::Add.is_logical());
        assert!(!BinOp::Eq.is_logical());
    }

    #[test]
    fn test_binop_display() {
        assert_eq!(format!("{}", BinOp::Add), "+");
        assert_eq!(format!("{}", BinOp::Sub), "-");
        assert_eq!(format!("{}", BinOp::Mul), "*");
        assert_eq!(format!("{}", BinOp::Div), "/");
        assert_eq!(format!("{}", BinOp::Eq), "=");
        assert_eq!(format!("{}", BinOp::Neq), "<>");
        assert_eq!(format!("{}", BinOp::Lt), "<");
        assert_eq!(format!("{}", BinOp::Lte), "<=");
        assert_eq!(format!("{}", BinOp::Gt), ">");
        assert_eq!(format!("{}", BinOp::Gte), ">=");
        assert_eq!(format!("{}", BinOp::And), "&&");
        assert_eq!(format!("{}", BinOp::Or), "||");
    }

    // ========================================================================
    // Expr Tests - Construction and Type Checking
    // ========================================================================

    #[test]
    fn test_expr_var() {
        let expr = Expr::Var("x".to_string());
        assert!(expr.is_var());
        assert!(!expr.is_literal());
        assert_eq!(expr.as_var(), Some("x"));
        assert_eq!(format!("{}", expr), "x");
    }

    #[test]
    fn test_expr_lit() {
        let expr = Expr::Lit(Literal::Int(42));
        assert!(expr.is_literal());
        assert!(!expr.is_var());
        assert_eq!(expr.as_literal(), Some(&Literal::Int(42)));
        assert_eq!(format!("{}", expr), "42");
    }

    #[test]
    fn test_expr_binop() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        };
        assert!(expr.is_binop());
        assert!(!expr.is_literal());
        assert_eq!(format!("{}", expr), "(1 + 2)");
    }

    #[test]
    fn test_expr_let() {
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(42))),
            body: Box::new(Expr::Var("x".to_string())),
        };
        assert!(expr.is_let());
        assert!(!expr.is_var());
        assert_eq!(format!("{}", expr), "(let x = 42 in x)");
    }

    #[test]
    fn test_expr_lambda() {
        let expr = Expr::Lambda {
            param: "x".to_string(),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(1))),
            }),
        };
        assert!(expr.is_lambda());
        assert!(!expr.is_app());
        assert_eq!(format!("{}", expr), "(fun x -> (x + 1))");
    }

    #[test]
    fn test_expr_app() {
        let expr = Expr::App {
            func: Box::new(Expr::Var("f".to_string())),
            arg: Box::new(Expr::Lit(Literal::Int(42))),
        };
        assert!(expr.is_app());
        assert!(!expr.is_lambda());
        assert_eq!(format!("{}", expr), "(f 42)");
    }

    #[test]
    fn test_expr_if() {
        let expr = Expr::If {
            cond: Box::new(Expr::Lit(Literal::Bool(true))),
            then_branch: Box::new(Expr::Lit(Literal::Int(1))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };
        assert!(expr.is_if());
        assert!(!expr.is_let());
        assert_eq!(format!("{}", expr), "(if true then 1 else 0)");
    }

    // ========================================================================
    // Complex Expression Tests
    // ========================================================================

    #[test]
    fn test_nested_binop() {
        // (1 + 2) * 3
        let expr = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Lit(Literal::Int(1))),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            }),
            right: Box::new(Expr::Lit(Literal::Int(3))),
        };
        assert!(expr.is_binop());
        assert_eq!(format!("{}", expr), "((1 + 2) * 3)");
    }

    #[test]
    fn test_let_with_complex_body() {
        // let x = 42 in x + 1
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(42))),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(1))),
            }),
        };
        assert!(expr.is_let());
        assert_eq!(format!("{}", expr), "(let x = 42 in (x + 1))");
    }

    #[test]
    fn test_nested_let() {
        // let x = 1 in let y = 2 in x + y
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(1))),
            body: Box::new(Expr::Let {
                name: "y".to_string(),
                value: Box::new(Expr::Lit(Literal::Int(2))),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::Var("y".to_string())),
                }),
            }),
        };
        assert!(expr.is_let());
        assert_eq!(format!("{}", expr), "(let x = 1 in (let y = 2 in (x + y)))");
    }

    #[test]
    fn test_lambda_application() {
        // (fun x -> x + 1) 42
        let expr = Expr::App {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::Lit(Literal::Int(1))),
                }),
            }),
            arg: Box::new(Expr::Lit(Literal::Int(42))),
        };
        assert!(expr.is_app());
        assert_eq!(format!("{}", expr), "((fun x -> (x + 1)) 42)");
    }

    #[test]
    fn test_if_with_comparison() {
        // if x > 0 then 1 else -1
        let expr = Expr::If {
            cond: Box::new(Expr::BinOp {
                op: BinOp::Gt,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(0))),
            }),
            then_branch: Box::new(Expr::Lit(Literal::Int(1))),
            else_branch: Box::new(Expr::Lit(Literal::Int(-1))),
        };
        assert!(expr.is_if());
        assert_eq!(format!("{}", expr), "(if (x > 0) then 1 else -1)");
    }

    #[test]
    fn test_expr_clone_and_equality() {
        let expr1 = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        };
        let expr2 = expr1.clone();
        assert_eq!(expr1, expr2);
    }

    #[test]
    fn test_all_literal_types_in_expr() {
        let exprs = vec![
            Expr::Lit(Literal::Int(42)),
            Expr::Lit(Literal::Float(2.5)),
            Expr::Lit(Literal::Bool(true)),
            Expr::Lit(Literal::Str("hello".to_string())),
            Expr::Lit(Literal::Unit),
        ];

        for expr in exprs {
            assert!(expr.is_literal());
            assert!(expr.as_literal().is_some());
        }
    }

    #[test]
    fn test_all_binop_types() {
        let ops = vec![
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Eq,
            BinOp::Neq,
            BinOp::Lt,
            BinOp::Lte,
            BinOp::Gt,
            BinOp::Gte,
            BinOp::And,
            BinOp::Or,
        ];

        for op in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Int(1))),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            };
            assert!(expr.is_binop());
        }
    }

    #[test]
    fn test_utility_methods_return_none_for_wrong_type() {
        let expr = Expr::Lit(Literal::Int(42));
        assert_eq!(expr.as_var(), None);

        let expr = Expr::Var("x".to_string());
        assert_eq!(expr.as_literal(), None);
    }

    // ========================================================================
    // Tuple Tests
    // ========================================================================

    #[test]
    fn test_tuple_empty() {
        // Empty tuple is represented as Literal::Unit, not Tuple(vec![])
        let expr = Expr::Lit(Literal::Unit);
        assert!(expr.is_literal());
        assert!(!expr.is_tuple());
        assert_eq!(format!("{}", expr), "()");
    }

    #[test]
    fn test_tuple_pair() {
        // (1, 2)
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]);
        assert!(expr.is_tuple());
        assert!(!expr.is_literal());
        assert_eq!(format!("{}", expr), "(1, 2)");
    }

    #[test]
    fn test_tuple_triple() {
        // (1, 2, 3)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "(1, 2, 3)");
    }

    #[test]
    fn test_tuple_nested() {
        // (1, (2, 3))
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Tuple(vec![Expr::Lit(Literal::Int(2)), Expr::Lit(Literal::Int(3))]),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "(1, (2, 3))");
    }

    #[test]
    fn test_tuple_nested_deep() {
        // ((1, 2), (3, 4))
        let expr = Expr::Tuple(vec![
            Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]),
            Expr::Tuple(vec![Expr::Lit(Literal::Int(3)), Expr::Lit(Literal::Int(4))]),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "((1, 2), (3, 4))");
    }

    #[test]
    fn test_tuple_mixed_types() {
        // (1, "hello", true)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Str("hello".to_string())),
            Expr::Lit(Literal::Bool(true)),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), r#"(1, "hello", true)"#);
    }

    #[test]
    fn test_tuple_with_variables() {
        // (x, y, z)
        let expr = Expr::Tuple(vec![
            Expr::Var("x".to_string()),
            Expr::Var("y".to_string()),
            Expr::Var("z".to_string()),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "(x, y, z)");
    }

    #[test]
    fn test_tuple_with_expressions() {
        // (x + 1, y * 2)
        let expr = Expr::Tuple(vec![
            Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(1))),
            },
            Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Var("y".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            },
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "((x + 1), (y * 2))");
    }

    #[test]
    fn test_tuple_as_tuple() {
        // Test as_tuple() helper
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]);

        let elements = expr.as_tuple();
        assert!(elements.is_some());
        assert_eq!(elements.unwrap().len(), 2);
    }

    #[test]
    fn test_tuple_as_tuple_none() {
        // Non-tuple should return None
        let expr = Expr::Lit(Literal::Int(42));
        assert_eq!(expr.as_tuple(), None);
    }

    #[test]
    fn test_tuple_single_element() {
        // (42,) - single element tuple
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(42))]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "(42)");
    }

    #[test]
    fn test_tuple_with_nested_let() {
        // (let x = 1 in x, 2)
        let expr = Expr::Tuple(vec![
            Expr::Let {
                name: "x".to_string(),
                value: Box::new(Expr::Lit(Literal::Int(1))),
                body: Box::new(Expr::Var("x".to_string())),
            },
            Expr::Lit(Literal::Int(2)),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "((let x = 1 in x), 2)");
    }

    #[test]
    fn test_tuple_with_lambda() {
        // ((fun x -> x + 1), 2)
        let expr = Expr::Tuple(vec![
            Expr::Lambda {
                param: "x".to_string(),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::Lit(Literal::Int(1))),
                }),
            },
            Expr::Lit(Literal::Int(2)),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "((fun x -> (x + 1)), 2)");
    }

    #[test]
    fn test_tuple_clone_and_equality() {
        let expr1 = Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]);
        let expr2 = expr1.clone();
        assert_eq!(expr1, expr2);
    }

    #[test]
    fn test_tuple_large() {
        // (1, 2, 3, 4, 5, 6, 7, 8)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
            Expr::Lit(Literal::Int(4)),
            Expr::Lit(Literal::Int(5)),
            Expr::Lit(Literal::Int(6)),
            Expr::Lit(Literal::Int(7)),
            Expr::Lit(Literal::Int(8)),
        ]);
        assert!(expr.is_tuple());
        assert_eq!(format!("{}", expr), "(1, 2, 3, 4, 5, 6, 7, 8)");
    }
}
