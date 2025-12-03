// Fusabi Standard Library - Terminal Control Functions
// Provides APIs for programmatic pane/window control in terminal emulators

use crate::value::Value;
use crate::vm::VmError;
use std::sync::{Mutex, OnceLock};

/// Trait for providing terminal control capabilities
/// Host applications should implement this trait and register it via `register_provider`
pub trait TerminalControlProvider: Send + Sync {
    /// Send text to the active pane
    fn send_text(&self, text: &str);

    /// Send key sequences to the active pane
    fn send_keys(&self, keys: &[String]);

    /// Split the active pane horizontally, returning the new pane ID if successful
    fn split_horizontal(&self) -> Option<i64>;

    /// Split the active pane vertically, returning the new pane ID if successful
    fn split_vertical(&self) -> Option<i64>;

    /// Close a pane by ID, returning true if successful
    fn close_pane(&self, pane_id: i64) -> bool;

    /// Focus a pane by ID, returning true if successful
    fn focus_pane(&self, pane_id: i64) -> bool;

    /// Create a new tab, returning the tab ID if successful
    fn create_tab(&self) -> Option<i64>;

    /// Close a tab by ID, returning true if successful
    fn close_tab(&self, tab_id: i64) -> bool;

    /// Set the title of a tab, returning true if successful
    fn set_tab_title(&self, tab_id: i64, title: &str) -> bool;

    /// Show a toast notification
    fn show_toast(&self, message: &str);
}

/// Global registry for the terminal control provider
static PROVIDER: OnceLock<Mutex<Option<Box<dyn TerminalControlProvider>>>> = OnceLock::new();

/// Register a terminal control provider
/// This should be called by host applications to enable terminal control functionality
pub fn register_provider(provider: Box<dyn TerminalControlProvider>) {
    let mutex = PROVIDER.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().unwrap();
    *guard = Some(provider);
}

/// Unregister the terminal control provider (useful for testing)
pub fn unregister_provider() {
    if let Some(mutex) = PROVIDER.get() {
        let mut guard = mutex.lock().unwrap();
        *guard = None;
    }
}

/// TerminalControl.sendText : string -> unit
/// Send text to the active pane
pub fn send_text(text: &Value) -> Result<Value, VmError> {
    match text {
        Value::Str(s) => {
            if let Some(mutex) = PROVIDER.get() {
                if let Some(provider) = mutex.lock().unwrap().as_ref() {
                    provider.send_text(s);
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: text.type_name(),
        }),
    }
}

/// TerminalControl.sendKeys : string list -> unit
/// Send key sequences to the active pane
pub fn send_keys(keys: &Value) -> Result<Value, VmError> {
    let key_strings = list_to_string_vec(keys)?;

    if let Some(mutex) = PROVIDER.get() {
        if let Some(provider) = mutex.lock().unwrap().as_ref() {
            provider.send_keys(&key_strings);
        }
    }

    Ok(Value::Unit)
}

/// TerminalControl.splitHorizontal : unit -> int option
/// Split the active pane horizontally, returning the new pane ID
pub fn split_horizontal(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let pane_id = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .and_then(|p| p.split_horizontal())
            } else {
                None
            };

            Ok(option_from_i64(pane_id))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalControl.splitVertical : unit -> int option
/// Split the active pane vertically, returning the new pane ID
pub fn split_vertical(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let pane_id = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .and_then(|p| p.split_vertical())
            } else {
                None
            };

            Ok(option_from_i64(pane_id))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalControl.closePane : int -> bool
/// Close a pane by ID
pub fn close_pane(pane_id: &Value) -> Result<Value, VmError> {
    match pane_id {
        Value::Int(id) => {
            let success = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map_or(false, |p| p.close_pane(*id))
            } else {
                false
            };

            Ok(Value::Bool(success))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: pane_id.type_name(),
        }),
    }
}

/// TerminalControl.focusPane : int -> bool
/// Focus a pane by ID
pub fn focus_pane(pane_id: &Value) -> Result<Value, VmError> {
    match pane_id {
        Value::Int(id) => {
            let success = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map_or(false, |p| p.focus_pane(*id))
            } else {
                false
            };

            Ok(Value::Bool(success))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: pane_id.type_name(),
        }),
    }
}

/// TerminalControl.createTab : unit -> int option
/// Create a new tab, returning the tab ID
pub fn create_tab(unit: &Value) -> Result<Value, VmError> {
    match unit {
        Value::Unit => {
            let tab_id = if let Some(mutex) = PROVIDER.get() {
                mutex.lock().unwrap().as_ref().and_then(|p| p.create_tab())
            } else {
                None
            };

            Ok(option_from_i64(tab_id))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "unit",
            got: unit.type_name(),
        }),
    }
}

/// TerminalControl.closeTab : int -> bool
/// Close a tab by ID
pub fn close_tab(tab_id: &Value) -> Result<Value, VmError> {
    match tab_id {
        Value::Int(id) => {
            let success = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map_or(false, |p| p.close_tab(*id))
            } else {
                false
            };

            Ok(Value::Bool(success))
        }
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: tab_id.type_name(),
        }),
    }
}

/// TerminalControl.setTabTitle : int -> string -> bool
/// Set the title of a tab
pub fn set_tab_title(tab_id: &Value, title: &Value) -> Result<Value, VmError> {
    match (tab_id, title) {
        (Value::Int(id), Value::Str(t)) => {
            let success = if let Some(mutex) = PROVIDER.get() {
                mutex
                    .lock()
                    .unwrap()
                    .as_ref()
                    .map_or(false, |p| p.set_tab_title(*id, t))
            } else {
                false
            };

            Ok(Value::Bool(success))
        }
        (Value::Int(_), _) => Err(VmError::TypeMismatch {
            expected: "string",
            got: title.type_name(),
        }),
        _ => Err(VmError::TypeMismatch {
            expected: "int",
            got: tab_id.type_name(),
        }),
    }
}

/// TerminalControl.showToast : string -> unit
/// Show a toast notification
pub fn show_toast(message: &Value) -> Result<Value, VmError> {
    match message {
        Value::Str(s) => {
            if let Some(mutex) = PROVIDER.get() {
                if let Some(provider) = mutex.lock().unwrap().as_ref() {
                    provider.show_toast(s);
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(VmError::TypeMismatch {
            expected: "string",
            got: message.type_name(),
        }),
    }
}

// Helper functions

/// Convert a Fusabi list of strings to a Vec<String>
fn list_to_string_vec(list: &Value) -> Result<Vec<String>, VmError> {
    let mut result = Vec::new();
    let mut current = list;

    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                match head.as_ref() {
                    Value::Str(s) => result.push(s.clone()),
                    _ => {
                        return Err(VmError::TypeMismatch {
                            expected: "string list",
                            got: "list with non-string element",
                        })
                    }
                }
                current = tail.as_ref();
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

/// Convert an Option<i64> to a Fusabi Option<int> value
fn option_from_i64(opt: Option<i64>) -> Value {
    match opt {
        Some(id) => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Int(id)],
        },
        None => Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockProvider {
        send_text_calls: Mutex<Vec<String>>,
        send_keys_calls: Mutex<Vec<Vec<String>>>,
        show_toast_calls: Mutex<Vec<String>>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                send_text_calls: Mutex::new(Vec::new()),
                send_keys_calls: Mutex::new(Vec::new()),
                show_toast_calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl TerminalControlProvider for MockProvider {
        fn send_text(&self, text: &str) {
            self.send_text_calls.lock().unwrap().push(text.to_string());
        }

        fn send_keys(&self, keys: &[String]) {
            self.send_keys_calls.lock().unwrap().push(keys.to_vec());
        }

        fn split_horizontal(&self) -> Option<i64> {
            Some(42)
        }

        fn split_vertical(&self) -> Option<i64> {
            Some(43)
        }

        fn close_pane(&self, _pane_id: i64) -> bool {
            true
        }

        fn focus_pane(&self, _pane_id: i64) -> bool {
            true
        }

        fn create_tab(&self) -> Option<i64> {
            Some(100)
        }

        fn close_tab(&self, _tab_id: i64) -> bool {
            true
        }

        fn set_tab_title(&self, _tab_id: i64, _title: &str) -> bool {
            true
        }

        fn show_toast(&self, message: &str) {
            self.show_toast_calls
                .lock()
                .unwrap()
                .push(message.to_string());
        }
    }

    #[test]
    fn test_send_text_no_provider() {
        unregister_provider();
        let result = send_text(&Value::Str("hello".to_string()));
        assert_eq!(result, Ok(Value::Unit));
    }

    #[test]
    fn test_send_text_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = send_text(&Value::Str("test message".to_string()));
        assert_eq!(result, Ok(Value::Unit));

        unregister_provider();
    }

    #[test]
    fn test_send_text_type_error() {
        unregister_provider();
        let result = send_text(&Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_send_keys_no_provider() {
        unregister_provider();
        let list = Value::Cons {
            head: Box::new(Value::Str("ctrl-c".to_string())),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Str("ctrl-v".to_string())),
                tail: Box::new(Value::Nil),
            }),
        };

        let result = send_keys(&list);
        assert_eq!(result, Ok(Value::Unit));
    }

    #[test]
    fn test_send_keys_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let list = Value::Cons {
            head: Box::new(Value::Str("Enter".to_string())),
            tail: Box::new(Value::Nil),
        };

        let result = send_keys(&list);
        assert_eq!(result, Ok(Value::Unit));

        unregister_provider();
    }

    #[test]
    fn test_send_keys_type_error_non_list() {
        unregister_provider();
        let result = send_keys(&Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_send_keys_type_error_non_string_element() {
        unregister_provider();
        let list = Value::Cons {
            head: Box::new(Value::Int(42)),
            tail: Box::new(Value::Nil),
        };

        let result = send_keys(&list);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_split_horizontal_no_provider() {
        unregister_provider();
        let result = split_horizontal(&Value::Unit);
        assert!(
            matches!(result, Ok(Value::Variant { variant_name, .. }) if variant_name == "None")
        );
    }

    #[test]
    fn test_split_horizontal_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = split_horizontal(&Value::Unit);
        match result {
            Ok(Value::Variant {
                variant_name,
                fields,
                ..
            }) if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(42));
            }
            _ => panic!("Expected Some(42)"),
        }

        unregister_provider();
    }

    #[test]
    fn test_split_horizontal_type_error() {
        unregister_provider();
        let result = split_horizontal(&Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_split_vertical_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = split_vertical(&Value::Unit);
        match result {
            Ok(Value::Variant {
                variant_name,
                fields,
                ..
            }) if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(43));
            }
            _ => panic!("Expected Some(43)"),
        }

        unregister_provider();
    }

    #[test]
    fn test_close_pane_no_provider() {
        unregister_provider();
        let result = close_pane(&Value::Int(1));
        assert_eq!(result, Ok(Value::Bool(false)));
    }

    #[test]
    fn test_close_pane_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = close_pane(&Value::Int(1));
        assert_eq!(result, Ok(Value::Bool(true)));

        unregister_provider();
    }

    #[test]
    fn test_close_pane_type_error() {
        unregister_provider();
        let result = close_pane(&Value::Str("not an int".to_string()));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_focus_pane_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = focus_pane(&Value::Int(2));
        assert_eq!(result, Ok(Value::Bool(true)));

        unregister_provider();
    }

    #[test]
    fn test_create_tab_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = create_tab(&Value::Unit);
        match result {
            Ok(Value::Variant {
                variant_name,
                fields,
                ..
            }) if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(100));
            }
            _ => panic!("Expected Some(100)"),
        }

        unregister_provider();
    }

    #[test]
    fn test_close_tab_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = close_tab(&Value::Int(10));
        assert_eq!(result, Ok(Value::Bool(true)));

        unregister_provider();
    }

    #[test]
    fn test_set_tab_title_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = set_tab_title(&Value::Int(10), &Value::Str("New Tab".to_string()));
        assert_eq!(result, Ok(Value::Bool(true)));

        unregister_provider();
    }

    #[test]
    fn test_set_tab_title_type_error_invalid_id() {
        unregister_provider();
        let result = set_tab_title(
            &Value::Str("not an int".to_string()),
            &Value::Str("Title".to_string()),
        );
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_set_tab_title_type_error_invalid_title() {
        unregister_provider();
        let result = set_tab_title(&Value::Int(10), &Value::Int(42));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_show_toast_no_provider() {
        unregister_provider();
        let result = show_toast(&Value::Str("notification".to_string()));
        assert_eq!(result, Ok(Value::Unit));
    }

    #[test]
    fn test_show_toast_with_provider() {
        unregister_provider();
        let provider = MockProvider::new();
        register_provider(Box::new(provider));

        let result = show_toast(&Value::Str("test toast".to_string()));
        assert_eq!(result, Ok(Value::Unit));

        unregister_provider();
    }

    #[test]
    fn test_show_toast_type_error() {
        unregister_provider();
        let result = show_toast(&Value::Bool(true));
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }

    #[test]
    fn test_list_to_string_vec() {
        let list = Value::Cons {
            head: Box::new(Value::Str("a".to_string())),
            tail: Box::new(Value::Cons {
                head: Box::new(Value::Str("b".to_string())),
                tail: Box::new(Value::Cons {
                    head: Box::new(Value::Str("c".to_string())),
                    tail: Box::new(Value::Nil),
                }),
            }),
        };

        let result = list_to_string_vec(&list).unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_list_to_string_vec_empty() {
        let result = list_to_string_vec(&Value::Nil).unwrap();
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_option_from_i64_some() {
        let result = option_from_i64(Some(42));
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "Some" => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0], Value::Int(42));
            }
            _ => panic!("Expected Some(42)"),
        }
    }

    #[test]
    fn test_option_from_i64_none() {
        let result = option_from_i64(None);
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } if variant_name == "None" => {
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected None"),
        }
    }
}
