// Fusabi File Standard Library
// Provides file I/O operations with line-based support

use crate::value::Value;
use crate::vm::VmError;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};

/// File.readLines : string -> string list
/// Reads a file and returns its contents as a list of lines
pub fn file_read_lines(path: &Value) -> Result<Value, VmError> {
    let path_str = match path {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: path.type_name(),
            })
        }
    };

    let file = fs::File::open(path_str).map_err(|e| {
        VmError::Runtime(format!("Failed to open file '{}': {}", path_str, e))
    })?;

    let reader = BufReader::new(file);
    let lines: Result<Vec<String>, _> = reader.lines().collect();
    let lines = lines.map_err(|e| {
        VmError::Runtime(format!("Failed to read file '{}': {}", path_str, e))
    })?;

    let mut result = Value::Nil;
    for line in lines.into_iter().rev() {
        result = Value::Cons {
            head: Box::new(Value::Str(line)),
            tail: Box::new(result),
        };
    }

    Ok(result)
}

/// File.writeLines : string -> string list -> unit
/// Writes a list of lines to a file (overwrites existing content)
pub fn file_write_lines(path: &Value, lines: &Value) -> Result<Value, VmError> {
    let path_str = match path {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: path.type_name(),
            })
        }
    };

    let mut line_vec = Vec::new();
    let mut current = lines.clone();
    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                match &*head {
                    Value::Str(s) => line_vec.push(s.clone()),
                    _ => {
                        return Err(VmError::TypeMismatch {
                            expected: "string list",
                            got: "list with non-string elements",
                        })
                    }
                }
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

    let mut file = fs::File::create(path_str).map_err(|e| {
        VmError::Runtime(format!("Failed to create file '{}': {}", path_str, e))
    })?;

    for line in line_vec {
        writeln!(file, "{}", line).map_err(|e| {
            VmError::Runtime(format!("Failed to write to file '{}': {}", path_str, e))
        })?;
    }

    Ok(Value::Unit)
}

/// File.appendLine : string -> string -> unit
/// Appends a single line to a file
pub fn file_append_line(path: &Value, line: &Value) -> Result<Value, VmError> {
    let path_str = match path {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: path.type_name(),
            })
        }
    };

    let line_str = match line {
        Value::Str(s) => s,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: line.type_name(),
            })
        }
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_str)
        .map_err(|e| {
            VmError::Runtime(format!("Failed to open file '{}' for append: {}", path_str, e))
        })?;

    writeln!(file, "{}", line_str).map_err(|e| {
        VmError::Runtime(format!("Failed to append to file '{}': {}", path_str, e))
    })?;

    Ok(Value::Unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_temp_file(name: &str, content: &str) -> String {
        let path = format!("/tmp/fusabi_test_{}", name);
        let mut file = fs::File::create(&path).unwrap();
        write!(file, "{}", content).unwrap();
        path
    }

    fn cleanup_temp_file(path: &str) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_file_read_lines() {
        let path = create_temp_file("read_lines.txt", "line1\nline2\nline3\n");
        let result = file_read_lines(&Value::Str(path.clone())).unwrap();
        
        let expected = Value::vec_to_cons(vec![
            Value::Str("line1".to_string()),
            Value::Str("line2".to_string()),
            Value::Str("line3".to_string()),
        ]);
        assert_eq!(result, expected);
        
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_read_lines_empty() {
        let path = create_temp_file("read_lines_empty.txt", "");
        let result = file_read_lines(&Value::Str(path.clone())).unwrap();
        assert_eq!(result, Value::Nil);
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_read_lines_not_found() {
        let result = file_read_lines(&Value::Str("/nonexistent/file.txt".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_read_lines_type_error() {
        let result = file_read_lines(&Value::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_write_lines() {
        let path = "/tmp/fusabi_test_write_lines.txt".to_string();
        let lines = Value::vec_to_cons(vec![
            Value::Str("hello".to_string()),
            Value::Str("world".to_string()),
        ]);
        
        let result = file_write_lines(&Value::Str(path.clone()), &lines).unwrap();
        assert_eq!(result, Value::Unit);
        
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello\nworld\n");
        
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_write_lines_empty() {
        let path = "/tmp/fusabi_test_write_lines_empty.txt".to_string();
        let result = file_write_lines(&Value::Str(path.clone()), &Value::Nil).unwrap();
        assert_eq!(result, Value::Unit);
        
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "");
        
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_write_lines_type_error_path() {
        let result = file_write_lines(&Value::Int(42), &Value::Nil);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_write_lines_type_error_lines() {
        let path = "/tmp/fusabi_test_type_error.txt".to_string();
        let lines = Value::vec_to_cons(vec![Value::Int(42)]);
        let result = file_write_lines(&Value::Str(path), &lines);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_append_line() {
        let path = "/tmp/fusabi_test_append_line.txt".to_string();
        let _ = fs::remove_file(&path);
        
        file_append_line(&Value::Str(path.clone()), &Value::Str("first".to_string())).unwrap();
        file_append_line(&Value::Str(path.clone()), &Value::Str("second".to_string())).unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "first\nsecond\n");
        
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_append_line_creates_file() {
        let path = "/tmp/fusabi_test_append_create.txt".to_string();
        let _ = fs::remove_file(&path);
        
        let result = file_append_line(&Value::Str(path.clone()), &Value::Str("new line".to_string()));
        assert!(result.is_ok());
        
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "new line\n");
        
        cleanup_temp_file(&path);
    }

    #[test]
    fn test_file_append_line_type_error_path() {
        let result = file_append_line(&Value::Int(42), &Value::Str("test".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_file_append_line_type_error_line() {
        let result = file_append_line(&Value::Str("/tmp/test.txt".to_string()), &Value::Int(42));
        assert!(result.is_err());
    }
}
