//! Lexer/Tokenizer for Mini-F# source code.
//!
//! This module implements the lexical analyzer that converts source text into
//! a stream of tokens. The lexer supports:
//!
//! - Literals: integers, floats, booleans, strings
//! - Keywords: let, rec, and, in, if, then, else, fun, true, false
//! - Identifiers: alphanumeric names starting with letter or underscore
//! - Operators: arithmetic, comparison, logical
//! - Punctuation: parentheses, arrows, commas
//! - Position tracking for error reporting
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::lexer::{Lexer, Token};
//!
//! let mut lexer = Lexer::new("let x = 42");
//! let tokens = lexer.tokenize().unwrap();
//! assert_eq!(tokens[0].token, Token::Let);
//! assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
//! ```

use std::fmt;

/// Token types in Mini-F# source code.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Integer literal (e.g., 42, -10)
    Int(i64),
    /// Floating-point literal (e.g., 3.15, -0.5)
    Float(f64),
    /// Boolean literal (true or false)
    Bool(bool),
    /// String literal (e.g., "hello")
    String(String),

    // Identifiers
    /// Identifier (variable or function name)
    Ident(String),

    // Keywords
    /// let keyword
    Let,
    /// rec keyword (for recursive bindings)
    Rec,
    /// and keyword (for mutual recursion)
    AndKeyword,
    /// in keyword
    In,
    /// if keyword
    If,
    /// then keyword
    Then,
    /// else keyword
    Else,
    /// fun keyword
    Fun,

    // Operators
    /// + operator
    Plus,
    /// - operator
    Minus,
    /// * operator
    Star,
    /// / operator
    Slash,
    /// = operator
    Eq,
    /// == operator (equality comparison)
    EqEq,
    /// <> operator (inequality)
    Neq,
    /// < operator
    Lt,
    /// <= operator
    Lte,
    /// > operator
    Gt,
    /// >= operator
    Gte,
    /// && operator
    And,
    /// || operator
    Or,

    // Punctuation
    /// ( left parenthesis
    LParen,
    /// ) right parenthesis
    RParen,
    /// -> arrow
    Arrow,
    /// , comma
    Comma,

    // Special
    /// End of file marker
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Int(n) => write!(f, "Int({})", n),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::Bool(b) => write!(f, "Bool({})", b),
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::Rec => write!(f, "rec"),
            Token::AndKeyword => write!(f, "and"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Fun => write!(f, "fun"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Eq => write!(f, "="),
            Token::EqEq => write!(f, "=="),
            Token::Neq => write!(f, "<>"),
            Token::Lt => write!(f, "<"),
            Token::Lte => write!(f, "<="),
            Token::Gt => write!(f, ">"),
            Token::Gte => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Arrow => write!(f, "->"),
            Token::Comma => write!(f, ","),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Position information for a token in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in source (0-indexed)
    pub offset: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position {
            line,
            column,
            offset,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A token with its position in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos {
    /// The token
    pub token: Token,
    /// Position in source
    pub pos: Position,
}

impl TokenWithPos {
    /// Create a new token with position.
    pub fn new(token: Token, pos: Position) -> Self {
        TokenWithPos { token, pos }
    }
}

/// Lexical analysis errors.
#[derive(Debug, Clone, PartialEq)]
pub enum LexError {
    /// Unexpected character encountered
    UnexpectedChar(char, Position),
    /// Unterminated string literal
    UnterminatedString(Position),
    /// Invalid number format
    InvalidNumber(String, Position),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar(ch, pos) => {
                write!(f, "Unexpected character '{}' at {}", ch, pos)
            }
            LexError::UnterminatedString(pos) => {
                write!(f, "Unterminated string literal at {}", pos)
            }
            LexError::InvalidNumber(s, pos) => {
                write!(f, "Invalid number '{}' at {}", s, pos)
            }
        }
    }
}

impl std::error::Error for LexError {}

/// Lexer for Mini-F# source code.
///
/// The lexer converts source text into a stream of tokens with position information.
pub struct Lexer {
    /// Input characters
    input: Vec<char>,
    /// Current position in input
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed)
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire input, returning a vector of tokens with positions.
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            let start_pos = self.current_position();
            let token = self.next_token()?;
            tokens.push(TokenWithPos::new(token, start_pos));
        }

        tokens.push(TokenWithPos::new(Token::Eof, self.current_position()));

        Ok(tokens)
    }

    /// Lex the next token from the input.
    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.current_char();

        match ch {
            '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier_or_keyword(),
            '"' => self.lex_string(),
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => self.lex_minus_or_arrow(),
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '/' => {
                self.advance();
                Ok(Token::Slash)
            }
            '=' => self.lex_eq_or_eqeq(),
            '<' => self.lex_lt_or_lte_or_neq(),
            '>' => self.lex_gt_or_gte(),
            '&' => self.lex_and(),
            '|' => self.lex_or(),
            '(' => {
                self.advance();
                Ok(Token::LParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            _ => Err(LexError::UnexpectedChar(ch, self.current_position())),
        }
    }

    /// Lex a number (integer or float).
    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let start_pos = self.current_position();

        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            self.advance();
        }

        // Check for float
        if !self.is_at_end()
            && self.current_char() == '.'
            && !self.is_at_end_or(1)
            && self.peek_char().is_ascii_digit()
        {
            self.advance(); // consume '.'
            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                self.advance();
            }
            let s: String = self.input[start..self.pos].iter().collect();
            s.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| LexError::InvalidNumber(s, start_pos))
        } else {
            let s: String = self.input[start..self.pos].iter().collect();
            s.parse::<i64>()
                .map(Token::Int)
                .map_err(|_| LexError::InvalidNumber(s, start_pos))
        }
    }

    /// Lex an identifier or keyword.
    fn lex_identifier_or_keyword(&mut self) -> Result<Token, LexError> {
        let start = self.pos;

        while !self.is_at_end()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            self.advance();
        }

        let s: String = self.input[start..self.pos].iter().collect();

        let token = match s.as_str() {
            "let" => Token::Let,
            "in" => Token::In,
            "rec" => Token::Rec,
            "and" => Token::AndKeyword,
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

    /// Lex a string literal.
    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start_pos = self.current_position();
        self.advance(); // consume opening "

        let mut s = String::new();

        while !self.is_at_end() && self.current_char() != '"' {
            let ch = self.current_char();
            if ch == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(LexError::UnterminatedString(start_pos));
                }
                let escaped = match self.current_char() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    c => c, // Unknown escape, just include the character
                };
                s.push(escaped);
                self.advance();
            } else {
                s.push(ch);
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(start_pos));
        }

        self.advance(); // consume closing "
        Ok(Token::String(s))
    }

    /// Lex - or ->.
    fn lex_minus_or_arrow(&mut self) -> Result<Token, LexError> {
        self.advance();
        if !self.is_at_end() && self.current_char() == '>' {
            self.advance();
            Ok(Token::Arrow)
        } else {
            Ok(Token::Minus)
        }
    }

    /// Lex = or ==.
    fn lex_eq_or_eqeq(&mut self) -> Result<Token, LexError> {
        self.advance();
        if !self.is_at_end() && self.current_char() == '=' {
            self.advance();
            Ok(Token::EqEq)
        } else {
            Ok(Token::Eq)
        }
    }

    /// Lex <, <=, or <>.
    fn lex_lt_or_lte_or_neq(&mut self) -> Result<Token, LexError> {
        self.advance();
        if !self.is_at_end() {
            match self.current_char() {
                '=' => {
                    self.advance();
                    Ok(Token::Lte)
                }
                '>' => {
                    self.advance();
                    Ok(Token::Neq)
                }
                _ => Ok(Token::Lt),
            }
        } else {
            Ok(Token::Lt)
        }
    }

    /// Lex > or >=.
    fn lex_gt_or_gte(&mut self) -> Result<Token, LexError> {
        self.advance();
        if !self.is_at_end() && self.current_char() == '=' {
            self.advance();
            Ok(Token::Gte)
        } else {
            Ok(Token::Gt)
        }
    }

    /// Lex &&.
    fn lex_and(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        self.advance();
        if !self.is_at_end() && self.current_char() == '&' {
            self.advance();
            Ok(Token::And)
        } else {
            Err(LexError::UnexpectedChar('&', pos))
        }
    }

    /// Lex ||.
    fn lex_or(&mut self) -> Result<Token, LexError> {
        let pos = self.current_position();
        self.advance();
        if !self.is_at_end() && self.current_char() == '|' {
            self.advance();
            Ok(Token::Or)
        } else {
            Err(LexError::UnexpectedChar('|', pos))
        }
    }

    /// Skip whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }

            match self.current_char() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 0; // Will be incremented to 1 by advance()
                    self.advance();
                }
                '/' if !self.is_at_end_or(1) && self.peek_char() == '/' => {
                    // Line comment
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    /// Get the current character without consuming it.
    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.pos]
        }
    }

    /// Peek at the next character without consuming it.
    fn peek_char(&self) -> char {
        if self.pos + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.pos + 1]
        }
    }

    /// Advance to the next character.
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
            self.column += 1;
        }
    }

    /// Check if we're at the end of input.
    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Check if we're at the end or at position + offset is beyond end.
    fn is_at_end_or(&self, offset: usize) -> bool {
        self.pos + offset >= self.input.len()
    }

    /// Get the current position.
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Basic Token Tests (TDD: Red-Green-Refactor)
    // ========================================================================

    #[test]
    fn test_lex_integer() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // Int + EOF
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Eof);
    }

    #[test]
    fn test_lex_multiple_integers() {
        let mut lexer = Lexer::new("1 23 456");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4); // 3 ints + EOF
        assert_eq!(tokens[0].token, Token::Int(1));
        assert_eq!(tokens[1].token, Token::Int(23));
        assert_eq!(tokens[2].token, Token::Int(456));
    }

    #[test]
    fn test_lex_float() {
        let mut lexer = Lexer::new("3.15");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2); // Float + EOF
        assert_eq!(tokens[0].token, Token::Float(3.15));
    }

    #[test]
    fn test_lex_multiple_floats() {
        let mut lexer = Lexer::new("1.5 2.0 3.15159");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token, Token::Float(1.5));
        assert_eq!(tokens[1].token, Token::Float(2.0));
        assert_eq!(tokens[2].token, Token::Float(3.15159));
    }

    #[test]
    fn test_lex_bool_true() {
        let mut lexer = Lexer::new("true");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Bool(true));
    }

    #[test]
    fn test_lex_bool_false() {
        let mut lexer = Lexer::new("false");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Bool(false));
    }

    #[test]
    fn test_lex_string_simple() {
        let mut lexer = Lexer::new(r#""hello""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::String("hello".to_string()));
    }

    #[test]
    fn test_lex_string_with_spaces() {
        let mut lexer = Lexer::new(r#""hello world""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::String("hello world".to_string()));
    }

    #[test]
    fn test_lex_string_with_escapes() {
        let mut lexer = Lexer::new(r#""hello\nworld\t!""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[0].token,
            Token::String("hello\nworld\t!".to_string())
        );
    }

    #[test]
    fn test_lex_string_empty() {
        let mut lexer = Lexer::new(r#""""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::String("".to_string()));
    }

    #[test]
    fn test_lex_identifier() {
        let mut lexer = Lexer::new("foo");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Ident("foo".to_string()));
    }

    #[test]
    fn test_lex_identifiers_various() {
        let mut lexer = Lexer::new("x foo bar_123 camelCase _internal");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[1].token, Token::Ident("foo".to_string()));
        assert_eq!(tokens[2].token, Token::Ident("bar_123".to_string()));
        assert_eq!(tokens[3].token, Token::Ident("camelCase".to_string()));
        assert_eq!(tokens[4].token, Token::Ident("_internal".to_string()));
    }

    // ========================================================================
    // Keyword Tests
    // ========================================================================

    #[test]
    fn test_lex_keyword_let() {
        let mut lexer = Lexer::new("let");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
    }

    #[test]
    fn test_lex_keyword_in() {
        let mut lexer = Lexer::new("in");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::In);
    }

    #[test]
    fn test_lex_keyword_if() {
        let mut lexer = Lexer::new("if");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::If);
    }

    #[test]
    fn test_lex_keyword_then() {
        let mut lexer = Lexer::new("then");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Then);
    }

    #[test]
    fn test_lex_keyword_else() {
        let mut lexer = Lexer::new("else");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Else);
    }

    #[test]
    fn test_lex_keyword_fun() {
        let mut lexer = Lexer::new("fun");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Fun);
    }

    #[test]
    fn test_lex_all_keywords() {
        let mut lexer = Lexer::new("let rec and in if then else fun");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Rec);
        assert_eq!(tokens[2].token, Token::AndKeyword);
        assert_eq!(tokens[3].token, Token::In);
        assert_eq!(tokens[4].token, Token::If);
        assert_eq!(tokens[5].token, Token::Then);
        assert_eq!(tokens[6].token, Token::Else);
        assert_eq!(tokens[7].token, Token::Fun);
    }

    // ========================================================================
    // Operator Tests
    // ========================================================================

    #[test]
    fn test_lex_arithmetic_operators() {
        let mut lexer = Lexer::new("+ - * /");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Plus);
        assert_eq!(tokens[1].token, Token::Minus);
        assert_eq!(tokens[2].token, Token::Star);
        assert_eq!(tokens[3].token, Token::Slash);
    }

    #[test]
    fn test_lex_comparison_operators() {
        let mut lexer = Lexer::new("= == <> < <= > >=");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Eq);
        assert_eq!(tokens[1].token, Token::EqEq);
        assert_eq!(tokens[2].token, Token::Neq);
        assert_eq!(tokens[3].token, Token::Lt);
        assert_eq!(tokens[4].token, Token::Lte);
        assert_eq!(tokens[5].token, Token::Gt);
        assert_eq!(tokens[6].token, Token::Gte);
    }

    #[test]
    fn test_lex_logical_operators() {
        let mut lexer = Lexer::new("&& ||");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::And);
        assert_eq!(tokens[1].token, Token::Or);
    }

    // ========================================================================
    // Punctuation Tests
    // ========================================================================

    #[test]
    fn test_lex_parentheses() {
        let mut lexer = Lexer::new("( )");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::RParen);
    }

    #[test]
    fn test_lex_arrow() {
        let mut lexer = Lexer::new("->");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Arrow);
    }

    #[test]
    fn test_lex_minus_vs_arrow() {
        let mut lexer = Lexer::new("- ->");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Minus);
        assert_eq!(tokens[1].token, Token::Arrow);
    }

    #[test]
    fn test_lex_comma() {
        let mut lexer = Lexer::new(",");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Comma);
    }

    #[test]
    fn test_lex_multiple_commas() {
        let mut lexer = Lexer::new(", , ,");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Comma);
        assert_eq!(tokens[1].token, Token::Comma);
        assert_eq!(tokens[2].token, Token::Comma);
    }

    #[test]
    fn test_lex_comma_separated_values() {
        let mut lexer = Lexer::new("1, 2, 3");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(1));
        assert_eq!(tokens[1].token, Token::Comma);
        assert_eq!(tokens[2].token, Token::Int(2));
        assert_eq!(tokens[3].token, Token::Comma);
        assert_eq!(tokens[4].token, Token::Int(3));
    }

    // ========================================================================
    // Position Tracking Tests
    // ========================================================================

    #[test]
    fn test_position_single_line() {
        let mut lexer = Lexer::new("let x");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 1);
        assert_eq!(tokens[1].pos.line, 1);
        assert_eq!(tokens[1].pos.column, 5);
    }

    #[test]
    fn test_position_multiple_lines() {
        let mut lexer = Lexer::new("let\nx");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 1);
        assert_eq!(tokens[1].pos.line, 2);
        assert_eq!(tokens[1].pos.column, 1);
    }

    #[test]
    fn test_position_with_spaces() {
        let mut lexer = Lexer::new("  let   x");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 3);
        assert_eq!(tokens[1].pos.line, 1);
        assert_eq!(tokens[1].pos.column, 9);
    }

    // ========================================================================
    // Whitespace and Comment Tests
    // ========================================================================

    #[test]
    fn test_skip_whitespace() {
        let mut lexer = Lexer::new("  \t\r\n  42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(42));
    }

    #[test]
    fn test_skip_line_comments() {
        let mut lexer = Lexer::new("42 // this is a comment\n 43");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Int(43));
    }

    #[test]
    fn test_comment_at_end() {
        let mut lexer = Lexer::new("42 // comment");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Eof);
    }

    // ========================================================================
    // Error Tests
    // ========================================================================

    #[test]
    fn test_error_unexpected_char() {
        let mut lexer = Lexer::new("@");
        let result = lexer.tokenize();
        assert!(result.is_err());
        match result.unwrap_err() {
            LexError::UnexpectedChar(ch, _) => assert_eq!(ch, '@'),
            _ => panic!("Expected UnexpectedChar error"),
        }
    }

    #[test]
    fn test_error_unterminated_string() {
        let mut lexer = Lexer::new(r#""hello"#);
        let result = lexer.tokenize();
        assert!(result.is_err());
        match result.unwrap_err() {
            LexError::UnterminatedString(_) => {}
            _ => panic!("Expected UnterminatedString error"),
        }
    }

    #[test]
    fn test_error_single_ampersand() {
        let mut lexer = Lexer::new("&");
        let result = lexer.tokenize();
        assert!(result.is_err());
        match result.unwrap_err() {
            LexError::UnexpectedChar(ch, _) => assert_eq!(ch, '&'),
            _ => panic!("Expected UnexpectedChar error"),
        }
    }

    #[test]
    fn test_error_single_pipe() {
        let mut lexer = Lexer::new("|");
        let result = lexer.tokenize();
        assert!(result.is_err());
        match result.unwrap_err() {
            LexError::UnexpectedChar(ch, _) => assert_eq!(ch, '|'),
            _ => panic!("Expected UnexpectedChar error"),
        }
    }

    // ========================================================================
    // Complex Expression Tests
    // ========================================================================

    #[test]
    fn test_lex_let_binding() {
        let mut lexer = Lexer::new("let x = 42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[2].token, Token::Eq);
        assert_eq!(tokens[3].token, Token::Int(42));
    }

    #[test]
    fn test_lex_lambda_expression() {
        let mut lexer = Lexer::new("fun x -> x + 1");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Fun);
        assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[2].token, Token::Arrow);
        assert_eq!(tokens[3].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[4].token, Token::Plus);
        assert_eq!(tokens[5].token, Token::Int(1));
    }

    #[test]
    fn test_lex_if_expression() {
        let mut lexer = Lexer::new("if x > 0 then 1 else -1");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::If);
        assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[2].token, Token::Gt);
        assert_eq!(tokens[3].token, Token::Int(0));
        assert_eq!(tokens[4].token, Token::Then);
        assert_eq!(tokens[5].token, Token::Int(1));
        assert_eq!(tokens[6].token, Token::Else);
        assert_eq!(tokens[7].token, Token::Minus);
        assert_eq!(tokens[8].token, Token::Int(1));
    }

    #[test]
    fn test_lex_arithmetic_expression() {
        let mut lexer = Lexer::new("(a + b) * c");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Ident("a".to_string()));
        assert_eq!(tokens[2].token, Token::Plus);
        assert_eq!(tokens[3].token, Token::Ident("b".to_string()));
        assert_eq!(tokens[4].token, Token::RParen);
        assert_eq!(tokens[5].token, Token::Star);
        assert_eq!(tokens[6].token, Token::Ident("c".to_string()));
    }

    #[test]
    fn test_lex_comparison_expression() {
        let mut lexer = Lexer::new("x == y && y <> z");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[1].token, Token::EqEq);
        assert_eq!(tokens[2].token, Token::Ident("y".to_string()));
        assert_eq!(tokens[3].token, Token::And);
        assert_eq!(tokens[4].token, Token::Ident("y".to_string()));
        assert_eq!(tokens[5].token, Token::Neq);
        assert_eq!(tokens[6].token, Token::Ident("z".to_string()));
    }

    #[test]
    fn test_lex_nested_let() {
        let mut lexer = Lexer::new("let x = 1 in let y = 2 in x + y");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[2].token, Token::Eq);
        assert_eq!(tokens[3].token, Token::Int(1));
        assert_eq!(tokens[4].token, Token::In);
        assert_eq!(tokens[5].token, Token::Let);
        assert_eq!(tokens[6].token, Token::Ident("y".to_string()));
        assert_eq!(tokens[7].token, Token::Eq);
        assert_eq!(tokens[8].token, Token::Int(2));
        assert_eq!(tokens[9].token, Token::In);
        assert_eq!(tokens[10].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[11].token, Token::Plus);
        assert_eq!(tokens[12].token, Token::Ident("y".to_string()));
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_lex_empty_input() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1); // Just EOF
        assert_eq!(tokens[0].token, Token::Eof);
    }

    #[test]
    fn test_lex_only_whitespace() {
        let mut lexer = Lexer::new("   \t\n  ");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1); // Just EOF
        assert_eq!(tokens[0].token, Token::Eof);
    }

    #[test]
    fn test_lex_only_comment() {
        let mut lexer = Lexer::new("// just a comment");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1); // Just EOF
        assert_eq!(tokens[0].token, Token::Eof);
    }

    #[test]
    fn test_lex_no_spaces_between_tokens() {
        let mut lexer = Lexer::new("let(x)=42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::LParen);
        assert_eq!(tokens[2].token, Token::Ident("x".to_string()));
        assert_eq!(tokens[3].token, Token::RParen);
        assert_eq!(tokens[4].token, Token::Eq);
        assert_eq!(tokens[5].token, Token::Int(42));
    }

    #[test]
    fn test_lex_mixed_integers_floats() {
        let mut lexer = Lexer::new("42 3.15 100 2.5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Float(3.15));
        assert_eq!(tokens[2].token, Token::Int(100));
        assert_eq!(tokens[3].token, Token::Float(2.5));
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::Let), "let");
        assert_eq!(format!("{}", Token::Int(42)), "Int(42)");
        assert_eq!(format!("{}", Token::Float(3.15)), "Float(3.15)");
        assert_eq!(
            format!("{}", Token::String("hi".to_string())),
            "String(\"hi\")"
        );
        assert_eq!(format!("{}", Token::Plus), "+");
        assert_eq!(format!("{}", Token::Arrow), "->");
        assert_eq!(format!("{}", Token::Comma), ",");
        assert_eq!(format!("{}", Token::Eof), "EOF");
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(10, 25, 100);
        assert_eq!(format!("{}", pos), "10:25");
    }

    #[test]
    fn test_error_display() {
        let err1 = LexError::UnexpectedChar('@', Position::new(1, 1, 0));
        assert!(format!("{}", err1).contains("Unexpected character"));

        let err2 = LexError::UnterminatedString(Position::new(1, 5, 4));
        assert!(format!("{}", err2).contains("Unterminated string"));

        let err3 = LexError::InvalidNumber("123abc".to_string(), Position::new(1, 1, 0));
        assert!(format!("{}", err3).contains("Invalid number"));
    }
    // ========================================================================
    // Let-Rec Keyword Tests (Issue #22)
    // ========================================================================

    #[test]
    fn test_lex_keyword_rec() {
        let mut lexer = Lexer::new("rec");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Rec);
    }

    #[test]
    fn test_lex_keyword_and() {
        let mut lexer = Lexer::new("and");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::AndKeyword);
    }

    #[test]
    fn test_lex_let_rec_simple() {
        let mut lexer = Lexer::new("let rec fact = fun n -> n");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Rec);
        assert_eq!(tokens[2].token, Token::Ident("fact".to_string()));
        assert_eq!(tokens[3].token, Token::Eq);
        assert_eq!(tokens[4].token, Token::Fun);
    }

    #[test]
    fn test_lex_let_rec_mutual() {
        let mut lexer = Lexer::new("let rec f = fun x -> x and g = fun y -> y in f");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[1].token, Token::Rec);
        assert_eq!(tokens[2].token, Token::Ident("f".to_string()));
        assert_eq!(tokens[3].token, Token::Eq);
        assert_eq!(tokens[8].token, Token::AndKeyword);
        assert_eq!(tokens[9].token, Token::Ident("g".to_string()));
    }

    // ========================================================================
    // Tuple Lexer Tests (Issue #24 Layer 2)
    // ========================================================================

    #[test]
    fn test_lex_tuple_simple() {
        let mut lexer = Lexer::new("(1, 2)");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Int(1));
        assert_eq!(tokens[2].token, Token::Comma);
        assert_eq!(tokens[3].token, Token::Int(2));
        assert_eq!(tokens[4].token, Token::RParen);
    }

    #[test]
    fn test_lex_tuple_triple() {
        let mut lexer = Lexer::new("(1, 2, 3)");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Int(1));
        assert_eq!(tokens[2].token, Token::Comma);
        assert_eq!(tokens[3].token, Token::Int(2));
        assert_eq!(tokens[4].token, Token::Comma);
        assert_eq!(tokens[5].token, Token::Int(3));
        assert_eq!(tokens[6].token, Token::RParen);
    }

    #[test]
    fn test_lex_tuple_with_trailing_comma() {
        let mut lexer = Lexer::new("(42,)");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Int(42));
        assert_eq!(tokens[2].token, Token::Comma);
        assert_eq!(tokens[3].token, Token::RParen);
    }

    #[test]
    fn test_lex_tuple_nested() {
        let mut lexer = Lexer::new("(1, (2, 3))");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Int(1));
        assert_eq!(tokens[2].token, Token::Comma);
        assert_eq!(tokens[3].token, Token::LParen);
        assert_eq!(tokens[4].token, Token::Int(2));
        assert_eq!(tokens[5].token, Token::Comma);
        assert_eq!(tokens[6].token, Token::Int(3));
        assert_eq!(tokens[7].token, Token::RParen);
        assert_eq!(tokens[8].token, Token::RParen);
    }
}
