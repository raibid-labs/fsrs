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
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::lexer::{Lexer, Token};
//!
//! // Tokenize source code
//! let mut lexer = Lexer::new("let x = 42");
//! let tokens = lexer.tokenize().unwrap();
//!
//! // Construct AST: 1 + 2
//! let expr = Expr::BinOp {
//!     op: BinOp::Add,
//!     left: Box::new(Expr::Lit(Literal::Int(1))),
//!     right: Box::new(Expr::Lit(Literal::Int(2))),
//! };
//! ```

pub mod ast;
pub mod lexer;

// Re-export commonly used types for convenience
pub use ast::{BinOp, Expr, Literal};
pub use lexer::{LexError, Lexer, Position, Token, TokenWithPos};
