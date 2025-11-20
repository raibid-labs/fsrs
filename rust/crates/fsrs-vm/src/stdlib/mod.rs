// FSRS Standard Library
// Provides built-in functions for List, String, and Option operations

pub mod list;
pub mod option;
pub mod string;

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;

/// Type alias for standard library functions
/// Takes a slice of arguments and returns a Result with Value or VmError
pub type StdlibFn = Box<dyn Fn(&[Value]) -> Result<Value, VmError>>;

/// Registry of all standard library functions
/// Provides lookup and registration of built-in functions
pub struct StdlibRegistry {
    functions: HashMap<String, StdlibFn>,
}

impl StdlibRegistry {
    /// Create a new stdlib registry with all built-in functions registered
    pub fn new() -> Self {
        let mut registry = StdlibRegistry {
            functions: HashMap::new(),
        };

        registry.register_list_functions();
        registry.register_string_functions();
        registry.register_option_functions();

        registry
    }

    /// Register all List module functions
    fn register_list_functions(&mut self) {
        self.register("List.length", wrap_unary(list::list_length));
        self.register("List.head", wrap_unary(list::list_head));
        self.register("List.tail", wrap_unary(list::list_tail));
        self.register("List.reverse", wrap_unary(list::list_reverse));
        self.register("List.isEmpty", wrap_unary(list::list_is_empty));
        self.register("List.append", wrap_binary(list::list_append));
        self.register("List.concat", wrap_unary(list::list_concat));
    }

    /// Register all String module functions
    fn register_string_functions(&mut self) {
        self.register("String.length", wrap_unary(string::string_length));
        self.register("String.trim", wrap_unary(string::string_trim));
        self.register("String.toLower", wrap_unary(string::string_to_lower));
        self.register("String.toUpper", wrap_unary(string::string_to_upper));
        self.register("String.split", wrap_binary(string::string_split));
        self.register("String.concat", wrap_unary(string::string_concat));
        self.register("String.contains", wrap_binary(string::string_contains));
        self.register("String.startsWith", wrap_binary(string::string_starts_with));
        self.register("String.endsWith", wrap_binary(string::string_ends_with));
    }

    /// Register all Option module functions
    fn register_option_functions(&mut self) {
        self.register("Option.isSome", wrap_unary(option::option_is_some));
        self.register("Option.isNone", wrap_unary(option::option_is_none));
        self.register(
            "Option.defaultValue",
            wrap_binary(option::option_default_value),
        );
    }

    /// Register a function with the given name
    fn register(&mut self, name: &str, f: StdlibFn) {
        self.functions.insert(name.to_string(), f);
    }

    /// Look up a function by name
    pub fn lookup(&self, name: &str) -> Option<&StdlibFn> {
        self.functions.get(name)
    }

    /// Call a function by name with the given arguments
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, VmError> {
        match self.lookup(name) {
            Some(f) => f(args),
            None => Err(VmError::Runtime(format!("Unknown function: {}", name))),
        }
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl Default for StdlibRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to wrap unary functions (1 argument)
fn wrap_unary<F>(f: F) -> StdlibFn
where
    F: Fn(&Value) -> Result<Value, VmError> + 'static,
{
    Box::new(move |args: &[Value]| {
        if args.len() != 1 {
            return Err(VmError::Runtime(format!(
                "Expected 1 argument, got {}",
                args.len()
            )));
        }
        f(&args[0])
    })
}

/// Helper to wrap binary functions (2 arguments)
fn wrap_binary<F>(f: F) -> StdlibFn
where
    F: Fn(&Value, &Value) -> Result<Value, VmError> + 'static,
{
    Box::new(move |args: &[Value]| {
        if args.len() != 2 {
            return Err(VmError::Runtime(format!(
                "Expected 2 arguments, got {}",
                args.len()
            )));
        }
        f(&args[0], &args[1])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = StdlibRegistry::new();
        assert!(registry.has_function("List.length"));
        assert!(registry.has_function("String.length"));
        assert!(registry.has_function("Option.isSome"));
    }

    #[test]
    fn test_registry_lookup() {
        let registry = StdlibRegistry::new();
        let list_length = registry.lookup("List.length");
        assert!(list_length.is_some());
    }

    #[test]
    fn test_registry_all_functions() {
        let registry = StdlibRegistry::new();
        let names = registry.function_names();
        assert!(names.len() >= 15); // At least 15 functions registered
    }

    #[test]
    fn test_registry_missing_function() {
        let registry = StdlibRegistry::new();
        assert!(!registry.has_function("NonExistent.function"));
        assert!(registry.lookup("NonExistent.function").is_none());
    }

    #[test]
    fn test_registry_call() {
        let registry = StdlibRegistry::new();
        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = registry.call("List.length", &[list]).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_registry_call_missing() {
        let registry = StdlibRegistry::new();
        let result = registry.call("NonExistent.function", &[]);
        assert!(result.is_err());
    }
}
