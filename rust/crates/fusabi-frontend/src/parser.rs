//! Recursive-descent parser for Mini-F# expressions.
//!
//! This module implements a recursive-descent parser that converts a stream of tokens
//! from the lexer into an Abstract Syntax Tree (AST). The parser supports:
//!
//! - Literals: integers, floats, booleans, strings, unit
//! - Variables and identifiers
//! - Let-bindings: `let x = expr in body`
//! - Multi-parameter functions (curried): `let f x y = expr in body`
//! - Lambda functions: `fun x -> body` and multi-parameter `fun x y -> body`
//! - Function application: `f x y`
//! - Binary operations: arithmetic, comparison, logical
//! - Conditional expressions: `if cond then expr1 else expr2`
//! - Tuples: `(1, 2)`, `(x, y, z)`, `(42,)` (single-element)
//! - Lists: `[1; 2; 3]`, `[]`
//! - Arrays: `[|1; 2; 3|]`, `arr.[0]`, `arr.[0] <- 99`
//! - Records: `type Person = { name: string }`, `{ name = "John" }`, `person.name`, `{ person with age = 31 }`
//! - Discriminated Unions: `type Option = Some of int | None`, `Some(42)`, `None`
//! - Cons operator: `1 :: [2; 3]`, `x :: xs`
//! - Unary minus: `-42`, `-x`
//! - Modules: `module Math = ...`, `open Math`
//! - Proper operator precedence
//! - Error recovery and reporting
//!
//! module     ::= declaration*
//! declaration::= type_def | let_binding
//! type_def   ::= "type" IDENT "=" "{" (IDENT ":" type_expr (";" IDENT ":" type_expr)* ";"?)? "}"
//!              | "type" IDENT "=" variant ("|" variant)*
//! variant    ::= IDENT ("of" type_expr ("*" type_expr)*)?
//! type_expr  ::= simple_type ("->" type_expr)? | simple_type ("*" simple_type)*
//! simple_type::= IDENT
//! let_binding::= "let" IDENT IDENT* "=" expr
//! # Grammar (Simplified)
//!
//! ```text
//! program    ::= import* module_def* expr?
//! import     ::= "open" IDENT ("." IDENT)*
//! module_def ::= "module" IDENT "=" module_item*
//! module_item::= let_binding | type_def | module_def
//! expr       ::= let_expr | if_expr | lambda_expr | or_expr
//! let_expr   ::= "let" IDENT IDENT* "=" expr "in" expr
//! if_expr    ::= "if" expr "then" expr "else" expr
//! lambda_expr::= "fun" IDENT+ "->" expr
//! or_expr    ::= and_expr ("||" and_expr)*
//! and_expr   ::= comp_expr ("&&" comp_expr)*
//! comp_expr  ::= cons_expr (("=" | "==" | "<>" | "<" | "<=" | ">" | ">=") cons_expr)?
//! cons_expr  ::= add_expr ("::" cons_expr)?
//! add_expr   ::= mul_expr (("+" | "-") mul_expr)*
//! mul_expr   ::= unary_expr (("*" | "/") unary_expr)*
//! unary_expr ::= "-" unary_expr | app_expr
//! app_expr   ::= postfix_expr (postfix_expr)*
//! postfix_expr ::= primary (".[" expr "]" ("<-" expr)?)*
//! primary    ::= INT | FLOAT | BOOL | STRING | IDENT | "(" expr ")" | tuple | list | array | "Array.length" primary | variant_construct
//! tuple      ::= "(" ")" | "(" expr ("," expr)* ","? ")"
//! list       ::= "[" "]" | "[" expr (";" expr)* ";"? "]"
//! array      ::= "[|" "]" | "[|" expr (";" expr)* ";"? "|]"
//! record_literal ::= "{" (IDENT "=" expr (";" IDENT "=" expr)* ";"?)? "}"
//! record_update  ::= "{" expr "with" IDENT "=" expr (";" IDENT "=" expr)* ";"? "}"
//! variant_construct ::= IDENT ("(" expr ("," expr)* ")")?
//! ```
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::parser::Parser;
//! use fusabi_frontend::lexer::Lexer;
//! use fusabi_frontend::ast::{Expr, Literal, BinOp};
//!
//! let mut lexer = Lexer::new("let x = 42 in x + 1");
//! let tokens = lexer.tokenize().unwrap();
//! let mut parser = Parser::new(tokens);
//! let ast = parser.parse().unwrap();
//!
//! // AST represents: let x = 42 in (x + 1)
//! assert!(ast.is_let());
//! ```
use crate::ast::{
    BinOp, DuTypeDef, Expr, Import, Literal, MatchArm, ModuleDef, ModuleItem, Pattern, Program,
    TypeDefinition, TypeExpr, VariantDef,
};
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
                    "Parse error: unexpected end of file, expected {}",
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

type Result<T> = std::result::Result<T, ParseError>;

/// Recursive-descent parser for Mini-F# expressions.
pub struct Parser {
    tokens: Vec<TokenWithPos>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse the token stream into an expression AST.
    ///
    /// This is the main entry point for parsing. It delegates to `parse_expr` for
    /// expression parsing. For backward compatibility with existing code.
    pub fn parse(&mut self) -> Result<Expr> {
        let expr = self.parse_expr()?;

        // Ensure we've consumed all tokens (except EOF)
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

    /// Parse a complete program with modules, imports, and main expression.
    ///
    /// This is the new entry point for parsing programs with module system support.
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut imports = vec![];
        let mut modules = vec![];

        // Parse imports first
        while self.peek() == Some(&Token::Open) {
            imports.push(self.parse_import()?);
        }

        // Parse module definitions
        while self.peek() == Some(&Token::Module) {
            modules.push(self.parse_module()?);
        }

        // Parse main expression (if any)
        let main_expr = if !self.is_at_end() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        Ok(Program {
            modules,
            imports,
            main_expr,
        })
    }

    /// Parse a module definition: module Math = <items>
    fn parse_module(&mut self) -> Result<ModuleDef> {
        self.expect_token(Token::Module)?;
        let name = self.expect_ident()?;
        self.expect_token(Token::Eq)?;

        let items = self.parse_module_items()?;

        Ok(ModuleDef { name, items })
    }

    /// Parse module items (let bindings, types, nested modules)
    fn parse_module_items(&mut self) -> Result<Vec<ModuleItem>> {
        let mut items = vec![];

        // Continue until we hit EOF or another module keyword
        while !self.is_at_end()
            && self.peek() != Some(&Token::Module)
            && self.peek() != Some(&Token::Open)
        {
            let tok = &self.current_token().token;

            match tok {
                Token::Let => {
                    let (name, expr) = self.parse_let_binding_parts()?;
                    items.push(ModuleItem::Let(name, expr));
                }
                Token::Type => {
                    let type_def = self.parse_type_def()?;
                    items.push(ModuleItem::TypeDef(type_def));
                }
                _ => break,
            }
        }

        Ok(items)
    }

    /// Helper: parse let binding and return (name, expr) without the 'in' part
    fn parse_let_binding_parts(&mut self) -> Result<(String, Expr)> {
        self.expect_token(Token::Let)?;

        // Check for "rec" keyword - skip for now in modules
        let _is_rec = self.match_token(&Token::Rec);

        let name = self.expect_ident()?;

        // Handle parameters: let f x y = ...
        let mut params = vec![];
        while self.peek_is_identifier() && !self.peek_is(&Token::Eq) {
            params.push(self.expect_ident()?);
        }

        self.expect_token(Token::Eq)?;
        let mut body = self.parse_expr()?;

        // Wrap in nested lambdas for curried functions
        for param in params.into_iter().rev() {
            body = Expr::Lambda {
                param,
                body: Box::new(body),
            };
        }

        Ok((name, body))
    }

    /// Parse an import statement: open Math or open Math.Geometry
    fn parse_import(&mut self) -> Result<Import> {
        self.expect_token(Token::Open)?;

        let mut path = vec![self.expect_ident()?];

        // Handle qualified names: open Math.Geometry
        while self.check(&Token::Dot) {
            self.advance();
            path.push(self.expect_ident()?);
        }

        Ok(Import {
            module_path: path,
            is_qualified: false,
        })
    }

    /// Parse a type definition (record or discriminated union)
    fn parse_type_def(&mut self) -> Result<TypeDefinition> {
        // For now, delegate to existing parse_du_type_def
        // In the future, this should also handle records
        let du_def = self.parse_du_type_def()?;
        Ok(TypeDefinition::Du(du_def))
    }

    // ========================================================================
    // Expression Parsing
    // ========================================================================

    /// Parse an expression
    fn parse_expr(&mut self) -> Result<Expr> {
        // Try let, if, lambda, match first, then fall through to parse_pipeline_expr
        let tok = &self.current_token().token;
        match tok {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::Fun => self.parse_lambda(),
            Token::Match => self.parse_match(),
            _ => self.parse_pipeline_expr(), // Start parsing from pipeline operator level
        }
    }

    /// Parse let-binding: let x = expr in body
    fn parse_let(&mut self) -> Result<Expr> {
        self.expect_token(Token::Let)?;

        // Check for "rec" keyword (recursive let)
        if self.match_token(&Token::Rec) {
            return self.parse_let_rec();
        }

        // Parse variable name
        let name = self.expect_ident()?;

        // Parse optional parameters (for function definitions)
        let mut params = vec![];
        while let Token::Ident(_) = &self.current_token().token {
            params.push(self.expect_ident()?);
        }

        // Expect '='
        self.expect_token(Token::Eq)?;

        // Parse value expression
        let value = self.parse_expr()?;

        // Expect 'in'
        self.expect_token(Token::In)?;

        // Parse body expression
        let body = self.parse_expr()?;

        // Convert multi-parameter function to nested lambdas
        let value = if params.is_empty() {
            value
        } else {
            params
                .into_iter()
                .rev()
                .fold(value, |acc, param| Expr::Lambda {
                    param,
                    body: Box::new(acc),
                })
        };

        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    /// Parse recursive let-binding: let rec f = expr in body
    fn parse_let_rec(&mut self) -> Result<Expr> {
        // Parse first binding name
        let first_name = self.expect_ident()?;

        // Parse optional parameters
        let mut params = vec![];
        while let Token::Ident(_) = &self.current_token().token {
            params.push(self.expect_ident()?);
        }

        // Expect '='
        self.expect_token(Token::Eq)?;

        // Parse value expression
        let first_value = self.parse_expr()?;

        // Convert multi-parameter function to nested lambdas
        let first_value = if params.is_empty() {
            first_value
        } else {
            params
                .into_iter()
                .rev()
                .fold(first_value, |acc, param| Expr::Lambda {
                    param,
                    body: Box::new(acc),
                })
        };

        // Check for 'and' keyword (mutually recursive bindings)
        if self.match_token(&Token::AndKeyword) {
            let mut bindings = vec![(first_name, first_value)];

            // Parse additional bindings
            loop {
                let name = self.expect_ident()?;

                // Parse optional parameters
                let mut params = vec![];
                while let Token::Ident(_) = &self.current_token().token {
                    params.push(self.expect_ident()?);
                }

                // Expect '='
                self.expect_token(Token::Eq)?;

                // Parse value expression
                let value = self.parse_expr()?;

                // Convert multi-parameter function to nested lambdas
                let value = if params.is_empty() {
                    value
                } else {
                    params
                        .into_iter()
                        .rev()
                        .fold(value, |acc, param| Expr::Lambda {
                            param,
                            body: Box::new(acc),
                        })
                };

                bindings.push((name, value));

                // Check for another 'and'
                if !self.match_token(&Token::AndKeyword) {
                    break;
                }
            }

            // Expect 'in'
            self.expect_token(Token::In)?;

            // Parse body
            let body = self.parse_expr()?;

            Ok(Expr::LetRecMutual {
                bindings,
                body: Box::new(body),
            })
        } else {
            // Single recursive binding
            // Expect 'in'
            self.expect_token(Token::In)?;

            // Parse body
            let body = self.parse_expr()?;

            Ok(Expr::LetRec {
                name: first_name,
                value: Box::new(first_value),
                body: Box::new(body),
            })
        }
    }

    /// Parse if-then-else expression
    fn parse_if(&mut self) -> Result<Expr> {
        self.expect_token(Token::If)?;

        let cond = self.parse_expr()?;

        self.expect_token(Token::Then)?;
        let then_branch = self.parse_expr()?;

        self.expect_token(Token::Else)?;
        let else_branch = self.parse_expr()?;

        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    /// Parse lambda expression: fun x -> body or fun x y z -> body
    /// Multi-parameter lambdas are desugared to nested single-parameter lambdas
    fn parse_lambda(&mut self) -> Result<Expr> {
        self.expect_token(Token::Fun)?;

        // Parse one or more parameters
        let mut params = vec![];
        params.push(self.expect_ident()?);

        // Continue parsing parameters until we hit the arrow
        while !matches!(self.current_token().token, Token::Arrow) {
            // Only parse identifiers as parameters
            if let Token::Ident(_) = &self.current_token().token {
                params.push(self.expect_ident()?);
            } else {
                break;
            }
        }

        self.expect_token(Token::Arrow)?;
        let mut body = self.parse_expr()?;

        // Build nested lambdas from right to left: fun x y -> body becomes fun x -> (fun y -> body)
        for param in params.into_iter().rev() {
            body = Expr::Lambda {
                param,
                body: Box::new(body),
            };
        }

        Ok(body)
    }

    /// Parse match expression: match expr with | pat -> expr | ...
    fn parse_match(&mut self) -> Result<Expr> {
        self.expect_token(Token::Match)?;

        // Parse scrutinee
        let scrutinee = Box::new(self.parse_expr()?);

        // Expect 'with'
        self.expect_token(Token::With)?;

        // Parse arms (at least one required)
        let mut arms = vec![];

        // First arm must start with |
        self.expect_token(Token::Pipe)?;

        loop {
            // Parse pattern
            let pattern = self.parse_pattern()?;

            // Expect ->
            self.expect_token(Token::Arrow)?;

            // Parse body
            let body = Box::new(self.parse_expr()?);

            arms.push(MatchArm { pattern, body });

            // Check for another arm (starts with |)
            if !self.match_token(&Token::Pipe) {
                break;
            }
        }

        if arms.is_empty() {
            let tok = self.current_token();
            return Err(ParseError::InvalidExpr {
                message: "match expression must have at least one arm".to_string(),
                pos: tok.pos,
            });
        }

        Ok(Expr::Match { scrutinee, arms })
    }

    /// Parse pipeline expression: expr |> func
    /// Desugars to func(expr)
    fn parse_pipeline_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_or_expr()?;

        while self.match_token(&Token::PipeRight) {
            let func_expr = self.parse_app_expr()?; // The function to pipe into
            
            // Desugar: expr |> func_expr  =>  func_expr expr
            expr = Expr::App {
                func: Box::new(func_expr),
                arg: Box::new(expr),
            };
        }
        Ok(expr)
    }


    /// Parse pattern in match expression
    fn parse_pattern(&mut self) -> Result<Pattern> {
        let tok = self.current_token();

        match &tok.token {
            // Wildcard: _
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            // Variable or Variant: x or Some or Some(...)
            Token::Ident(name) => {
                let variant_name = name.clone();
                self.advance();

                // Check if this is a variant constructor with arguments: Some(x)
                if self.match_token(&Token::LParen) {
                    // Parse variant arguments
                    let mut patterns = vec![];

                    // Check for empty args: Some()
                    if !matches!(self.current_token().token, Token::RParen) {
                        loop {
                            patterns.push(self.parse_pattern()?);

                            if self.match_token(&Token::Comma) {
                                // Check for trailing comma before RParen
                                if matches!(self.current_token().token, Token::RParen) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    self.expect_token(Token::RParen)?;
                    Ok(Pattern::Variant {
                        variant: variant_name,
                        patterns,
                    })
                } else {
                    // No parens - could be simple variant or variable binding
                    // In pattern context, uppercase identifiers are typically variants
                    // For now, we treat as variable - type checker will disambiguate
                    Ok(Pattern::Var(variant_name))
                }
            }
            // Literal: 42, true, "hello"
            Token::Int(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            Token::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Pattern::Literal(Literal::Float(val)))
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
            // Tuple or grouped pattern: (p1, p2, ...) or (p)
            Token::LParen => {
                self.advance(); // consume '('

                // Empty tuple: ()
                if self.match_token(&Token::RParen) {
                    return Ok(Pattern::Tuple(vec![]));
                }

                // Parse first pattern
                let first_pat = self.parse_pattern()?;

                // Check if it's a tuple (has comma) or grouped pattern (no comma)
                if self.match_token(&Token::Comma) {
                    // It's a tuple: (p1, p2, ...)
                    let mut patterns = vec![first_pat];

                    // Check for trailing comma or continue with more patterns
                    if !matches!(self.current_token().token, Token::RParen) {
                        loop {
                            patterns.push(self.parse_pattern()?);

                            if self.match_token(&Token::Comma) {
                                // Check for trailing comma before RParen
                                if matches!(self.current_token().token, Token::RParen) {
                                    break;
                                }
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
                    Ok(first_pat)
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
    }

    /// Parse logical OR expression
    fn parse_or_expr(&mut self) -> Result<Expr> {
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
    fn parse_and_expr(&mut self) -> Result<Expr> {
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
    fn parse_comp_expr(&mut self) -> Result<Expr> {
        let left = self.parse_cons_expr()?;

        if let Some(op) = self.match_comparison_op() {
            let right = self.parse_cons_expr()?;
            Ok(Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    /// Parse cons expression (list cons operator ::)
    fn parse_cons_expr(&mut self) -> Result<Expr> {
        let left = self.parse_add_expr()?;

        if self.match_token(&Token::ColonColon) {
            let right = self.parse_cons_expr()?; // Right-associative
            Ok(Expr::Cons {
                head: Box::new(left),
                tail: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    /// Parse addition/subtraction expression
    fn parse_add_expr(&mut self) -> Result<Expr> {
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
    fn parse_mul_expr(&mut self) -> Result<Expr> {
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
    fn parse_unary_expr(&mut self) -> Result<Expr> {
        if self.match_token(&Token::Minus) {
            let expr = self.parse_unary_expr()?; // Right-associative
            Ok(Expr::BinOp {
                op: BinOp::Sub,
                left: Box::new(Expr::Lit(Literal::Int(0))),
                right: Box::new(expr),
            })
        } else {
            self.parse_app_expr()
        }
    }

    /// Parse function application
    fn parse_app_expr(&mut self) -> Result<Expr> {
        let mut func = self.parse_postfix_expr()?;

        // Repeatedly parse arguments while we see primary expressions
        while self.is_primary_start() {
            let arg = self.parse_postfix_expr()?;
            func = Expr::App {
                func: Box::new(func),
                arg: Box::new(arg),
            };
        }

        Ok(func)
    }

    /// Parse postfix expressions (array indexing, array update, record access)
    fn parse_postfix_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            let tok = &self.current_token().token;
            match tok {
                // Array indexing: arr.[index] or arr.[index] <- value
                Token::Dot => {
                    self.advance(); // consume '.'
                                    // Check if it's array indexing: .[
                    if self.match_token(&Token::LBracket) {
                        let index = Box::new(self.parse_expr()?);
                        self.expect_token(Token::RBracket)?;

                        // Check for array update: <- value
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
                    } else {
                        // Record field access: record.field
                        let field = self.expect_ident()?;
                        expr = Expr::RecordAccess {
                            record: Box::new(expr),
                            field,
                        };
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse record literal or record update: { name = "John" } or { person with age = 31 }
    fn parse_record_literal(&mut self) -> Result<Expr> {
        self.expect_token(Token::LBrace)?;

        // Empty record: {}
        if self.match_token(&Token::RBrace) {
            return Ok(Expr::RecordLiteral {
                type_name: String::new(),
                fields: vec![],
            });
        }

        // Check if this is a record update: { record with field = value }
        // We need to lookahead to distinguish between:
        // - { person with age = 31 }  (record update)
        // - { name = "John" }          (record literal)

        // Save position to potentially backtrack
        let save_pos = self.pos;

        // Try to parse first identifier
        if let Ok(_first_ident) = self.expect_ident() {
            // Check if followed by 'with' keyword
            if self.match_token(&Token::With) {
                // This is a record update: { record with ... }
                // The first_ident is actually an expression (variable), so restore and parse it properly
                self.pos = save_pos;
                let record = Box::new(self.parse_expr()?);

                // Expect 'with' keyword
                self.expect_token(Token::With)?;

                // Parse update fields
                let mut fields = vec![];

                loop {
                    let field_name = self.expect_ident()?;
                    self.expect_token(Token::Eq)?;
                    let value = self.parse_expr()?;
                    fields.push((field_name, Box::new(value)));

                    // Check for semicolon or closing brace
                    if self.match_token(&Token::Semicolon) {
                        // Check for trailing semicolon before }
                        if matches!(self.current_token().token, Token::RBrace) {
                            break;
                        }
                    } else if matches!(self.current_token().token, Token::RBrace) {
                        // No semicolon, but closing brace - this is ok for last field
                        break;
                    } else {
                        // No semicolon and no closing brace - error
                        let tok = self.current_token();
                        return Err(ParseError::UnexpectedToken {
                            expected: "';' or '}'".to_string(),
                            found: tok.token.clone(),
                            pos: tok.pos,
                        });
                    }
                }

                self.expect_token(Token::RBrace)?;

                return Ok(Expr::RecordUpdate { record, fields });
            }
        }

        // If we get here, it's a record literal
        // Restore position and parse as record literal
        self.pos = save_pos;

        let mut fields = vec![];

        // Parse fields
        loop {
            let field_name = self.expect_ident()?;
            self.expect_token(Token::Eq)?;
            let value = self.parse_expr()?;
            fields.push((field_name, Box::new(value)));

            // Check for semicolon or closing brace
            if self.match_token(&Token::Semicolon) {
                // Check for trailing semicolon before }
                if matches!(self.current_token().token, Token::RBrace) {
                    break;
                }
            } else if matches!(self.current_token().token, Token::RBrace) {
                // No semicolon, but closing brace - this is ok for last field
                break;
            } else {
                // No semicolon and no closing brace - error
                let tok = self.current_token();
                return Err(ParseError::UnexpectedToken {
                    expected: "';' or '}'".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                });
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Expr::RecordLiteral {
            type_name: String::new(), // Inferred by typechecker
            fields,
        })
    }

    /// Parse variant constructor: Some(42), Left, Rectangle(10.0, 20.0)
    fn parse_variant_construct(&mut self, variant_name: String) -> Result<Expr> {
        // Check if this is a variant constructor with arguments: Some(42)
        if self.match_token(&Token::LParen) {
            // Parse variant field values
            let mut fields = vec![];

            // Check for empty args: Some()
            if !matches!(self.current_token().token, Token::RParen) {
                loop {
                    fields.push(Box::new(self.parse_expr()?));

                    if self.match_token(&Token::Comma) {
                        // Check for trailing comma before RParen
                        if matches!(self.current_token().token, Token::RParen) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            self.expect_token(Token::RParen)?;
            Ok(Expr::VariantConstruct {
                type_name: String::new(), // Filled by typechecker
                variant: variant_name,
                fields,
            })
        } else {
            // Simple variant without fields: None, Left
            Ok(Expr::VariantConstruct {
                type_name: String::new(),
                variant: variant_name,
                fields: vec![],
            })
        }
    }

    /// Check if an identifier starts with uppercase letter (heuristic for variant constructor)
    fn is_uppercase_ident(s: &str) -> bool {
        s.chars().next().is_some_and(|c| c.is_uppercase())
    }

    /// Parse primary expression (literals, variables, parenthesized expressions, tuples, lists)
    fn parse_primary(&mut self) -> Result<Expr> {
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

                // Check if this could be a variant constructor
                // Use uppercase heuristic: if identifier starts with uppercase, it's likely a variant
                // Exception: if followed by '.', it's likely a module access (e.g. String.length)
                if (Self::is_uppercase_ident(&val) && !self.check(&Token::Dot))
                    || matches!(self.current_token().token, Token::LParen)
                {
                    // This looks like a variant constructor
                    self.parse_variant_construct(val)
                } else {
                    // Just a regular variable
                    Ok(Expr::Var(val))
                }
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
            Token::LBrace => {
                // Record literal or record update: { name = "John" } or { person with age = 31 }
                self.parse_record_literal()
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
    }

    // ========================================================================
    // Type Expression Parsing (Issue #15 Layer 2)
    // ========================================================================

    #[allow(dead_code)]
    /// Parse type expression
    fn parse_type_expr(&mut self) -> Result<TypeExpr> {
        self.parse_type_function()
    }

    #[allow(dead_code)]
    /// Parse function type: T1 -> T2
    fn parse_type_function(&mut self) -> Result<TypeExpr> {
        let mut left = self.parse_type_tuple()?;
        if self.peek() == Some(&Token::Arrow) {
            self.advance();
            let right = self.parse_type_function()?;
            left = TypeExpr::Function(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Parse tuple type: T1 * T2 * T3
    #[allow(dead_code)]
    fn parse_type_tuple(&mut self) -> Result<TypeExpr> {
        let mut types = vec![self.parse_type_primary()?];
        while self.peek() == Some(&Token::Star) {
            self.advance();
            types.push(self.parse_type_primary()?);
        }
        if types.len() == 1 {
            Ok(types.into_iter().next().unwrap())
        } else {
            Ok(TypeExpr::Tuple(types))
        }
    }

    #[allow(dead_code)]
    /// Parse primary type expression
    fn parse_type_primary(&mut self) -> Result<TypeExpr> {
        match self.peek() {
            Some(Token::Ident(name)) => {
                let type_name = name.clone();
                self.advance();
                Ok(TypeExpr::Named(type_name))
            }
            Some(Token::LParen) => {
                self.advance();
                let ty = self.parse_type_expr()?;
                self.expect_token(Token::RParen)?;
                Ok(ty)
            }
            _ => {
                let tok = self.current_token();
                Err(ParseError::UnexpectedToken {
                    expected: "type".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                })
            }
        }
    }

    // ========================================================================
    // Discriminated Union Parsing (Issue #29 Layer 2)
    // ========================================================================

    /// Parse discriminated union type definition
    /// type Option = Some of int | None
    /// type Shape = Circle of float | Rectangle of float * float
    #[allow(dead_code)]
    pub fn parse_du_type_def(&mut self) -> Result<DuTypeDef> {
        // Expect 'type'
        self.expect_token(Token::Type)?;

        // Parse type name
        let name = self.expect_ident()?;

        // Expect '='
        self.expect_token(Token::Eq)?;

        // Parse variants separated by |
        let mut variants = vec![];

        // First variant (no leading |)
        variants.push(self.parse_variant_def()?);

        // Additional variants (each starts with |)
        while self.match_token(&Token::Pipe) {
            variants.push(self.parse_variant_def()?);
        }

        Ok(DuTypeDef { name, variants })
    }

    /// Parse a single variant definition
    /// Some of int
    /// None
    /// Rectangle of float * float
    #[allow(dead_code)]
    fn parse_variant_def(&mut self) -> Result<VariantDef> {
        // Parse variant name
        let name = self.expect_ident()?;

        // Check for 'of' keyword (variant with fields)
        if self.match_token(&Token::Of) {
            // Parse field types separated by *
            let mut fields = vec![];

            // First field type
            fields.push(self.parse_type_primary()?);

            // Additional field types (each starts with *)
            while self.peek() == Some(&Token::Star) {
                self.advance(); // consume *
                fields.push(self.parse_type_primary()?);
            }

            Ok(VariantDef::new(name, fields))
        } else {
            // Simple variant with no fields
            Ok(VariantDef::new_simple(name))
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
                | Token::LBrace
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

    /// Check if a token matches without consuming
    fn check(&self, expected: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.current_token().token == expected
    }

    /// Check if current token is an identifier
    fn peek_is_identifier(&self) -> bool {
        matches!(self.peek(), Some(Token::Ident(_)))
    }

    /// Check if current token matches a specific token
    fn peek_is(&self, token: &Token) -> bool {
        self.peek() == Some(token)
    }

    /// Expect a specific token, returning error if not found
    fn expect_token(&mut self, expected: Token) -> Result<()> {
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
    fn expect_ident(&mut self) -> Result<String> {
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

    #[allow(dead_code)]
    /// Peek at the current token without position info
    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.pos].token)
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
    fn parse(input: &str) -> Result<Expr> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // Helper function to parse a program
    fn parse_program(input: &str) -> Result<Program> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

    // ========================================================================
    // Module System Tests (Phase 3 Cycle 2)
    // ========================================================================

    #[test]
    fn test_parse_simple_module() {
        let source = r#"
            module Math =
                let add x y = x + y
                let multiply x y = x * y
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].name, "Math");
        assert_eq!(program.modules[0].items.len(), 2);
    }

    #[test]
    fn test_parse_module_with_single_binding() {
        let source = "module Test = let x = 42";

        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].name, "Test");
        assert_eq!(program.modules[0].items.len(), 1);

        match &program.modules[0].items[0] {
            ModuleItem::Let(name, _) => assert_eq!(name, "x"),
            _ => panic!("Expected Let item"),
        }
    }

    #[test]
    fn test_parse_import_simple() {
        let source = r#"
            open Math

            add 5 10
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.imports[0].module_path, vec!["Math"]);
        assert!(program.main_expr.is_some());
    }

    #[test]
    fn test_parse_import_qualified() {
        let source = "open Math.Geometry";

        let program = parse_program(source).unwrap();

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.imports[0].module_path, vec!["Math", "Geometry"]);
    }

    #[test]
    fn test_parse_module_with_function() {
        let source = r#"
            module Utils =
                let identity x = x
                let const x y = x
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].items.len(), 2);
    }

    #[test]
    fn test_parse_complete_program() {
        let source = r#"
            module Math =
                let add x y = x + y
                let multiply x y = x * y

            module Utils =
                let identity x = x
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 2);
        assert_eq!(program.imports.len(), 0);
        assert!(program.main_expr.is_none());
    }

    #[test]
    fn test_parse_program_modules_only() {
        let source = r#"
            module A =
                let x = 1

            module B =
                let y = 2
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 2);
        assert_eq!(program.imports.len(), 0);
        assert!(program.main_expr.is_none());
    }

    #[test]
    fn test_parse_program_imports_and_expr() {
        let source = r#"
            open Math
            open Utils

            add 1 2
        "#;

        let program = parse_program(source).unwrap();

        assert_eq!(program.imports.len(), 2);
        assert!(program.main_expr.is_some());
    }

    #[test]
    fn test_parse_empty_module() {
        let source = "module Empty =";

        // This should parse successfully with an empty module
        let program = parse_program(source).unwrap();

        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].items.len(), 0);
    }

    // ========================================================================
    // Literal Tests
    // ========================================================================

    #[test]
    fn test_parse_int() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Int(42)));
    }

    #[test]
    fn test_parse_float() {
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
    fn test_parse_string() {
        let expr = parse("\"hello\"").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Str("hello".to_string())));
    }

    #[test]
    fn test_parse_unit() {
        let expr = parse("()").unwrap();
        assert_eq!(expr, Expr::Lit(Literal::Unit));
    }

    // ========================================================================
    // Variable Tests
    // ========================================================================

    #[test]
    fn test_parse_var() {
        let expr = parse("x").unwrap();
        assert_eq!(expr, Expr::Var("x".to_string()));
    }

    #[test]
    fn test_parse_var_long_name() {
        let expr = parse("myVariable123").unwrap();
        assert_eq!(expr, Expr::Var("myVariable123".to_string()));
    }

    // ========================================================================
    // Binary Operation Tests
    // ========================================================================

    #[test]
    fn test_parse_add() {
        let expr = parse("1 + 2").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_parse_sub() {
        let expr = parse("5 - 3").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Sub, .. }));
    }

    #[test]
    fn test_parse_mul() {
        let expr = parse("4 * 5").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Mul, .. }));
    }

    #[test]
    fn test_parse_div() {
        let expr = parse("10 / 2").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Div, .. }));
    }

    #[test]
    fn test_parse_eq() {
        let expr = parse("x = y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Eq, .. }));
    }

    #[test]
    fn test_parse_neq() {
        let expr = parse("x <> y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Neq, .. }));
    }

    #[test]
    fn test_parse_lt() {
        let expr = parse("x < y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Lt, .. }));
    }

    #[test]
    fn test_parse_lte() {
        let expr = parse("x <= y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Lte, .. }));
    }

    #[test]
    fn test_parse_gt() {
        let expr = parse("x > y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Gt, .. }));
    }

    #[test]
    fn test_parse_gte() {
        let expr = parse("x >= y").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Gte, .. }));
    }

    #[test]
    fn test_parse_and() {
        let expr = parse("true && false").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::And, .. }));
    }

    #[test]
    fn test_parse_or() {
        let expr = parse("true || false").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Or, .. }));
    }

    // ========================================================================
    // Precedence Tests
    // ========================================================================

    #[test]
    fn test_parse_precedence_mul_add() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let expr = parse("1 + 2 * 3").unwrap();
        if let Expr::BinOp {
            op: BinOp::Add,
            left,
            right,
        } = expr
        {
            assert_eq!(*left, Expr::Lit(Literal::Int(1)));
            assert!(matches!(*right, Expr::BinOp { op: BinOp::Mul, .. }));
        } else {
            panic!("Expected Add at root");
        }
    }

    #[test]
    fn test_parse_precedence_add_mul() {
        // 2 * 3 + 1 should be (2 * 3) + 1
        let expr = parse("2 * 3 + 1").unwrap();
        if let Expr::BinOp {
            op: BinOp::Add,
            left,
            right,
        } = expr
        {
            assert!(matches!(*left, Expr::BinOp { op: BinOp::Mul, .. }));
            assert_eq!(*right, Expr::Lit(Literal::Int(1)));
        } else {
            panic!("Expected Add at root");
        }
    }

    #[test]
    fn test_parse_precedence_comp_add() {
        // x + 1 < y + 2 should be (x + 1) < (y + 2)
        let expr = parse("x + 1 < y + 2").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Lt, .. }));
    }

    #[test]
    fn test_parse_precedence_and_or() {
        // a || b && c should be a || (b && c)
        let expr = parse("a || b && c").unwrap();
        if let Expr::BinOp {
            op: BinOp::Or,
            left,
            right,
        } = expr
        {
            assert_eq!(*left, Expr::Var("a".to_string()));
            assert!(matches!(*right, Expr::BinOp { op: BinOp::And, .. }));
        } else {
            panic!("Expected Or at root");
        }
    }

    // ========================================================================
    // Let-Binding Tests
    // ========================================================================

    #[test]
    fn test_parse_let_simple() {
        let expr = parse("let x = 42 in x").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_let_with_expr() {
        let expr = parse("let x = 1 + 2 in x * 3").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_let_nested() {
        let expr = parse("let x = 1 in let y = 2 in x + y").unwrap();
        assert!(expr.is_let());
        if let Expr::Let { body, .. } = expr {
            assert!(body.is_let());
        }
    }

    #[test]
    fn test_parse_let_function() {
        let expr = parse("let f x = x + 1 in f 5").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_let_function_multi_param() {
        let expr = parse("let add x y = x + y in add 1 2").unwrap();
        assert!(expr.is_let());
    }

    // ========================================================================
    // Lambda Tests
    // ========================================================================

    #[test]
    fn test_parse_lambda_simple() {
        let expr = parse("fun x -> x + 1").unwrap();
        assert!(expr.is_lambda());
    }

    #[test]
    fn test_parse_lambda_nested() {
        let expr = parse("fun x -> fun y -> x + y").unwrap();
        assert!(expr.is_lambda());
        if let Expr::Lambda { body, .. } = expr {
            assert!(body.is_lambda());
        }
    }

    #[test]
    fn test_parse_lambda_multi_param() {
        // Multi-parameter lambda: fun x y -> x + y
        let expr = parse("fun x y -> x + y").unwrap();
        assert!(expr.is_lambda());
        // Should be desugared to: fun x -> (fun y -> x + y)
        if let Expr::Lambda { param, body } = expr {
            assert_eq!(param, "x");
            assert!(body.is_lambda());
            if let Expr::Lambda {
                param: inner_param,
                body: inner_body,
            } = *body
            {
                assert_eq!(inner_param, "y");
                assert!(inner_body.is_binop());
            }
        }
    }

    #[test]
    fn test_parse_lambda_triple_param() {
        // Triple-parameter lambda: fun x y z -> x + y + z
        let expr = parse("fun x y z -> x + y + z").unwrap();
        assert!(expr.is_lambda());
        // Should be: fun x -> (fun y -> (fun z -> x + y + z))
        if let Expr::Lambda { param, body } = expr {
            assert_eq!(param, "x");
            if let Expr::Lambda { param, body } = *body {
                assert_eq!(param, "y");
                if let Expr::Lambda { param, body: _ } = *body {
                    assert_eq!(param, "z");
                }
            }
        }
    }

    // ========================================================================
    // Function Application Tests
    // ========================================================================

    #[test]
    fn test_parse_app_simple() {
        let expr = parse("f x").unwrap();
        assert!(expr.is_app());
    }

    #[test]
    fn test_parse_app_multi() {
        let expr = parse("f x y").unwrap();
        assert!(expr.is_app());
        // Should be ((f x) y)
        if let Expr::App { func, .. } = expr {
            assert!(func.is_app());
        }
    }

    #[test]
    fn test_parse_app_with_literal() {
        let expr = parse("f 42").unwrap();
        assert!(expr.is_app());
    }

    // ========================================================================
    // If-Then-Else Tests
    // ========================================================================

    #[test]
    fn test_parse_if_simple() {
        let expr = parse("if true then 1 else 2").unwrap();
        assert!(expr.is_if());
    }

    #[test]
    fn test_parse_if_with_comparison() {
        let expr = parse("if x > 0 then 1 else -1").unwrap();
        assert!(expr.is_if());
    }

    #[test]
    fn test_parse_if_nested() {
        let expr = parse("if x > 0 then if y > 0 then 1 else 2 else 3").unwrap();
        assert!(expr.is_if());
    }

    // ========================================================================
    // Tuple Tests
    // ========================================================================

    #[test]
    fn test_parse_tuple_pair() {
        let expr = parse("(1, 2)").unwrap();
        assert!(expr.is_tuple());
        if let Expr::Tuple(elements) = expr {
            assert_eq!(elements.len(), 2);
        }
    }

    #[test]
    fn test_parse_tuple_triple() {
        let expr = parse("(1, 2, 3)").unwrap();
        assert!(expr.is_tuple());
        if let Expr::Tuple(elements) = expr {
            assert_eq!(elements.len(), 3);
        }
    }

    #[test]
    fn test_parse_tuple_trailing_comma() {
        let expr = parse("(1,)").unwrap();
        assert!(expr.is_tuple());
        if let Expr::Tuple(elements) = expr {
            assert_eq!(elements.len(), 1);
        }
    }

    #[test]
    fn test_parse_tuple_nested() {
        let expr = parse("((1, 2), (3, 4))").unwrap();
        assert!(expr.is_tuple());
        if let Expr::Tuple(elements) = expr {
            assert_eq!(elements.len(), 2);
            assert!(elements[0].is_tuple());
            assert!(elements[1].is_tuple());
        }
    }

    #[test]
    fn test_parse_grouped_expr() {
        let expr = parse("(1 + 2)").unwrap();
        // Should be just the BinOp, not a tuple
        assert!(!expr.is_tuple());
        assert!(expr.is_binop());
    }

    // ========================================================================
    // List Tests
    // ========================================================================

    #[test]
    fn test_parse_list_empty() {
        let expr = parse("[]").unwrap();
        assert!(expr.is_list());
        if let Expr::List(elements) = expr {
            assert_eq!(elements.len(), 0);
        }
    }

    #[test]
    fn test_parse_list_single() {
        let expr = parse("[1]").unwrap();
        assert!(expr.is_list());
        if let Expr::List(elements) = expr {
            assert_eq!(elements.len(), 1);
        }
    }

    #[test]
    fn test_parse_list_multiple() {
        let expr = parse("[1; 2; 3]").unwrap();
        assert!(expr.is_list());
        if let Expr::List(elements) = expr {
            assert_eq!(elements.len(), 3);
        }
    }

    #[test]
    fn test_parse_list_trailing_semicolon() {
        let expr = parse("[1; 2; 3;]").unwrap();
        assert!(expr.is_list());
        if let Expr::List(elements) = expr {
            assert_eq!(elements.len(), 3);
        }
    }

    #[test]
    fn test_parse_cons() {
        let expr = parse("1 :: [2; 3]").unwrap();
        assert!(expr.is_cons());
    }

    #[test]
    fn test_parse_cons_nested() {
        let expr = parse("1 :: 2 :: [3]").unwrap();
        assert!(expr.is_cons());
        // Should be right-associative: 1 :: (2 :: [3])
        if let Expr::Cons { tail, .. } = expr {
            assert!(tail.is_cons());
        }
    }

    // ========================================================================
    // Array Tests
    // ========================================================================

    #[test]
    fn test_parse_array_empty() {
        let expr = parse("[||]").unwrap();
        assert!(expr.is_array());
        if let Expr::Array(elements) = expr {
            assert_eq!(elements.len(), 0);
        }
    }

    #[test]
    fn test_parse_array_single() {
        let expr = parse("[|1|]").unwrap();
        assert!(expr.is_array());
        if let Expr::Array(elements) = expr {
            assert_eq!(elements.len(), 1);
        }
    }

    #[test]
    fn test_parse_array_multiple() {
        let expr = parse("[|1; 2; 3|]").unwrap();
        assert!(expr.is_array());
        if let Expr::Array(elements) = expr {
            assert_eq!(elements.len(), 3);
        }
    }

    #[test]
    fn test_parse_array_trailing_semicolon() {
        let expr = parse("[|1; 2; 3;|]").unwrap();
        assert!(expr.is_array());
        if let Expr::Array(elements) = expr {
            assert_eq!(elements.len(), 3);
        }
    }

    #[test]
    fn test_parse_array_index() {
        let expr = parse("arr.[0]").unwrap();
        assert!(expr.is_array_index());
    }

    #[test]
    fn test_parse_array_update() {
        let expr = parse("arr.[0] <- 99").unwrap();
        assert!(expr.is_array_update());
    }

    #[test]
    fn test_parse_array_length() {
        let expr = parse("Array.length arr").unwrap();
        assert!(expr.is_array_length());
    }

    // ========================================================================
    // Unary Minus Tests
    // ========================================================================

    #[test]
    fn test_parse_unary_minus() {
        let expr = parse("-42").unwrap();
        // Unary minus is represented as 0 - expr
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Sub, .. }));
    }

    #[test]
    fn test_parse_unary_minus_var() {
        let expr = parse("-x").unwrap();
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Sub, .. }));
    }

    // ========================================================================
    // Let Rec Tests
    // ========================================================================

    #[test]
    fn test_parse_let_rec_simple() {
        let source = "let rec f = fun n -> if n <= 1 then 1 else n * f (n - 1) in f 5";
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    #[test]
    fn test_parse_let_rec_with_params() {
        let source = "let rec fact n = if n <= 1 then 1 else n * fact (n - 1) in fact 5";
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    #[test]
    fn test_parse_let_rec_no_params() {
        let source = "let rec x = 42 in x";
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    #[test]
    fn test_parse_let_rec_nested() {
        let source = "let rec f = fun x -> let rec g = fun y -> y + 1 in g x in f 5";
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    // ========================================================================
    // Mutual Recursion Tests
    // ========================================================================

    #[test]
    fn test_parse_let_rec_mutual_simple() {
        let source = r#"
            let rec even n = if n = 0 then true else odd (n - 1)
            and odd n = if n = 0 then false else even (n - 1)
            in even 10
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec_mutual());
        if let Expr::LetRecMutual { bindings, .. } = expr {
            assert_eq!(bindings.len(), 2);
        }
    }

    #[test]
    fn test_parse_let_rec_mutual_three_functions() {
        let source = r#"
            let rec f x = g x
            and g x = h x
            and h x = x + 1
            in f 5
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec_mutual());
        if let Expr::LetRecMutual { bindings, .. } = expr {
            assert_eq!(bindings.len(), 3);
        }
    }

    #[test]
    fn test_parse_let_rec_mutual_even_odd() {
        let source = r#"
            let rec
                isEven n = if n = 0 then true else isOdd (n - 1)
            and isOdd n = if n = 0 then false else isEven (n - 1)
            in isEven 4
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec_mutual());
    }

    #[test]
    fn test_parse_mutual_recursion_complex() {
        let source = r#"
            let rec f x y = if x > 0 then g (x - 1) y else y
            and g x y = if y > 0 then f x (y - 1) else x
            in f 3 3
        "#;
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec_mutual());
    }

    #[test]
    fn test_parse_recursive_closure() {
        let source = "let rec f = fun n -> if n <= 1 then 1 else f (n - 1) + f (n - 2) in f 10";
        let expr = parse(source).unwrap();
        assert!(expr.is_let_rec());
    }

    // ========================================================================
    // Match Expression Tests (Issue #27 Layer 2)
    // ========================================================================

    #[test]
    fn test_parse_match_simple() {
        let source = "match x with | 0 -> \"zero\" | _ -> \"nonzero\"";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
    }

    #[test]
    fn test_parse_match_with_wildcard() {
        let source = "match x with | _ -> 42";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_wildcard());
    }

    #[test]
    fn test_parse_match_with_var() {
        let source = "match x with | n -> n + 1";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_var());
    }

    #[test]
    fn test_parse_match_with_literals() {
        let source = r#"match x with | 0 -> "zero" | 1 -> "one" | _ -> "many""#;
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 3);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_literal());
        assert!(arms[2].pattern.is_wildcard());
    }

    #[test]
    fn test_parse_match_tuple_pattern() {
        let source = "match p with | (x, y) -> x + y";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
    }

    #[test]
    fn test_parse_match_tuple_pattern_nested() {
        let source = "match p with | ((a, b), c) -> a + b + c";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 2);
        assert!(tuple_patterns[0].is_tuple());
    }

    #[test]
    fn test_parse_match_tuple_pattern_with_wildcard() {
        let source = "match p with | (x, _) -> x";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 2);
        assert!(tuple_patterns[0].is_var());
        assert!(tuple_patterns[1].is_wildcard());
    }

    #[test]
    fn test_parse_match_bool_pattern() {
        let source = "match flag with | true -> 1 | false -> 0";
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].pattern.is_literal());
        assert!(arms[1].pattern.is_literal());
    }

    #[test]
    fn test_parse_match_string_pattern() {
        let source = r#"match s with | "hello" -> 1 | _ -> 0"#;
        let expr = parse(source).unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].pattern.is_literal());
    }

    #[test]
    fn test_parse_match_nested() {
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
        let expr = parse("match x with | 0 -> fun y -> y + 1 | _ -> fun y -> y").unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 2);
        assert!(arms[0].body.is_lambda());
        assert!(arms[1].body.is_lambda());
    }

    #[test]
    fn test_parse_match_single_arm() {
        let expr = parse("match x with | n -> n").unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_var());
    }

    #[test]
    fn test_parse_match_tuple_single_element() {
        let expr = parse("match pair with | (x,) -> x").unwrap();
        assert!(expr.is_match());
        let (_, arms) = expr.as_match().unwrap();
        assert_eq!(arms.len(), 1);
        assert!(arms[0].pattern.is_tuple());
        let tuple_patterns = arms[0].pattern.as_tuple().unwrap();
        assert_eq!(tuple_patterns.len(), 1);
    }

    // ========================================================================
    // Record Update Tests
    // ========================================================================

    #[test]
    fn test_parse_record_update_simple() {
        let source = "{ person with age = 31 }";
        let expr = parse(source).unwrap();
        assert!(expr.is_record_update());
        if let Expr::RecordUpdate { record, fields } = expr {
            assert!(matches!(*record, Expr::Var(_)));
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "age");
        }
    }

    #[test]
    fn test_parse_record_update_multiple_fields() {
        let source = "{ person with age = 31; city = \"NYC\" }";
        let expr = parse(source).unwrap();
        assert!(expr.is_record_update());
        if let Expr::RecordUpdate { fields, .. } = expr {
            assert_eq!(fields.len(), 2);
        }
    }

    #[test]
    fn test_parse_record_literal_vs_update() {
        // Test that we can distinguish record literal from update
        let literal = parse("{ name = \"John\"; age = 30 }").unwrap();
        assert!(literal.is_record_literal());
        assert!(!literal.is_record_update());

        let update = parse("{ person with age = 31 }").unwrap();
        assert!(update.is_record_update());
        assert!(!update.is_record_literal());
    }
}
