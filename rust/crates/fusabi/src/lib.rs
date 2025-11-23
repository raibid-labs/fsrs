//! Fusabi Demo Host Library
//!
//! This library provides the core functionality for executing Mini-F# scripts
//! through the complete Fusabi pipeline: Lexer -> Parser -> Compiler -> VM.
//!
//! # Type Checking Support
//!
//! The library now supports optional type checking through compilation options.
//! You can run programs with or without type checking enabled.

use fusabi_frontend::compiler::CompileOptions;
use fusabi_frontend::{Compiler, Lexer, Parser};
use fusabi_vm::{Value, Vm, FZB_MAGIC, deserialize_chunk};
use std::error::Error;
use std::fmt;
use std::fs;
use std::string::FromUtf8Error;

pub mod host_api;
pub use host_api::FusabiEngine;

/// Unified error type for the Fusabi pipeline
#[derive(Debug)]
pub enum FusabiError {
    /// IO error reading source file
    Io(std::io::Error),
    /// Lexer error during tokenization
    Lex(fusabi_frontend::LexError),
    /// Parser error during parsing
    Parse(fusabi_frontend::ParseError),
    /// Compiler error during bytecode generation
    Compile(fusabi_frontend::CompileError),
    /// VM runtime error during execution
    Runtime(fusabi_vm::VmError),
    /// Error during bytecode serialization/deserialization
    Serde(Box<dyn std::error::Error + Send + Sync + 'static>),
    /// UTF-8 error when reading source file
    Utf8(FromUtf8Error),
}

impl fmt::Display for FusabiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FusabiError::Io(e) => write!(f, "IO Error: {}", e),
            FusabiError::Lex(e) => write!(f, "Lexer Error: {}", e),
            FusabiError::Parse(e) => write!(f, "Parser Error: {}", e),
            FusabiError::Compile(e) => write!(f, "Compiler Error: {}", e),
            FusabiError::Runtime(e) => write!(f, "Runtime Error: {}", e),
            FusabiError::Serde(e) => write!(f, "Serialization Error: {}", e),
            FusabiError::Utf8(e) => write!(f, "UTF-8 Error: {}", e),
        }
    }
}

impl Error for FusabiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FusabiError::Io(e) => Some(e),
            FusabiError::Lex(e) => Some(e),
            FusabiError::Parse(e) => Some(e),
            FusabiError::Compile(e) => Some(e),
            FusabiError::Runtime(e) => Some(e),
            FusabiError::Serde(e) => Some(e.as_ref()),
            FusabiError::Utf8(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for FusabiError {
    fn from(err: std::io::Error) -> Self {
        FusabiError::Io(err)
    }
}

impl From<fusabi_frontend::LexError> for FusabiError {
    fn from(err: fusabi_frontend::LexError) -> Self {
        FusabiError::Lex(err)
    }
}

impl From<fusabi_frontend::ParseError> for FusabiError {
    fn from(err: fusabi_frontend::ParseError) -> Self {
        FusabiError::Parse(err)
    }
}

impl From<fusabi_frontend::CompileError> for FusabiError {
    fn from(err: fusabi_frontend::CompileError) -> Self {
        FusabiError::Compile(err)
    }
}

impl From<fusabi_vm::VmError> for FusabiError {
    fn from(err: fusabi_vm::VmError) -> Self {
        FusabiError::Runtime(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for FusabiError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        FusabiError::Serde(err)
    }
}

impl From<FromUtf8Error> for FusabiError {
    fn from(err: FromUtf8Error) -> Self {
        FusabiError::Utf8(err)
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
pub fn run_source(source: &str) -> Result<Value, FusabiError> {
    run_source_with_options(source, RunOptions::default())
}

/// Execute Mini-F# source code with type checking enabled
pub fn run_source_checked(source: &str) -> Result<Value, FusabiError> {
    let options = RunOptions {
        enable_type_checking: true,
        ..Default::default()
    };
    run_source_with_options(source, options)
}

/// Execute Mini-F# source code with custom run options
pub fn run_source_with_options(source: &str, options: RunOptions) -> Result<Value, FusabiError> {
    if options.verbose {
        println!("=== Fusabi Execution Pipeline ===");
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
    // Register standard library functions and globals
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    
    let result = vm.execute(chunk)?;
    if options.verbose {
        println!("  Result: {:?}", result);
        println!();
    }

    Ok(result)
}

/// Execute a Mini-F# script from a file (backward compatible)
pub fn run_file(path: &str) -> Result<Value, FusabiError> {
    let bytes = fs::read(path)?;

    if bytes.starts_with(FZB_MAGIC) {
        // It's a pre-compiled bytecode file (.fzb)
        let chunk = deserialize_chunk(&bytes)?;
        
        let mut vm = Vm::new();
        fusabi_vm::stdlib::register_stdlib(&mut vm);
        let result = vm.execute(chunk)?;
        Ok(result)
    } else {
        // It's a source file (.fsx), compile it
        let source = String::from_utf8(bytes)?;
        run_source(&source)
    }
}

/// Execute a Mini-F# script from a file with type checking enabled
pub fn run_file_checked(path: &str) -> Result<Value, FusabiError> {
    let source = fs::read_to_string(path)?;
    run_source_checked(&source)
}

/// Execute a Mini-F# script from a file with custom options
pub fn run_file_with_options(path: &str, options: RunOptions) -> Result<Value, FusabiError> {
    let source = fs::read_to_string(path)?;
    run_source_with_options(&source, options)
}

/// Execute source with optional disassembly output (backward compatible)
pub fn run_source_with_disasm(source: &str, name: &str) -> Result<Value, FusabiError> {
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
    // Register standard library functions and globals
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    
    let result = vm.execute(chunk)?;

    Ok(result)
}

/// Execute file with optional disassembly output
pub fn run_file_with_disasm(path: &str) -> Result<Value, FusabiError> {
    let bytes = fs::read(path)?;

    let chunk = if bytes.starts_with(FZB_MAGIC) {
        // It's a pre-compiled bytecode file (.fzb)
        println!("Loading pre-compiled bytecode: {}", path);
        deserialize_chunk(&bytes)?
    } else {
        // It's a source file (.fsx), compile it
        let source = String::from_utf8(bytes)?;
        // Stage 1: Lexical Analysis
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()?;

        // Stage 2: Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        // Stage 3: Compilation
        Compiler::compile(&ast)?
    };

    // Disassemble the chunk
    println!("\n=== Disassembly of '{}' ===", path);
    chunk.disassemble();
    println!("=== End Disassembly ===\n");

    // Stage 4: Execution
    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk)?;

    Ok(result)
}
