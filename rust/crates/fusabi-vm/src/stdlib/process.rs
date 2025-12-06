// Fusabi Process Standard Library
// Provides process execution and environment variable access
//
// SECURITY NOTE: This module executes real system commands and modifies environment variables.
// It is designed for plugin/script runtime environments where controlled system access is desired.
// Consider your security requirements before exposing these functions to untrusted code.

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Process.run : string -> string list -> ProcessResult
/// Runs a command with the given arguments and returns the result.
/// The command is executed directly (not through a shell).
///
/// Example:
///   Process.run "echo" ["hello"; "world"]
///   // Returns { exitCode = 0; stdout = "hello world\n"; stderr = "" }
pub fn process_run(cmd: &Value, args: &Value) -> Result<Value, VmError> {
    let cmd_str = cmd.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: cmd.type_name(),
    })?;

    // Convert list to Vec<String>
    let args_vec = list_to_string_vec(args)?;

    // Execute the command
    let output = Command::new(cmd_str)
        .args(&args_vec)
        .output()
        .map_err(|e| VmError::Runtime(format!("Failed to execute command '{}': {}", cmd_str, e)))?;

    // Build ProcessResult record
    create_process_result(
        output.status.code().unwrap_or(-1),
        output.stdout,
        output.stderr,
    )
}

/// Process.runShell : string -> ProcessResult
/// Runs a shell command string and returns the result.
/// The command is executed through the system shell (sh on Unix, cmd.exe on Windows).
///
/// Example:
///   Process.runShell "echo hello | grep h"
///   // Returns { exitCode = 0; stdout = "hello\n"; stderr = "" }
pub fn process_run_shell(cmd: &Value) -> Result<Value, VmError> {
    let cmd_str = cmd.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: cmd.type_name(),
    })?;

    // Execute through shell
    #[cfg(unix)]
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .output()
        .map_err(|e| VmError::Runtime(format!("Failed to execute shell command: {}", e)))?;

    #[cfg(windows)]
    let output = Command::new("cmd")
        .arg("/C")
        .arg(cmd_str)
        .output()
        .map_err(|e| VmError::Runtime(format!("Failed to execute shell command: {}", e)))?;

    // Build ProcessResult record
    create_process_result(
        output.status.code().unwrap_or(-1),
        output.stdout,
        output.stderr,
    )
}

/// Process.env : string -> string option
/// Gets an environment variable value.
/// Returns Some(value) if the variable exists, None otherwise.
///
/// Example:
///   Process.env "PATH"
///   // Returns Some("/usr/bin:/bin:...")
pub fn process_env(var_name: &Value) -> Result<Value, VmError> {
    let var_name_str = var_name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: var_name.type_name(),
    })?;

    match std::env::var(var_name_str) {
        Ok(value) => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Str(value)],
        }),
        Err(_) => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        }),
    }
}

/// Process.setEnv : string -> string -> unit
/// Sets an environment variable for the current process and child processes.
/// Note: This only affects the current process and its children, not the parent process.
///
/// Example:
///   Process.setEnv "MY_VAR" "my_value"
///   // Returns ()
pub fn process_set_env(var_name: &Value, value: &Value) -> Result<Value, VmError> {
    let var_name_str = var_name.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: var_name.type_name(),
    })?;

    let value_str = value.as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: value.type_name(),
    })?;

    std::env::set_var(var_name_str, value_str);
    Ok(Value::Unit)
}

/// Process.cwd : unit -> string
/// Gets the current working directory.
///
/// Example:
///   Process.cwd ()
///   // Returns "/home/user/project"
pub fn process_cwd(_unit: &Value) -> Result<Value, VmError> {
    let cwd = std::env::current_dir()
        .map_err(|e| VmError::Runtime(format!("Failed to get current directory: {}", e)))?;

    let cwd_str = cwd.to_str().ok_or_else(|| {
        VmError::Runtime("Current directory path contains invalid Unicode".to_string())
    })?;

    Ok(Value::Str(cwd_str.to_string()))
}

// Helper functions

/// Convert a Fusabi list to Vec<String>, validating that all elements are strings
fn list_to_string_vec(list: &Value) -> Result<Vec<String>, VmError> {
    let mut result = Vec::new();
    let mut current = list.clone();

    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                let arg_str = head.as_str().ok_or_else(|| VmError::TypeMismatch {
                    expected: "string list",
                    got: "list with non-string elements",
                })?;
                result.push(arg_str.to_string());
                current = (*tail).clone();
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "list",
                    got: current.type_name(),
                })
            }
        }
    }

    Ok(result)
}

/// Create a ProcessResult record from exit code and output
fn create_process_result(
    exit_code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> Result<Value, VmError> {
    let stdout_str = String::from_utf8(stdout).unwrap_or_else(|e| {
        // If stdout is not valid UTF-8, replace invalid sequences
        String::from_utf8_lossy(&e.into_bytes()).to_string()
    });

    let stderr_str = String::from_utf8(stderr).unwrap_or_else(|e| {
        // If stderr is not valid UTF-8, replace invalid sequences
        String::from_utf8_lossy(&e.into_bytes()).to_string()
    });

    let mut fields = HashMap::new();
    fields.insert("exitCode".to_string(), Value::Int(exit_code as i64));
    fields.insert("stdout".to_string(), Value::Str(stdout_str));
    fields.insert("stderr".to_string(), Value::Str(stderr_str));

    Ok(Value::Record(Arc::new(Mutex::new(fields))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_run_echo() {
        let cmd = Value::Str("echo".to_string());
        let args = Value::vec_to_cons(vec![Value::Str("hello".to_string())]);
        let result = process_run(&cmd, &args).unwrap();

        // Verify it's a record
        assert!(result.is_record());

        // Check exit code
        let exit_code = result.record_get("exitCode").unwrap();
        assert_eq!(exit_code, Value::Int(0));

        // Check stdout contains "hello"
        let stdout = result.record_get("stdout").unwrap();
        if let Value::Str(s) = stdout {
            assert!(s.contains("hello"));
        } else {
            panic!("stdout should be a string");
        }

        // Check stderr is empty
        let stderr = result.record_get("stderr").unwrap();
        assert_eq!(stderr, Value::Str("".to_string()));
    }

    #[test]
    fn test_process_run_multiple_args() {
        let cmd = Value::Str("echo".to_string());
        let args = Value::vec_to_cons(vec![
            Value::Str("hello".to_string()),
            Value::Str("world".to_string()),
        ]);
        let result = process_run(&cmd, &args).unwrap();

        let exit_code = result.record_get("exitCode").unwrap();
        assert_eq!(exit_code, Value::Int(0));

        let stdout = result.record_get("stdout").unwrap();
        if let Value::Str(s) = stdout {
            assert!(s.contains("hello"));
            assert!(s.contains("world"));
        } else {
            panic!("stdout should be a string");
        }
    }

    #[test]
    fn test_process_run_empty_args() {
        let cmd = Value::Str("pwd".to_string());
        let args = Value::Nil;
        let result = process_run(&cmd, &args).unwrap();

        let exit_code = result.record_get("exitCode").unwrap();
        assert_eq!(exit_code, Value::Int(0));

        let stdout = result.record_get("stdout").unwrap();
        if let Value::Str(s) = stdout {
            assert!(!s.is_empty());
        } else {
            panic!("stdout should be a string");
        }
    }

    #[test]
    fn test_process_run_command_not_found() {
        let cmd = Value::Str("this_command_does_not_exist_12345".to_string());
        let args = Value::Nil;
        let result = process_run(&cmd, &args);

        assert!(result.is_err());
        if let Err(VmError::Runtime(msg)) = result {
            assert!(msg.contains("Failed to execute command"));
        } else {
            panic!("Expected Runtime error");
        }
    }

    #[test]
    fn test_process_run_type_error_cmd() {
        let cmd = Value::Int(42);
        let args = Value::Nil;
        let result = process_run(&cmd, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_run_type_error_args() {
        let cmd = Value::Str("echo".to_string());
        let args = Value::Int(42);
        let result = process_run(&cmd, &args);

        assert!(result.is_err());
    }

    #[test]
    fn test_process_run_args_not_strings() {
        let cmd = Value::Str("echo".to_string());
        let args = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = process_run(&cmd, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, .. }) = result {
            assert_eq!(expected, "string list");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_run_shell_basic() {
        let cmd = Value::Str("echo hello".to_string());
        let result = process_run_shell(&cmd).unwrap();

        let exit_code = result.record_get("exitCode").unwrap();
        assert_eq!(exit_code, Value::Int(0));

        let stdout = result.record_get("stdout").unwrap();
        if let Value::Str(s) = stdout {
            assert!(s.contains("hello"));
        } else {
            panic!("stdout should be a string");
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_process_run_shell_pipe() {
        let cmd = Value::Str("echo hello | grep hello".to_string());
        let result = process_run_shell(&cmd).unwrap();

        let exit_code = result.record_get("exitCode").unwrap();
        assert_eq!(exit_code, Value::Int(0));

        let stdout = result.record_get("stdout").unwrap();
        if let Value::Str(s) = stdout {
            assert!(s.contains("hello"));
        } else {
            panic!("stdout should be a string");
        }
    }

    #[test]
    fn test_process_run_shell_type_error() {
        let cmd = Value::Int(42);
        let result = process_run_shell(&cmd);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_env_existing() {
        // Set a test environment variable
        std::env::set_var("FUSABI_TEST_VAR", "test_value");

        let var_name = Value::Str("FUSABI_TEST_VAR".to_string());
        let result = process_env(&var_name).unwrap();

        assert!(result.is_variant());
        assert!(result.is_variant_named("Some"));

        let fields = result.variant_fields().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], Value::Str("test_value".to_string()));

        // Clean up
        std::env::remove_var("FUSABI_TEST_VAR");
    }

    #[test]
    fn test_process_env_nonexistent() {
        let var_name = Value::Str("THIS_VAR_DOES_NOT_EXIST_12345".to_string());
        let result = process_env(&var_name).unwrap();

        assert!(result.is_variant());
        assert!(result.is_variant_named("None"));

        let fields = result.variant_fields().unwrap();
        assert_eq!(fields.len(), 0);
    }

    #[test]
    fn test_process_env_type_error() {
        let var_name = Value::Int(42);
        let result = process_env(&var_name);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_set_env() {
        let var_name = Value::Str("FUSABI_TEST_SET_VAR".to_string());
        let value = Value::Str("test_set_value".to_string());
        let result = process_set_env(&var_name, &value).unwrap();

        assert_eq!(result, Value::Unit);

        // Verify the variable was set
        assert_eq!(
            std::env::var("FUSABI_TEST_SET_VAR").unwrap(),
            "test_set_value"
        );

        // Clean up
        std::env::remove_var("FUSABI_TEST_SET_VAR");
    }

    #[test]
    fn test_process_set_env_type_error_name() {
        let var_name = Value::Int(42);
        let value = Value::Str("value".to_string());
        let result = process_set_env(&var_name, &value);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_set_env_type_error_value() {
        let var_name = Value::Str("TEST_VAR".to_string());
        let value = Value::Int(42);
        let result = process_set_env(&var_name, &value);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, got }) = result {
            assert_eq!(expected, "string");
            assert_eq!(got, "int");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_process_cwd() {
        let unit = Value::Unit;
        let result = process_cwd(&unit).unwrap();

        if let Value::Str(s) = result {
            // Should be a non-empty path
            assert!(!s.is_empty());
            // Should be an absolute path
            assert!(s.starts_with('/') || s.chars().nth(1) == Some(':'));
        } else {
            panic!("cwd should return a string");
        }
    }

    #[test]
    fn test_list_to_string_vec_empty() {
        let list = Value::Nil;
        let result = list_to_string_vec(&list).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_list_to_string_vec_single() {
        let list = Value::vec_to_cons(vec![Value::Str("hello".to_string())]);
        let result = list_to_string_vec(&list).unwrap();
        assert_eq!(result, vec!["hello".to_string()]);
    }

    #[test]
    fn test_list_to_string_vec_multiple() {
        let list = Value::vec_to_cons(vec![
            Value::Str("one".to_string()),
            Value::Str("two".to_string()),
            Value::Str("three".to_string()),
        ]);
        let result = list_to_string_vec(&list).unwrap();
        assert_eq!(
            result,
            vec!["one".to_string(), "two".to_string(), "three".to_string()]
        );
    }

    #[test]
    fn test_list_to_string_vec_non_string() {
        let list = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = list_to_string_vec(&list);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_to_string_vec_not_list() {
        let not_list = Value::Int(42);
        let result = list_to_string_vec(&not_list);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_process_result() {
        let exit_code = 0;
        let stdout = b"hello world".to_vec();
        let stderr = b"".to_vec();

        let result = create_process_result(exit_code, stdout, stderr).unwrap();

        assert!(result.is_record());
        assert_eq!(result.record_get("exitCode").unwrap(), Value::Int(0));
        assert_eq!(
            result.record_get("stdout").unwrap(),
            Value::Str("hello world".to_string())
        );
        assert_eq!(
            result.record_get("stderr").unwrap(),
            Value::Str("".to_string())
        );
    }

    #[test]
    fn test_create_process_result_with_stderr() {
        let exit_code = 1;
        let stdout = b"".to_vec();
        let stderr = b"error occurred".to_vec();

        let result = create_process_result(exit_code, stdout, stderr).unwrap();

        assert_eq!(result.record_get("exitCode").unwrap(), Value::Int(1));
        assert_eq!(
            result.record_get("stdout").unwrap(),
            Value::Str("".to_string())
        );
        assert_eq!(
            result.record_get("stderr").unwrap(),
            Value::Str("error occurred".to_string())
        );
    }

    #[test]
    fn test_create_process_result_invalid_utf8() {
        let exit_code = 0;
        // Invalid UTF-8 sequence
        let stdout = vec![0xFF, 0xFE, 0xFD];
        let stderr = vec![];

        let result = create_process_result(exit_code, stdout, stderr).unwrap();

        assert!(result.is_record());
        // Should have replaced invalid sequences with � or similar
        let stdout_val = result.record_get("stdout").unwrap();
        assert!(stdout_val.as_str().is_some());
    }
}
