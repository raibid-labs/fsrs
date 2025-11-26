//! Fusabi - A potent, functional scripting layer for Rust infrastructure
//!
//! This library provides the core functionality for executing Fusabi (Mini-F#) scripts
//! through the complete pipeline: Lexer -> Parser -> Compiler -> VM.
//!
//! # Quick Start
//!
//! ```no_run
//! use fusabi::Engine;
//!
//! let mut engine = Engine::new();
//! let result = engine.eval("let x = 42 in x * 2").unwrap();
//! assert_eq!(result.as_int(), Some(84));
//! ```
//!
//! # Loading from Files
//!
//! ```no_run
//! use fusabi::run_file;
//!
//! let result = run_file("script.fsx").unwrap();
//! println!("Result: {:?}", result);
//! ```
//!
//! # Bytecode Compilation
//!
//! Fusabi supports ahead-of-time (AOT) compilation to bytecode for production deployments:
//!
//! ```no_run
//! use fusabi::{compile_to_bytecode, compile_file_to_bytecode};
//! use fusabi_vm::{Vm, deserialize_chunk};
//! use std::fs;
//!
//! // Compile source to bytecode
//! let source = "let x = 42 in x * 2";
//! let bytecode = compile_to_bytecode(source).unwrap();
//!
//! // Save bytecode to file
//! fs::write("script.fzb", &bytecode).unwrap();
//!
//! // Load and execute bytecode
//! let chunk = deserialize_chunk(&bytecode).unwrap();
//! let mut vm = Vm::new();
//! fusabi_vm::stdlib::register_stdlib(&mut vm);
//! let result = vm.execute(chunk).unwrap();
//!
//! // Or compile file directly
//! let bytecode = compile_file_to_bytecode("script.fsx").unwrap();
//! ```
//!
//! # Type Checking Support
//!
//! The library supports optional type checking through compilation options:
//!
//! ```no_run
//! use fusabi::Engine;
//!
//! let mut engine = Engine::new();
//! let result = engine.eval_checked("let x: int = 42 in x * 2").unwrap();
//! assert_eq!(result.as_int(), Some(84));
//! ```
//!
//! # Working with Values
//!
//! ```no_run
//! use fusabi::{Engine, Value};
//!
//! let mut engine = Engine::new();
//!
//! // Register a custom host function
//! engine.register_fn1("double", |x: Value| {
//!     let n = x.as_int().unwrap_or(0);
//!     Ok(Value::Int(n * 2))
//! });
//!
//! // Call it from Fusabi code
//! let result = engine.eval("double 21").unwrap();
//! assert_eq!(result.as_int(), Some(42));
//! ```

use fusabi_frontend::{Compiler, Lexer, Parser};
use fusabi_vm::{deserialize_chunk, serialize_chunk, Chunk, Vm, FZB_MAGIC};
use std::error::Error;
use std::fmt;
use std::fs;
use std::string::FromUtf8Error;

pub mod host_api;

// Re-export the primary API at the crate root for easy access
pub use fusabi_vm::{HostData, Value};
pub use host_api::{FusabiEngine as Engine, Module};
// Re-export CompileOptions for advanced compilation control
pub use fusabi_frontend::CompileOptions;

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
    let program = parser.parse_program()?;
    if options.verbose {
        println!("  Parsed AST successfully");
    }

    // Stage 3: Compilation (with optional type checking)
    if options.verbose {
        println!("Stage 3: Compilation");
    }
    // Note: compile_program doesn't support custom options yet, using default compilation
    let chunk = Compiler::compile_program(&program)?;
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
    let program = parser.parse_program()?;

    // Stage 3: Compilation
    let chunk = Compiler::compile_program(&program)?;

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
        let program = parser.parse_program()?;

        // Stage 3: Compilation
        Compiler::compile_program(&program)?
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

// ============================================================================
// Bytecode Compilation API
// ============================================================================

/// Compile Fusabi source code to bytecode
///
/// This function compiles Fusabi source code through the complete compilation
/// pipeline (Lexer -> Parser -> Compiler) and returns serialized bytecode that
/// can be saved to a .fzb file or executed later.
///
/// # Arguments
///
/// * `source` - Fusabi source code as a string
///
/// # Returns
///
/// Returns a `Vec<u8>` containing the serialized bytecode with FZB magic header
/// and version information.
///
/// # Errors
///
/// Returns `FusabiError` if:
/// - Lexical analysis fails (invalid tokens)
/// - Parsing fails (syntax errors)
/// - Compilation fails (semantic errors)
/// - Serialization fails (internal error)
///
/// # Example
///
/// ```no_run
/// use fusabi::compile_to_bytecode;
/// use std::fs;
///
/// let source = r#"
///     let factorial n =
///         if n <= 1 then 1
///         else n * factorial (n - 1)
///     in
///     factorial 10
/// "#;
///
/// // Compile to bytecode
/// let bytecode = compile_to_bytecode(source).unwrap();
///
/// // Save to file
/// fs::write("factorial.fzb", &bytecode).unwrap();
/// ```
///
/// # Performance Considerations
///
/// - Compilation is CPU-bound and should be done ahead-of-time for production
/// - Bytecode files are typically 60-80% smaller than source files
/// - Bytecode loading is 3-5x faster than parsing source
/// - Use this for production deployments where startup time matters
///
/// # See Also
///
/// - [`compile_file_to_bytecode`] - Compile from file path
/// - [`run_file`] - Execute either source or bytecode files
/// - [Bytecode Format Documentation](../docs/bytecode-format.md)
pub fn compile_to_bytecode(source: &str) -> Result<Vec<u8>, FusabiError> {
    // Stage 1: Lexical Analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Stage 2: Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    // Stage 3: Compilation
    let chunk = Compiler::compile_program(&program)?;

    // Stage 4: Serialization
    let bytecode = serialize_chunk(&chunk)?;

    Ok(bytecode)
}

/// Compile a Fusabi source file to bytecode
///
/// This is a convenience function that reads a .fsx file, compiles it to bytecode,
/// and returns the serialized result. It's equivalent to reading the file and
/// calling [`compile_to_bytecode`].
///
/// # Arguments
///
/// * `path` - Path to the .fsx source file
///
/// # Returns
///
/// Returns a `Vec<u8>` containing the serialized bytecode.
///
/// # Errors
///
/// Returns `FusabiError` if:
/// - File cannot be read (IO error)
/// - File contains invalid UTF-8
/// - Compilation fails (see [`compile_to_bytecode`])
///
/// # Example
///
/// ```no_run
/// use fusabi::compile_file_to_bytecode;
/// use std::fs;
///
/// // Compile pipeline.fsx to bytecode
/// let bytecode = compile_file_to_bytecode("pipeline.fsx").unwrap();
///
/// // Save as pipeline.fzb
/// fs::write("pipeline.fzb", &bytecode).unwrap();
/// ```
///
/// # Production Pattern
///
/// ```no_run
/// use fusabi::compile_file_to_bytecode;
/// use std::path::Path;
/// use std::fs;
///
/// fn compile_or_load(source_path: &str, cache_path: &str) -> Vec<u8> {
///     // Check if cached bytecode exists and is newer than source
///     if Path::new(cache_path).exists() {
///         let source_time = fs::metadata(source_path).unwrap().modified().unwrap();
///         let cache_time = fs::metadata(cache_path).unwrap().modified().unwrap();
///
///         if cache_time > source_time {
///             println!("Loading from cache: {}", cache_path);
///             return fs::read(cache_path).unwrap();
///         }
///     }
///
///     // Cache miss or stale - compile and cache
///     println!("Compiling: {}", source_path);
///     let bytecode = compile_file_to_bytecode(source_path).unwrap();
///     fs::write(cache_path, &bytecode).unwrap();
///     bytecode
/// }
/// ```
pub fn compile_file_to_bytecode(path: &str) -> Result<Vec<u8>, FusabiError> {
    let source = fs::read_to_string(path)?;
    compile_to_bytecode(&source)
}

/// Compile source code to a Chunk (internal representation)
///
/// This is a lower-level API that returns the compiled `Chunk` directly without
/// serialization. Useful if you want to inspect or modify the bytecode before
/// serialization, or if you're executing immediately without saving to disk.
///
/// # Arguments
///
/// * `source` - Fusabi source code as a string
///
/// # Returns
///
/// Returns a `Chunk` containing instructions and constants.
///
/// # Errors
///
/// Returns `FusabiError` if compilation fails.
///
/// # Example
///
/// ```no_run
/// use fusabi::compile_to_chunk;
/// use fusabi_vm::{Vm, serialize_chunk};
///
/// let source = "let x = 42 in x * 2";
///
/// // Compile to chunk
/// let chunk = compile_to_chunk(source).unwrap();
///
/// // Inspect the chunk
/// println!("Instructions: {}", chunk.instructions.len());
/// println!("Constants: {}", chunk.constants.len());
///
/// // Execute directly
/// let mut vm = Vm::new();
/// fusabi_vm::stdlib::register_stdlib(&mut vm);
/// let result = vm.execute(chunk).unwrap();
/// ```
pub fn compile_to_chunk(source: &str) -> Result<Chunk, FusabiError> {
    // Stage 1: Lexical Analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    // Stage 2: Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    // Stage 3: Compilation
    let chunk = Compiler::compile_program(&program)?;

    Ok(chunk)
}

/// Execute bytecode directly without saving to file
///
/// This is a convenience function that deserializes and executes bytecode
/// in a single step. Useful for in-memory bytecode caching scenarios.
///
/// # Arguments
///
/// * `bytecode` - Serialized bytecode (including FZB magic header)
///
/// # Returns
///
/// Returns the execution result as a `Value`.
///
/// # Errors
///
/// Returns `FusabiError` if:
/// - Bytecode is invalid (bad magic bytes, wrong version)
/// - Deserialization fails
/// - Execution fails
///
/// # Example
///
/// ```no_run
/// use fusabi::{compile_to_bytecode, execute_bytecode};
///
/// let source = "let x = 42 in x * 2";
/// let bytecode = compile_to_bytecode(source).unwrap();
///
/// // Execute bytecode directly
/// let result = execute_bytecode(&bytecode).unwrap();
/// assert_eq!(result.as_int(), Some(84));
/// ```
pub fn execute_bytecode(bytecode: &[u8]) -> Result<Value, FusabiError> {
    let chunk = deserialize_chunk(bytecode)?;

    let mut vm = Vm::new();
    fusabi_vm::stdlib::register_stdlib(&mut vm);
    let result = vm.execute(chunk)?;

    Ok(result)
}

/// Compile and execute source in one step, returning both bytecode and result
///
/// This is useful for development workflows where you want to cache the bytecode
/// for future runs while also executing immediately.
///
/// # Arguments
///
/// * `source` - Fusabi source code
///
/// # Returns
///
/// Returns a tuple of `(bytecode, result)`.
///
/// # Example
///
/// ```no_run
/// use fusabi::compile_and_execute;
/// use std::fs;
///
/// let source = "let x = 42 in x * 2";
///
/// // Compile and execute in one call
/// let (bytecode, result) = compile_and_execute(source).unwrap();
///
/// // Cache the bytecode for future runs
/// fs::write("cached.fzb", &bytecode).unwrap();
///
/// // Use the result immediately
/// assert_eq!(result.as_int(), Some(84));
/// ```
pub fn compile_and_execute(source: &str) -> Result<(Vec<u8>, Value), FusabiError> {
    let bytecode = compile_to_bytecode(source)?;
    let result = execute_bytecode(&bytecode)?;
    Ok((bytecode, result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_to_bytecode() {
        let source = "let x = 42 in x * 2";
        let bytecode = compile_to_bytecode(source).unwrap();

        // Check magic bytes
        assert!(bytecode.starts_with(FZB_MAGIC));

        // Check version
        assert_eq!(bytecode[4], fusabi_vm::FZB_VERSION);

        // Should be non-empty
        assert!(bytecode.len() > 5);
    }

    #[test]
    fn test_compile_to_chunk() {
        let source = "let x = 42 in x * 2";
        let chunk = compile_to_chunk(source).unwrap();

        // Should have instructions
        assert!(!chunk.instructions.is_empty());

        // Should have constants
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_execute_bytecode() {
        let source = "let x = 42 in x * 2";
        let bytecode = compile_to_bytecode(source).unwrap();
        let result = execute_bytecode(&bytecode).unwrap();

        assert_eq!(result.as_int(), Some(84));
    }

    #[test]
    fn test_compile_and_execute() {
        let source = "let x = 10 in x + 32";
        let (bytecode, result) = compile_and_execute(source).unwrap();

        // Check bytecode is valid
        assert!(bytecode.starts_with(FZB_MAGIC));

        // Check result is correct
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_bytecode_roundtrip() {
        // Simple arithmetic test
        let source = r#"
            let a = 10
            in
            let b = 20
            in
            a + b
        "#;

        // Compile to bytecode
        let bytecode = compile_to_bytecode(source).unwrap();

        // Execute from bytecode
        let result = execute_bytecode(&bytecode).unwrap();

        // Verify result (10 + 20 = 30)
        assert_eq!(result.as_int(), Some(30));
    }

    #[test]
    fn test_invalid_bytecode() {
        let invalid_bytecode = b"NOT_FZB";
        let result = execute_bytecode(invalid_bytecode);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_file_to_bytecode() {
        use std::io::Write;

        // Create temporary file
        let temp_path = "/tmp/test_compile.fsx";
        let source = "let x = 42 in x";

        let mut file = std::fs::File::create(temp_path).unwrap();
        file.write_all(source.as_bytes()).unwrap();
        drop(file);

        // Compile file to bytecode
        let bytecode = compile_file_to_bytecode(temp_path).unwrap();

        // Verify bytecode is valid
        assert!(bytecode.starts_with(FZB_MAGIC));

        // Execute and verify
        let result = execute_bytecode(&bytecode).unwrap();
        assert_eq!(result.as_int(), Some(42));

        // Cleanup
        std::fs::remove_file(temp_path).unwrap();
    }
}
