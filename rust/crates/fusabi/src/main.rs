//! Fusabi - Functional Scripting for Rust
//!
//! Command-line interface for executing Mini-F# scripts through the Fusabi pipeline.
//!
//! # Usage
//!
//! ```bash
//! # Run a script file (JIT execution)
//! fus run examples/hello.fsx
//!
//! # Compile to bytecode
//! fus grind examples/arithmetic.fsx
//!
//! # Run with disassembly output
//! fus run --disasm examples/arithmetic.fsx
//!
//! # Evaluate an expression directly
//! fus run -e "let x = 42 in x + 1"
//!
//! # Package manager (coming soon)
//! fus root install some-package
//!
//! # Show help
//! fus --help
//! ```

use fusabi_demo::{run_file, run_file_with_disasm, run_source, run_source_with_disasm};
use std::env;
use std::fs;
use std::process;
use fusabi_frontend::{Lexer, Parser, Compiler};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP_TEXT: &str = r#"
Fusabi - Small. Potent. Functional.

USAGE:
    fus <COMMAND> [OPTIONS] [FILE]
    fus run -e <EXPRESSION>

COMMANDS:
    run                 JIT execution of .fsx script (default)
    grind               Compile script to .fzb bytecode
    root                Package manager (coming soon)

OPTIONS:
    -h, --help          Show this help message
    -v, --version       Show version information
    -e, --eval <EXPR>   Evaluate an expression directly (run mode only)
    -d, --disasm        Show bytecode disassembly before execution

ARGUMENTS:
    FILE                Path to .fsx script file

EXAMPLES:
    # JIT execute a script
    fus run examples/arithmetic.fsx

    # Compile to bytecode
    fus grind examples/arithmetic.fsx

    # Evaluate an expression
    fus run -e "let x = 10 in x + 5"

    # Show bytecode disassembly
    fus run --disasm examples/conditionals.fsx

    # Package manager (placeholder)
    fus root install some-package

For more information, see: https://github.com/fusabi-lang/fusabi
"#;

struct Config {
    mode: Mode,
    disasm: bool,
}

enum Mode {
    RunFile(String),
    Eval(String),
    Grind(String),
    Root(Vec<String>),
    Help,
    Version,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    let mut mode = None;
    let mut disasm = false;
    let mut i = 1;

    // Check for global flags first
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                mode = Some(Mode::Help);
                break;
            }
            "-v" | "--version" => {
                mode = Some(Mode::Version);
                break;
            }
            _ => break,
        }
    }

    // If we haven't matched a global flag, parse command
    if mode.is_none() && i < args.len() {
        let command = args[i].as_str();
        match command {
            "run" => {
                i += 1;
                // Parse run options
                while i < args.len() {
                    match args[i].as_str() {
                        "-d" | "--disasm" => {
                            disasm = true;
                            i += 1;
                        }
                        "-e" | "--eval" => {
                            if i + 1 >= args.len() {
                                return Err("--eval requires an expression argument".to_string());
                            }
                            mode = Some(Mode::Eval(args[i + 1].clone()));
                            i += 2;
                        }
                        arg if arg.starts_with('-') => {
                            return Err(format!("Unknown option: {}", arg));
                        }
                        file => {
                            mode = Some(Mode::RunFile(file.to_string()));
                            break;
                        }
                    }
                }
                // Default to hello.fus if no file specified
                if mode.is_none() {
                    mode = Some(Mode::RunFile("examples/hello.fus".to_string()));
                }
            }
            "grind" => {
                i += 1;
                if i >= args.len() {
                    return Err("grind command requires a script file".to_string());
                }
                mode = Some(Mode::Grind(args[i].clone()));
            }
            "root" => {
                i += 1;
                let subcommands: Vec<String> = args[i..].to_vec();
                mode = Some(Mode::Root(subcommands));
            }
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: {}", arg));
            }
            // If no command specified, treat first arg as file for run mode
            file => {
                mode = Some(Mode::RunFile(file.to_string()));
            }
        }
    }

    let mode = mode.unwrap_or_else(|| Mode::RunFile("examples/hello.fus".to_string()));

    Ok(Config { mode, disasm })
}

fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    match config.mode {
        Mode::Help => {
            print!("{}", HELP_TEXT);
            Ok(())
        }
        Mode::Version => {
            println!("fus version {}", VERSION);
            Ok(())
        }
        Mode::Eval(expr) => {
            let result = if config.disasm {
                run_source_with_disasm(&expr, "<eval>")?
            } else {
                run_source(&expr)?
            };
            println!("{}", result);
            Ok(())
        }
        Mode::RunFile(path) => {
            let result = if config.disasm {
                run_file_with_disasm(&path)?
            } else {
                run_file(&path)?
            };
            println!("{}", result);
            Ok(())
        }
        Mode::Grind(path) => {
            grind_command(&path);
            Ok(())
        }
        Mode::Root(subcommands) => {
            println!("Fusabi Package Manager - Coming Soon");
            if !subcommands.is_empty() {
                println!("Requested: fus root {}", subcommands.join(" "));
            }
            println!("\nPlanned features:");
            println!("  - fus root install <package>  # Install package");
            println!("  - fus root search <query>     # Search packages");
            println!("  - fus root update             # Update packages");
            println!("  - fus root init               # Initialize project");
            Ok(())
        }
    }
}

fn grind_command(file_path: &str) {
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().expect("Lex error");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("Parse error");
    let chunk = Compiler::compile(&ast).expect("Compile error");
    
    let bytes = match fusabi_vm::serialize_chunk(&chunk) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Serialization error: {}", e);
            process::exit(1);
        }
    };
    
    let output_path = file_path.replace(".fsx", ".fzb");
    if let Err(e) = fs::write(&output_path, &bytes) {
        eprintln!("Failed to write to '{}': {}", output_path, e);
        process::exit(1);
    }

    println!("Compiled {} ({} bytes) -> {}", file_path, bytes.len(), output_path);
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!("Try 'fus --help' for more information.");
            process::exit(1);
        }
    };

    if let Err(err) = run(config) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn test_help_text_not_empty() {
        assert!(!HELP_TEXT.is_empty());
    }
}
