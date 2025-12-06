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
//! # Package manager commands (delegates to fpm)
//! fus pm init              # Initialize a new package
//! fus pm build             # Build the package
//! fus pm run               # Run the package
//! fus pm add <package>     # Add a dependency
//!
//! # Show help
//! fus --help
//! ```

use colored::*;
use fusabi::{run_file, run_file_with_disasm, run_source, run_source_with_disasm};
use fusabi_frontend::{Compiler, Lexer, Parser};
use std::env;
use std::fs;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const BANNER: &str = r#"
   ___                _     _
  / __\   _ ___  __ _| |__ (_)
 / _\| | | / __|/ _` | '_ \| |
/ /  | |_| \__ \ (_| | |_) | |
\/    \__,_|___/\__,_|_.__/|_|
"#;

fn print_help() {
    println!("{}", BANNER.truecolor(153, 204, 51));
    println!(
        "{}",
        "Small. Potent. Functional."
            .italic()
            .truecolor(128, 128, 128)
    );
    println!();
    println!("{}", "USAGE:".bold());
    println!("    fus <COMMAND> [OPTIONS] [FILE]");
    println!("    fus run -e <EXPRESSION>");
    println!();
    println!("{}", "COMMANDS:".bold());
    println!(
        "    {}                 JIT execution of .fsx script (default)",
        "run".truecolor(153, 204, 51)
    );
    println!(
        "    {}               Compile script to .fzb bytecode",
        "grind".truecolor(153, 204, 51)
    );
    println!(
        "    {}                  Package manager (delegates to fpm)",
        "pm".truecolor(153, 204, 51)
    );
    println!();
    println!("{}", "OPTIONS:".bold());
    println!("    -h, --help          Show this help message");
    println!("    -v, --version       Show version information");
    println!("    -e, --eval <EXPR>   Evaluate an expression directly (run mode only)");
    println!("    -d, --disasm        Show bytecode disassembly before execution");
    println!();
    println!("{}", "ARGUMENTS:".bold());
    println!("    FILE                Path to .fsx script file");
    println!();
    println!("{}", "EXAMPLES:".bold());
    println!(
        "    {}",
        "# JIT execute a script".italic().truecolor(128, 128, 128)
    );
    println!("    fus run examples/arithmetic.fsx");
    println!();
    println!(
        "    {}",
        "# Compile to bytecode".italic().truecolor(128, 128, 128)
    );
    println!("    fus grind examples/arithmetic.fsx");
    println!();
    println!(
        "    {}",
        "# Evaluate an expression".italic().truecolor(128, 128, 128)
    );
    println!("    fus run -e \"let x = 10 in x + 5\"");
    println!();
    println!(
        "    {}",
        "# Show bytecode disassembly"
            .italic()
            .truecolor(128, 128, 128)
    );
    println!("    fus run --disasm examples/conditionals.fsx");
    println!();
    println!(
        "    {}",
        "# Package manager (init, build, run, add)"
            .italic()
            .truecolor(128, 128, 128)
    );
    println!("    fus pm init");
    println!("    fus pm build");
    println!("    fus pm run");
    println!();
    println!(
        "For more information, see: {}",
        "https://github.com/fusabi-lang/fusabi".cyan()
    );
}

struct Config {
    mode: Mode,
    disasm: bool,
}

enum Mode {
    RunFile(String),
    Eval(String),
    Grind(String),
    Pm(Vec<String>),
    Help,
    Version,
}

fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();

    let mut mode = None;
    let mut disasm = false;
    let mut i = 1;

    // Check for global flags first
    if i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                mode = Some(Mode::Help);
            }
            "-v" | "--version" => {
                mode = Some(Mode::Version);
            }
            _ => {}
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
            "pm" => {
                i += 1;
                let subcommands: Vec<String> = args[i..].to_vec();
                mode = Some(Mode::Pm(subcommands));
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
            print_help();
            Ok(())
        }
        Mode::Version => {
            println!(
                "{} {}",
                "fus version".truecolor(153, 204, 51).bold(),
                VERSION
            );
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
        Mode::Pm(subcommands) => pm_command(subcommands),
    }
}

fn grind_command(file_path: &str) {
    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{} reading file '{}': {}",
                "Error".truecolor(183, 65, 14).bold(),
                file_path,
                e
            );
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "Lex error:".truecolor(183, 65, 14).bold(), e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "Parse error:".truecolor(183, 65, 14).bold(), e);
            process::exit(1);
        }
    };

    let chunk = match Compiler::compile_program(&program) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{} {}", "Compile error:".truecolor(183, 65, 14).bold(), e);
            process::exit(1);
        }
    };

    let bytes = match fusabi_vm::serialize_chunk(&chunk) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "{} {}",
                "Serialization error:".truecolor(183, 65, 14).bold(),
                e
            );
            process::exit(1);
        }
    };

    let output_path = file_path.replace(".fsx", ".fzb");
    if let Err(e) = fs::write(&output_path, &bytes) {
        eprintln!(
            "{} Failed to write to '{}': {}",
            "Error:".truecolor(183, 65, 14).bold(),
            output_path,
            e
        );
        process::exit(1);
    }

    println!(
        "{} {} ({} bytes) -> {}",
        "Compiled".truecolor(153, 204, 51).bold(),
        file_path,
        bytes.len(),
        output_path
    );
}

/// Delegates to fpm (Fusabi Package Manager) subprocess.
fn pm_command(subcommands: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Show help if no subcommand provided
    if subcommands.is_empty() {
        println!(
            "{}",
            "Fusabi Package Manager".truecolor(153, 204, 51).bold()
        );
        println!();
        println!("{}", "USAGE:".bold());
        println!("    fus pm <COMMAND> [OPTIONS]");
        println!();
        println!("{}", "COMMANDS:".bold());
        println!(
            "    {}             Initialize a new Fusabi package",
            "init".truecolor(153, 204, 51)
        );
        println!(
            "    {}            Build the current package",
            "build".truecolor(153, 204, 51)
        );
        println!(
            "    {}              Run the current package",
            "run".truecolor(153, 204, 51)
        );
        println!(
            "    {}              Add a dependency",
            "add".truecolor(153, 204, 51)
        );
        println!();
        println!("{}", "EXAMPLES:".bold());
        println!(
            "    {}",
            "# Initialize a new package"
                .italic()
                .truecolor(128, 128, 128)
        );
        println!("    fus pm init");
        println!();
        println!(
            "    {}",
            "# Build and run the package"
                .italic()
                .truecolor(128, 128, 128)
        );
        println!("    fus pm build && fus pm run");
        println!();
        println!(
            "    {}",
            "# Add a dependency".italic().truecolor(128, 128, 128)
        );
        println!("    fus pm add json --version 1.0.0");
        println!();
        println!(
            "{} You can also use '{}' directly for additional options.",
            "Tip:".italic().truecolor(128, 128, 128),
            "fpm".truecolor(153, 204, 51)
        );
        return Ok(());
    }

    // Try to execute fpm with the subcommands
    let status = Command::new("fpm").args(&subcommands).status();

    match status {
        Ok(exit_status) => {
            if !exit_status.success() {
                process::exit(exit_status.code().unwrap_or(1));
            }
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "{} fpm (Fusabi Package Manager) not found in PATH",
                "Error:".truecolor(183, 65, 14).bold()
            );
            eprintln!();
            eprintln!(
                "{} Install fpm with: {}",
                "Hint:".italic().truecolor(128, 128, 128),
                "cargo install --path rust/crates/fusabi-pm".cyan()
            );
            process::exit(1);
        }
        Err(e) => {
            eprintln!(
                "{} Failed to execute fpm: {}",
                "Error:".truecolor(183, 65, 14).bold(),
                e
            );
            process::exit(1);
        }
    }
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{} {}", "Error:".truecolor(183, 65, 14).bold(), err);
            eprintln!(
                "{} Try 'fus --help' for more information.",
                "Hint:".italic().truecolor(128, 128, 128)
            );
            process::exit(1);
        }
    };

    if let Err(err) = run(config) {
        eprintln!("{} {}", "Error:".truecolor(183, 65, 14).bold(), err);
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
    fn test_banner_not_empty() {
        assert!(!BANNER.is_empty());
    }
}
