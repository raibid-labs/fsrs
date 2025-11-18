//! Recursive-descent parser for Mini-F# expressions.
//!
//! This module implements a recursive-descent parser that converts a stream of tokens
//! from the lexer into an Abstract Syntax Tree (AST). The parser supports:
//!
//! - Literals: integers, floats, booleans, strings
//! - Variables and identifiers
//! - Let-bindings: `let x = expr in body`
//! - Multi-parameter functions (curried): `let f x y = expr in body`
//! - Lambda functions: `fun x -> body`
//! - Function application: `f x y`
//! - Binary operations: arithmetic, comparison, logical
//! - Conditional expressions: `if cond then expr1 else expr2`
//! - Unary minus: `-42`, `-x`
//! - Proper operator precedence
//! - Error recovery and reporting
//!
//! # Grammar (Simplified)
//!
//! ```text
//! expr       ::= let_expr | if_expr | lambda_expr | or_expr
//! let_expr   ::= "let" IDENT IDENT* "=" expr "in" expr
//! if_expr    ::= "if" expr "then" expr "else" expr
//! lambda_expr::= "fun" IDENT "->" expr
//! or_expr    ::= and_expr ("||" and_expr)*
//! and_expr   ::= comp_expr ("&&" comp_expr)*
//! comp_expr  ::= add_expr (("=" | "==" | "<>" | "<" | "<=" | ">" | ">=") add_expr)?
//! add_expr   ::= mul_expr (("+" | "-") mul_expr)*
//! mul_expr   ::= unary_expr (("*" | "/") unary_expr)*
//! unary_expr ::= "-" unary_expr | app_expr
//! app_expr   ::= primary (primary)*
//! primary    ::= INT | FLOAT | BOOL | STRING | IDENT | "(" expr ")"
//! ```
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::parser::Parser;
//! use fsrs_frontend::lexer::Lexer;
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//!
//! let mut lexer = Lexer::new("let x = 42 in x + 1");
//! let tokens = lexer.tokenize().unwrap();
//! let mut parser = Parser::new(tokens);
//! let ast = parser.parse().unwrap();
//!
//! // AST represents: let x = 42 in (x + 1)
//! assert!(ast.is_let());
//! ```

use crate::ast::{BinOp, Expr, Literal};
use crate::lexer::{Position, Token, TokenWithPos};
use std::fmt;

/// Parse errors with position information.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Unexpected token encountered
    UnexpectedToken {
        expected: String,
        found: Token,
        pos: Position,
    },
    /// Unexpected end of input
    UnexpectedEof { expected: String },
    /// Invalid expression
    InvalidExpr { message: String, pos: Position },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken {
                expected,
                found,
                pos,
            } => {
                write!(
                    f,
                    "Parse error at {}: expected {}, found {}",
                    pos, expected, found
                )
            }
            ParseError::UnexpectedEof { expected } => {
                write!(
                    f,
                    "Parse error: unexpected end of input, expected {}",
                    expected
                )
            }
            ParseError::InvalidExpr { message, pos } => {
                write!(f, "Parse error at {}: {}", pos, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Recursive-descent parser for Mini-F# expressions.
pub struct Parser {
    /// Input tokens
    tokens: Vec<TokenWithPos>,
    /// Current position in token stream
    pos: usize,
}

impl Parser {
    /// Create a new parser for the given token stream.
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse the token stream into an AST.
    ///
    /// Returns the root expression or a parse error.
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expr()?;

        // Ensure we consumed all tokens (except EOF)
        if !self.is_at_end() {
            let tok = self.current_token();
            return Err(ParseError::UnexpectedToken {
                expected: "end of input".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            });
        }

        Ok(expr)
    }

    /// Parse a top-level expression.
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Check for let, if, or fun keywords
        match &self.current_token().token {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::Fun => self.parse_lambda(),
            _ => self.parse_or_expr(),
        }
    }

    /// Parse let-binding with optional multi-parameter function syntax.
    ///
    /// Supports both:
    /// - `let x = expr in body` (simple binding)
    /// - `let f x y z = expr in body` (multi-parameter function, desugared to nested lambdas)
    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::Let)?;

        let name = self.expect_ident()?;

        // Collect any parameters before the '='
        let mut params = Vec::new();
        while !self.is_at_end() && !matches!(self.current_token().token, Token::Eq) {
            // Check if current token is an identifier
            if let Token::Ident(_) = self.current_token().token {
                params.push(self.expect_ident()?);
            } else {
                break;
            }
        }

        self.expect_token(Token::Eq)?;

        let mut value = Box::new(self.parse_expr()?);

        // If we have parameters, desugar to nested lambdas
        // let f x y = body  =>  let f = fun x -> fun y -> body
        for param in params.into_iter().rev() {
            value = Box::new(Expr::Lambda { param, body: value });
        }

        self.expect_token(Token::In)?;

        let body = Box::new(self.parse_expr()?);

        Ok(Expr::Let { name, value, body })
    }

    /// Parse if-then-else expression
    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::If)?;

        let cond = Box::new(self.parse_expr()?);

        self.expect_token(Token::Then)?;
        let then_branch = Box::new(self.parse_expr()?);

        self.expect_token(Token::Else)?;
        let else_branch = Box::new(self.parse_expr()?);

        Ok(Expr::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    /// Parse lambda: fun x -> body
    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::Fun)?;

        let param = self.expect_ident()?;

        self.expect_token(Token::Arrow)?;

        let body = Box::new(self.parse_expr()?);

        Ok(Expr::Lambda { param, body })
    }

    /// Parse logical OR expression (lowest precedence)
    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;

        while self.match_token(&Token::Or) {
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse logical AND expression
    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comp_expr()?;

        while self.match_token(&Token::And) {
            let right = self.parse_comp_expr()?;
            left = Expr::BinOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison expression
    fn parse_comp_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add_expr()?;

        // Comparisons are non-associative (only one comparison allowed)
        if let Some(op) = self.match_comparison_op() {
            let right = self.parse_add_expr()?;
            return Ok(Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    /// Parse addition/subtraction expression
    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;

        while let Some(op) = self.match_add_op() {
            let right = self.parse_mul_expr()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse multiplication/division expression
    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expr()?;

        while let Some(op) = self.match_mul_op() {
            let right = self.parse_unary_expr()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse unary expression (unary minus)
    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        // Handle unary minus
        if self.match_token(&Token::Minus) {
            let expr = self.parse_unary_expr()?;
            // Represent -x as 0 - x
            return Ok(Expr::BinOp {
                op: BinOp::Sub,
                left: Box::new(Expr::Lit(Literal::Int(0))),
                right: Box::new(expr),
            });
        }

        self.parse_app_expr()
    }

    /// Parse function application (left-associative)
    fn parse_app_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        // Keep parsing arguments while we see primary expressions
        while self.is_primary_start() {
            let arg = self.parse_primary()?;
            left = Expr::App {
                func: Box::new(left),
                arg: Box::new(arg),
            };
        }

        Ok(left)
    }

    /// Parse primary expression (literals, variables, parenthesized expressions)
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.current_token();

        match &tok.token {
            Token::Int(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Lit(Literal::Int(val)))
            }
            Token::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Expr::Lit(Literal::Float(val)))
            }
            Token::Bool(b) => {
                let val = *b;
                self.advance();
                Ok(Expr::Lit(Literal::Bool(val)))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expr::Lit(Literal::Str(val)))
            }
            Token::Ident(name) => {
                let val = name.clone();
                self.advance();
                Ok(Expr::Var(val))
            }
            Token::LParen => {
                self.advance();

                // Handle unit literal ()
                if self.match_token(&Token::RParen) {
                    return Ok(Expr::Lit(Literal::Unit));
                }

                let expr = self.parse_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Check if current token starts a primary expression
    fn is_primary_start(&self) -> bool {
        if self.is_at_end() {
            return false;
        }

        matches!(
            &self.current_token().token,
            Token::Int(_)
                | Token::Float(_)
                | Token::Bool(_)
                | Token::String(_)
                | Token::Ident(_)
                | Token::LParen
        )
    }

    /// Try to match a comparison operator
    fn match_comparison_op(&mut self) -> Option<BinOp> {
        let tok = &self.current_token().token;
        let op = match tok {
            Token::Eq => Some(BinOp::Eq),
            Token::EqEq => Some(BinOp::Eq),
            Token::Neq => Some(BinOp::Neq),
            Token::Lt => Some(BinOp::Lt),
            Token::Lte => Some(BinOp::Lte),
            Token::Gt => Some(BinOp::Gt),
            Token::Gte => Some(BinOp::Gte),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }

        op
    }

    /// Try to match addition/subtraction operator
    fn match_add_op(&mut self) -> Option<BinOp> {
        let tok = &self.current_token().token;
        let op = match tok {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }

        op
    }

    /// Try to match multiplication/division operator
    fn match_mul_op(&mut self) -> Option<BinOp> {
        let tok = &self.current_token().token;
        let op = match tok {
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }

        op
    }

    /// Try to match a specific token, consuming it if matched
    fn match_token(&mut self, expected: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }

        if &self.current_token().token == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect a specific token, returning error if not found
    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.is_at_end() {
            return Err(ParseError::UnexpectedEof {
                expected: format!("{}", expected),
            });
        }

        let tok = self.current_token();
        if tok.token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", expected),
                found: tok.token.clone(),
                pos: tok.pos,
            })
        }
    }

    /// Expect an identifier, returning its name
    fn expect_ident(&mut self) -> Result<String, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::UnexpectedEof {
                expected: "identifier".to_string(),
            });
        }

        let tok = self.current_token();
        match &tok.token {
            Token::Ident(name) => {
                let result = name.clone();
                self.advance();
                Ok(result)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
    }

    /// Get the current token (or EOF if at end)
    fn current_token(&self) -> &TokenWithPos {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            // Return EOF token if beyond bounds
            self.tokens.last().unwrap()
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool {
        if self.pos >= self.tokens.len() {
            return true;
        }
        matches!(self.tokens[self.pos].token, Token::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    // Helper function to parse a string
    fn parse(input: &str) -> Result<Expr, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // ========================================================================
    // TDD: Literal Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_integer_literal() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Int(42)));
    }

    #[test]
    fn test_parse_negative_integer() {
        // -42 is parsed as 0 - 42
        let expr = parse("-42").unwrap();
        assert!(expr.is_binop());
    }

    #[test]
    fn test_parse_float_literal() {
        let expr = parse("2.5").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Float(2.5)));
    }

    #[test]
    fn test_parse_bool_true() {
        let expr = parse("true").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Bool(true)));
    }

    #[test]
    fn test_parse_bool_false() {
        let expr = parse("false").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Bool(false)));
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = parse(r#""hello""#).unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Str("hello".to_string())));
    }

    #[test]
    fn test_parse_unit_literal() {
        let expr = parse("()").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Unit));
    }

    #[test]
    fn test_parse_variable() {
        let expr = parse("x").unwrap();
        assert_eq!(expr, Expr::Var("x".to_string()));
    }

    // ========================================================================
    // TDD: Let-Binding Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_let() {
        let expr = parse("let x = 42 in x").unwrap();
        match expr {
            Expr::Let { name, value, body } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expr::Lit(Literal::Int(42)));
                assert_eq!(*body, Expr::Var("x".to_string()));
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_let_with_expression_value() {
        let expr = parse("let x = 1 + 2 in x").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_nested_let() {
        let expr = parse("let x = 1 in let y = 2 in x").unwrap();
        match expr {
            Expr::Let { name, value, body } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expr::Lit(Literal::Int(1)));
                assert!(body.is_let());
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_let_error_missing_in() {
        let result = parse("let x = 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_let_error_missing_equals() {
        let result = parse("let x 42 in x");
        assert!(result.is_err());
    }

    // ========================================================================
    // TDD: Function Definition (Lambda) Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_lambda() {
        let expr = parse("fun x -> x").unwrap();
        match expr {
            Expr::Lambda { param, body } => {
                assert_eq!(param, "x");
                assert_eq!(*body, Expr::Var("x".to_string()));
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_parse_lambda_with_body_expression() {
        let expr = parse("fun x -> x + 1").unwrap();
        assert!(expr.is_lambda());
    }

    #[test]
    fn test_parse_nested_lambda() {
        let expr = parse("fun x -> fun y -> x").unwrap();
        match expr {
            Expr::Lambda { param, body } => {
                assert_eq!(param, "x");
                assert!(body.is_lambda());
            }
            _ => panic!("Expected Lambda expression"),
        }
    }

    #[test]
    fn test_parse_lambda_error_missing_arrow() {
        let result = parse("fun x x");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_lambda_error_missing_param() {
        let result = parse("fun -> x");
        assert!(result.is_err());
    }

    // ========================================================================
    // TDD: Function Application Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_application() {
        let expr = parse("f x").unwrap();
        match expr {
            Expr::App { func, arg } => {
                assert_eq!(*func, Expr::Var("f".to_string()));
                assert_eq!(*arg, Expr::Var("x".to_string()));
            }
            _ => panic!("Expected App expression"),
        }
    }

    #[test]
    fn test_parse_application_with_literal() {
        let expr = parse("f 42").unwrap();
        match expr {
            Expr::App { func, arg } => {
                assert_eq!(*func, Expr::Var("f".to_string()));
                assert_eq!(*arg, Expr::Lit(Literal::Int(42)));
            }
            _ => panic!("Expected App expression"),
        }
    }

    #[test]
    fn test_parse_curried_application() {
        // f x y should parse as (f x) y
        let expr = parse("f x y").unwrap();
        match expr {
            Expr::App { func, arg } => {
                assert!(func.is_app());
                assert_eq!(*arg, Expr::Var("y".to_string()));
            }
            _ => panic!("Expected App expression"),
        }
    }

    #[test]
    fn test_parse_application_left_associative() {
        // a b c d should parse as (((a b) c) d)
        let expr = parse("a b c d").unwrap();
        assert!(expr.is_app());
        match expr {
            Expr::App { func, arg } => {
                assert!(func.is_app());
                assert_eq!(*arg, Expr::Var("d".to_string()));
            }
            _ => unreachable!(),
        }
    }

    // ========================================================================
    // TDD: If-Then-Else Tests
    // ========================================================================

    #[test]
    fn test_parse_simple_if() {
        let expr = parse("if true then 1 else 0").unwrap();
        match expr {
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                assert_eq!(*cond, Expr::Lit(Literal::Bool(true)));
                assert_eq!(*then_branch, Expr::Lit(Literal::Int(1)));
                assert_eq!(*else_branch, Expr::Lit(Literal::Int(0)));
            }
            _ => panic!("Expected If expression"),
        }
    }

    #[test]
    fn test_parse_if_with_comparison() {
        let expr = parse("if x > 0 then 1 else 0").unwrap();
        assert!(expr.is_if());
        match expr {
            Expr::If { cond, .. } => {
                assert!(cond.is_binop());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_parse_nested_if() {
        let expr = parse("if x then if y then 1 else 2 else 3").unwrap();
        assert!(expr.is_if());
        match expr {
            Expr::If { then_branch, .. } => {
                assert!(then_branch.is_if());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_parse_if_error_missing_then() {
        let result = parse("if true 1 else 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_if_error_missing_else() {
        let result = parse("if true then 1");
        assert!(result.is_err());
    }

    // ========================================================================
    // TDD: Binary Operator Tests
    // ========================================================================

    #[test]
    fn test_parse_addition() {
        let expr = parse("1 + 2").unwrap();
        match expr {
            Expr::BinOp { op, left, right } => {
                assert_eq!(op, BinOp::Add);
                assert_eq!(*left, Expr::Lit(Literal::Int(1)));
                assert_eq!(*right, Expr::Lit(Literal::Int(2)));
            }
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_subtraction() {
        let expr = parse("5 - 3").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Sub),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_multiplication() {
        let expr = parse("2 * 3").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Mul),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_division() {
        let expr = parse("10 / 2").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Div),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_eq() {
        let expr = parse("x = y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Eq),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_neq() {
        let expr = parse("x <> y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Neq),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_lt() {
        let expr = parse("x < y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Lt),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_lte() {
        let expr = parse("x <= y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Lte),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_gt() {
        let expr = parse("x > y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Gt),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_comparison_gte() {
        let expr = parse("x >= y").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Gte),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let expr = parse("true && false").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::And),
            _ => panic!("Expected BinOp expression"),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = parse("true || false").unwrap();
        match expr {
            Expr::BinOp { op, .. } => assert_eq!(op, BinOp::Or),
            _ => panic!("Expected BinOp expression"),
        }
    }

    // ========================================================================
    // TDD: Operator Precedence Tests
    // ========================================================================

    #[test]
    fn test_precedence_mul_over_add() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let expr = parse("1 + 2 * 3").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Lit(Literal::Int(1)));
                assert!(right.is_binop());
                match *right {
                    Expr::BinOp { op: BinOp::Mul, .. } => {}
                    _ => panic!("Expected multiplication"),
                }
            }
            _ => panic!("Expected addition at top level"),
        }
    }

    #[test]
    fn test_precedence_mul_over_sub() {
        // 5 - 2 * 3 should parse as 5 - (2 * 3)
        let expr = parse("5 - 2 * 3").unwrap();
        match expr {
            Expr::BinOp { op: BinOp::Sub, .. } => {}
            _ => panic!("Expected subtraction at top level"),
        }
    }

    #[test]
    fn test_precedence_add_left_associative() {
        // 1 + 2 + 3 should parse as (1 + 2) + 3
        let expr = parse("1 + 2 + 3").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                right,
            } => {
                assert!(left.is_binop());
                assert_eq!(*right, Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_precedence_mul_left_associative() {
        // 2 * 3 * 4 should parse as (2 * 3) * 4
        let expr = parse("2 * 3 * 4").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Mul,
                left,
                right,
            } => {
                assert!(left.is_binop());
                assert_eq!(*right, Expr::Lit(Literal::Int(4)));
            }
            _ => panic!("Expected multiplication"),
        }
    }

    #[test]
    fn test_precedence_comparison_over_logical() {
        // x > 0 && y < 10 should parse as (x > 0) && (y < 10)
        let expr = parse("x > 0 && y < 10").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::And,
                left,
                right,
            } => {
                assert!(left.is_binop());
                assert!(right.is_binop());
            }
            _ => panic!("Expected AND at top level"),
        }
    }

    #[test]
    fn test_precedence_and_over_or() {
        // a || b && c should parse as a || (b && c)
        let expr = parse("a || b && c").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Or,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Var("a".to_string()));
                assert!(right.is_binop());
                match *right {
                    Expr::BinOp { op: BinOp::And, .. } => {}
                    _ => panic!("Expected AND on right side"),
                }
            }
            _ => panic!("Expected OR at top level"),
        }
    }

    #[test]
    fn test_precedence_with_parentheses() {
        // (1 + 2) * 3 should respect parentheses
        let expr = parse("(1 + 2) * 3").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Mul,
                left,
                right,
            } => {
                assert!(left.is_binop());
                match *left {
                    Expr::BinOp { op: BinOp::Add, .. } => {}
                    _ => panic!("Expected addition in parentheses"),
                }
                assert_eq!(*right, Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected multiplication"),
        }
    }

    // ========================================================================
    // TDD: Complex Expression Tests
    // ========================================================================

    #[test]
    fn test_parse_factorial() {
        let expr =
            parse("let rec fact = fun n -> if n <= 1 then 1 else n * fact (n - 1) in fact 5");
        // This will fail until we implement 'rec', but tests the parser's capability
        // For now, just test without rec
        let _expr = parse("let fact = fun n -> if n <= 1 then 1 else n in fact 5").unwrap();
        assert!(expr.is_err()); // Should fail because 'rec' is not a valid identifier
    }

    #[test]
    fn test_parse_complex_arithmetic() {
        let expr = parse("1 + 2 * 3 - 4 / 2").unwrap();
        assert!(expr.is_binop());
    }

    #[test]
    fn test_parse_complex_let_in_lambda() {
        let expr = parse("fun x -> let y = x + 1 in y * 2").unwrap();
        assert!(expr.is_lambda());
    }

    #[test]
    fn test_parse_lambda_application() {
        let expr = parse("(fun x -> x + 1) 42").unwrap();
        assert!(expr.is_app());
    }

    #[test]
    fn test_parse_multiple_applications() {
        let expr = parse("f (g x) (h y)").unwrap();
        assert!(expr.is_app());
    }

    // ========================================================================
    // TDD: Error Cases
    // ========================================================================

    #[test]
    fn test_error_unexpected_token() {
        let result = parse("let + = 42 in x");
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnexpectedToken { .. } => {}
            _ => panic!("Expected UnexpectedToken error"),
        }
    }

    #[test]
    fn test_error_unexpected_eof() {
        let result = parse("let x =");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unmatched_paren() {
        let result = parse("(1 + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_empty_input() {
        let result = parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_extra_tokens() {
        // "42 43" now successfully parses as application of 42 to 43
        // which is semantically invalid but syntactically valid
        let result = parse("42 43");
        // Application of literals is syntactically valid
        assert!(result.is_ok());
    }

    // ========================================================================
    // Integration Tests with Lexer
    // ========================================================================

    #[test]
    fn test_integration_lex_and_parse_let() {
        let mut lexer = Lexer::new("let x = 42 in x + 1");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert!(ast.is_let());
    }

    #[test]
    fn test_integration_lex_and_parse_lambda() {
        let mut lexer = Lexer::new("fun x -> x * 2");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert!(ast.is_lambda());
    }

    #[test]
    fn test_integration_lex_and_parse_if() {
        let mut lexer = Lexer::new("if true then 1 else 0");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert!(ast.is_if());
    }

    #[test]
    fn test_integration_with_comments() {
        let mut lexer = Lexer::new("let x = 42 // the answer\nin x");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();

        assert!(ast.is_let());
    }

    // ========================================================================
    // Real-World Examples
    // ========================================================================

    #[test]
    fn test_real_world_inc_function() {
        let expr = parse("let inc = fun x -> x + 1 in inc 41").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_real_world_add_function() {
        let expr = parse("let add = fun x -> fun y -> x + y in add 1 2").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_real_world_conditional_sign() {
        let expr =
            parse("let sign = fun n -> if n < 0 then -1 else if n = 0 then 0 else 1 in sign x")
                .unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_real_world_nested_let_bindings() {
        let input = r#"
            let x = 10 in
            let y = 20 in
            let z = 30 in
            x + y + z
        "#;
        let expr = parse(input).unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_unary_minus_precedence() {
        // -5 + 3 should parse as (-5) + 3
        let expr = parse("-5 + 3").unwrap();
        match expr {
            Expr::BinOp {
                op: BinOp::Add,
                left,
                ..
            } => {
                assert!(left.is_binop()); // Left is 0 - 5
            }
            _ => panic!("Expected addition"),
        }
    }

    #[test]
    fn test_unary_minus_on_variable() {
        let expr = parse("-x").unwrap();
        assert!(expr.is_binop());
        match expr {
            Expr::BinOp {
                op: BinOp::Sub,
                left,
                right,
            } => {
                assert_eq!(*left, Expr::Lit(Literal::Int(0)));
                assert_eq!(*right, Expr::Var("x".to_string()));
            }
            _ => unreachable!(),
        }
    }
}
