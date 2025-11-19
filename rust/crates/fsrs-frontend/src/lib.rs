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
//! - `compiler`: Bytecode compiler (AST → Bytecode)
//! - `types`: Type system infrastructure for Hindley-Milner type inference
//! - `inference`: Type inference engine (Hindley-Milner algorithm)
//! - `span`: Source location tracking for error reporting
//! - `error`: Error types with beautiful formatting and suggestions
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::lexer::Lexer;
//! use fsrs_frontend::parser::Parser;
//! use fsrs_frontend::compiler::Compiler;
//! use fsrs_frontend::inference::TypeInference;
//! use fsrs_frontend::types::TypeEnv;
//!
//! // Full pipeline: source -> tokens -> AST -> type check -> bytecode
//! let source = "let x = 42 in x + 1";
//! let mut lexer = Lexer::new(source);
//! let tokens = lexer.tokenize().unwrap();
//! let mut parser = Parser::new(tokens);
//! let ast = parser.parse().unwrap();
//!
//! // Type check
//! let mut infer = TypeInference::new();
//! let env = TypeEnv::new();
//! let ty = infer.infer_and_solve(&ast, &env).unwrap();
//!
//! // Compile
//! let chunk = Compiler::compile(&ast).unwrap();
//!
//! // Chunk is ready for VM execution
//! assert!(chunk.instructions.len() > 0);
//! ```

pub mod ast;
pub mod compiler;
pub mod error;
pub mod inference;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod types;

// Re-export commonly used types for convenience
pub use ast::{BinOp, Expr, Literal};
pub use compiler::{CompileError, Compiler};
pub use error::{TypeError, TypeErrorKind};
pub use inference::TypeInference;
pub use lexer::{LexError, Lexer, Position, Token, TokenWithPos};
pub use parser::{ParseError, Parser};
pub use span::Span;
pub use types::{Substitution, Type, TypeEnv, TypeScheme, TypeVar};
