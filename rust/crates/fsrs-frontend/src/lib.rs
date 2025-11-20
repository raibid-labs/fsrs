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
//! - `typed_ast`: Optional typed AST with type annotations
//! - `span`: Source location tracking for error reporting
//! - `error`: Error types with beautiful formatting and suggestions
//! - `modules`: Module system for code organization
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::lexer::Lexer;
//! use fsrs_frontend::parser::Parser;
//! use fsrs_frontend::compiler::{Compiler, CompileOptions};
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
//! // Compile without type checking (backward compatible)
//! let chunk = Compiler::compile(&ast).unwrap();
//!
//! // Or compile with type checking enabled
//! let options = CompileOptions {
//!     enable_type_checking: true,
//!     ..Default::default()
//! };
//! let chunk_checked = Compiler::compile_with_options(&ast, options).unwrap();
//!
//! // Chunk is ready for VM execution
//! assert!(chunk.instructions.len() > 0);
//! ```

pub mod ast;
pub mod compiler;
pub mod error;
pub mod inference;
pub mod lexer;
pub mod modules;
pub mod parser;
pub mod span;
pub mod typed_ast;
pub mod types;

// Re-export commonly used types for convenience
pub use ast::{BinOp, Expr, Literal, ModuleDef, ModuleItem, Pattern, Program};
pub use compiler::{CompileError, CompileOptions, Compiler};
pub use error::{TypeError, TypeErrorKind};
pub use inference::TypeInference;
pub use lexer::{LexError, Lexer, Position, Token, TokenWithPos};
pub use modules::{Module, ModulePath, ModuleRegistry, TypeDefinition as ModuleTypeDef};
pub use parser::{ParseError, Parser};
pub use span::Span;
pub use typed_ast::{TypedExpr, TypedPattern};
pub use types::{Substitution, Type, TypeEnv, TypeScheme, TypeVar};

use fsrs_vm::chunk::Chunk;

/// Convenience function to compile a program from source code
///
/// This is a high-level API that performs the full compilation pipeline:
/// 1. Lexing (source -> tokens)
/// 2. Parsing (tokens -> Program AST)
/// 3. Compilation (Program AST -> Bytecode)
///
/// # Example
///
/// ```rust
/// use fsrs_frontend::compile_program_from_source;
///
/// // Note: Module parsing is available but not all features are implemented yet
/// let source = "42";
///
/// let chunk = compile_program_from_source(source).unwrap();
/// assert!(chunk.instructions.len() > 0);
/// ```
pub fn compile_program_from_source(source: &str) -> Result<Chunk, CompilationError> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(CompilationError::LexError)?;
    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(CompilationError::ParseError)?;
    let chunk = Compiler::compile_program(&program).map_err(CompilationError::CompileError)?;
    Ok(chunk)
}

/// Unified error type for the compilation pipeline
#[derive(Debug)]
pub enum CompilationError {
    LexError(LexError),
    ParseError(ParseError),
    CompileError(CompileError),
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::LexError(e) => write!(f, "Lexer error: {}", e),
            CompilationError::ParseError(e) => write!(f, "Parse error: {}", e),
            CompilationError::CompileError(e) => write!(f, "Compile error: {}", e),
        }
    }
}

impl std::error::Error for CompilationError {}
