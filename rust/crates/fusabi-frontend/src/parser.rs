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
//! - Lists: `[1, 2, 3]`, `[1; 2; 3]`, `[]`
//! - Arrays: `[|1; 2; 3|]`, `arr.[0]`, `arr.[0] <- 99`
//! - Records: `type Person = { name: string }`, `{ name = "John" }`, `person.name`, `{ person with age = 31 }`
//! - Discriminated Unions: `type Option = Some of int | None`, `Some(42)`, `None`, `Some 42`
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
//! list       ::= "[" "]" | "[" expr (("," | ";") expr)* ("," | ";")? "]"
//! array      ::= "[|" "]" | "[|" expr (";" expr)* ";"? "|]"
//! record_literal ::= "{" (IDENT "=" expr (";" IDENT "=" expr)* ";"?)? "}"
//! record_update  ::= "{" expr "with" IDENT "=" expr (";" IDENT "=" expr)* ";"? "}"
//! variant_construct ::= IDENT ("(" expr ("," expr)* ")")? | IDENT expr
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
    BinOp, CEStatement, DuTypeDef, Expr, Import, Literal, MatchArm, ModuleDef, ModuleItem, Pattern,
    Program, TypeDefinition, TypeExpr, VariantDef,
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

/// Result of parsing a `let` construct, which could be a ModuleItem or an Expr
enum LetResult {
    Item(ModuleItem),
    Expr(Expr),
}

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
        let mut items = vec![];
        let mut main_expr = None;

        // Parse imports first
        while self.peek() == Some(&Token::Open) {
            imports.push(self.parse_import()?);
        }

        // Parse module definitions
        while self.peek() == Some(&Token::Module) {
            modules.push(self.parse_module()?);
        }

        // Parse top-level items and main expression
        while !self.is_at_end() {
            let tok = self.current_token();
            match tok.token {
                Token::Let => {
                    // Check if it's a top-level binding or a let expression
                    // We parse the binding part first
                    let binding = self.parse_let_binding_or_expr()?;
                    match binding {
                        LetResult::Item(item) => items.push(item),
                        LetResult::Expr(expr) => {
                            main_expr = Some(expr);
                            // If we found a main expression (e.g., let ... in ...),
                            // we shouldn't expect more top-level items unless they are part of that expression
                            // (which parse_expr handles).
                            // However, if there's trailing junk, verify EOF?
                            // For now, let's assume this completes the program or fail if more follows?
                            // F# doesn't really allow `let ... in ...; let ...` at top level.
                            break;
                        }
                    }
                }
                Token::Do => {
                    // do expr is syntax sugar for let _ = expr
                    self.advance();
                    let expr = self.parse_expr()?;
                    items.push(ModuleItem::Let(None, expr));
                }
                Token::Type => {
                    let type_def = self.parse_type_def()?;
                    items.push(ModuleItem::TypeDef(type_def));
                }
                _ => {
                    // Assume main expression
                    let expr = self.parse_expr()?;
                    main_expr = Some(expr);
                    break;
                }
            }
        }

        Ok(Program {
            modules,
            imports,
            items,
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
                    // Inside a module, we expect items, but let's handle potential ambiguity properly
                    // although technically 'in' is not valid inside a module unless it's inside an expr.
                    // But parse_let_binding_or_expr handles top-level context.
                    // For modules, we expect declarations.
                    // However, parse_let_binding_parts is what we used before.
                    // Let's use parse_let_binding_or_expr and ensure it returns an Item.
                    let result = self.parse_let_binding_or_expr()?;
                    match result {
                        LetResult::Item(item) => items.push(item),
                        LetResult::Expr(_) => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "module item".to_string(),
                                found: Token::In,
                                pos: self.current_token().pos,
                            });
                        }
                    }
                }
                Token::Do => {
                    // do expr is syntax sugar for let _ = expr
                    self.advance();
                    let expr = self.parse_expr()?;
                    items.push(ModuleItem::Let(None, expr));
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

    /// Helper: parse a let construct that could be a binding or an expression
    fn parse_let_binding_or_expr(&mut self) -> Result<LetResult> {
        self.expect_token(Token::Let)?;

        // Check for "rec" keyword
        let is_rec = self.match_token(&Token::Rec);

        if is_rec {
            // Recursive binding(s)
            let first_name = self.expect_ident()?;
            let mut params = vec![];
            while let Token::Ident(_) = &self.current_token().token {
                params.push(self.expect_ident()?);
            }

            self.expect_token(Token::Eq)?;
            let mut first_value = self.parse_expr()?;

            // Desugar params
            if !params.is_empty() {
                first_value =
                    params
                        .into_iter()
                        .rev()
                        .fold(first_value, |acc, param| Expr::Lambda {
                            param,
                            body: Box::new(acc),
                        });
            }

            // Check for mutual recursion ('and')
            if self.match_token(&Token::AndKeyword) {
                let mut bindings = vec![(first_name, first_value)];

                loop {
                    let name = self.expect_ident()?;
                    let mut params = vec![];
                    while let Token::Ident(_) = &self.current_token().token {
                        params.push(self.expect_ident()?);
                    }
                    self.expect_token(Token::Eq)?;
                    let mut value = self.parse_expr()?;

                    if !params.is_empty() {
                        value = params
                            .into_iter()
                            .rev()
                            .fold(value, |acc, param| Expr::Lambda {
                                param,
                                body: Box::new(acc),
                            });
                    }
                    bindings.push((name, value));

                    if !self.match_token(&Token::AndKeyword) {
                        break;
                    }
                }

                // Now check for 'in'
                if self.match_token(&Token::In) {
                    let body = self.parse_expr()?;
                    Ok(LetResult::Expr(Expr::LetRecMutual {
                        bindings,
                        body: Box::new(body),
                    }))
                } else {
                    // It's a LetRec item.
                    // Convert (name, val) pairs.
                    // ModuleItem::LetRec takes a Vec.
                    Ok(LetResult::Item(ModuleItem::LetRec(bindings)))
                }
            } else {
                // Single recursive binding
                // Expect 'in' - wait, for top-level item, 'in' is NOT expected.
                // If 'in' is present, it is an expression.
                if self.match_token(&Token::In) {
                    let body = self.parse_expr()?;
                    Ok(LetResult::Expr(Expr::LetRec {
                        name: first_name,
                        value: Box::new(first_value),
                        body: Box::new(body),
                    }))
                } else {
                    Ok(LetResult::Item(ModuleItem::LetRec(vec![(
                        first_name,
                        first_value,
                    )])))
                }
            }
        } else {
            // Simple Let
            // Check for discard pattern '_'
            let name = match &self.current_token().token {
                Token::Underscore => {
                    self.advance();
                    None
                }
                Token::Ident(id) if id == "_" => {
                    self.advance();
                    None
                }
                _ => Some(self.expect_ident()?),
            };

            let mut params = vec![];
            while let Token::Ident(_) = &self.current_token().token {
                params.push(self.expect_ident()?);
            }

            self.expect_token(Token::Eq)?;
            let mut value = self.parse_expr()?;

            if !params.is_empty() {
                value = params
                    .into_iter()
                    .rev()
                    .fold(value, |acc, param| Expr::Lambda {
                        param,
                        body: Box::new(acc),
                    });
            }

            if self.match_token(&Token::In) {
                let body = self.parse_expr()?;
                // Discard variables not allowed in let...in expressions
                if name.is_none() {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: Token::Ident("_".to_string()),
                        pos: self.current_token().pos,
                    });
                }
                Ok(LetResult::Expr(Expr::Let {
                    name: name.unwrap(),
                    value: Box::new(value),
                    body: Box::new(body),
                }))
            } else {
                Ok(LetResult::Item(ModuleItem::Let(name, value)))
            }
        }
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
        // Try let, if, lambda, match, while first, then fall through to parse_pipeline_expr
        let tok = &self.current_token().token;
        match tok {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::Fun => self.parse_lambda(),
            Token::Match => self.parse_match(),
            Token::While => self.parse_while(),
            Token::Break => {
                self.advance();
                Ok(Expr::Break)
            }
            Token::Continue => {
                self.advance();
                Ok(Expr::Continue)
            }
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

        let name = self.expect_ident()?;

        // Parse optional parameter list (for multi-parameter functions)
        // Example: let f x y = ...
        let mut params = vec![];
        while let Token::Ident(_) = &self.current_token().token {
            params.push(self.expect_ident()?);
        }

        self.expect_token(Token::Eq)?;
        let mut value = self.parse_expr()?;

        // If we have params, desugar into nested lambdas
        // let f x y = body  =>  let f = fun x -> fun y -> body
        if !params.is_empty() {
            value = params
                .into_iter()
                .rev()
                .fold(value, |acc, param| Expr::Lambda {
                    param,
                    body: Box::new(acc),
                });
        }

        self.expect_token(Token::In)?;
        let body = self.parse_expr()?;

        Ok(Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    /// Parse recursive let-binding: let rec f x = ... in body
    fn parse_let_rec(&mut self) -> Result<Expr> {
        // Expect function name
        let first_name = self.expect_ident()?;

        // Parse parameters
        let mut params = vec![];
        while let Token::Ident(_) = &self.current_token().token {
            params.push(self.expect_ident()?);
        }

        self.expect_token(Token::Eq)?;
        let mut first_value = self.parse_expr()?;

        // Desugar params into nested lambdas
        if !params.is_empty() {
            first_value = params
                .into_iter()
                .rev()
                .fold(first_value, |acc, param| Expr::Lambda {
                    param,
                    body: Box::new(acc),
                });
        }

        // Check for mutual recursion ('and')
        if self.match_token(&Token::AndKeyword) {
            let mut bindings = vec![(first_name, first_value)];

            loop {
                let name = self.expect_ident()?;
                let mut params = vec![];
                while let Token::Ident(_) = &self.current_token().token {
                    params.push(self.expect_ident()?);
                }
                self.expect_token(Token::Eq)?;
                let mut value = self.parse_expr()?;

                if !params.is_empty() {
                    value = params
                        .into_iter()
                        .rev()
                        .fold(value, |acc, param| Expr::Lambda {
                            param,
                            body: Box::new(acc),
                        });
                }
                bindings.push((name, value));

                if !self.match_token(&Token::AndKeyword) {
                    break;
                }
            }

            self.expect_token(Token::In)?;
            let body = self.parse_expr()?;

            Ok(Expr::LetRecMutual {
                bindings,
                body: Box::new(body),
            })
        } else {
            // Single recursive function
            self.expect_token(Token::In)?;
            let body = self.parse_expr()?;

            Ok(Expr::LetRec {
                name: first_name,
                value: Box::new(first_value),
                body: Box::new(body),
            })
        }
    }

    /// Parse conditional: if cond then expr1 else expr2
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

    /// Parse lambda function: fun x -> body or fun x y -> body
    fn parse_lambda(&mut self) -> Result<Expr> {
        self.expect_token(Token::Fun)?;

        // Parse parameter list
        let mut params = vec![];
        while let Token::Ident(_) = &self.current_token().token {
            params.push(self.expect_ident()?);
        }

        if params.is_empty() {
            let tok = self.current_token();
            return Err(ParseError::UnexpectedToken {
                expected: "parameter".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            });
        }

        self.expect_token(Token::Arrow)?;
        let body = self.parse_expr()?;

        // Desugar multi-param lambda into nested lambdas
        // fun x y -> body  =>  fun x -> fun y -> body
        Ok(params
            .into_iter()
            .rev()
            .fold(body, |acc, param| Expr::Lambda {
                param,
                body: Box::new(acc),
            }))
    }

    /// Parse match expression: match expr with | pattern -> expr
    fn parse_match(&mut self) -> Result<Expr> {
        self.expect_token(Token::Match)?;

        let scrutinee = Box::new(self.parse_expr()?);

        self.expect_token(Token::With)?;

        let mut arms = vec![];

        // Parse match arms
        loop {
            // Optional leading pipe
            self.match_token(&Token::Pipe);

            let pattern = self.parse_pattern()?;

            self.expect_token(Token::Arrow)?;

            let body = Box::new(self.parse_expr()?);

            arms.push(MatchArm { pattern, body });

            // Check if there's another arm (starts with |)
            if !self.check(&Token::Pipe) {
                break;
            }
        }

        Ok(Expr::Match { scrutinee, arms })
    }

    /// Parse while loop: while cond do body
    fn parse_while(&mut self) -> Result<Expr> {
        self.expect_token(Token::While)?;

        let cond = Box::new(self.parse_expr()?);

        self.expect_token(Token::Do)?;

        let body = Box::new(self.parse_expr()?);

        Ok(Expr::While { cond, body })
    }

    /// Parse a pattern for match expressions
    fn parse_pattern(&mut self) -> Result<Pattern> {
        let tok = self.current_token();

        match &tok.token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Ident(name) => {
                let val = name.clone();
                self.advance();

                // Check if this is a variant constructor with patterns
                if Self::is_uppercase_ident(&val) {
                    // Variant pattern
                    if self.match_token(&Token::LParen) {
                        // Variant with nested patterns: Some(x), Circle(r)
                        let mut patterns = vec![];

                        if !matches!(self.current_token().token, Token::RParen) {
                            loop {
                                patterns.push(self.parse_pattern()?);

                                if self.match_token(&Token::Comma) {
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
                            variant: val,
                            patterns,
                        })
                    } else {
                        // Simple variant without patterns: None, Left
                        Ok(Pattern::Variant {
                            variant: val,
                            patterns: vec![],
                        })
                    }
                } else {
                    // Variable pattern
                    Ok(Pattern::Var(val))
                }
            }
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
            Token::LParen => {
                self.advance(); // consume '('

                // Handle unit pattern: ()
                if self.match_token(&Token::RParen) {
                    return Ok(Pattern::Literal(Literal::Unit));
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

    /// Parse unary expressions (unary minus)
    fn parse_unary_expr(&mut self) -> Result<Expr> {
        if self.match_token(&Token::Minus) {
            let expr = self.parse_unary_expr()?;
            // Desugar unary minus as (0 - expr)
            Ok(Expr::BinOp {
                op: BinOp::Sub,
                left: Box::new(Expr::Lit(Literal::Int(0))),
                right: Box::new(expr),
            })
        } else {
            self.parse_app_expr()
        }
    }

    /// Parse pipeline expression: expr |> func
    fn parse_pipeline_expr(&mut self) -> Result<Expr> {
        let mut left = self.parse_or_expr()?;

        while self.match_token(&Token::PipeRight) {
            let func_expr = self.parse_app_expr()?; // The function to pipe into
            left = Expr::App {
                func: Box::new(func_expr),
                arg: Box::new(left),
            };
        }

        Ok(left)
    }

    /// Parse function application
    fn parse_app_expr(&mut self) -> Result<Expr> {
        let mut func = self.parse_postfix_expr()?;

        // Repeatedly parse arguments while we see primary expressions
        while self.is_primary_start() {
            // Heuristic: if the argument is at column 1 (start of line),
            // treat it as a new statement/expression, not an argument.
            if self.current_token().pos.column == 1 {
                break;
            }

            let arg = self.parse_postfix_expr()?;
            func = Expr::App {
                func: Box::new(func),
                arg: Box::new(arg),
            };
        }

        // Post-process: convert variant constructor applications to VariantConstruct
        // This handles `Some 42` syntax by converting App(Some, 42) to VariantConstruct
        func = self.convert_variant_app_to_construct(func);

        Ok(func)
    }

    /// Convert function applications of variant constructors into VariantConstruct nodes.
    ///
    /// This enables F#-style variant syntax like `Some 42` instead of just `Some(42)`.
    /// The function recursively collects all arguments applied to a variant constructor
    /// and creates a single VariantConstruct with those arguments as fields.
    ///
    /// Examples:
    /// - `Some 42` (App { func: VariantConstruct("Some", []), arg: 42 })
    ///   becomes `VariantConstruct("Some", [42])`
    /// - `Rectangle 10 20` becomes `VariantConstruct("Rectangle", [10, 20])`
    fn convert_variant_app_to_construct(&self, expr: Expr) -> Expr {
        match expr {
            Expr::App { func, arg } => {
                // Check if the function is a VariantConstruct (possibly nested in more Apps)
                let (variant_opt, mut args) = self.extract_variant_and_args(*func, vec![]);
                args.push(*arg);

                if let Some((type_name, variant)) = variant_opt {
                    // Convert to VariantConstruct with all collected arguments as fields
                    Expr::VariantConstruct {
                        type_name,
                        variant,
                        fields: args.into_iter().map(Box::new).collect(),
                    }
                } else {
                    // Not a variant application, just return the original App
                    // Rebuild from the collected expressions
                    self.rebuild_app_from_args(args)
                }
            }
            _ => expr,
        }
    }
    /// Extract variant constructor and arguments from a chain of App nodes.
    /// Returns (Some((type_name, variant)), args) if leftmost is a VariantConstruct,
    /// or (None, args) otherwise.
    fn extract_variant_and_args(
        &self,
        func: Expr,
        mut current_args: Vec<Expr>,
    ) -> (Option<(String, String)>, Vec<Expr>) {
        match func {
            Expr::VariantConstruct {
                type_name,
                variant,
                fields,
            } => {
                // Found the variant constructor
                // Prepend any existing fields to our arguments
                let mut all_fields: Vec<Expr> = fields.into_iter().map(|b| *b).collect();
                all_fields.extend(current_args);
                (Some((type_name, variant)), all_fields)
            }
            Expr::App {
                func: inner_func,
                arg,
            } => {
                // Another App in the chain, collect the argument and recurse
                current_args.insert(0, *arg);
                self.extract_variant_and_args(*inner_func, current_args)
            }
            _ => {
                // Not a variant constructor, return None and include the leftmost expr in args
                current_args.insert(0, func);
                (None, current_args)
            }
        }
    }

    /// Extract the leftmost expression and all arguments from a chain of App nodes.
    fn extract_app_chain(&self, expr: Expr) -> (Expr, Vec<Expr>) {
        match expr {
            Expr::App { func, arg } => {
                let (leftmost, mut args) = self.extract_app_chain(*func);
                args.push(*arg);
                (leftmost, args)
            }
            _ => (expr, vec![]),
        }
    }

    /// Rebuild App nodes from a list of expressions (left-associative).
    fn rebuild_app_from_args(&self, mut exprs: Vec<Expr>) -> Expr {
        if exprs.is_empty() {
            return Expr::Lit(Literal::Unit); // Shouldn't happen
        }

        let mut result = exprs.remove(0);
        for arg in exprs {
            result = Expr::App {
                func: Box::new(result),
                arg: Box::new(arg),
            };
        }
        result
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
                        // Could be record field access or method call
                        let field = self.expect_ident()?;

                        // Check if the base expression is a module (uppercase identifier)
                        // If so, use record access (for F#-style Module.function calls)
                        // not method call syntax
                        let is_module =
                            matches!(&expr, Expr::Var(name) if Self::is_uppercase_ident(name));

                        // Check if followed by '(' to distinguish method call from field access
                        // But don't use method call for module access (List.map, Array.ofList, etc.)
                        if self.current_token().token == Token::LParen && !is_module {
                            // Method call: obj.method(args)
                            self.advance(); // consume '('

                            let mut args = vec![];

                            // Parse arguments
                            if !matches!(self.current_token().token, Token::RParen) {
                                loop {
                                    args.push(self.parse_expr()?);

                                    if !self.match_token(&Token::Comma) {
                                        break;
                                    }
                                }
                            }

                            self.expect_token(Token::RParen)?;

                            expr = Expr::MethodCall {
                                receiver: Box::new(expr),
                                method_name: field,
                                args,
                            };
                        } else {
                            // Regular field access (also for module function access)
                            expr = Expr::RecordAccess {
                                record: Box::new(expr),
                                field,
                            };
                        }
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
                    } else {
                        break;
                    }
                }

                self.expect_token(Token::RBrace)?;

                return Ok(Expr::RecordUpdate { record, fields });
            }
        }

        // Not a record update, restore position and parse as record literal
        self.pos = save_pos;

        // Parse record literal fields
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
            } else {
                break;
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Expr::RecordLiteral {
            type_name: String::new(),
            fields,
        })
    }

    /// Parse anonymous record literal: {| name = "John"; age = 30 |}
    /// Anonymous records use the {| ... |} syntax and work exactly like regular records
    /// but without requiring a type declaration.
    fn parse_anonymous_record_literal(&mut self) -> Result<Expr> {
        self.expect_token(Token::LBracePipe)?;

        // Empty anonymous record: {||}
        if self.match_token(&Token::PipeRBrace) {
            return Ok(Expr::RecordLiteral {
                type_name: String::new(),
                fields: vec![],
            });
        }

        // Anonymous records don't support the 'with' update syntax in the literal itself
        // Parse fields
        let mut fields = vec![];

        loop {
            let field_name = self.expect_ident()?;
            self.expect_token(Token::Eq)?;
            let value = self.parse_expr()?;
            fields.push((field_name, Box::new(value)));

            // Check for semicolon or closing brace
            if self.match_token(&Token::Semicolon) {
                // Check for trailing semicolon before |}
                if matches!(self.current_token().token, Token::PipeRBrace) {
                    break;
                }
            } else if matches!(self.current_token().token, Token::PipeRBrace) {
                // No semicolon, but closing brace - this is ok for last field
                break;
            } else {
                // No semicolon and no closing brace - error
                let tok = self.current_token();
                return Err(ParseError::UnexpectedToken {
                    expected: "';' or '|}'".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                });
            }
        }

        self.expect_token(Token::PipeRBrace)?;

        Ok(Expr::RecordLiteral {
            type_name: String::new(), // Anonymous records have no type name
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
            // This will be converted to VariantConstruct with fields if followed by arguments
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

                // Check for Array.length (special case for compatibility)
                if val == "Array" && self.match_token(&Token::Dot) {
                    let method = self.expect_ident()?;
                    if method == "length" {
                        let arr = Box::new(self.parse_postfix_expr()?);
                        return Ok(Expr::ArrayLength(arr));
                    } else {
                        // For other Array methods, treat as module access like List.map
                        let record = Expr::Var("Array".to_string());
                        return Ok(Expr::RecordAccess {
                            record: Box::new(record),
                            field: method,
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
                    // Just grouped expression: (e)
                    self.expect_token(Token::RParen)?;
                    Ok(first_expr)
                }
            }
            Token::LBracket => self.parse_list(),
            Token::LBracketPipe => self.parse_array(),
            Token::LBracePipe => self.parse_anonymous_record_literal(),
            Token::LBrace => self.parse_record_literal(),
            Token::Async => self.parse_computation_expr(),
            _ => {
                let tok = self.current_token();
                Err(ParseError::UnexpectedToken {
                    expected: "expression".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                })
            }
        }
    }

    /// Parse list: [1, 2, 3] or [1; 2; 3] or []
    ///
    /// Supports both comma and semicolon separators for backward compatibility.
    /// Trailing separators are allowed: [1, 2, 3,] or [1; 2; 3;]
    fn parse_list(&mut self) -> Result<Expr> {
        self.expect_token(Token::LBracket)?;

        // Empty list: []
        if self.match_token(&Token::RBracket) {
            return Ok(Expr::List(vec![]));
        }

        // Parse list elements
        let mut elements = vec![];

        loop {
            elements.push(self.parse_expr()?);

            // Check for comma or semicolon separator
            if self.match_token(&Token::Comma) || self.match_token(&Token::Semicolon) {
                // Check for trailing separator before ]
                if matches!(self.current_token().token, Token::RBracket) {
                    break;
                }
                // Otherwise continue parsing
            } else {
                break;
            }
        }

        self.expect_token(Token::RBracket)?;

        Ok(Expr::List(elements))
    }

    /// Parse array: [|1; 2; 3|] or [||]
    fn parse_array(&mut self) -> Result<Expr> {
        self.expect_token(Token::LBracketPipe)?;

        // Empty array: [||]
        if self.match_token(&Token::PipeRBracket) {
            return Ok(Expr::Array(vec![]));
        }

        // Parse array elements
        let mut elements = vec![];

        loop {
            elements.push(self.parse_expr()?);

            // Check for semicolon separator
            if self.match_token(&Token::Semicolon) {
                // Check for trailing semicolon before |]
                if matches!(self.current_token().token, Token::PipeRBracket) {
                    break;
                }
                // Otherwise continue parsing
            } else {
                break;
            }
        }

        self.expect_token(Token::PipeRBracket)?;

        Ok(Expr::Array(elements))
    }

    /// Parse DU type definition: type Option = Some of int | None
    pub fn parse_du_type_def(&mut self) -> Result<DuTypeDef> {
        self.expect_token(Token::Type)?;

        let type_name = self.expect_ident()?;

        self.expect_token(Token::Eq)?;

        // Parse variants
        let mut variants = vec![];

        // Optional leading pipe
        if self.match_token(&Token::Pipe) {
            // If we consumed a leading pipe, there must be a variant name following
            // This catches errors like "type T = | " or "type T = | |"
            let tok = self.current_token();
            if !matches!(&tok.token, Token::Ident(_)) {
                return Err(ParseError::UnexpectedToken {
                    expected: "variant name".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                });
            }
        }

        loop {
            let variant_name = self.expect_ident()?;

            // Check for 'of' keyword (variant with fields)
            let fields = if self.match_token(&Token::Of) {
                let mut field_types = vec![];

                loop {
                    let ty = self.parse_type_expr()?;
                    field_types.push(ty);

                    // Check for * separator (for tuple-like fields)
                    if !self.match_token(&Token::Star) {
                        break;
                    }
                }

                field_types
            } else {
                vec![]
            };

            variants.push(VariantDef {
                name: variant_name,
                fields,
            });

            // Check for another variant (starts with |)
            if !self.match_token(&Token::Pipe) {
                break;
            }
        }

        Ok(DuTypeDef {
            name: type_name,
            variants,
        })
    }

    /// Parse type expression for type annotations
    fn parse_type_expr(&mut self) -> Result<TypeExpr> {
        // Parse a simple type or function type
        // Note: tuple types (int * int) are handled by the caller (parse_du_type_def)
        // because the * separator is used differently in DU definitions
        let left = self.parse_simple_type()?;

        // Check for function type: int -> string
        if self.match_token(&Token::Arrow) {
            let right = self.parse_type_expr()?; // Right-associative
            return Ok(TypeExpr::Function(Box::new(left), Box::new(right)));
        }

        Ok(left)
    }

    /// Parse simple type (identifier)
    fn parse_simple_type(&mut self) -> Result<TypeExpr> {
        let name = self.expect_ident()?;
        Ok(TypeExpr::Named(name))
    }

    // ========================================================================
    // Computation Expression Parsing
    // ========================================================================

    /// Parse computation expression: async { statements... }
    ///
    /// A computation expression is of the form:
    ///   async { let! x = expr; do! expr; return expr }
    ///
    /// Future: could support other builders like "seq", "option", "result"
    fn parse_computation_expr(&mut self) -> Result<Expr> {
        // Get builder name from current token
        let builder = match &self.current_token().token {
            Token::Async => {
                self.advance(); // consume 'async'
                "async".to_string()
            }
            _ => {
                let tok = self.current_token();
                return Err(ParseError::UnexpectedToken {
                    expected: "computation expression keyword".to_string(),
                    found: tok.token.clone(),
                    pos: tok.pos,
                });
            }
        };

        // Expect '{'
        self.expect_token(Token::LBrace)?;

        // Parse CE statements
        let mut body = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            body.push(self.parse_ce_statement()?);
            // Optional semicolons between statements
            self.match_token(&Token::Semicolon);
        }

        // Expect '}'
        self.expect_token(Token::RBrace)?;

        Ok(Expr::ComputationExpr { builder, body })
    }

    /// Parse a single CE statement
    ///
    /// CE statements include:
    /// - let! x = expr    (bind operation)
    /// - let x = expr     (regular binding)
    /// - do! expr         (bind with unit result)
    /// - return expr      (return value)
    /// - return! expr     (return from another CE)
    /// - yield expr       (yield value for sequence)
    /// - yield! expr      (yield from another sequence)
    /// - expr             (plain expression)
    fn parse_ce_statement(&mut self) -> Result<CEStatement> {
        let tok = &self.current_token().token;

        match tok {
            Token::LetBang => {
                self.advance(); // consume let!
                let name = if self.match_token(&Token::Underscore) {
                    "_".to_string()
                } else {
                    self.expect_ident()?
                };
                self.expect_token(Token::Eq)?;
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::LetBang { name, value })
            }
            Token::Let => {
                self.advance(); // consume let
                let name = if self.match_token(&Token::Underscore) {
                    "_".to_string()
                } else {
                    self.expect_ident()?
                };
                self.expect_token(Token::Eq)?;
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::Let { name, value })
            }
            Token::DoBang => {
                self.advance(); // consume do!
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::DoBang { value })
            }
            Token::Return => {
                self.advance(); // consume return
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::Return { value })
            }
            Token::ReturnBang => {
                self.advance(); // consume return!
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::ReturnBang { value })
            }
            Token::Yield => {
                self.advance(); // consume yield
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::Yield { value })
            }
            Token::YieldBang => {
                self.advance(); // consume yield!
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::YieldBang { value })
            }
            _ => {
                // Plain expression
                let value = Box::new(self.parse_expr()?);
                Ok(CEStatement::Expr { value })
            }
        }
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Get the current token (or EOF if at end)
    fn current_token(&self) -> &TokenWithPos {
        if self.is_at_end() {
            // Return a special EOF token
            static EOF_TOKEN: std::sync::OnceLock<TokenWithPos> = std::sync::OnceLock::new();
            EOF_TOKEN.get_or_init(|| TokenWithPos {
                token: Token::Eof,
                pos: Position::new(0, 0, 0),
            })
        } else {
            &self.tokens[self.pos]
        }
    }

    /// Peek at the current token without consuming it
    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            None
        } else {
            Some(&self.tokens[self.pos].token)
        }
    }

    /// Check if current token matches without consuming
    fn check(&self, token: &Token) -> bool {
        !self.is_at_end() && &self.tokens[self.pos].token == token
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    /// Check if we're at EOF
    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.tokens[self.pos].token, Token::Eof)
    }

    /// Match and consume token if it matches
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect a specific token and consume it
    fn expect_token(&mut self, expected: Token) -> Result<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            let tok = self.current_token();
            Err(ParseError::UnexpectedToken {
                expected: format!("{}", expected),
                found: tok.token.clone(),
                pos: tok.pos,
            })
        }
    }

    /// Expect an identifier and return it
    fn expect_ident(&mut self) -> Result<String> {
        let tok = self.current_token();
        match &tok.token {
            Token::Ident(name) => {
                let val = name.clone();
                self.advance();
                Ok(val)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: tok.token.clone(),
                pos: tok.pos,
            }),
        }
    }

    /// Check if current token could start a primary expression
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

    /// Try to match comparison operator
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

    /// Try to match addition operator
    fn match_add_op(&mut self) -> Option<BinOp> {
        let tok = &self.current_token().token;
        let op = match tok {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            Token::PlusPlus => Some(BinOp::Concat),
            _ => None,
        };

        if op.is_some() {
            self.advance();
        }

        op
    }

    /// Try to match multiplication operator
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_str(input: &str) -> Result<Expr> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_int() {
        let expr = parse_str("42").unwrap();
        assert!(matches!(expr, Expr::Lit(Literal::Int(42))));
    }

    #[test]
    fn test_parse_let() {
        let expr = parse_str("let x = 42 in x").unwrap();
        assert!(expr.is_let());
    }

    #[test]
    fn test_parse_application() {
        let expr = parse_str("f x").unwrap();
        assert!(expr.is_app());
    }

    #[test]
    fn test_parse_lambda() {
        let expr = parse_str("fun x -> x").unwrap();
        assert!(expr.is_lambda());
    }

    #[test]
    fn test_parse_string_concat() {
        let expr = parse_str(r#""hello" ++ "world""#).unwrap();
        match expr {
            Expr::BinOp { op, left, right } => {
                assert_eq!(op, BinOp::Concat);
                assert!(matches!(*left, Expr::Lit(Literal::Str(_))));
                assert!(matches!(*right, Expr::Lit(Literal::Str(_))));
            }
            _ => panic!("Expected BinOp with Concat"),
        }
    }

    #[test]
    fn test_parse_string_concat_chain() {
        let expr = parse_str(r#""a" ++ "b" ++ "c""#).unwrap();
        // Should parse as left-associative: ("a" ++ "b") ++ "c"
        match expr {
            Expr::BinOp { op, left, right: _ } => {
                assert_eq!(op, BinOp::Concat);
                match *left {
                    Expr::BinOp { op, .. } => {
                        assert_eq!(op, BinOp::Concat);
                    }
                    _ => panic!("Expected nested BinOp"),
                }
            }
            _ => panic!("Expected BinOp"),
        }
    }

    #[test]
    fn test_parse_concat_precedence() {
        // ++ should have same precedence as +, so this should parse correctly
        let expr = parse_str(r#""http://" ++ host ++ ":" ++ port"#).unwrap();
        match expr {
            Expr::BinOp { op, .. } => {
                assert_eq!(op, BinOp::Concat);
            }
            _ => panic!("Expected BinOp"),
        }
    }

    // ========================================================================
    // Computation Expression Tests
    // ========================================================================

    #[test]
    fn test_parse_async_block_empty() {
        let mut lexer = Lexer::new("async { }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        assert!(expr.is_computation_expr());
        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 0);
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_async_block_return() {
        let mut lexer = Lexer::new("async { return 42 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        assert!(expr.is_computation_expr());
        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    CEStatement::Return { value } => {
                        assert!(matches!(**value, Expr::Lit(Literal::Int(42))));
                    }
                    _ => panic!("Expected Return statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_let_bang() {
        let mut lexer = Lexer::new("async { let! x = op; return x }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        assert!(expr.is_computation_expr());
        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 2);

                // First statement: let! x = op
                match &body[0] {
                    CEStatement::LetBang { name, value } => {
                        assert_eq!(name, "x");
                        assert!(matches!(**value, Expr::Var(_)));
                    }
                    _ => panic!("Expected LetBang statement"),
                }

                // Second statement: return x
                match &body[1] {
                    CEStatement::Return { value } => {
                        assert!(matches!(**value, Expr::Var(_)));
                    }
                    _ => panic!("Expected Return statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_do_bang() {
        let mut lexer = Lexer::new("async { do! print 42 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    CEStatement::DoBang { value } => {
                        assert!(matches!(**value, Expr::App { .. }));
                    }
                    _ => panic!("Expected DoBang statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_regular_let_in_ce() {
        let mut lexer = Lexer::new("async { let x = 42; return x }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 2);

                // First statement: let x = 42
                match &body[0] {
                    CEStatement::Let { name, value } => {
                        assert_eq!(name, "x");
                        assert!(matches!(**value, Expr::Lit(Literal::Int(42))));
                    }
                    _ => panic!("Expected Let statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_return_bang() {
        let mut lexer = Lexer::new("async { return! other }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    CEStatement::ReturnBang { value } => {
                        assert!(matches!(**value, Expr::Var(_)));
                    }
                    _ => panic!("Expected ReturnBang statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_yield() {
        let mut lexer = Lexer::new("async { yield 1; yield 2 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 2);
                match &body[0] {
                    CEStatement::Yield { value } => {
                        assert!(matches!(**value, Expr::Lit(Literal::Int(1))));
                    }
                    _ => panic!("Expected Yield statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_yield_bang() {
        let mut lexer = Lexer::new("async { yield! sequence }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    CEStatement::YieldBang { value } => {
                        assert!(matches!(**value, Expr::Var(_)));
                    }
                    _ => panic!("Expected YieldBang statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_plain_expr_in_ce() {
        let mut lexer = Lexer::new("async { 1 + 2 }");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
                match &body[0] {
                    CEStatement::Expr { value } => {
                        assert!(matches!(**value, Expr::BinOp { .. }));
                    }
                    _ => panic!("Expected Expr statement"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_complex_async_block() {
        let input = r#"
            async {
                let! x = fetchData;
                let y = x + 1;
                do! printValue y;
                return y
            }
        "#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 4);

                // Verify statement types
                assert!(matches!(body[0], CEStatement::LetBang { .. }));
                assert!(matches!(body[1], CEStatement::Let { .. }));
                assert!(matches!(body[2], CEStatement::DoBang { .. }));
                assert!(matches!(body[3], CEStatement::Return { .. }));
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_nested_async_blocks() {
        let input = "async { let! x = async { return 42 }; return x }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 2);

                // First statement should have nested CE
                match &body[0] {
                    CEStatement::LetBang { value, .. } => {
                        assert!(matches!(**value, Expr::ComputationExpr { .. }));
                    }
                    _ => panic!("Expected LetBang"),
                }
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_async_with_optional_semicolons() {
        // Test that semicolons are optional between statements
        let input = "async { let x = 1 let y = 2 return x }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_parse_async_with_trailing_semicolon() {
        let input = "async { return 42; }";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();

        match expr {
            Expr::ComputationExpr { builder, body } => {
                assert_eq!(builder, "async");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ComputationExpr"),
        }
    }

    #[test]
    fn test_computation_expr_helpers() {
        let expr = Expr::ComputationExpr {
            builder: "async".to_string(),
            body: vec![],
        };

        assert!(expr.is_computation_expr());
        assert!(!expr.is_let());
        assert!(!expr.is_if());

        let result = expr.as_computation_expr();
        assert!(result.is_some());
        let (builder, body) = result.unwrap();
        assert_eq!(builder, "async");
        assert_eq!(body.len(), 0);
    }
}
