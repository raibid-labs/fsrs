// Fusabi Events Standard Library
// Provides event hook system for terminal lifecycle events and custom events

use crate::value::Value;
use crate::vm::{Vm, VmError};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event handler storage
/// Maps event names to lists of (handler_id, handler_value) pairs
type HandlerRegistry = HashMap<String, Vec<(u64, Value)>>;

lazy_static! {
    /// Global event handler registry
    static ref EVENT_HANDLERS: Arc<Mutex<HandlerRegistry>> = Arc::new(Mutex::new(HashMap::new()));

    /// Counter for generating unique handler IDs
    static ref HANDLER_ID_COUNTER: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

/// Generate a unique handler ID
fn next_handler_id() -> u64 {
    let mut counter = HANDLER_ID_COUNTER.lock().unwrap();
    *counter += 1;
    *counter
}

/// Events.on : string -> ('a -> unit) -> int
/// Register a handler for an event. Returns a handler ID that can be used to remove the handler.
/// The handler function receives the event data and should return unit.
///
/// Example:
///   let handlerId = Events.on "WindowFocusChanged" (fun gained -> printfn (sprintf "Focus: %b" gained))
pub fn events_on(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Events.on expects 2 arguments (event_name, handler), got {}",
            args.len()
        )));
    }

    let event_name = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: args[0].type_name(),
            })
        }
    };

    let handler = args[1].clone();

    // Verify handler is callable
    match &handler {
        Value::Closure(_) | Value::NativeFn { .. } => {}
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "function",
                got: handler.type_name(),
            })
        }
    }

    let handler_id = next_handler_id();

    {
        let mut registry = EVENT_HANDLERS.lock().unwrap();
        registry
            .entry(event_name)
            .or_insert_with(Vec::new)
            .push((handler_id, handler));
    }

    Ok(Value::Int(handler_id as i64))
}

/// Events.off : int -> bool
/// Remove a handler by its ID. Returns true if the handler was found and removed.
///
/// Example:
///   Events.off handlerId
pub fn events_off(handler_id: &Value) -> Result<Value, VmError> {
    let id = match handler_id {
        Value::Int(id) => *id as u64,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: handler_id.type_name(),
            })
        }
    };

    let mut registry = EVENT_HANDLERS.lock().unwrap();
    let mut found = false;

    for handlers in registry.values_mut() {
        let initial_len = handlers.len();
        handlers.retain(|(hid, _)| *hid != id);
        if handlers.len() < initial_len {
            found = true;
            break;
        }
    }

    Ok(Value::Bool(found))
}

/// Events.emit : string -> 'a -> unit
/// Emit an event with data, calling all registered handlers.
/// Handlers are called synchronously in registration order.
///
/// Example:
///   Events.emit "WindowFocusChanged" true
pub fn events_emit(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Events.emit expects 2 arguments (event_name, data), got {}",
            args.len()
        )));
    }

    let event_name = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: args[0].type_name(),
            })
        }
    };

    let event_data = args[1].clone();

    // Get handlers (clone to avoid holding lock during calls)
    let handlers: Vec<Value> = {
        let registry = EVENT_HANDLERS.lock().unwrap();
        registry
            .get(&event_name)
            .map(|handlers| handlers.iter().map(|(_, h)| h.clone()).collect())
            .unwrap_or_default()
    };

    // Call each handler with the event data
    for handler in handlers {
        vm.call_value(handler, &[event_data.clone()])?;
    }

    Ok(Value::Unit)
}

/// Events.emitAsync : string -> 'a -> Async<unit>
/// Emit an event asynchronously. Returns immediately while handlers run.
/// Note: In current implementation, this is synchronous but designed for future async support.
///
/// Example:
///   Events.emitAsync "Bell" ()
pub fn events_emit_async(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    // For now, emitAsync behaves the same as emit
    // Future implementation could spawn handlers in separate threads
    events_emit(vm, args)
}

/// Events.once : string -> ('a -> unit) -> int
/// Register a one-time handler that automatically removes itself after being called once.
/// Returns a handler ID.
///
/// Example:
///   Events.once "Startup" (fun _ -> printfn "App started!")
pub fn events_once(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(format!(
            "Events.once expects 2 arguments (event_name, handler), got {}",
            args.len()
        )));
    }

    let event_name = match &args[0] {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: args[0].type_name(),
            })
        }
    };

    let handler = args[1].clone();

    // Verify handler is callable
    match &handler {
        Value::Closure(_) | Value::NativeFn { .. } => {}
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "function",
                got: handler.type_name(),
            })
        }
    }

    let handler_id = next_handler_id();

    // Store with a special marker for one-time handlers
    // We'll use negative IDs internally to mark one-time handlers
    {
        let mut registry = EVENT_HANDLERS.lock().unwrap();
        registry
            .entry(format!("__once__{}", event_name))
            .or_insert_with(Vec::new)
            .push((handler_id, handler));
    }

    Ok(Value::Int(handler_id as i64))
}

/// Events.clear : string -> unit
/// Remove all handlers for a specific event.
///
/// Example:
///   Events.clear "WindowResized"
pub fn events_clear(event_name: &Value) -> Result<Value, VmError> {
    let name = match event_name {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: event_name.type_name(),
            })
        }
    };

    let mut registry = EVENT_HANDLERS.lock().unwrap();
    registry.remove(&name);
    registry.remove(&format!("__once__{}", name));

    Ok(Value::Unit)
}

/// Events.clearAll : unit -> unit
/// Remove all event handlers.
///
/// Example:
///   Events.clearAll ()
pub fn events_clear_all(_unit: &Value) -> Result<Value, VmError> {
    let mut registry = EVENT_HANDLERS.lock().unwrap();
    registry.clear();
    Ok(Value::Unit)
}

/// Events.handlers : string -> int
/// Get the count of handlers registered for an event.
///
/// Example:
///   let count = Events.handlers "WindowFocusChanged"
pub fn events_handlers(event_name: &Value) -> Result<Value, VmError> {
    let name = match event_name {
        Value::Str(s) => s.clone(),
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "string",
                got: event_name.type_name(),
            })
        }
    };

    let registry = EVENT_HANDLERS.lock().unwrap();
    let count = registry.get(&name).map(|h| h.len()).unwrap_or(0)
        + registry
            .get(&format!("__once__{}", name))
            .map(|h| h.len())
            .unwrap_or(0);

    Ok(Value::Int(count as i64))
}

/// Events.list : unit -> string list
/// Get a list of all event names that have handlers registered.
///
/// Example:
///   let events = Events.list ()
pub fn events_list(_unit: &Value) -> Result<Value, VmError> {
    let registry = EVENT_HANDLERS.lock().unwrap();
    let mut event_names: Vec<String> = registry
        .keys()
        .filter(|k| !k.starts_with("__once__"))
        .cloned()
        .collect();

    // Add unique once events
    for key in registry.keys() {
        if key.starts_with("__once__") {
            let name = key.strip_prefix("__once__").unwrap().to_string();
            if !event_names.contains(&name) {
                event_names.push(name);
            }
        }
    }

    event_names.sort();

    let values: Vec<Value> = event_names.into_iter().map(Value::Str).collect();
    Ok(Value::vec_to_cons(values))
}

/// Internal function to emit events and handle once handlers
pub fn emit_event_internal(vm: &mut Vm, event_name: &str, data: Value) -> Result<(), VmError> {
    // Get regular handlers
    let handlers: Vec<Value> = {
        let registry = EVENT_HANDLERS.lock().unwrap();
        registry
            .get(event_name)
            .map(|handlers| handlers.iter().map(|(_, h)| h.clone()).collect())
            .unwrap_or_default()
    };

    // Get and remove one-time handlers
    let once_handlers: Vec<Value> = {
        let mut registry = EVENT_HANDLERS.lock().unwrap();
        let once_key = format!("__once__{}", event_name);
        registry
            .remove(&once_key)
            .map(|handlers| handlers.into_iter().map(|(_, h)| h).collect())
            .unwrap_or_default()
    };

    // Call all handlers
    for handler in handlers.into_iter().chain(once_handlers.into_iter()) {
        vm.call_value(handler, &[data.clone()])?;
    }

    Ok(())
}

/// Reset the event system (for testing)
#[cfg(test)]
pub fn reset_events() {
    let mut registry = EVENT_HANDLERS.lock().unwrap();
    registry.clear();
    let mut counter = HANDLER_ID_COUNTER.lock().unwrap();
    *counter = 0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkBuilder;
    use crate::closure::Closure;

    fn create_test_vm() -> Vm {
        Vm::new()
    }

    fn create_mock_handler() -> Value {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::new(chunk);
        Value::Closure(Arc::new(closure))
    }

    #[test]
    fn test_events_on_registers_handler() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        let args = vec![Value::Str("TestEvent".to_string()), handler];
        let result = events_on(&mut vm, &args);

        assert!(result.is_ok());
        if let Ok(Value::Int(id)) = result {
            assert!(id > 0);
        } else {
            panic!("Expected Int handler ID");
        }
    }

    #[test]
    fn test_events_on_wrong_arg_count() {
        reset_events();
        let mut vm = create_test_vm();

        let args = vec![Value::Str("TestEvent".to_string())];
        let result = events_on(&mut vm, &args);

        assert!(result.is_err());
    }

    #[test]
    fn test_events_on_wrong_event_type() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        let args = vec![Value::Int(42), handler];
        let result = events_on(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, .. }) = result {
            assert_eq!(expected, "string");
        }
    }

    #[test]
    fn test_events_on_wrong_handler_type() {
        reset_events();
        let mut vm = create_test_vm();

        let args = vec![Value::Str("TestEvent".to_string()), Value::Int(42)];
        let result = events_on(&mut vm, &args);

        assert!(result.is_err());
        if let Err(VmError::TypeMismatch { expected, .. }) = result {
            assert_eq!(expected, "function");
        }
    }

    #[test]
    fn test_events_off_removes_handler() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        let args = vec![Value::Str("TestEvent".to_string()), handler];
        let handler_id = events_on(&mut vm, &args).unwrap();

        let result = events_off(&handler_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_events_off_not_found() {
        reset_events();

        let result = events_off(&Value::Int(9999));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_events_off_wrong_type() {
        reset_events();

        let result = events_off(&Value::Str("not an id".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_events_clear_removes_all_handlers_for_event() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        // Register multiple handlers
        let args = vec![Value::Str("TestEvent".to_string()), handler.clone()];
        events_on(&mut vm, &args).unwrap();
        events_on(&mut vm, &args).unwrap();

        // Verify handlers exist
        let count = events_handlers(&Value::Str("TestEvent".to_string())).unwrap();
        assert_eq!(count, Value::Int(2));

        // Clear handlers
        let result = events_clear(&Value::Str("TestEvent".to_string()));
        assert!(result.is_ok());

        // Verify handlers are gone
        let count = events_handlers(&Value::Str("TestEvent".to_string())).unwrap();
        assert_eq!(count, Value::Int(0));
    }

    #[test]
    fn test_events_clear_wrong_type() {
        reset_events();

        let result = events_clear(&Value::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_events_clear_all() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        // Register handlers for multiple events
        events_on(
            &mut vm,
            &[Value::Str("Event1".to_string()), handler.clone()],
        )
        .unwrap();
        events_on(&mut vm, &[Value::Str("Event2".to_string()), handler]).unwrap();

        // Clear all
        let result = events_clear_all(&Value::Unit);
        assert!(result.is_ok());

        // Verify all are gone
        let list = events_list(&Value::Unit).unwrap();
        assert_eq!(list, Value::Nil);
    }

    #[test]
    fn test_events_handlers_count() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        // No handlers initially
        let count = events_handlers(&Value::Str("TestEvent".to_string())).unwrap();
        assert_eq!(count, Value::Int(0));

        // Add handlers
        let args = vec![Value::Str("TestEvent".to_string()), handler.clone()];
        events_on(&mut vm, &args).unwrap();
        events_on(&mut vm, &args).unwrap();
        events_on(&mut vm, &args).unwrap();

        let count = events_handlers(&Value::Str("TestEvent".to_string())).unwrap();
        assert_eq!(count, Value::Int(3));
    }

    #[test]
    fn test_events_handlers_wrong_type() {
        reset_events();

        let result = events_handlers(&Value::Int(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_events_list_empty() {
        reset_events();

        let result = events_list(&Value::Unit).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_events_list_with_events() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        events_on(&mut vm, &[Value::Str("Alpha".to_string()), handler.clone()]).unwrap();
        events_on(&mut vm, &[Value::Str("Beta".to_string()), handler.clone()]).unwrap();
        events_on(&mut vm, &[Value::Str("Gamma".to_string()), handler]).unwrap();

        let result = events_list(&Value::Unit).unwrap();

        // Convert to vec for easier testing
        let vec = result.list_to_vec().unwrap();
        assert_eq!(vec.len(), 3);
        assert!(vec.contains(&Value::Str("Alpha".to_string())));
        assert!(vec.contains(&Value::Str("Beta".to_string())));
        assert!(vec.contains(&Value::Str("Gamma".to_string())));
    }

    #[test]
    fn test_events_once_registers_handler() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        let args = vec![Value::Str("OneTimeEvent".to_string()), handler];
        let result = events_once(&mut vm, &args);

        assert!(result.is_ok());
        if let Ok(Value::Int(id)) = result {
            assert!(id > 0);
        } else {
            panic!("Expected Int handler ID");
        }
    }

    #[test]
    fn test_events_emit_wrong_arg_count() {
        reset_events();
        let mut vm = create_test_vm();

        let args = vec![Value::Str("TestEvent".to_string())];
        let result = events_emit(&mut vm, &args);

        assert!(result.is_err());
    }

    #[test]
    fn test_events_emit_wrong_event_type() {
        reset_events();
        let mut vm = create_test_vm();

        let args = vec![Value::Int(42), Value::Unit];
        let result = events_emit(&mut vm, &args);

        assert!(result.is_err());
    }

    #[test]
    fn test_events_emit_no_handlers() {
        reset_events();
        let mut vm = create_test_vm();

        let args = vec![Value::Str("NoHandlers".to_string()), Value::Unit];
        let result = events_emit(&mut vm, &args);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_unique_handler_ids() {
        reset_events();
        let mut vm = create_test_vm();
        let handler = create_mock_handler();

        let args = vec![Value::Str("TestEvent".to_string()), handler.clone()];

        let id1 = events_on(&mut vm, &args).unwrap();
        let id2 = events_on(&mut vm, &args).unwrap();
        let id3 = events_on(&mut vm, &args).unwrap();

        // All IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
