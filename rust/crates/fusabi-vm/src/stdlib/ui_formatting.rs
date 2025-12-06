// Fusabi Standard Library - UI Formatting Module
// Provides callback-based APIs for status bar and UI element formatting

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global storage for formatter callbacks
/// Maps handler ID to Fusabi closures
static FORMATTERS: Mutex<Option<Formatters>> = Mutex::new(None);

/// Thread-local counter for generating unique handler IDs
static NEXT_HANDLER_ID: Mutex<i64> = Mutex::new(1);

/// Storage structure for different formatter types
struct Formatters {
    tab_formatters: HashMap<i64, Value>,
    status_left_formatters: HashMap<i64, Value>,
    status_right_formatters: HashMap<i64, Value>,
}

impl Formatters {
    fn new() -> Self {
        Self {
            tab_formatters: HashMap::new(),
            status_left_formatters: HashMap::new(),
            status_right_formatters: HashMap::new(),
        }
    }
}

/// Initialize the formatter storage if not already initialized
fn ensure_formatters_initialized() {
    let mut formatters = FORMATTERS.lock().unwrap();
    if formatters.is_none() {
        *formatters = Some(Formatters::new());
    }
}

/// Generate a new unique handler ID
fn next_handler_id() -> i64 {
    let mut id = NEXT_HANDLER_ID.lock().unwrap();
    let current = *id;
    *id += 1;
    current
}

/// UIFormatting.onFormatTab : (TabInfo -> StatusSegment list) -> int
/// Registers a formatter callback for tab rendering
/// Returns a handler ID that can be used to remove the formatter
pub fn on_format_tab(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "UIFormatting.onFormatTab expects 1 argument, got {}",
            args.len()
        )));
    }

    let formatter = &args[0];

    // Verify it's a callable value (Closure or NativeFn)
    if !matches!(formatter, Value::Closure(_) | Value::NativeFn { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "function",
            got: formatter.type_name(),
        });
    }

    ensure_formatters_initialized();
    let handler_id = next_handler_id();

    let mut formatters = FORMATTERS.lock().unwrap();
    if let Some(ref mut fmt) = *formatters {
        fmt.tab_formatters.insert(handler_id, formatter.clone());
    }

    Ok(Value::Int(handler_id))
}

/// UIFormatting.onFormatStatusLeft : (StatusInfo -> StatusSegment list) -> int
/// Registers a formatter callback for left status area
/// Returns a handler ID that can be used to remove the formatter
pub fn on_format_status_left(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "UIFormatting.onFormatStatusLeft expects 1 argument, got {}",
            args.len()
        )));
    }

    let formatter = &args[0];

    // Verify it's a callable value
    if !matches!(formatter, Value::Closure(_) | Value::NativeFn { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "function",
            got: formatter.type_name(),
        });
    }

    ensure_formatters_initialized();
    let handler_id = next_handler_id();

    let mut formatters = FORMATTERS.lock().unwrap();
    if let Some(ref mut fmt) = *formatters {
        fmt.status_left_formatters
            .insert(handler_id, formatter.clone());
    }

    Ok(Value::Int(handler_id))
}

/// UIFormatting.onFormatStatusRight : (StatusInfo -> StatusSegment list) -> int
/// Registers a formatter callback for right status area
/// Returns a handler ID that can be used to remove the formatter
pub fn on_format_status_right(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "UIFormatting.onFormatStatusRight expects 1 argument, got {}",
            args.len()
        )));
    }

    let formatter = &args[0];

    // Verify it's a callable value
    if !matches!(formatter, Value::Closure(_) | Value::NativeFn { .. }) {
        return Err(VmError::TypeMismatch {
            expected: "function",
            got: formatter.type_name(),
        });
    }

    ensure_formatters_initialized();
    let handler_id = next_handler_id();

    let mut formatters = FORMATTERS.lock().unwrap();
    if let Some(ref mut fmt) = *formatters {
        fmt.status_right_formatters
            .insert(handler_id, formatter.clone());
    }

    Ok(Value::Int(handler_id))
}

/// UIFormatting.removeFormatter : int -> bool
/// Removes a formatter by its handler ID
/// Returns true if a formatter was removed, false if not found
pub fn remove_formatter(args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "UIFormatting.removeFormatter expects 1 argument, got {}",
            args.len()
        )));
    }

    let handler_id = match &args[0] {
        Value::Int(id) => *id,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: args[0].type_name(),
            })
        }
    };

    ensure_formatters_initialized();
    let mut formatters = FORMATTERS.lock().unwrap();

    if let Some(ref mut fmt) = *formatters {
        let removed = fmt.tab_formatters.remove(&handler_id).is_some()
            || fmt.status_left_formatters.remove(&handler_id).is_some()
            || fmt.status_right_formatters.remove(&handler_id).is_some();

        Ok(Value::Bool(removed))
    } else {
        Ok(Value::Bool(false))
    }
}

/// UIFormatting.clearFormatters : unit -> unit
/// Removes all registered formatters
pub fn clear_formatters(args: &[Value]) -> Result<Value, VmError> {
    if !args.is_empty() {
        return Err(VmError::Runtime(format!(
            "UIFormatting.clearFormatters expects 0 arguments, got {}",
            args.len()
        )));
    }

    ensure_formatters_initialized();
    let mut formatters = FORMATTERS.lock().unwrap();

    if let Some(ref mut fmt) = *formatters {
        fmt.tab_formatters.clear();
        fmt.status_left_formatters.clear();
        fmt.status_right_formatters.clear();
    }

    Ok(Value::Unit)
}

// ============================================================================
// Host-side API for invoking formatters
// ============================================================================

/// Create a TabInfo record from individual components
/// TabInfo: { index: int, title: string, active: bool, hasActivity: bool }
pub fn create_tab_info(index: i64, title: String, active: bool, has_activity: bool) -> Value {
    let mut fields = HashMap::new();
    fields.insert("index".to_string(), Value::Int(index));
    fields.insert("title".to_string(), Value::Str(title));
    fields.insert("active".to_string(), Value::Bool(active));
    fields.insert("hasActivity".to_string(), Value::Bool(has_activity));
    Value::Record(Arc::new(Mutex::new(fields)))
}

/// Create a StatusInfo record from individual components
/// StatusInfo: { currentTab: int, totalTabs: int, time: string }
pub fn create_status_info(current_tab: i64, total_tabs: i64, time: String) -> Value {
    let mut fields = HashMap::new();
    fields.insert("currentTab".to_string(), Value::Int(current_tab));
    fields.insert("totalTabs".to_string(), Value::Int(total_tabs));
    fields.insert("time".to_string(), Value::Str(time));
    Value::Record(Arc::new(Mutex::new(fields)))
}

/// Convert a StatusSegment Value to Rust-friendly components
/// StatusSegment: { text: string, fgColor: string option, bgColor: string option, bold: bool }
pub fn extract_status_segment(
    segment: &Value,
) -> Result<(String, Option<String>, Option<String>, bool), VmError> {
    match segment {
        Value::Record(fields) => {
            let fields = fields.lock().unwrap();

            let text = match fields.get("text") {
                Some(Value::Str(s)) => s.clone(),
                _ => {
                    return Err(VmError::Runtime(
                        "StatusSegment.text must be a string".to_string(),
                    ))
                }
            };

            let fg_color = match fields.get("fgColor") {
                Some(Value::Variant {
                    variant_name,
                    fields: f,
                    ..
                }) if variant_name == "Some" && f.len() == 1 => {
                    if let Value::Str(s) = &f[0] {
                        Some(s.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let bg_color = match fields.get("bgColor") {
                Some(Value::Variant {
                    variant_name,
                    fields: f,
                    ..
                }) if variant_name == "Some" && f.len() == 1 => {
                    if let Value::Str(s) = &f[0] {
                        Some(s.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let bold = match fields.get("bold") {
                Some(Value::Bool(b)) => *b,
                _ => false,
            };

            Ok((text, fg_color, bg_color, bold))
        }
        _ => Err(VmError::Runtime(
            "StatusSegment must be a record".to_string(),
        )),
    }
}

/// Invoke all registered tab formatters and collect results
/// Returns a list of segment lists, one for each formatter
pub fn invoke_tab_formatters(vm: &mut Vm, tab_info: Value) -> Result<Vec<Vec<Value>>, VmError> {
    ensure_formatters_initialized();
    let formatters = FORMATTERS.lock().unwrap();

    let mut results = Vec::new();

    if let Some(ref fmt) = *formatters {
        for (_id, formatter) in &fmt.tab_formatters {
            let result = vm.call_value(formatter.clone(), &[tab_info.clone()])?;

            // Convert result (list) to vector
            let segments = match &result {
                Value::Nil => vec![],
                Value::Cons { .. } => result.list_to_vec().ok_or(VmError::Runtime(
                    "Malformed list returned from formatter".into(),
                ))?,
                _ => {
                    return Err(VmError::Runtime(
                        "Tab formatter must return a list of StatusSegments".to_string(),
                    ))
                }
            };

            results.push(segments);
        }
    }

    Ok(results)
}

/// Invoke all registered status left formatters and collect results
pub fn invoke_status_left_formatters(
    vm: &mut Vm,
    status_info: Value,
) -> Result<Vec<Vec<Value>>, VmError> {
    ensure_formatters_initialized();
    let formatters = FORMATTERS.lock().unwrap();

    let mut results = Vec::new();

    if let Some(ref fmt) = *formatters {
        for (_id, formatter) in &fmt.status_left_formatters {
            let result = vm.call_value(formatter.clone(), &[status_info.clone()])?;

            // Convert result (list) to vector
            let segments = match &result {
                Value::Nil => vec![],
                Value::Cons { .. } => result.list_to_vec().ok_or(VmError::Runtime(
                    "Malformed list returned from formatter".into(),
                ))?,
                _ => {
                    return Err(VmError::Runtime(
                        "Status left formatter must return a list of StatusSegments".to_string(),
                    ))
                }
            };

            results.push(segments);
        }
    }

    Ok(results)
}

/// Invoke all registered status right formatters and collect results
pub fn invoke_status_right_formatters(
    vm: &mut Vm,
    status_info: Value,
) -> Result<Vec<Vec<Value>>, VmError> {
    ensure_formatters_initialized();
    let formatters = FORMATTERS.lock().unwrap();

    let mut results = Vec::new();

    if let Some(ref fmt) = *formatters {
        for (_id, formatter) in &fmt.status_right_formatters {
            let result = vm.call_value(formatter.clone(), &[status_info.clone()])?;

            // Convert result (list) to vector
            let segments = match &result {
                Value::Nil => vec![],
                Value::Cons { .. } => result.list_to_vec().ok_or(VmError::Runtime(
                    "Malformed list returned from formatter".into(),
                ))?,
                _ => {
                    return Err(VmError::Runtime(
                        "Status right formatter must return a list of StatusSegments".to_string(),
                    ))
                }
            };

            results.push(segments);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Chunk;
    use crate::closure::Closure;
    use crate::vm::Vm;

    #[test]
    fn test_on_format_tab_registration() {
        let mut vm = Vm::new();

        // Create a mock closure
        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        let result = on_format_tab(&mut vm, &[closure.clone()]);
        assert!(result.is_ok());

        // Verify we got an integer handler ID
        match result.unwrap() {
            Value::Int(id) => assert!(id > 0),
            _ => panic!("Expected Int handler ID"),
        }
    }

    #[test]
    fn test_on_format_status_left_registration() {
        let mut vm = Vm::new();

        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        let result = on_format_status_left(&mut vm, &[closure]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Int(id) => assert!(id > 0),
            _ => panic!("Expected Int handler ID"),
        }
    }

    #[test]
    fn test_on_format_status_right_registration() {
        let mut vm = Vm::new();

        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        let result = on_format_status_right(&mut vm, &[closure]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Int(id) => assert!(id > 0),
            _ => panic!("Expected Int handler ID"),
        }
    }

    #[test]
    fn test_remove_formatter() {
        let mut vm = Vm::new();

        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        // Register a formatter
        let handler_id = match on_format_tab(&mut vm, &[closure]).unwrap() {
            Value::Int(id) => id,
            _ => panic!("Expected Int handler ID"),
        };

        // Remove it
        let result = remove_formatter(&[Value::Int(handler_id)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));

        // Try to remove again (should return false)
        let result = remove_formatter(&[Value::Int(handler_id)]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_clear_formatters() {
        let mut vm = Vm::new();

        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        // Register multiple formatters
        on_format_tab(&mut vm, &[closure.clone()]).unwrap();
        on_format_status_left(&mut vm, &[closure.clone()]).unwrap();
        on_format_status_right(&mut vm, &[closure]).unwrap();

        // Clear all
        let result = clear_formatters(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);

        // Verify all are cleared (removing non-existent should return false)
        let result = remove_formatter(&[Value::Int(1)]);
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_create_tab_info() {
        let tab_info = create_tab_info(0, "Test Tab".to_string(), true, false);

        match tab_info {
            Value::Record(fields) => {
                let fields = fields.lock().unwrap();
                assert_eq!(fields.get("index"), Some(&Value::Int(0)));
                assert_eq!(
                    fields.get("title"),
                    Some(&Value::Str("Test Tab".to_string()))
                );
                assert_eq!(fields.get("active"), Some(&Value::Bool(true)));
                assert_eq!(fields.get("hasActivity"), Some(&Value::Bool(false)));
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_create_status_info() {
        let status_info = create_status_info(1, 5, "12:34:56".to_string());

        match status_info {
            Value::Record(fields) => {
                let fields = fields.lock().unwrap();
                assert_eq!(fields.get("currentTab"), Some(&Value::Int(1)));
                assert_eq!(fields.get("totalTabs"), Some(&Value::Int(5)));
                assert_eq!(
                    fields.get("time"),
                    Some(&Value::Str("12:34:56".to_string()))
                );
            }
            _ => panic!("Expected Record"),
        }
    }

    #[test]
    fn test_extract_status_segment() {
        let mut fields = HashMap::new();
        fields.insert("text".to_string(), Value::Str("Hello".to_string()));
        fields.insert(
            "fgColor".to_string(),
            Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                fields: vec![Value::Str("red".to_string())],
            },
        );
        fields.insert(
            "bgColor".to_string(),
            Value::Variant {
                type_name: "Option".to_string(),
                variant_name: "None".to_string(),
                fields: vec![],
            },
        );
        fields.insert("bold".to_string(), Value::Bool(true));

        let segment = Value::Record(Arc::new(Mutex::new(fields)));
        let result = extract_status_segment(&segment);

        assert!(result.is_ok());
        let (text, fg_color, bg_color, bold) = result.unwrap();
        assert_eq!(text, "Hello");
        assert_eq!(fg_color, Some("red".to_string()));
        assert_eq!(bg_color, None);
        assert_eq!(bold, true);
    }

    #[test]
    fn test_on_format_tab_invalid_args() {
        let mut vm = Vm::new();

        // Test with non-function
        let result = on_format_tab(&mut vm, &[Value::Int(42)]);
        assert!(result.is_err());

        // Test with wrong number of args
        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));
        let result = on_format_tab(&mut vm, &[closure.clone(), closure]);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_formatter_invalid_args() {
        // Test with non-int
        let result = remove_formatter(&[Value::Str("hello".to_string())]);
        assert!(result.is_err());

        // Test with wrong number of args
        let result = remove_formatter(&[Value::Int(1), Value::Int(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_formatters_invalid_args() {
        // Test with arguments when none expected
        let result = clear_formatters(&[Value::Unit]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_handler_ids() {
        let mut vm = Vm::new();
        let closure = Value::Closure(Arc::new(Closure::with_arity(Chunk::new(), 1)));

        let id1 = match on_format_tab(&mut vm, &[closure.clone()]).unwrap() {
            Value::Int(id) => id,
            _ => panic!("Expected Int"),
        };

        let id2 = match on_format_tab(&mut vm, &[closure.clone()]).unwrap() {
            Value::Int(id) => id,
            _ => panic!("Expected Int"),
        };

        let id3 = match on_format_status_left(&mut vm, &[closure]).unwrap() {
            Value::Int(id) => id,
            _ => panic!("Expected Int"),
        };

        // All IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
