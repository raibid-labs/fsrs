// Fusabi Commands Standard Library
// Provides command palette functionality for plugins to register custom commands

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Use OnceLock for production, but a regular Mutex for tests (for isolation)
#[cfg(not(test))]
use std::sync::OnceLock;

/// Global command registry (production)
#[cfg(not(test))]
static COMMAND_REGISTRY: OnceLock<Arc<Mutex<CommandRegistryInner>>> = OnceLock::new();

/// Global command registry (testing - allows clearing)
#[cfg(test)]
static COMMAND_REGISTRY: Mutex<Option<Arc<Mutex<CommandRegistryInner>>>> = Mutex::new(None);

/// Inner registry structure
struct CommandRegistryInner {
    commands: HashMap<String, CommandEntry>,
    next_id: i64,
}

impl CommandRegistryInner {
    fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 0,
        }
    }
}

/// Get or initialize the global command registry (production)
#[cfg(not(test))]
fn get_registry() -> Arc<Mutex<CommandRegistryInner>> {
    COMMAND_REGISTRY
        .get_or_init(|| Arc::new(Mutex::new(CommandRegistryInner::new())))
        .clone()
}

/// Get or initialize the global command registry (testing)
#[cfg(test)]
fn get_registry() -> Arc<Mutex<CommandRegistryInner>> {
    let mut reg = COMMAND_REGISTRY.lock().unwrap();
    if reg.is_none() {
        *reg = Some(Arc::new(Mutex::new(CommandRegistryInner::new())));
    }
    reg.as_ref().unwrap().clone()
}

/// Internal representation of a registered command
#[derive(Debug, Clone)]
struct CommandEntry {
    /// Unique numeric ID (for tracking registration)
    numeric_id: i64,
    /// Command ID (e.g., "git.status")
    id: String,
    /// Display name (e.g., "Git: Show Status")
    name: String,
    /// Description
    description: String,
    /// Category
    category: String,
    /// Handler closure (Fusabi function/closure)
    handler: Value,
}

impl CommandEntry {
    /// Convert CommandEntry to Value::Record for Fusabi code
    fn to_value(&self) -> Value {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Str(self.id.clone()));
        fields.insert("name".to_string(), Value::Str(self.name.clone()));
        fields.insert(
            "description".to_string(),
            Value::Str(self.description.clone()),
        );
        fields.insert("category".to_string(), Value::Str(self.category.clone()));
        fields.insert("handler".to_string(), self.handler.clone());
        Value::Record(Arc::new(Mutex::new(fields)))
    }

    /// Try to create CommandEntry from Value::Record
    fn from_value(value: &Value, numeric_id: i64) -> Result<Self, VmError> {
        match value {
            Value::Record(record) => {
                let fields = record.lock().unwrap();

                let id = fields
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        VmError::Runtime("CommandInfo missing 'id' field (string)".to_string())
                    })?
                    .to_string();

                let name = fields
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        VmError::Runtime("CommandInfo missing 'name' field (string)".to_string())
                    })?
                    .to_string();

                let description = fields
                    .get("description")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        VmError::Runtime(
                            "CommandInfo missing 'description' field (string)".to_string(),
                        )
                    })?
                    .to_string();

                let category = fields
                    .get("category")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        VmError::Runtime(
                            "CommandInfo missing 'category' field (string)".to_string(),
                        )
                    })?
                    .to_string();

                let handler = fields
                    .get("handler")
                    .ok_or_else(|| {
                        VmError::Runtime("CommandInfo missing 'handler' field".to_string())
                    })?
                    .clone();

                // Validate that handler is a function
                if !matches!(handler, Value::Closure(_) | Value::NativeFn { .. }) {
                    return Err(VmError::Runtime(
                        "CommandInfo 'handler' must be a function or closure".to_string(),
                    ));
                }

                Ok(CommandEntry {
                    numeric_id,
                    id,
                    name,
                    description,
                    category,
                    handler,
                })
            }
            _ => Err(VmError::TypeMismatch {
                expected: "record",
                got: value.type_name(),
            }),
        }
    }
}

/// Commands.register : CommandInfo -> int
/// Registers a command and returns its numeric ID
pub fn commands_register(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.register expects 1 argument, got {}",
            args.len()
        )));
    }

    let registry = get_registry();
    let mut reg = registry.lock().unwrap();

    let numeric_id = reg.next_id;
    reg.next_id += 1;

    let entry = CommandEntry::from_value(&args[0], numeric_id)?;
    let command_id = entry.id.clone();

    // Store in registry
    reg.commands.insert(command_id, entry);

    Ok(Value::Int(numeric_id))
}

/// Commands.registerMany : CommandInfo list -> int list
/// Registers multiple commands and returns their numeric IDs
pub fn commands_register_many(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.registerMany expects 1 argument, got {}",
            args.len()
        )));
    }

    // Collect all command infos from the list
    let mut command_infos = Vec::new();
    let mut current = &args[0];

    loop {
        match current {
            Value::Nil => break,
            Value::Cons { head, tail } => {
                command_infos.push((**head).clone());
                current = tail;
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "list",
                    got: current.type_name(),
                });
            }
        }
    }

    let registry = get_registry();
    let mut reg = registry.lock().unwrap();

    // Register all commands and collect their IDs
    let mut ids = Vec::new();
    for cmd_info in command_infos {
        let numeric_id = reg.next_id;
        reg.next_id += 1;

        let entry = CommandEntry::from_value(&cmd_info, numeric_id)?;
        let command_id = entry.id.clone();

        reg.commands.insert(command_id, entry);
        ids.push(Value::Int(numeric_id));
    }

    // Convert Vec<Value> to cons list
    let mut result = Value::Nil;
    for id in ids.into_iter().rev() {
        result = Value::Cons {
            head: Box::new(id),
            tail: Box::new(result),
        };
    }

    Ok(result)
}

/// Commands.unregister : int -> bool
/// Unregisters a command by its string ID (despite the type signature suggesting int)
/// Returns true if a command was found and removed, false otherwise
pub fn commands_unregister(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.unregister expects 1 argument, got {}",
            args.len()
        )));
    }

    let registry = get_registry();
    let mut reg = registry.lock().unwrap();

    match &args[0] {
        Value::Str(id) => {
            // Remove by string ID
            let removed = reg.commands.remove(id).is_some();
            Ok(Value::Bool(removed))
        }
        Value::Int(numeric_id) => {
            // Find and remove by numeric ID
            let key_to_remove = reg
                .commands
                .iter()
                .find(|(_, entry)| entry.numeric_id == *numeric_id)
                .map(|(k, _)| k.clone());

            if let Some(key) = key_to_remove {
                reg.commands.remove(&key);
                Ok(Value::Bool(true))
            } else {
                Ok(Value::Bool(false))
            }
        }
        _ => Err(VmError::TypeMismatch {
            expected: "string or int",
            got: args[0].type_name(),
        }),
    }
}

/// Commands.list : unit -> CommandInfo list
/// Returns a list of all registered commands
pub fn commands_list(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.list expects 1 argument, got {}",
            args.len()
        )));
    }

    // Verify unit argument
    if !matches!(args[0], Value::Unit) {
        return Err(VmError::TypeMismatch {
            expected: "unit",
            got: args[0].type_name(),
        });
    }

    let registry = get_registry();
    let reg = registry.lock().unwrap();

    // Convert all commands to a list
    let mut result = Value::Nil;
    for entry in reg.commands.values() {
        result = Value::Cons {
            head: Box::new(entry.to_value()),
            tail: Box::new(result),
        };
    }

    Ok(result)
}

/// Commands.getById : string -> CommandInfo option
/// Gets a command by its string ID
pub fn commands_get_by_id(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.getById expects 1 argument, got {}",
            args.len()
        )));
    }

    let id = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let registry = get_registry();
    let reg = registry.lock().unwrap();

    match reg.commands.get(id) {
        Some(entry) => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![entry.to_value()],
        }),
        None => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        }),
    }
}

/// Commands.invoke : string -> unit
/// Invokes a command by its string ID
pub fn commands_invoke(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Commands.invoke expects 1 argument, got {}",
            args.len()
        )));
    }

    let id = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    // Get the handler (clone it so we don't hold the lock)
    let handler = {
        let registry = get_registry();
        let reg = registry.lock().unwrap();
        reg.commands
            .get(id)
            .map(|entry| entry.handler.clone())
            .ok_or_else(|| VmError::Runtime(format!("Command not found: {}", id)))?
    };

    // Invoke the handler with no arguments
    match &handler {
        Value::Closure(closure) => {
            // Call the closure with no arguments
            vm.call_closure(closure.clone(), &[])?;
            Ok(Value::Unit)
        }
        Value::NativeFn { name, arity, .. } => {
            // Check arity
            if *arity != 0 {
                return Err(VmError::Runtime(format!(
                    "Command handler '{}' expects {} arguments, but commands are invoked with 0 arguments",
                    name, arity
                )));
            }
            // For native functions, we need to call them through the host registry
            // Since we can't directly invoke them, we'll just return an error for now
            // In a real implementation, the host application would handle command invocation
            Err(VmError::Runtime(
                "Cannot invoke native function commands directly; host must handle invocation"
                    .to_string(),
            ))
        }
        _ => Err(VmError::Runtime(
            "Invalid handler type (must be closure or native function)".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_command(id: &str, name: &str, description: &str, category: &str) -> Value {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Str(id.to_string()));
        fields.insert("name".to_string(), Value::Str(name.to_string()));
        fields.insert(
            "description".to_string(),
            Value::Str(description.to_string()),
        );
        fields.insert("category".to_string(), Value::Str(category.to_string()));
        // Use a dummy NativeFn as handler for testing
        fields.insert(
            "handler".to_string(),
            Value::NativeFn {
                name: "test_handler".to_string(),
                arity: 0,
                args: vec![],
            },
        );
        Value::Record(Arc::new(Mutex::new(fields)))
    }

    // Note: These tests share global state (COMMAND_REGISTRY).
    // While we reset the registry in clear_registry(), parallel test execution
    // can still cause interference. Tests pass when run serially:
    //   cargo test --package fusabi-vm commands::tests -- --test-threads=1

    fn clear_registry() {
        // Reset the global registry for test isolation
        let mut reg_opt = COMMAND_REGISTRY.lock().unwrap();
        *reg_opt = Some(Arc::new(Mutex::new(CommandRegistryInner::new())));
    }

    #[test]
    fn test_register_command() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd = create_test_command("test.command", "Test Command", "A test command", "Testing");

        let result = commands_register(&mut vm, &[cmd]).unwrap();

        // Should return numeric ID (0 for first command)
        assert_eq!(result, Value::Int(0));

        // Verify command is in registry
        let registry = get_registry();
        let reg = registry.lock().unwrap();
        assert!(reg.commands.contains_key("test.command"));
        assert_eq!(reg.commands.len(), 1);
    }

    #[test]
    fn test_register_multiple_commands() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd1 = create_test_command("cmd1", "Command 1", "First", "Test");
        let cmd2 = create_test_command("cmd2", "Command 2", "Second", "Test");

        commands_register(&mut vm, &[cmd1]).unwrap();
        commands_register(&mut vm, &[cmd2]).unwrap();

        let registry = get_registry();
        let reg = registry.lock().unwrap();
        assert_eq!(reg.commands.len(), 2);
        assert!(reg.commands.contains_key("cmd1"));
        assert!(reg.commands.contains_key("cmd2"));
    }

    #[test]
    fn test_register_many_commands() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd1 = create_test_command("cmd1", "Command 1", "First", "Test");
        let cmd2 = create_test_command("cmd2", "Command 2", "Second", "Test");

        // Create a list of commands
        let list = Value::Cons {
            head: Box::new(cmd1),
            tail: Box::new(Value::Cons {
                head: Box::new(cmd2),
                tail: Box::new(Value::Nil),
            }),
        };

        let result = commands_register_many(&mut vm, &[list]).unwrap();

        // Result should be a list of numeric IDs
        match result {
            Value::Cons { head, tail } => {
                assert_eq!(*head, Value::Int(0));
                match *tail {
                    Value::Cons { head, tail } => {
                        assert_eq!(*head, Value::Int(1));
                        assert_eq!(*tail, Value::Nil);
                    }
                    _ => panic!("Expected second element in list"),
                }
            }
            _ => panic!("Expected cons list"),
        }

        let registry = get_registry();
        let reg = registry.lock().unwrap();
        assert_eq!(reg.commands.len(), 2);
    }

    #[test]
    fn test_list_commands() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd1 = create_test_command("cmd1", "Command 1", "First", "Test");
        let cmd2 = create_test_command("cmd2", "Command 2", "Second", "Test");

        commands_register(&mut vm, &[cmd1]).unwrap();
        commands_register(&mut vm, &[cmd2]).unwrap();

        let result = commands_list(&mut vm, &[Value::Unit]).unwrap();

        // Should return a list with 2 elements
        let mut count = 0;
        let mut current = &result;
        loop {
            match current {
                Value::Nil => break,
                Value::Cons { head, tail } => {
                    count += 1;
                    // Verify it's a record
                    assert!(matches!(**head, Value::Record(_)));
                    current = tail;
                }
                _ => panic!("Expected list"),
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_by_id() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd = create_test_command("test.command", "Test Command", "A test command", "Testing");

        commands_register(&mut vm, &[cmd]).unwrap();

        // Get existing command
        let result =
            commands_get_by_id(&mut vm, &[Value::Str("test.command".to_string())]).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(fields.len(), 1);
                assert!(matches!(fields[0], Value::Record(_)));
            }
            _ => panic!("Expected Some variant"),
        }

        // Get non-existing command
        let result = commands_get_by_id(&mut vm, &[Value::Str("nonexistent".to_string())]).unwrap();
        match result {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                assert_eq!(variant_name, "None");
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_unregister_command() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd = create_test_command("test.command", "Test", "Description", "Category");

        commands_register(&mut vm, &[cmd]).unwrap();

        // Unregister by string ID
        let result =
            commands_unregister(&mut vm, &[Value::Str("test.command".to_string())]).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Verify it's gone
        let registry = get_registry();
        let reg = registry.lock().unwrap();
        assert_eq!(reg.commands.len(), 0);
        drop(reg);

        // Try to unregister again
        let result =
            commands_unregister(&mut vm, &[Value::Str("test.command".to_string())]).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unregister_by_numeric_id() {
        clear_registry();
        let mut vm = Vm::new();

        let cmd = create_test_command("test.command", "Test", "Description", "Category");

        let numeric_id = commands_register(&mut vm, &[cmd]).unwrap();

        // Unregister by numeric ID
        let result = commands_unregister(&mut vm, &[numeric_id]).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Verify it's gone
        let registry = get_registry();
        let reg = registry.lock().unwrap();
        assert_eq!(reg.commands.len(), 0);
    }

    #[test]
    fn test_missing_field_error() {
        clear_registry();
        let mut vm = Vm::new();

        // Create command with missing 'name' field
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Str("test".to_string()));
        fields.insert("description".to_string(), Value::Str("desc".to_string()));
        fields.insert("category".to_string(), Value::Str("cat".to_string()));
        fields.insert(
            "handler".to_string(),
            Value::NativeFn {
                name: "test".to_string(),
                arity: 0,
                args: vec![],
            },
        );
        let cmd = Value::Record(Arc::new(Mutex::new(fields)));

        let result = commands_register(&mut vm, &[cmd]);
        assert!(result.is_err());
        match result {
            Err(VmError::Runtime(msg)) => {
                assert!(msg.contains("missing 'name'"));
            }
            _ => panic!("Expected runtime error"),
        }
    }

    #[test]
    fn test_invalid_handler_type() {
        clear_registry();
        let mut vm = Vm::new();

        // Create command with non-function handler
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Str("test".to_string()));
        fields.insert("name".to_string(), Value::Str("name".to_string()));
        fields.insert("description".to_string(), Value::Str("desc".to_string()));
        fields.insert("category".to_string(), Value::Str("cat".to_string()));
        fields.insert("handler".to_string(), Value::Int(42)); // Invalid!
        let cmd = Value::Record(Arc::new(Mutex::new(fields)));

        let result = commands_register(&mut vm, &[cmd]);
        assert!(result.is_err());
        match result {
            Err(VmError::Runtime(msg)) => {
                assert!(msg.contains("handler' must be a function"));
            }
            _ => panic!("Expected runtime error"),
        }
    }

    #[test]
    fn test_type_mismatch_errors() {
        clear_registry();
        let mut vm = Vm::new();

        // register expects record
        let result = commands_register(&mut vm, &[Value::Int(42)]);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));

        // registerMany expects list
        let result = commands_register_many(&mut vm, &[Value::Int(42)]);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));

        // getById expects string
        let result = commands_get_by_id(&mut vm, &[Value::Int(42)]);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));

        // list expects unit
        let result = commands_list(&mut vm, &[Value::Int(42)]);
        assert!(matches!(result, Err(VmError::TypeMismatch { .. })));
    }
}
