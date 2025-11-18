//! FSRS Demo Host Application
//!
//! Command-line interface for executing Mini-F# scripts through the FSRS pipeline.
//!
//! # Usage
//!
//! ```bash
//! # Run a script file
//! fsrs-demo examples/hello.fsrs
//!
//! # Run with disassembly output
//! fsrs-demo --disasm examples/arithmetic.fsrs
//!
//! # Evaluate an expression directly
//! fsrs-demo -e "let x = 42 in x + 1"
//!
//! # Show help
//! fsrs-demo --help
//! ```

use fsrs_demo::{run_file, run_file_with_disasm, run_source, run_source_with_disasm};
use std::env;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP_TEXT: &str = r#"
FSRS Demo Host - Mini-F# Script Engine

USAGE:
    fsrs-demo [OPTIONS] [FILE]
    fsrs-demo -e <EXPRESSION>

OPTIONS:
    -h, --help          Show this help message
    -v, --version       Show version information
    -e, --eval <EXPR>   Evaluate an expression directly
    -d, --disasm        Show bytecode disassembly before execution

ARGUMENTS:
    FILE                Path to .fsrs script file (default: examples/hello.fsrs)

EXAMPLES:
    # Run a script file
    fsrs-demo examples/arithmetic.fsrs

    # Evaluate an expression
    fsrs-demo -e "let x = 10 in x + 5"

    # Show bytecode disassembly
    fsrs-demo --disasm examples/conditionals.fsrs

    # Run default hello world example
    fsrs-demo

For more information, see: https://github.com/raibid-labs/fsrs
"#;

struct Config {
    mode: Mode,
    disasm: bool,
}

enum Mode {
    RunFile(String),
    Eval(String),
    Help,
    Version,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    let mut mode = None;
    let mut disasm = false;
    let mut i = 1;

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
                i += 1;
            }
        }
    }

    let mode = mode.unwrap_or_else(|| Mode::RunFile("examples/hello.fsrs".to_string()));

    Ok(Config { mode, disasm })
}

fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    match config.mode {
        Mode::Help => {
            print!("{}", HELP_TEXT);
            Ok(())
        }
        Mode::Version => {
            println!("fsrs-demo version {}", VERSION);
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
    }
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            eprintln!("Try 'fsrs-demo --help' for more information.");
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
