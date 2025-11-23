//! Lexer/Tokenizer for Mini-F# source code.
//!
//! This module implements the lexical analyzer that converts source text into
//! a stream of tokens. The lexer supports:
//!
//! - Literals: integers, floats, booleans, strings
//! - Keywords: let, rec, and, in, if, then, else, fun, true, false
//! - Identifiers: alphanumeric names starting with letter or underscore
//! - Operators: arithmetic, comparison, logical, cons (::)
//! - Punctuation: parentheses, arrows, commas, brackets, semicolons
//! - Array syntax: [|, |], <-, .
//! - Position tracking for error reporting
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::lexer::{Lexer, Token};
//!
//! let mut lexer = Lexer::new("let x = 42");
//! let tokens = lexer.tokenize().unwrap();
//! assert_eq!(tokens[0].token, Token::Let);
//! assert_eq!(tokens[1].token, Token::Ident("x".to_string()));
//! ```

pub use crate::span::{Position, Span};
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
    /// match keyword
    Match,
    /// type keyword
    Type,
    /// with keyword (for record updates)
    With,
    /// of keyword (for discriminated unions)
    Of,
    /// open keyword (for importing modules)
    Open,
    /// module keyword (for module definitions)
    Module,

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
    /// :: operator (cons)
    ColonColon,
    /// <- operator (array assignment)
    LArrow,
    /// |> operator (pipeline)
    PipeRight,

    // Punctuation
    /// ( left parenthesis
    LParen,
    /// ) right parenthesis
    RParen,
    /// [ left bracket
    LBracket,
    /// ] right bracket
    RBracket,
    /// [| left array bracket
    LBracketPipe,
    /// |] right array bracket
    PipeRBracket,
    /// -> arrow
    Arrow,
    /// , comma
    Comma,
    /// ; semicolon
    Semicolon,
    /// . dot
    Dot,
    /// { left brace
    LBrace,
    /// } right brace
    RBrace,
    /// : colon
    Colon,
    /// | pipe
    Pipe,
    /// _ underscore
    Underscore,

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
            Token::Match => write!(f, "match"),
            Token::Type => write!(f, "type"),
            Token::With => write!(f, "with"),
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
            Token::ColonColon => write!(f, "::"),
            Token::LArrow => write!(f, "<-"),
            Token::PipeRight => write!(f, "|>"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBracketPipe => write!(f, "[|"),
            Token::PipeRBracket => write!(f, "|]"),
            Token::Arrow => write!(f, "->"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Dot => write!(f, "."),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Pipe => write!(f, "|"),
            Token::Underscore => write!(f, "_"),
            Token::Of => write!(f, "of"),
            Token::Open => write!(f, "open"),
            Token::Module => write!(f, "module"),
            Token::Eof => write!(f, "EOF"),
        }
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

/// A token with its span in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithSpan {
    /// The token
    pub token: Token,
    /// Span in source
    pub span: Span,
}

impl TokenWithSpan {
    /// Create a new token with span.
    pub fn new(token: Token, span: Span) -> Self {
        TokenWithSpan { token, span }
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

    /// Tokenize with span information.
    pub fn tokenize_with_spans(&mut self) -> Result<Vec<TokenWithSpan>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }

            let start_pos = self.current_position();
            let token = self.next_token()?;
            let end_pos = self.current_position();
            let span = Span::new(start_pos, end_pos);
            tokens.push(TokenWithSpan::new(token, span));
        }

        let eof_pos = self.current_position();
        tokens.push(TokenWithSpan::new(Token::Eof, Span::point(eof_pos)));

        Ok(tokens)
    }

    /// Lex the next token from the input.
    fn next_token(&mut self) -> Result<Token, LexError> {
        let ch = self.current_char();

        match ch {
            '0'..='9' => self.lex_number(),
            'a'..='z' | 'A'..='Z' => self.lex_identifier_or_keyword(),
            '_' => {
                // Peek ahead to see if this is an identifier like _foo or just _
                if !self.is_at_end_or(1)
                    && (self.peek_char().is_alphanumeric() || self.peek_char() == '_')
                {
                    self.lex_identifier_or_keyword()
                } else {
                    self.advance();
                    Ok(Token::Underscore)
                }
            }
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
            '<' => self.lex_lt_or_lte_or_neq_or_larrow(),
            '>' => self.lex_gt_or_gte(),
            '&' => self.lex_and(),
            '|' => {
                self.advance();
                if !self.is_at_end() && self.current_char() == '|' {
                    self.advance();
                    Ok(Token::Or)
                } else if !self.is_at_end() && self.current_char() == '>' {
                    self.advance();
                    Ok(Token::PipeRight)
                } else if !self.is_at_end() && self.current_char() == ']' {
                    self.advance();
                    Ok(Token::PipeRBracket)
                } else {
                    Ok(Token::Pipe)
                }
            }
            ':' => self.lex_colon_or_coloncolon(),
            '(' => {
                self.advance();
                Ok(Token::LParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RParen)
            }
            '[' => self.lex_lbracket_or_lbracket_pipe(),
            ']' => {
                self.advance();
                Ok(Token::RBracket)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            '{' => {
                self.advance();
                Ok(Token::LBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RBrace)
            }
            '.' => {
                self.advance();
                Ok(Token::Dot)
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
            "match" => Token::Match,
            "and" => Token::AndKeyword,
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "fun" => Token::Fun,
            "type" => Token::Type,
            "with" => Token::With,
            "of" => Token::Of,
            "open" => Token::Open,
            "module" => Token::Module,
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

    /// Lex <, <=, <>, or <-.
    fn lex_lt_or_lte_or_neq_or_larrow(&mut self) -> Result<Token, LexError> {
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
                '-' => {
                    self.advance();
                    Ok(Token::LArrow)
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


    /// Lex : or ::.
    fn lex_colon_or_coloncolon(&mut self) -> Result<Token, LexError> {
        let _pos = self.current_position();
        self.advance();
        if !self.is_at_end() && self.current_char() == ':' {
            self.advance();
            Ok(Token::ColonColon)
        } else {
            Ok(Token::Colon)
        }
    }

    /// Lex [ or [|.
    fn lex_lbracket_or_lbracket_pipe(&mut self) -> Result<Token, LexError> {
        self.advance();
        if !self.is_at_end() && self.current_char() == '|' {
            self.advance();
            Ok(Token::LBracketPipe)
        } else {
            Ok(Token::LBracket)
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

// Tests are preserved from original file - they test TokenWithPos
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_integer() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token, Token::Int(42));
        assert_eq!(tokens[1].token, Token::Eof);
    }

    #[test]
    fn test_tokenize_with_spans() {
        let mut lexer = Lexer::new("let x = 42");
        let tokens = lexer.tokenize_with_spans().unwrap();
        assert_eq!(tokens.len(), 5); // let, x, =, 42, EOF
        assert_eq!(tokens[0].token, Token::Let);
        assert!(tokens[0].span.is_single_line());
    }
}
