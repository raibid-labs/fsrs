# Issue #003: Parser Implementation

## Overview
Implement the recursive-descent parser that transforms token streams into AST. This is the second stage of the compilation pipeline.

## Labels
- `feature`
- `phase-1: mvp`
- `priority: high`
- `foundational`
- `component: frontend`
- `effort: m` (3-4 days)

## Milestone
Phase 1.1: Frontend Foundation (Week 1)

## Dependencies
- #001 (Core AST) - **MUST BE COMPLETE** - Parser produces AST nodes
- #002 (Lexer) - Recommended but not blocking - Can use mock tokens for development

## Acceptance Criteria
- [ ] Parse let-bindings: `let x = expr`
- [ ] Parse function definitions: `let f x = body`
- [ ] Parse if/then/else: `if cond then t else f`
- [ ] Parse binary operations with correct precedence
- [ ] Parse function application
- [ ] Parse lambdas: `fun x -> body`
- [ ] Parse parenthesized expressions
- [ ] Error recovery and clear error messages
- [ ] 30+ parser tests covering all constructs

## Technical Specification

### File Location
`rust/crates/fsrs-frontend/src/parser.rs`

### Core Parser Structure

```rust
use crate::ast::{Expr, Literal, BinOp};
use crate::lexer::{Token, TokenWithPos, Position};

pub struct Parser {
    tokens: Vec<TokenWithPos>,
    pos: usize,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: Token,
        pos: Position,
    },
    UnexpectedEof,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithPos>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr()
    }

    // Recursive descent parsing
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_let_or_comparison()
    }

    fn parse_let_or_comparison(&mut self) -> Result<Expr, ParseError> {
        if self.check(&Token::Let) {
            self.parse_let()
        } else if self.check(&Token::If) {
            self.parse_if()
        } else if self.check(&Token::Fun) {
            self.parse_lambda()
        } else {
            self.parse_comparison()
        }
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.consume(&Token::Let)?;

        let name = match self.current_token() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current_token().clone(),
                pos: self.current_position(),
            }),
        };

        self.consume(&Token::Eq)?;
        let value = Box::new(self.parse_expr()?);
        self.consume(&Token::In)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Expr::Let { name, value, body })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.consume(&Token::If)?;
        let cond = Box::new(self.parse_expr()?);
        self.consume(&Token::Then)?;
        let then_branch = Box::new(self.parse_expr()?);
        self.consume(&Token::Else)?;
        let else_branch = Box::new(self.parse_expr()?);

        Ok(Expr::If { cond, then_branch, else_branch })
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        self.consume(&Token::Fun)?;

        let param = match self.current_token() {
            Token::Ident(s) => {
                let p = s.clone();
                self.advance();
                p
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: "parameter name".to_string(),
                found: self.current_token().clone(),
                pos: self.current_position(),
            }),
        };

        self.consume(&Token::Arrow)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Expr::Lambda { param, body })
    }

    // Binary operations with precedence
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;

        while matches!(
            self.current_token(),
            Token::EqEq | Token::Neq | Token::Lt | Token::Lte | Token::Gt | Token::Gte
        ) {
            let op = match self.current_token() {
                Token::EqEq => BinOp::Eq,
                Token::Neq => BinOp::Neq,
                Token::Lt => BinOp::Lt,
                Token::Lte => BinOp::Lte,
                Token::Gt => BinOp::Gt,
                Token::Gte => BinOp::Gte,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_addition()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;

        while matches!(self.current_token(), Token::Plus | Token::Minus) {
            let op = match self.current_token() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_multiplication()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_application()?;

        while matches!(self.current_token(), Token::Star | Token::Slash) {
            let op = match self.current_token() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_application()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_application(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        // Left-associative function application
        while !self.is_at_end()
            && !matches!(
                self.current_token(),
                Token::Plus | Token::Minus | Token::Star | Token::Slash
                | Token::Then | Token::Else | Token::In
                | Token::RParen | Token::Eof
            )
        {
            let arg = self.parse_primary()?;
            expr = Expr::App {
                func: Box::new(expr),
                arg: Box::new(arg),
            };
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current_token() {
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
                let var = name.clone();
                self.advance();
                Ok(Expr::Var(var))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(&Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current_token().clone(),
                pos: self.current_position(),
            }),
        }
    }

    // Helper methods
    fn current_token(&self) -> &Token {
        if self.is_at_end() {
            &Token::Eof
        } else {
            &self.tokens[self.pos].token
        }
    }

    fn current_position(&self) -> Position {
        if self.is_at_end() {
            self.tokens.last().unwrap().pos
        } else {
            self.tokens[self.pos].pos
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.current_token(), Token::Eof)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(token)
    }

    fn consume(&mut self, expected: &Token) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token().clone(),
                pos: self.current_position(),
            })
        }
    }
}
```

## Testing Requirements

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expr(input: &str) -> Expr {
        let mut lexer = crate::lexer::Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn test_literal() {
        let expr = parse_expr("42");
        assert!(matches!(expr, Expr::Lit(Literal::Int(42))));
    }

    #[test]
    fn test_variable() {
        let expr = parse_expr("x");
        assert!(matches!(expr, Expr::Var(_)));
    }

    #[test]
    fn test_addition() {
        let expr = parse_expr("1 + 2");
        assert!(matches!(expr, Expr::BinOp { op: BinOp::Add, .. }));
    }

    #[test]
    fn test_precedence() {
        let expr = parse_expr("1 + 2 * 3");
        // Should parse as 1 + (2 * 3)
        if let Expr::BinOp { op: BinOp::Add, right, .. } = expr {
            assert!(matches!(*right, Expr::BinOp { op: BinOp::Mul, .. }));
        } else {
            panic!("Expected addition at top level");
        }
    }

    #[test]
    fn test_let_binding() {
        let expr = parse_expr("let x = 10 in x + 1");
        assert!(matches!(expr, Expr::Let { .. }));
    }

    #[test]
    fn test_if_expression() {
        let expr = parse_expr("if true then 1 else 0");
        assert!(matches!(expr, Expr::If { .. }));
    }

    #[test]
    fn test_lambda() {
        let expr = parse_expr("fun x -> x + 1");
        assert!(matches!(expr, Expr::Lambda { .. }));
    }

    #[test]
    fn test_function_application() {
        let expr = parse_expr("f x");
        assert!(matches!(expr, Expr::App { .. }));
    }

    #[test]
    fn test_complex_expression() {
        let expr = parse_expr("let add x y = x + y in add 1 2");
        // Should parse without errors
        assert!(matches!(expr, Expr::Let { .. }));
    }
}
```

## Estimated Effort
**3-4 days**

## Related Issues
- Depends on #001 (AST)
- Uses #002 (Lexer)
- Blocks #007 (Compiler)

## Notes
⚠️ **FOUNDATIONAL**: Compiler depends on this