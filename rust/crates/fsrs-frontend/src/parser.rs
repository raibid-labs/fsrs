//! Recursive-descent parser for Mini-F# expressions.
//!
//! This module implements a recursive-descent parser that converts a stream of tokens
//! from the lexer into an Abstract Syntax Tree (AST). The parser supports:
//!
//! - Literals: integers, floats, booleans, strings, unit
//! - Variables and identifiers
//! - Let-bindings: `let x = expr in body`
//! - Multi-parameter functions (curried): `let f x y = expr in body`
//! - Lambda functions: `fun x -> body`
//! - Function application: `f x y`
//! - Binary operations: arithmetic, comparison, logical
//! - Conditional expressions: `if cond then expr1 else expr2`
//! - Tuples: `(1, 2)`, `(x, y, z)`, `(42,)` (single-element)
//! - Lists: `[1; 2; 3]`, `[]`
//! - Arrays: `[|1; 2; 3|]`, `arr.[0]`, `arr.[0] <- 99`
//! - Records: `type Person = { name: string }`, `{ name = "John" }`, `person.name`
//! - Cons operator: `1 :: [2; 3]`, `x :: xs`
//! - Unary minus: `-42`, `-x`
//! - Proper operator precedence
//! - Error recovery and reporting
//!
//! module     ::= declaration*
//! declaration::= type_def | let_binding
//! type_def   ::= "type" IDENT "=" "{" (IDENT ":" type_expr (";" IDENT ":" type_expr)* ";"?)? "}"
//! type_expr  ::= simple_type ("->" type_expr)? | simple_type ("*" simple_type)*
//! simple_type::= IDENT
//! let_binding::= "let" IDENT IDENT* "=" expr
//! # Grammar (Simplified)
//!
//! ```text
//! expr       ::= let_expr | if_expr | lambda_expr | or_expr
//! let_expr   ::= "let" IDENT IDENT* "=" expr "in" expr
//! if_expr    ::= "if" expr "then" expr "else" expr
//! lambda_expr::= "fun" IDENT "->" expr
//! or_expr    ::= and_expr ("||" and_expr)*
//! and_expr   ::= comp_expr ("&&" comp_expr)*
//! comp_expr  ::= cons_expr (("=" | "==" | "<>" | "<" | "<=" | ">" | ">=") cons_expr)?
//! cons_expr  ::= add_expr ("::" cons_expr)?
//! add_expr   ::= mul_expr (("+" | "-") mul_expr)*
//! mul_expr   ::= unary_expr (("*" | "/") unary_expr)*
//! unary_expr ::= "-" unary_expr | app_expr
//! app_expr   ::= postfix_expr (postfix_expr)*
//! postfix_expr ::= primary (".[" expr "]" ("<-" expr)?)*
//! primary    ::= INT | FLOAT | BOOL | STRING | IDENT | "(" expr ")" | tuple | list | array | "Array.length" primary
//! tuple      ::= "(" ")" | "(" expr ("," expr)* ","? ")"
//! list       ::= "[" "]" | "[" expr (";" expr)* ";"? "]"
//! array      ::= "[|" "]" | "[|" expr (";" expr)* ";"? "|]"
//! record_literal ::= "{" (IDENT "=" expr (";" IDENT "=" expr)* ";"?)? "}"
//! record_update  ::= "{" expr "with" IDENT "=" expr (";" IDENT "=" expr)* ";"? "}"
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
use crate::ast::{BinOp, Expr, Literal, MatchArm, Pattern};
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
            Token::Match => self.parse_match(),
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

        // Check for 'rec' keyword
        let is_rec = self.match_token(&Token::Rec);

        if is_rec {
            return self.parse_let_rec();
        }

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

    /// Parse recursive let-binding: `let rec name = expr in body`
    /// or mutually recursive: `let rec f = ... and g = ... in body`
    fn parse_let_rec(&mut self) -> Result<Expr, ParseError> {
        // Parse first binding
        let name = self.expect_ident()?;

        // Collect any parameters before the '='
        let mut params = Vec::new();
        while !self.is_at_end()
            && !matches!(self.current_token().token, Token::Eq | Token::AndKeyword)
        {
            if let Token::Ident(_) = self.current_token().token {
                params.push(self.expect_ident()?);
            } else {
                break;
            }
        }

        self.expect_token(Token::Eq)?;

        let mut value = Box::new(self.parse_expr()?);

        // Desugar parameters to nested lambdas
        for param in params.into_iter().rev() {
            value = Box::new(Expr::Lambda { param, body: value });
        }

        // Collect first binding
        let mut bindings = vec![(name.clone(), *value.clone())];

        // Check for 'and' keyword for mutual recursion
        while self.match_token(&Token::AndKeyword) {
            let and_name = self.expect_ident()?;

            // Collect parameters for this binding
            let mut and_params = Vec::new();
            while !self.is_at_end()
                && !matches!(self.current_token().token, Token::Eq | Token::AndKeyword)
            {
                if let Token::Ident(_) = self.current_token().token {
                    and_params.push(self.expect_ident()?);
                } else {
                    break;
                }
            }

            self.expect_token(Token::Eq)?;

            let mut and_value = Box::new(self.parse_expr()?);

            // Desugar parameters
            for param in and_params.into_iter().rev() {
                and_value = Box::new(Expr::Lambda {
                    param,
                    body: and_value,
                });
            }

            bindings.push((and_name, *and_value));
        }

        self.expect_token(Token::In)?;

        let body = Box::new(self.parse_expr()?);

        // If single binding, use LetRec; otherwise LetRecMutual
        if bindings.len() == 1 {
            let (name, value) = bindings.into_iter().next().unwrap();
            Ok(Expr::LetRec {
                name,
                value: Box::new(value),
                body,
            })
        } else {
            Ok(Expr::LetRecMutual { bindings, body })
        }
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

    /// Parse match expression: `match expr with | pattern -> expr | pattern -> expr ...`
    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::Match)?;

        // Parse scrutinee
        let scrutinee = Box::new(self.parse_or_expr()?);

        self.expect_token(Token::With)?;

        // Expect first pipe
        self.expect_token(Token::Pipe)?;

        let mut arms = vec![];

        // Parse arms
        loop {
            // Parse pattern
            let pattern = self.parse_pattern()?;

            // Expect arrow
            self.expect_token(Token::Arrow)?;

            // Parse body expression
            let body = Box::new(self.parse_expr()?);

            arms.push(MatchArm { pattern, body });

            // Check for more arms
            if self.match_token(&Token::Pipe) {
                // Continue parsing next arm
            } else {
                // No more arms
                break;
            }
        }

        Ok(Expr::Match { scrutinee, arms })
    }

    /// Parse pattern: `_` | IDENT | INT | BOOL | STRING | `(` pattern (, pattern)* `)`
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let tok = self.current_token();

        match &tok.token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Var(name))
            }
            Token::Int(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            Token::Bool(b) => {
                let val = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(val)))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::Str(val)))
            }
            Token::LParen => {
                self.advance(); // consume '('

                // Empty tuple pattern ()
                if self.match_token(&Token::RParen) {
                    return Ok(Pattern::Tuple(vec![]));
                }

                // Parse first pattern
                let first_pattern = self.parse_pattern()?;

                // Check if it's a tuple (has comma) or grouped pattern (no comma)
                if self.match_token(&Token::Comma) {
                    // It's a tuple pattern: (p1, p2, ...)
                    let mut patterns = vec![first_pattern];

                    // Check for trailing comma or continue with more patterns
                    if !matches!(self.current_token().token, Token::RParen) {
                        // Parse remaining patterns
                        loop {
                            patterns.push(self.parse_pattern()?);

                            if self.match_token(&Token::Comma) {
                                // Check for trailing comma before RParen
                                if matches!(self.current_token().token, Token::RParen) {
                                    break;
                                }
                                // Otherwise continue parsing
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect_token(Token::RParen)?;
                    Ok(Pattern::Tuple(patterns))
                } else {
                    // No comma, it's a grouped pattern: (p)
                    self.expect_token(Token::RParen)?;
                    Ok(first_pattern)
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern (wildcard, identifier, literal, or tuple)".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
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
        let left = self.parse_cons_expr()?;

        // Comparisons are non-associative (only one comparison allowed)
        if let Some(op) = self.match_comparison_op() {
            let right = self.parse_cons_expr()?;
            return Ok(Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    /// Parse cons expression (right-associative)
    /// cons_expr ::= add_expr ("::" cons_expr)?
    fn parse_cons_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add_expr()?;

        // Right-associative: 1 :: 2 :: [] parses as 1 :: (2 :: [])
        if self.match_token(&Token::ColonColon) {
            let tail = self.parse_cons_expr()?; // Recursive call for right-associativity
            return Ok(Expr::Cons {
                head: Box::new(left),
                tail: Box::new(tail),
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
        let mut left = self.parse_postfix_expr()?;

        // Keep parsing arguments while we see primary expressions
        while self.is_primary_start() {
            let arg = self.parse_postfix_expr()?;
            left = Expr::App {
                func: Box::new(left),
                arg: Box::new(arg),
            };
        }

        Ok(left)
    }

    /// Parse postfix expression (array indexing and updates)
    /// postfix_expr ::= primary (".[" expr "]" ("<-" expr)?)*
    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        // Handle array indexing and updates: arr.[idx] or arr.[idx] <- value
        while self.match_token(&Token::Dot) {
            if !self.match_token(&Token::LBracket) {
                let tok = self.current_token();
                return Err(ParseError::UnexpectedToken {
                    expected: "'[' for array indexing".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                });
            }

            let index = Box::new(self.parse_expr()?);
            self.expect_token(Token::RBracket)?;

            // Check for update: arr.[idx] <- value
            if self.match_token(&Token::LArrow) {
                let value = Box::new(self.parse_expr()?);
                expr = Expr::ArrayUpdate {
                    array: Box::new(expr),
                    index,
                    value,
                };
            } else {
                expr = Expr::ArrayIndex {
                    array: Box::new(expr),
                    index,
                };
            }
        }

        Ok(expr)
    }

    /// Parse primary expression (literals, variables, parenthesized expressions, tuples, lists)
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

                // Check for Array.length
                if val == "Array" && self.match_token(&Token::Dot) {
                    let method = self.expect_ident()?;
                    if method == "length" {
                        let arr = Box::new(self.parse_postfix_expr()?);
                        return Ok(Expr::ArrayLength(arr));
                    } else {
                        let tok = self.current_token();
                        return Err(ParseError::InvalidExpr {
                            message: format!("Unknown Array method: {}", method),
                            pos: tok.pos,
                        });
                    }
                }

                Ok(Expr::Var(val))
            }
            Token::LParen => {
                self.advance(); // consume '('

                // Handle empty tuple/unit: ()
                if self.match_token(&Token::RParen) {
                    return Ok(Expr::Lit(Literal::Unit));
                }

                // Parse first expression
                let first_expr = self.parse_expr()?;

                // Check if it's a tuple (has comma) or grouped expression (no comma)
                if self.match_token(&Token::Comma) {
                    // It's a tuple: (e1, e2, ...)
                    let mut elements = vec![first_expr];

                    // Check for trailing comma: (e,) or continue with more elements
                    if !matches!(self.current_token().token, Token::RParen) {
                        // Parse remaining elements
                        loop {
                            elements.push(self.parse_expr()?);

                            if self.match_token(&Token::Comma) {
                                // Check for trailing comma before RParen
                                if matches!(self.current_token().token, Token::RParen) {
                                    break;
                                }
                                // Otherwise continue parsing
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect_token(Token::RParen)?;
                    Ok(Expr::Tuple(elements))
                } else {
                    // No comma, it's a grouped expression: (e)
                    self.expect_token(Token::RParen)?;
                    Ok(first_expr)
                }
            }
            Token::LBracket => {
                self.advance(); // consume '['

                let mut elements = vec![];

                // Empty list: []
                if self.match_token(&Token::RBracket) {
                    return Ok(Expr::List(elements));
                }

                // Parse first element
                elements.push(self.parse_expr()?);

                // Parse remaining elements: ; e1 ; e2 ...
                while self.match_token(&Token::Semicolon) {
                    // Check for trailing semicolon before RBracket
                    if matches!(self.current_token().token, Token::RBracket) {
                        break; // trailing semicolon
                    }
                    elements.push(self.parse_expr()?);
                }

                self.expect_token(Token::RBracket)?;
                Ok(Expr::List(elements))
            }
            Token::LBracketPipe => {
                self.advance(); // consume '[|'
                let mut elements = vec![];

                // Empty array: [||]
                if self.match_token(&Token::PipeRBracket) {
                    return Ok(Expr::Array(elements));
                }

                // Parse first element
                elements.push(self.parse_expr()?);

                // Parse remaining elements: ; e1 ; e2 ...
                while self.match_token(&Token::Semicolon) {
                    // Check for trailing semicolon before |]
                    if matches!(self.current_token().token, Token::PipeRBracket) {
                        break; // trailing semicolon
                    }
                    elements.push(self.parse_expr()?);
                }

                self.expect_token(Token::PipeRBracket)?;
                Ok(Expr::Array(elements))
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
                | Token::LBracket
                | Token::LBracketPipe
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
        // Now that 'rec' is implemented, this should parse successfully
        let expr =
            parse("let rec fact = fun n -> if n <= 1 then 1 else n * fact (n - 1) in fact 5");
        assert!(expr.is_ok());
        assert!(expr.unwrap().is_let_rec());
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

    // ========================================================================
    // Tuple Parser Tests (Issue #24 Layer 2)
    // ========================================================================

    #[test]
    fn test_parse_tuple_empty() {
        // () is unit literal, not empty tuple
        let expr = parse("()").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Unit));
        assert!(!expr.is_tuple());
    }

    #[test]
    fn test_parse_tuple_pair() {
        // (1, 2)
        let expr = parse("(1, 2)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert_eq!(elements[1], Expr::Lit(Literal::Int(2)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_triple() {
        // (1, 2, 3)
        let expr = parse("(1, 2, 3)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert_eq!(elements[1], Expr::Lit(Literal::Int(2)));
                assert_eq!(elements[2], Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_single_element() {
        // (42,) is a single-element tuple
        let expr = parse("(42,)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(42)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_grouped_expression() {
        // (42) is just 42, not a tuple
        let expr = parse("(42)").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Int(42)));
        assert!(!expr.is_tuple());
    }

    #[test]
    fn test_parse_tuple_nested() {
        // (1, (2, 3))
        let expr = parse("(1, (2, 3))").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert!(elements[1].is_tuple());
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_with_variables() {
        // (x, y, z)
        let expr = parse("(x, y, z)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Var("x".to_string()));
                assert_eq!(elements[1], Expr::Var("y".to_string()));
                assert_eq!(elements[2], Expr::Var("z".to_string()));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_with_expressions() {
        // (x + 1, y * 2)
        let expr = parse("(x + 1, y * 2)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_binop());
                assert!(elements[1].is_binop());
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_trailing_comma() {
        // (1, 2,) should be same as (1, 2)
        let expr = parse("(1, 2,)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_deeply_nested() {
        // ((1, 2), (3, 4))
        let expr = parse("((1, 2), (3, 4))").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_tuple());
                assert!(elements[1].is_tuple());
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_mixed_types() {
        // (1, "hello", true)
        let expr = parse(r#"(1, "hello", true)"#).unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert_eq!(elements[1], Expr::Lit(Literal::Str("hello".to_string())));
                assert_eq!(elements[2], Expr::Lit(Literal::Bool(true)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_with_let() {
        // (let x = 1 in x, 2)
        let expr = parse("(let x = 1 in x, 2)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_let());
                assert_eq!(elements[1], Expr::Lit(Literal::Int(2)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_with_lambda() {
        // (fun x -> x + 1, 42)
        let expr = parse("(fun x -> x + 1, 42)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_lambda());
                assert_eq!(elements[1], Expr::Lit(Literal::Int(42)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_with_if() {
        // (if true then 1 else 2, 3)
        let expr = parse("(if true then 1 else 2, 3)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_if());
                assert_eq!(elements[1], Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_tuple_in_application() {
        // f (1, 2) should parse as application of f to tuple (1, 2)
        let expr = parse("f (1, 2)").unwrap();
        assert!(expr.is_app());
        match expr {
            Expr::App { func, arg } => {
                assert_eq!(*func, Expr::Var("f".to_string()));
                assert!(arg.is_tuple());
            }
            _ => panic!("Expected App"),
        }
    }

    #[test]
    fn test_parse_tuple_large() {
        // (1, 2, 3, 4, 5, 6, 7, 8)
        let expr = parse("(1, 2, 3, 4, 5, 6, 7, 8)").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 8);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    // ========================================================================
    // List Parser Tests (Issue #25 Layer 2)
    // ========================================================================

    #[test]
    fn test_parse_list_empty() {
        // []
        let expr = parse("[]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_single() {
        // [1]
        let expr = parse("[1]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_multiple() {
        // [1; 2; 3]
        let expr = parse("[1; 2; 3]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert_eq!(elements[1], Expr::Lit(Literal::Int(2)));
                assert_eq!(elements[2], Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_trailing_semicolon() {
        // [1; 2;]
        let expr = parse("[1; 2;]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_nested() {
        // [1; [2; 3]]
        let expr = parse("[1; [2; 3]]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert!(elements[1].is_list());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_cons_empty() {
        // 1 :: []
        let expr = parse("1 :: []").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, tail } => {
                assert_eq!(*head, Expr::Lit(Literal::Int(1)));
                assert!(tail.is_list());
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_parse_cons_list() {
        // 1 :: [2; 3]
        let expr = parse("1 :: [2; 3]").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, tail } => {
                assert_eq!(*head, Expr::Lit(Literal::Int(1)));
                assert!(tail.is_list());
                match *tail {
                    Expr::List(elements) => {
                        assert_eq!(elements.len(), 2);
                    }
                    _ => panic!("Expected list in tail"),
                }
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_parse_cons_chain_right_assoc() {
        // 1 :: 2 :: [] should parse as 1 :: (2 :: [])
        let expr = parse("1 :: 2 :: []").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, tail } => {
                assert_eq!(*head, Expr::Lit(Literal::Int(1)));
                assert!(tail.is_cons()); // Right-associative
                match *tail {
                    Expr::Cons { head, tail } => {
                        assert_eq!(*head, Expr::Lit(Literal::Int(2)));
                        assert!(tail.is_list());
                    }
                    _ => panic!("Expected cons in tail"),
                }
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_parse_cons_in_list() {
        // [1 :: [2; 3]]
        let expr = parse("[1 :: [2; 3]]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 1);
                assert!(elements[0].is_cons());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_mixed_operators_list_cons() {
        // x + 1 :: [y * 2]
        let expr = parse("x + 1 :: [y * 2]").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, tail } => {
                assert!(head.is_binop());
                assert!(tail.is_list());
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_parse_list_with_let() {
        // let xs = [1; 2] in xs
        let expr = parse("let xs = [1; 2] in xs").unwrap();
        assert!(expr.is_let());
        match expr {
            Expr::Let { value, .. } => {
                assert!(value.is_list());
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_parse_list_with_lambda() {
        // let f = fun x -> [x; x + 1] in f 1
        let expr = parse("let f = fun x -> [x; x + 1] in f 1").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_list_with_variables() {
        // [x; y; z]
        let expr = parse("[x; y; z]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Var("x".to_string()));
                assert_eq!(elements[1], Expr::Var("y".to_string()));
                assert_eq!(elements[2], Expr::Var("z".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_with_expressions() {
        // [x + 1; y * 2]
        let expr = parse("[x + 1; y * 2]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_binop());
                assert!(elements[1].is_binop());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_cons_with_variables() {
        // x :: xs
        let expr = parse("x :: xs").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, tail } => {
                assert_eq!(*head, Expr::Var("x".to_string()));
                assert_eq!(*tail, Expr::Var("xs".to_string()));
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_precedence_cons_lower_than_arithmetic() {
        // 1 + 2 :: [3] should parse as (1 + 2) :: [3]
        let expr = parse("1 + 2 :: [3]").unwrap();
        assert!(expr.is_cons());
        match expr {
            Expr::Cons { head, .. } => {
                assert!(head.is_binop());
            }
            _ => panic!("Expected Cons"),
        }
    }

    #[test]
    fn test_precedence_cons_higher_than_comparison() {
        // x :: xs = y :: ys should parse as (x :: xs) = (y :: ys)
        let expr = parse("x :: xs = y :: ys").unwrap();
        assert!(expr.is_binop());
        match expr {
            Expr::BinOp {
                op: BinOp::Eq,
                left,
                right,
            } => {
                assert!(left.is_cons());
                assert!(right.is_cons());
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_list_large() {
        // [1; 2; 3; 4; 5; 6; 7; 8]
        let expr = parse("[1; 2; 3; 4; 5; 6; 7; 8]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 8);
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_list_of_tuples() {
        // [(1, 2); (3, 4)]
        let expr = parse("[(1, 2); (3, 4)]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_tuple());
                assert!(elements[1].is_tuple());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_tuple_of_lists() {
        // ([1; 2], [3; 4])
        let expr = parse("([1; 2], [3; 4])").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_list());
                assert!(elements[1].is_list());
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_cons_precedence_with_comparison() {
        // 1 :: 2 :: [] = [1; 2] should parse as (1 :: (2 :: [])) = [1; 2]
        let expr = parse("1 :: 2 :: [] = [1; 2]").unwrap();
        assert!(expr.is_binop());
    }

    #[test]
    fn test_parse_list_with_if_elements() {
        // [if true then 1 else 0; 2]
        let expr = parse("[if true then 1 else 0; 2]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_if());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_complex_cons_chain() {
        // 1 :: 2 :: 3 :: []
        let expr = parse("1 :: 2 :: 3 :: []").unwrap();
        assert!(expr.is_cons());
        // Check right-associativity: 1 :: (2 :: (3 :: []))
        match expr {
            Expr::Cons { head, tail } => {
                assert_eq!(*head, Expr::Lit(Literal::Int(1)));
                assert!(tail.is_cons());
            }
            _ => panic!("Expected Cons"),
        }
    }

    // ========================================================================
    // Array Parser Tests (Issue #26 Layer 2)
    // ========================================================================

    #[test]
    fn test_parse_array_empty() {
        // [||]
        let expr = parse("[||]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_single_element() {
        // [|42|]
        let expr = parse("[|42|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(42)));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_multi_element() {
        // [|1; 2; 3|]
        let expr = parse("[|1; 2; 3|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert_eq!(elements[1], Expr::Lit(Literal::Int(2)));
                assert_eq!(elements[2], Expr::Lit(Literal::Int(3)));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_trailing_semicolon() {
        // [|1; 2;|]
        let expr = parse("[|1; 2;|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_nested() {
        // [|1; [|2; 3|]|]
        let expr = parse("[|1; [|2; 3|]|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], Expr::Lit(Literal::Int(1)));
                assert!(elements[1].is_array());
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_with_variables() {
        // [|x; y; z|]
        let expr = parse("[|x; y; z|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 3);
                assert_eq!(elements[0], Expr::Var("x".to_string()));
                assert_eq!(elements[1], Expr::Var("y".to_string()));
                assert_eq!(elements[2], Expr::Var("z".to_string()));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_with_expressions() {
        // [|x + 1; y * 2|]
        let expr = parse("[|x + 1; y * 2|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_binop());
                assert!(elements[1].is_binop());
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_index_simple() {
        // arr.[0]
        let expr = parse("arr.[0]").unwrap();
        assert!(expr.is_array_index());
        match expr {
            Expr::ArrayIndex { array, index } => {
                assert_eq!(*array, Expr::Var("arr".to_string()));
                assert_eq!(*index, Expr::Lit(Literal::Int(0)));
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }

    #[test]
    fn test_parse_array_index_variable() {
        // arr.[i]
        let expr = parse("arr.[i]").unwrap();
        assert!(expr.is_array_index());
        match expr {
            Expr::ArrayIndex { array, index } => {
                assert_eq!(*array, Expr::Var("arr".to_string()));
                assert_eq!(*index, Expr::Var("i".to_string()));
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }

    #[test]
    fn test_parse_array_index_expression() {
        // arr.[i + 1]
        let expr = parse("arr.[i + 1]").unwrap();
        assert!(expr.is_array_index());
        match expr {
            Expr::ArrayIndex { array, index } => {
                assert_eq!(*array, Expr::Var("arr".to_string()));
                assert!(index.is_binop());
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }

    #[test]
    fn test_parse_array_index_chained() {
        // matrix.[i].[j]
        let expr = parse("matrix.[i].[j]").unwrap();
        assert!(expr.is_array_index());
        match expr {
            Expr::ArrayIndex { array, index } => {
                assert!(array.is_array_index());
                assert_eq!(*index, Expr::Var("j".to_string()));
            }
            _ => panic!("Expected ArrayIndex"),
        }
    }

    #[test]
    fn test_parse_array_update_simple() {
        // arr.[0] <- 99
        let expr = parse("arr.[0] <- 99").unwrap();
        assert!(expr.is_array_update());
        match expr {
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => {
                assert_eq!(*array, Expr::Var("arr".to_string()));
                assert_eq!(*index, Expr::Lit(Literal::Int(0)));
                assert_eq!(*value, Expr::Lit(Literal::Int(99)));
            }
            _ => panic!("Expected ArrayUpdate"),
        }
    }

    #[test]
    fn test_parse_array_update_with_expression() {
        // arr.[i] <- x + 1
        let expr = parse("arr.[i] <- x + 1").unwrap();
        assert!(expr.is_array_update());
        match expr {
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => {
                assert_eq!(*array, Expr::Var("arr".to_string()));
                assert_eq!(*index, Expr::Var("i".to_string()));
                assert!(value.is_binop());
            }
            _ => panic!("Expected ArrayUpdate"),
        }
    }

    #[test]
    fn test_parse_array_update_chained() {
        // matrix.[i].[j] <- 42
        let expr = parse("matrix.[i].[j] <- 42").unwrap();
        assert!(expr.is_array_update());
        match expr {
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => {
                assert!(array.is_array_index());
                assert_eq!(*index, Expr::Var("j".to_string()));
                assert_eq!(*value, Expr::Lit(Literal::Int(42)));
            }
            _ => panic!("Expected ArrayUpdate"),
        }
    }

    #[test]
    fn test_parse_array_length_simple() {
        // Array.length arr
        let expr = parse("Array.length arr").unwrap();
        assert!(expr.is_array_length());
        match expr {
            Expr::ArrayLength(arr) => {
                assert_eq!(*arr, Expr::Var("arr".to_string()));
            }
            _ => panic!("Expected ArrayLength"),
        }
    }

    #[test]
    fn test_parse_array_length_with_literal() {
        // Array.length [|1; 2; 3|]
        let expr = parse("Array.length [|1; 2; 3|]").unwrap();
        assert!(expr.is_array_length());
        match expr {
            Expr::ArrayLength(arr) => {
                assert!(arr.is_array());
            }
            _ => panic!("Expected ArrayLength"),
        }
    }

    #[test]
    fn test_parse_array_length_with_index() {
        // Array.length arr.[0]
        let expr = parse("Array.length arr.[0]").unwrap();
        assert!(expr.is_array_length());
        match expr {
            Expr::ArrayLength(arr) => {
                assert!(arr.is_array_index());
            }
            _ => panic!("Expected ArrayLength"),
        }
    }

    #[test]
    fn test_parse_array_in_let() {
        // let arr = [|1; 2; 3|] in arr.[0]
        let expr = parse("let arr = [|1; 2; 3|] in arr.[0]").unwrap();
        assert!(expr.is_let());
        match expr {
            Expr::Let { value, body, .. } => {
                assert!(value.is_array());
                assert!(body.is_array_index());
            }
            _ => panic!("Expected Let"),
        }
    }

    #[test]
    fn test_parse_array_in_lambda() {
        // fun x -> [|x; x + 1|]
        let expr = parse("fun x -> [|x; x + 1|]").unwrap();
        assert!(expr.is_lambda());
        match expr {
            Expr::Lambda { body, .. } => {
                assert!(body.is_array());
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_parse_mixed_array_list() {
        // [[|1|]; [|2|]]
        let expr = parse("[[|1|]; [|2|]]").unwrap();
        assert!(expr.is_list());
        match expr {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_array());
                assert!(elements[1].is_array());
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_parse_array_of_tuples() {
        // [|(1, 2); (3, 4)|]
        let expr = parse("[|(1, 2); (3, 4)|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_tuple());
                assert!(elements[1].is_tuple());
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_tuple_of_arrays() {
        // ([|1; 2|], [|3; 4|])
        let expr = parse("([|1; 2|], [|3; 4|])").unwrap();
        assert!(expr.is_tuple());
        match expr {
            Expr::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_array());
                assert!(elements[1].is_array());
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_parse_array_with_if_elements() {
        // [|if true then 1 else 0; 2|]
        let expr = parse("[|if true then 1 else 0; 2|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(elements[0].is_if());
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_parse_array_large() {
        // [|1; 2; 3; 4; 5; 6; 7; 8|]
        let expr = parse("[|1; 2; 3; 4; 5; 6; 7; 8|]").unwrap();
        assert!(expr.is_array());
        match expr {
            Expr::Array(elements) => {
                assert_eq!(elements.len(), 8);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_error_array_unclosed() {
        // [|1; 2
        let result = parse("[|1; 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_array_index_missing_bracket() {
        // arr.0
        let result = parse("arr.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array_index_nested_in_expr() {
        // x + arr.[0]
        let expr = parse("x + arr.[0]").unwrap();
        assert!(expr.is_binop());
        match expr {
            Expr::BinOp { right, .. } => {
                assert!(right.is_array_index());
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_parse_complex_array_expression() {
        // let arr = [|1; 2; 3|] in Array.length arr + arr.[0]
        let expr = parse("let arr = [|1; 2; 3|] in Array.length arr + arr.[0]").unwrap();
        assert!(expr.is_let());
        match expr {
            Expr::Let { body, .. } => {
                assert!(body.is_binop());
            }
            _ => panic!("Expected Let"),
        }
    }

    // ========================================================================
    // Pattern Matching Parser Tests (Issue #27 Layer 2)
    // ========================================================================

    #[test]
    fn test_parse_match_literal_int() {
        let source = "match x with | 0 -> \"zero\" | 1 -> \"one\" | _ -> \"many\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (scrutinee, arms) = expr.as_match().unwrap();
        assert_eq!(scrutinee.as_var(), Some("x"));
        assert_eq!(arms.len(), 3);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_literal());
        assert!(arms[2].pattern.is_wildcard());
    }

    #[test]
    fn test_parse_match_literal_bool() {
        let source = "match flag with | true -> 1 | false -> 0";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_literal());
    }

    #[test]
    fn test_parse_match_variable_pattern() {
        let source = "match x with | n -> n + 1";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_var());
        assert_eq!(arms[0].pattern.as_var(), Some("n"));
    }

    #[test]
    fn test_parse_match_wildcard() {
        let source = "match x with | _ -> 42";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_wildcard());
    }

    #[test]
    fn test_parse_match_string_literal() {
        let source = r#"match cmd with | "quit" -> 0 | "help" -> 1 | _ -> 2"#;
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 3);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_literal());
    }

    #[test]
    fn test_parse_match_tuple_pattern() {
        let source = "match pair with | (0, 0) -> \"origin\" | (x, y) -> \"point\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].pattern.is_tuple());
        assert!(arms[1].pattern.is_tuple());
    }

    #[test]
    fn test_parse_match_nested_tuple() {
        let source = "match triple with | (a, (b, c)) -> a + b + c";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 2);
        assert!(tuple_patterns[1].is_tuple());
    }

    #[test]
    fn test_parse_match_tuple_with_wildcard() {
        let source = "match pair with | (0, _) -> \"x\" | (_, 0) -> \"y\" | _ -> \"z\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 3);
        assert!(arms[0].pattern.is_tuple());
        assert!(arms[1].pattern.is_tuple());
    }

    #[test]
    fn test_parse_match_complex_scrutinee() {
        let source = "match x + 1 with | 0 -> \"zero\" | _ -> \"other\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (scrutinee, arms) = expr.as_match().unwrap();
        assert!(scrutinee.is_binop());
        assert_eq!(arms.len(), 2);
    }

    #[test]
    fn test_parse_match_in_let_binding() {
        let source =
            "let classify = fun n -> match n with | 0 -> \"zero\" | _ -> \"nonzero\" in classify";
        let expr = parse(source).unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_match_as_function_body() {
        let source = "fun n -> match n with | 0 -> \"zero\" | 1 -> \"one\" | _ -> \"many\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_lambda());
        match expr {
            Expr::Lambda { body, .. } => {
                assert!(body.is_match());
            }
            _ => panic!("Expected Lambda"),
        }
    }

    #[test]
    fn test_parse_nested_match() {
        let source = "match x with | 0 -> (match y with | 0 -> 1 | _ -> 2) | _ -> 3";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_match());
    }

    #[test]
    fn test_parse_match_complex_body() {
        let source = "match n with | 0 -> if true then 1 else 2 | _ -> 3";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_if());
    }

    #[test]
    fn test_parse_match_multiple_arms() {
        let source =
            "match n with | 0 -> \"a\" | 1 -> \"b\" | 2 -> \"c\" | 3 -> \"d\" | _ -> \"e\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 5);
    }

    #[test]
    fn test_parse_match_empty_tuple_pattern() {
        let source = "match unit with | () -> 42";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        assert_eq!(arms[0].pattern.as_tuple().unwrap().len(), 0);
    }

    #[test]
    fn test_parse_match_grouped_pattern() {
        let source = "match x with | (n) -> n + 1";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        // Grouped pattern (n) is just n
        assert!(arms[0].pattern.is_var());
    }

    #[test]
    fn test_parse_match_mixed_patterns() {
        let source = "match data with | 0 -> \"zero\" | x -> \"var\" | _ -> \"wild\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 3);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_var());
        assert!(arms[2].pattern.is_wildcard());
    }

    #[test]
    fn test_parse_match_tuple_triple() {
        let source = "match triple with | (a, b, c) -> a + b + c";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        assert_eq!(arms[0].pattern.as_tuple().unwrap().len(), 3);
    }

    #[test]
    fn test_parse_match_with_arithmetic_in_body() {
        let source = "match n with | 0 -> 1 + 2 * 3 | _ -> 4 - 5";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_binop());
        assert!(arms[1].body.is_binop());
    }

    #[test]
    fn test_parse_match_in_complex_expression() {
        let source = "let f = fun x -> 1 + (match x with | 0 -> 10 | _ -> 20) in f 5";
        let expr = parse(source).unwrap();
        assert!(expr.is_let());
    }

    // Error cases
    #[test]
    fn test_parse_match_missing_with() {
        let source = "match x | 0 -> 1";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_match_missing_pipe() {
        let source = "match x with 0 -> 1";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_match_missing_arrow() {
        let source = "match x with | 0 1";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pattern_invalid() {
        let source = "match x with | -> 1";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_match_empty_arms() {
        let source = "match x with";
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_match_tuple_pattern_nested_complex() {
        let source = "match quad with | ((a, b), (c, d)) -> a + b + c + d";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 2);
        assert!(tuple_patterns[0].is_tuple());
        assert!(tuple_patterns[1].is_tuple());
    }

    #[test]
    fn test_parse_match_real_world_example() {
        let source = r#"
            let classify_point = fun p ->
                match p with
                | (0, 0) -> "origin"
                | (0, _) -> "y-axis"
                | (_, 0) -> "x-axis"
                | (x, y) -> "quadrant"
            in classify_point (1, 2)
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_match_fibonacci_style() {
        let source = r#"
            let rec fib = fun n ->
                match n with
                | 0 -> 0
                | 1 -> 1
                | n -> fib (n - 1) + fib (n - 2)
            in fib 10
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    #[test]
    fn test_parse_match_with_let_in_body() {
        let source = "match x with | 0 -> let y = 1 in y | _ -> 2";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_let());
    }

    #[test]
    fn test_parse_match_with_lambda_in_body() {
        let source = "match x with | 0 -> fun y -> y + 1 | _ -> fun y -> y";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_lambda());
        assert!(arms[1].body.is_lambda());
    }

    #[test]
    fn test_parse_match_single_arm() {
        let source = "match x with | n -> n";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_var());
    }

    #[test]
    fn test_parse_match_tuple_single_element() {
        let source = "match pair with | (x,) -> x";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 1);
    }
}
