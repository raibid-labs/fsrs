//! Console module for user input operations
//!
//! Provides functions for reading user input from stdin and writing to stdout.

use crate::value::Value;
use crate::vm::VmError;
use std::io::{self, BufRead, Read, Write};

/// Console.readLine : unit -> string
/// Reads a line of input from stdin (blocking)
///
/// Returns the input string with trailing newline removed.
///
/// Example:
///   let name = Console.readLine ()
///   // User types "Alice" and presses Enter
///   // Returns "Alice"
pub fn console_read_line(input: &Value) -> Result<Value, VmError> {
    // Verify unit argument
    if *input != Value::Unit {
        return Err(VmError::TypeMismatch {
            expected: "unit",
            got: input.type_name(),
        });
    }

    let mut buffer = String::new();
    io::stdout()
        .flush()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    io::stdin()
        .lock()
        .read_line(&mut buffer)
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    // Remove trailing newline
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }

    Ok(Value::Str(buffer))
}

/// Console.readKey : unit -> string
/// Reads a single keypress from stdin (blocking)
/// Returns the key as a string (e.g., "a", "Enter", "ArrowUp")
///
/// Note: This provides basic single-character reading. Special keys are detected
/// based on common escape sequences.
///
/// Example:
///   let key = Console.readKey ()
///   // User presses 'a'
///   // Returns "a"
pub fn console_read_key(input: &Value) -> Result<Value, VmError> {
    // Verify unit argument
    if *input != Value::Unit {
        return Err(VmError::TypeMismatch {
            expected: "unit",
            got: input.type_name(),
        });
    }

    // Read a single byte/character
    // For cross-platform compatibility, we read one character at a time
    let mut buffer = [0u8; 1];
    io::stdin()
        .lock()
        .read_exact(&mut buffer)
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    let key = match buffer[0] {
        b'\n' | b'\r' => "Enter".to_string(),
        b'\x1b' => "Escape".to_string(),
        b'\t' => "Tab".to_string(),
        b'\x7f' | b'\x08' => "Backspace".to_string(),
        c if c.is_ascii() => (c as char).to_string(),
        _ => String::from_utf8_lossy(&buffer).to_string(),
    };

    Ok(Value::Str(key))
}

/// Console.write : string -> unit
/// Writes a string to stdout without newline
///
/// Example:
///   Console.write "Enter your name: "
///   // Outputs "Enter your name: " without newline
pub fn console_write(text: &Value) -> Result<Value, VmError> {
    match text {
        Value::Str(s) => {
            print!("{}", s);
            io::stdout()
                .flush()
                .map_err(|e| VmError::Runtime(e.to_string()))?;
            Ok(Value::Unit)
        }
        other => Err(VmError::TypeMismatch {
            expected: "string",
            got: other.type_name(),
        }),
    }
}

/// Console.writeLine : string -> unit
/// Writes a string to stdout with newline
///
/// Example:
///   Console.writeLine "Hello, World!"
///   // Outputs "Hello, World!" followed by newline
pub fn console_write_line(text: &Value) -> Result<Value, VmError> {
    match text {
        Value::Str(s) => {
            println!("{}", s);
            Ok(Value::Unit)
        }
        other => Err(VmError::TypeMismatch {
            expected: "string",
            got: other.type_name(),
        }),
    }
}

/// Console.clear : unit -> unit
/// Clears the terminal screen using ANSI escape codes
///
/// Example:
///   Console.clear ()
///   // Clears the screen and moves cursor to top-left
pub fn console_clear(input: &Value) -> Result<Value, VmError> {
    // Verify unit argument
    if *input != Value::Unit {
        return Err(VmError::TypeMismatch {
            expected: "unit",
            got: input.type_name(),
        });
    }

    // ANSI escape code to clear screen and move cursor to top-left
    print!("\x1b[2J\x1b[H");
    io::stdout()
        .flush()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    Ok(Value::Unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_write_valid() {
        let input = Value::Str("test".to_string());
        let result = console_write(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_console_write_type_error() {
        let input = Value::Int(42);
        let result = console_write(&input);
        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_console_write_line_valid() {
        let input = Value::Str("test".to_string());
        let result = console_write_line(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_console_write_line_type_error() {
        let input = Value::Int(42);
        let result = console_write_line(&input);
        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_console_clear_valid() {
        let input = Value::Unit;
        let result = console_clear(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_console_clear_type_error() {
        let input = Value::Int(42);
        let result = console_clear(&input);
        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "unit");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }
}
