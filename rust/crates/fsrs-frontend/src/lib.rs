//! FSRS Frontend - Parser, Typechecker, and Bytecode Compiler
//!
//! This crate implements the frontend of the FSRS (F# Script Runtime System),
//! responsible for parsing F# source code into an AST, performing type checking,
//! and compiling to bytecode for the FSRS VM.
//!
//! # Modules
//!
//! - `ast`: Core AST (Abstract Syntax Tree) definitions
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//!
//! // Construct: 1 + 2
//! let expr = Expr::BinOp {
//!     op: BinOp::Add,
//!     left: Box::new(Expr::Lit(Literal::Int(1))),
//!     right: Box::new(Expr::Lit(Literal::Int(2))),
//! };
//! ```

pub mod ast;

// Re-export commonly used types for convenience
pub use ast::{BinOp, Expr, Literal};
