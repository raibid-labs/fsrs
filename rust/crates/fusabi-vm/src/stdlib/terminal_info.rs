// Fusabi TerminalInfo Standard Library
// Provides APIs to query terminal state and process information
//
// This module uses a pluggable backend pattern where host applications
// (like terminal emulators) can register a TerminalInfoProvider implementation
// to provide terminal-specific functionality.

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// ProcessInfo record structure
/// Fields: name (string), pid (int), commandLine (string option)
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: i64,
    pub command_line: Option<String>,
}

impl ProcessInfo {
    /// Convert ProcessInfo to a Fusabi Value::Record
    pub fn to_value(&self) -> Value {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::Str(self.name.clone()));
        fields.insert("pid".to_string(), Value::Int(self.pid));

        // Convert command_line to Option variant
        let command_line_value = match &self.command_line {
            Some(cmd) => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![Value::Str(cmd.clone())],
            },
            None => Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            },
        };
        fields.insert("commandLine".to_string(), command_line_value);

        Value::Record(Arc::new(Mutex::new(fields)))
    }
}

/// Trait that host applications implement to provide terminal information
pub trait TerminalInfoProvider: Send + Sync {
    /// Get information about the foreground process
    fn get_foreground_process(&self) -> Option<ProcessInfo>;

    /// Get the current working directory
    fn get_current_working_dir(&self) -> Option<String>;

    /// Get a specific line from the terminal scrollback buffer
    fn get_line(&self, line_number: i64) -> Option<String>;

    /// Get a range of lines from the terminal scrollback buffer
    fn get_lines(&self, start: i64, end: i64) -> Vec<String>;

    /// Get the window title
    fn get_window_title(&self) -> String;

    /// Get the tab title
    fn get_tab_title(&self) -> String;

    /// Get the terminal size as (columns, rows)
    fn get_terminal_size(&self) -> (i64, i64);
}

/// Global provider registry using OnceLock for thread-safe initialization
static PROVIDER: OnceLock<Arc<Mutex<Option<Box<dyn TerminalInfoProvider>>>>> = OnceLock::new();

/// Initialize the provider storage
fn get_provider_storage() -> &'static Arc<Mutex<Option<Box<dyn TerminalInfoProvider>>>> {
    PROVIDER.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// Register a terminal info provider
/// This should be called by host applications to provide terminal functionality
pub fn register_provider(provider: Box<dyn TerminalInfoProvider>) {
    let storage = get_provider_storage();
    let mut guard = storage.lock().unwrap();
    *guard = Some(provider);
}

/// Unregister the current provider (useful for testing)
pub fn unregister_provider() {
    let storage = get_provider_storage();
    let mut guard = storage.lock().unwrap();
    *guard = None;
}

// Helper function to get the provider
fn with_provider<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&dyn TerminalInfoProvider) -> R,
{
    let storage = get_provider_storage();
    let guard = storage.lock().unwrap();
    guard.as_ref().map(|provider| f(provider.as_ref()))
}

/// TerminalInfo.getForegroundProcess : unit -> ProcessInfo option
/// Returns information about the foreground process if available
pub fn get_foreground_process(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let result = with_provider(|provider| provider.get_foreground_process());

            match result {
                Some(Some(process_info)) => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    fields: vec![process_info.to_value()],
                }),
                _ => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                }),
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalInfo.getCurrentWorkingDir : unit -> string option
/// Returns the current working directory if available
pub fn get_current_working_dir(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let result = with_provider(|provider| provider.get_current_working_dir());

            match result {
                Some(Some(cwd)) => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    fields: vec![Value::Str(cwd)],
                }),
                _ => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                }),
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalInfo.getLine : int -> string option
/// Returns the content of a specific line from the scrollback buffer
pub fn get_line(line_number: &Value) -> Result<Value, VmError> {
    match line_number {
        Value::Int(n) => {
            let result = with_provider(|provider| provider.get_line(*n));

            match result {
                Some(Some(line)) => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    fields: vec![Value::Str(line)],
                }),
                _ => Ok(Value::Variant {
                    type_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    fields: vec![],
                }),
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: line_number.type_name(),
        }),
    }
}

/// TerminalInfo.getLines : int -> int -> string list
/// Returns a list of lines from the scrollback buffer between start and end
pub fn get_lines(start: &Value, end: &Value) -> Result<Value, VmError> {
    match (start, end) {
        (Value::Int(start_n), Value::Int(end_n)) => {
            let lines =
                with_provider(|provider| provider.get_lines(*start_n, *end_n)).unwrap_or_default();

            // Build list in reverse order
            let mut result = Value::Nil;
            for line in lines.iter().rev() {
                result = Value::Cons {
                    head: Box::new(Value::Str(line.clone())),
                    tail: Box::new(result),
                };
            }

            Ok(result)
        }
        (Value::Int(_), _) => Err(VmError::TypeMismatch {
            expected: "int",
            got: end.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: start.type_name(),
        }),
    }
}

/// TerminalInfo.getWindowTitle : unit -> string
/// Returns the window title, or empty string if no provider is registered
pub fn get_window_title(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let title = with_provider(|provider| provider.get_window_title())
                .unwrap_or_else(|| String::new());

            Ok(Value::Str(title))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalInfo.getTabTitle : unit -> string
/// Returns the tab title, or empty string if no provider is registered
pub fn get_tab_title(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let title =
                with_provider(|provider| provider.get_tab_title()).unwrap_or_else(|| String::new());

            Ok(Value::Str(title))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalInfo.getTerminalSize : unit -> (int * int)
/// Returns the terminal size as a tuple (columns, rows)
/// Returns (0, 0) if no provider is registered
pub fn get_terminal_size(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let (cols, rows) =
                with_provider(|provider| provider.get_terminal_size()).unwrap_or((0, 0));

            Ok(Value::Tuple(vec![Value::Int(cols), Value::Int(rows)]))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Test mutex to ensure tests run serially and don't interfere with each other
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    // Mock provider for testing
    struct MockTerminalProvider {
        foreground_process: Option<ProcessInfo>,
        cwd: Option<String>,
        lines: Vec<String>,
        window_title: String,
        tab_title: String,
        terminal_size: (i64, i64),
    }

    impl TerminalInfoProvider for MockTerminalProvider {
        fn get_foreground_process(&self) -> Option<ProcessInfo> {
            self.foreground_process.clone()
        }

        fn get_current_working_dir(&self) -> Option<String> {
            self.cwd.clone()
        }

        fn get_line(&self, line_number: i64) -> Option<String> {
            if line_number >= 0 && (line_number as usize) < self.lines.len() {
                Some(self.lines[line_number as usize].clone())
            } else {
                None
            }
        }

        fn get_lines(&self, start: i64, end: i64) -> Vec<String> {
            let start_idx = start.max(0) as usize;
            let end_idx = (end.max(0) as usize).min(self.lines.len());

            if start_idx >= end_idx {
                vec![]
            } else {
                self.lines[start_idx..end_idx].to_vec()
            }
        }

        fn get_window_title(&self) -> String {
            self.window_title.clone()
        }

        fn get_tab_title(&self) -> String {
            self.tab_title.clone()
        }

        fn get_terminal_size(&self) -> (i64, i64) {
            self.terminal_size
        }
    }

    #[test]
    fn test_get_foreground_process_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_foreground_process(&Value::Unit).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected Option::None variant"),
        }
    }

    #[test]
    fn test_get_foreground_process_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: Some(ProcessInfo {
                name: "bash".to_string(),
                pid: 1234,
                command_line: Some("/bin/bash".to_string()),
            }),
            cwd: None,
            lines: vec![],
            window_title: String::new(),
            tab_title: String::new(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_foreground_process(&Value::Unit).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);

                // Check the ProcessInfo record
                if let Value::Record(record) = &fields[0] {
                    let r = record.lock().unwrap();
                    assert!(matches!(r.get("name"), Some(Value::Str(s)) if s == "bash"));
                    assert!(matches!(r.get("pid"), Some(Value::Int(1234))));
                } else {
                    panic!("Expected Record in Some variant");
                }
            }
            _ => panic!("Expected Option::Some variant"),
        }

        unregister_provider();
    }

    #[test]
    fn test_get_foreground_process_type_error() {
        let result = get_foreground_process(&Value::Int(42));
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "unit",
                ..
            })
        ));
    }

    #[test]
    fn test_get_current_working_dir_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_current_working_dir(&Value::Unit).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected Option::None variant"),
        }
    }

    #[test]
    fn test_get_current_working_dir_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: Some("/home/user".to_string()),
            lines: vec![],
            window_title: String::new(),
            tab_title: String::new(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_current_working_dir(&Value::Unit).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                assert!(matches!(&fields[0], Value::Str(s) if s == "/home/user"));
            }
            _ => panic!("Expected Option::Some variant"),
        }

        unregister_provider();
    }

    #[test]
    fn test_get_line_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_line(&Value::Int(0)).unwrap();

        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected Option::None variant"),
        }
    }

    #[test]
    fn test_get_line_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: None,
            lines: vec![
                "line 0".to_string(),
                "line 1".to_string(),
                "line 2".to_string(),
            ],
            window_title: String::new(),
            tab_title: String::new(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_line(&Value::Int(1)).unwrap();

        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                assert!(matches!(&fields[0], Value::Str(s) if s == "line 1"));
            }
            _ => panic!("Expected Option::Some variant"),
        }

        // Test out of bounds
        let result = get_line(&Value::Int(10)).unwrap();
        match result {
            Value::Variant { variant_name, .. } => {
                assert_eq!(variant_name, "None");
            }
            _ => panic!("Expected Option::None variant"),
        }

        unregister_provider();
    }

    #[test]
    fn test_get_line_type_error() {
        let result = get_line(&Value::Str("not an int".to_string()));
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_get_lines_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_lines(&Value::Int(0), &Value::Int(2)).unwrap();

        // Should return empty list
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_get_lines_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: None,
            lines: vec![
                "line 0".to_string(),
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
            window_title: String::new(),
            tab_title: String::new(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_lines(&Value::Int(1), &Value::Int(3)).unwrap();

        // Should return list containing "line 1" and "line 2"
        let mut count = 0;
        let mut current = result;
        loop {
            match current {
                Value::Nil => break,
                Value::Cons { head, tail } => {
                    count += 1;
                    current = *tail;
                    // Verify it's a string
                    assert!(matches!(*head, Value::Str(_)));
                }
                _ => panic!("Expected list structure"),
            }
        }
        assert_eq!(count, 2);

        unregister_provider();
    }

    #[test]
    fn test_get_lines_type_error() {
        let result = get_lines(&Value::Str("not an int".to_string()), &Value::Int(2));
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "int",
                ..
            })
        ));

        let result = get_lines(&Value::Int(0), &Value::Str("not an int".to_string()));
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "int",
                ..
            })
        ));
    }

    #[test]
    fn test_get_window_title_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_window_title(&Value::Unit).unwrap();

        assert!(matches!(result, Value::Str(s) if s.is_empty()));
    }

    #[test]
    fn test_get_window_title_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: None,
            lines: vec![],
            window_title: "Terminal Window".to_string(),
            tab_title: String::new(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_window_title(&Value::Unit).unwrap();

        assert!(matches!(result, Value::Str(s) if s == "Terminal Window"));

        unregister_provider();
    }

    #[test]
    fn test_get_tab_title_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_tab_title(&Value::Unit).unwrap();

        assert!(matches!(result, Value::Str(s) if s.is_empty()));
    }

    #[test]
    fn test_get_tab_title_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: None,
            lines: vec![],
            window_title: String::new(),
            tab_title: "Tab 1".to_string(),
            terminal_size: (80, 24),
        });

        register_provider(provider);

        let result = get_tab_title(&Value::Unit).unwrap();

        assert!(matches!(result, Value::Str(s) if s == "Tab 1"));

        unregister_provider();
    }

    #[test]
    fn test_get_terminal_size_no_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        unregister_provider();

        let result = get_terminal_size(&Value::Unit).unwrap();

        match result {
            Value::Tuple(values) => {
                assert_eq!(values.len(), 2);
                assert!(matches!(values[0], Value::Int(0)));
                assert!(matches!(values[1], Value::Int(0)));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_get_terminal_size_with_provider() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let provider = Box::new(MockTerminalProvider {
            foreground_process: None,
            cwd: None,
            lines: vec![],
            window_title: String::new(),
            tab_title: String::new(),
            terminal_size: (120, 40),
        });

        register_provider(provider);

        let result = get_terminal_size(&Value::Unit).unwrap();

        match result {
            Value::Tuple(values) => {
                assert_eq!(values.len(), 2);
                assert!(matches!(values[0], Value::Int(120)));
                assert!(matches!(values[1], Value::Int(40)));
            }
            _ => panic!("Expected tuple"),
        }

        unregister_provider();
    }

    #[test]
    fn test_get_terminal_size_type_error() {
        let result = get_terminal_size(&Value::Int(42));
        assert!(matches!(
            result,
            Err(VmError::TypeMismatch {
                expected: "unit",
                ..
            })
        ));
    }

    #[test]
    fn test_process_info_to_value() {
        let process_info = ProcessInfo {
            name: "vim".to_string(),
            pid: 9999,
            command_line: Some("/usr/bin/vim file.txt".to_string()),
        };

        let value = process_info.to_value();

        match value {
            Value::Record(record) => {
                let r = record.lock().unwrap();
                assert!(matches!(r.get("name"), Some(Value::Str(s)) if s == "vim"));
                assert!(matches!(r.get("pid"), Some(Value::Int(9999))));

                // Check commandLine is Some variant
                if let Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) = r.get("commandLine")
                {
                    assert_eq!(variant_name, "Some");
                    assert_eq!(fields.len(), 1);
                    assert!(matches!(&fields[0], Value::Str(s) if s == "/usr/bin/vim file.txt"));
                } else {
                    panic!("Expected Option variant for commandLine");
                }
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_process_info_to_value_no_command_line() {
        let process_info = ProcessInfo {
            name: "vim".to_string(),
            pid: 9999,
            command_line: None,
        };

        let value = process_info.to_value();

        match value {
            Value::Record(record) => {
                let r = record.lock().unwrap();

                // Check commandLine is None variant
                if let Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) = r.get("commandLine")
                {
                    assert_eq!(variant_name, "None");
                    assert_eq!(fields.len(), 0);
                } else {
                    panic!("Expected Option variant for commandLine");
                }
            }
            _ => panic!("Expected Record"),
        }
    }
}
