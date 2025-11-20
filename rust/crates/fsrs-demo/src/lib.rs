//! FSRS Demo Host Library
//!
//! This library provides the core functionality for executing Mini-F# scripts
//! through the complete FSRS pipeline: Lexer -> Parser -> Compiler -> VM.
//!
//! # Type Checking Support
//!
//! The library now supports optional type checking through compilation options.
//! You can run programs with or without type checking enabled.

use fsrs_frontend::compiler::CompileOptions;
use fsrs_frontend::{Compiler, Lexer, Parser};
use fsrs_vm::{Value, Vm};
use std::error::Error;
use std::fmt;
use std::fs;

pub mod host_api;
pub use host_api::FsrsEngine;

/// Unified error type for the FSRS pipeline
#[derive(Debug)]
pub enum FsrsError {
    /// IO error reading source file
    Io(std::io::Error),
    /// Lexer error during tokenization
    Lex(fsrs_frontend::LexError),
    /// Parser error during parsing
    Parse(fsrs_frontend::ParseError),
    /// Compiler error during bytecode generation
    Compile(fsrs_frontend::CompileError),
    /// VM runtime error during execution
    Runtime(fsrs_vm::VmError),
}

impl fmt::Display for FsrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsrsError::Io(e) => write!(f, "IO Error: {}", e),
            FsrsError::Lex(e) => write!(f, "Lexer Error: {}", e),
            FsrsError::Parse(e) => write!(f, "Parser Error: {}", e),
            FsrsError::Compile(e) => write!(f, "Compiler Error: {}", e),
            FsrsError::Runtime(e) => write!(f, "Runtime Error: {}", e),
        }
    }
}

impl Error for FsrsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FsrsError::Io(e) => Some(e),
            FsrsError::Lex(e) => Some(e),
            FsrsError::Parse(e) => Some(e),
            FsrsError::Compile(e) => Some(e),
            FsrsError::Runtime(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for FsrsError {
    fn from(err: std::io::Error) -> Self {
        FsrsError::Io(err)
    }
}

impl From<fsrs_frontend::LexError> for FsrsError {
    fn from(err: fsrs_frontend::LexError) -> Self {
        FsrsError::Lex(err)
    }
}

impl From<fsrs_frontend::ParseError> for FsrsError {
    fn from(err: fsrs_frontend::ParseError) -> Self {
        FsrsError::Parse(err)
    }
}

impl From<fsrs_frontend::CompileError> for FsrsError {
    fn from(err: fsrs_frontend::CompileError) -> Self {
        FsrsError::Compile(err)
    }
}

impl From<fsrs_vm::VmError> for FsrsError {
    fn from(err: fsrs_vm::VmError) -> Self {
        FsrsError::Runtime(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Enable type checking before compilation
    pub enable_type_checking: bool,
    /// Verbose output for debugging
    pub verbose: bool,
    /// Strict mode - treat warnings as errors
    pub strict_mode: bool,
}

/// Execute Mini-F# source code from a string (backward compatible)
pub fn run_source(source: &str) -> Result<Value, FsrsError> {
    run_source_with_options(source, RunOptions::default())
}

/// Execute Mini-F# source code with type checking enabled
pub fn run_source_checked(source: &str) -> Result<Value, FsrsError> {
    let options = RunOptions {
        enable_type_checking: true,
        ..Default::default()
    };
    run_source_with_options(source, options)
}

/// Execute Mini-F# source code with custom run options
pub fn run_source_with_options(source: &str, options: RunOptions) -> Result<Value, FsrsError> {
    if options.verbose {
        println!("=== FSRS Execution Pipeline ===");
        println!("Type checking: {}", options.enable_type_checking);
        println!("Strict mode: {}", options.strict_mode);
        println!();
    }

    // Stage 1: Lexical Analysis
    if options.verbose {
        println!("Stage 1: Lexical Analysis");
    }
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    if options.verbose {
        println!("  Generated {} tokens", tokens.len());
    }

    // Stage 2: Parsing
    if options.verbose {
        println!("Stage 2: Parsing");
    }
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    if options.verbose {
        println!("  Parsed AST successfully");
    }

    // Stage 3: Compilation (with optional type checking)
    if options.verbose {
        println!("Stage 3: Compilation");
    }
    let compile_options = CompileOptions {
        enable_type_checking: options.enable_type_checking,
        strict_mode: options.strict_mode,
        allow_warnings: !options.strict_mode,
    };
    let chunk = Compiler::compile_with_options(&ast, compile_options)?;
    if options.verbose {
        println!("  Generated {} instructions", chunk.instructions.len());
        println!("  Constant pool size: {}", chunk.constants.len());
    }

    // Stage 4: Execution
    if options.verbose {
        println!("Stage 4: Execution");
    }
    let mut vm = Vm::new();
    let result = vm.execute(chunk)?;
    if options.verbose {
        println!("  Result: {:?}", result);
        println!();
    }

    Ok(result)
}

/// Execute a Mini-F# script from a file (backward compatible)
pub fn run_file(path: &str) -> Result<Value, FsrsError> {
    let source = fs::read_to_string(path)?;
    run_source(&source)
}

/// Execute a Mini-F# script from a file with type checking enabled
pub fn run_file_checked(path: &str) -> Result<Value, FsrsError> {
    let source = fs::read_to_string(path)?;
    run_source_checked(&source)
}

/// Execute a Mini-F# script from a file with custom options
pub fn run_file_with_options(path: &str, options: RunOptions) -> Result<Value, FsrsError> {
    let source = fs::read_to_string(path)?;
    run_source_with_options(&source, options)
}

/// Execute source with optional disassembly output (backward compatible)
pub fn run_source_with_disasm(source: &str, name: &str) -> Result<Value, FsrsError> {
    // Stage 1: Lexical Analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Stage 2: Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // Stage 3: Compilation
    let chunk = Compiler::compile(&ast)?;

    // Disassemble the chunk
    println!("\n=== Disassembly of '{}' ===", name);
    chunk.disassemble();
    println!("=== End Disassembly ===\n");

    // Stage 4: Execution
    let mut vm = Vm::new();
    let result = vm.execute(chunk)?;

    Ok(result)
}

/// Execute file with optional disassembly output
pub fn run_file_with_disasm(path: &str) -> Result<Value, FsrsError> {
    let source = fs::read_to_string(path)?;
    run_source_with_disasm(&source, path)
}
