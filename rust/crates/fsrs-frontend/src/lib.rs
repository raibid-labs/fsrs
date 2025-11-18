//! FSRS Frontend - Parser, Typechecker, and Bytecode Compiler
//!
//! This crate implements the frontend of the FSRS (F# Script Runtime System),
//! responsible for parsing F# source code into an AST, performing type checking,
//! and compiling to bytecode for the FSRS VM.
//!
//! # Modules
//!
//! - `ast`: Core AST (Abstract Syntax Tree) definitions
//! - `lexer`: Lexer/Tokenizer for Mini-F# source code
//! - `parser`: Recursive-descent parser for Mini-F# expressions
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::lexer::Lexer;
//! use fsrs_frontend::parser::Parser;
//!
//! // Full pipeline: source -> tokens -> AST
//! let source = "let x = 42 in x + 1";
//! let mut lexer = Lexer::new(source);
//! let tokens = lexer.tokenize().unwrap();
//! let mut parser = Parser::new(tokens);
//! let ast = parser.parse().unwrap();
//!
//! // AST represents: let x = 42 in (x + 1)
//! assert!(ast.is_let());
//! ```

pub mod ast;
pub mod lexer;
pub mod parser;

// Re-export commonly used types for convenience
pub use ast::{BinOp, Expr, Literal};
pub use lexer::{LexError, Lexer, Position, Token, TokenWithPos};
pub use parser::{ParseError, Parser};
