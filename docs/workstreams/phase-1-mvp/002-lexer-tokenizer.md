# Issue #002: Lexer and Tokenizer Implementation

## Overview
Implement the lexical analyzer (lexer/tokenizer) that converts Mini-F# source code into a stream of tokens. This is the first stage of the compilation pipeline.

## Labels
- `feature`
- `phase-1: mvp`
- `priority: high`
- `foundational`
- `parallel-safe`
- `component: frontend`
- `effort: s` (1-2 days)

## Milestone
Phase 1.1: Frontend Foundation (Week 1)

## Dependencies
None - Can work in parallel with #001 (AST)

## Acceptance Criteria
- [ ] Token enum defined with all Phase 1 token types
- [ ] Lexer struct with tokenize method
- [ ] Support for keywords: `let`, `in`, `if`, `then`, `else`, `fun`, `true`, `false`
- [ ] Support for identifiers and literals (int, float, bool, string)
- [ ] Support for operators and punctuation
- [ ] Position tracking for error reporting
- [ ] Comprehensive unit tests covering all token types
- [ ] Error handling for invalid characters

## Technical Specification

### File Location
`rust/crates/fsrs-frontend/src/lexer.rs`

### Token Types

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    // Identifiers
    Ident(String),

    // Keywords
    Let,
    In,
    If,
    Then,
    Else,
    Fun,

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Eq,         // =
    EqEq,       // ==
    Neq,        // <>
    Lt,         // <
    Lte,        // <=
    Gt,         // >
    Gte,        // >=
    And,        // &&
    Or,         // ||

    // Punctuation
    LParen,     // (
    RParen,     // )
    Arrow,      // ->

    // Special
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos {
    pub token: Token,
    pub pos: Position,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}
```

### Implementation

```rust
impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            let start_pos = self.current_position();
            let token = self.next_token()?;
            tokens.push(TokenWithPos {
                token,
                pos: start_pos,
            });
        }

        tokens.push(TokenWithPos {
            token: Token::Eof,
            pos: self.current_position(),
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.current_char();

        match ch {
            '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier_or_keyword(),
            '"' => self.lex_string(),
            '+' => { self.advance(); Ok(Token::Plus) }
            '-' => self.lex_minus_or_arrow(),
            '*' => { self.advance(); Ok(Token::Star) }
            '/' => { self.advance(); Ok(Token::Slash) }
            '=' => self.lex_eq_or_eqeq(),
            '<' => self.lex_lt_or_lte_or_neq(),
            '>' => self.lex_gt_or_gte(),
            '&' => self.lex_and(),
            '|' => self.lex_or(),
            '(' => { self.advance(); Ok(Token::LParen) }
            ')' => { self.advance(); Ok(Token::RParen) }
            _ => Err(LexError::UnexpectedChar(ch, self.current_position())),
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while self.current_char().is_ascii_digit() {
            self.advance();
        }

        // Check for float
        if self.current_char() == '.' && self.peek_char().is_ascii_digit() {
            self.advance(); // consume '.'
            while self.current_char().is_ascii_digit() {
                self.advance();
            }
            let s: String = self.input[start..self.pos].iter().collect();
            Ok(Token::Float(s.parse().unwrap()))
        } else {
            let s: String = self.input[start..self.pos].iter().collect();
            Ok(Token::Int(s.parse().unwrap()))
        }
    }

    fn lex_identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        while self.current_char().is_alphanumeric() || self.current_char() == '_' {
            self.advance();
        }
        let s: String = self.input[start..self.pos].iter().collect();

        let token = match s.as_str() {
            "let" => Token::Let,
            "in" => Token::In,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "fun" => Token::Fun,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Ident(s),
        };

        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current_char() {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }
                '/' if self.peek_char() == '/' => {
                    // Line comment
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.pos]
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn current_position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
            offset: self.pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    UnexpectedChar(char, Position),
    UnterminatedString(Position),
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("+ - * /");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 5); // 4 tokens + EOF
        assert_eq!(tokens[0].token, Token::Plus);
        assert_eq!(tokens[1].token, Token::Minus);
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("let in if then else");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::In);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_123 camelCase");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token, Token::Ident(_)));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Float(3.14));
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = Lexer::new("let\nx");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[1].pos.line, 2);
    }
}
```

## Estimated Effort
**1-2 days**

## Related Issues
- Used by #003 (Parser)
- Parallel with #001 (AST)

## Notes
✅ **PARALLEL-SAFE**: Can develop independently