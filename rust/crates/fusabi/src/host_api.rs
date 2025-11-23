// High-level Host Interop API for Fusabi
// Provides ergonomic embedding API for Rust applications

use fusabi_vm::{HostRegistry, Value, Vm, VmError};
use fusabi_vm::stdlib::{list, option, string};
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;
use std::cell::RefCell;

/// High-level API for embedding Fusabi in Rust applications
///
/// # Example
/// ```no_run
/// use fusabi_demo::host_api::FusabiEngine;
/// use fusabi_vm::Value;
///
/// let mut engine = FusabiEngine::new();
///
/// // Register a host function
/// engine.register_fn1("double", |x: Value| {
///     let n = x.as_int().unwrap_or(0);
///     Ok(Value::Int(n * 2))
/// });
///
/// // Call the host function
/// let result = engine.call_host("double", &[Value::Int(21)]).unwrap();
/// assert_eq!(result.as_int(), Some(42));
/// ```
pub struct FusabiEngine {
    #[allow(dead_code)]
    vm: Vm,
    host_registry: Rc<RefCell<HostRegistry>>,
    global_bindings: HashMap<String, Value>,
}

impl FusabiEngine {
    /// Create a new Fusabi engine
    pub fn new() -> Self {
        let vm = Vm::new();
        let host_registry = vm.host_registry.clone();
        
        let mut engine = FusabiEngine {
            vm,
            host_registry,
            global_bindings: HashMap::new(),
        };
        engine.register_stdlib_functions();
        engine
    }

    /// Register all standard library functions into the engine's HostRegistry
    fn register_stdlib_functions(&mut self) {
        // List functions
        self.register_fn1("List.length", |v| list::list_length(&v));
        self.register_fn1("List.head", |v| list::list_head(&v));
        self.register_fn1("List.tail", |v| list::list_tail(&v));
        self.register_fn1("List.reverse", |v| list::list_reverse(&v));
        self.register_fn1("List.isEmpty", |v| list::list_is_empty(&v));
        self.register_fn2("List.append", |v1, v2| list::list_append(&v1, &v2));
        self.register_fn1("List.concat", |v| list::list_concat(&v));
        // List.map requires VM context, so use raw register
        self.register_raw("List.map", |vm, args| list::list_map(vm, args));

        // String functions
        self.register_fn1("String.length", |v| string::string_length(&v));
        self.register_fn1("String.trim", |v| string::string_trim(&v));
        self.register_fn1("String.toLower", |v| string::string_to_lower(&v));
        self.register_fn1("String.toUpper", |v| string::string_to_upper(&v));
        self.register_fn2("String.split", |v1, v2| string::string_split(&v1, &v2));
        self.register_fn1("String.concat", |v| string::string_concat(&v));
        self.register_fn2("String.contains", |v1, v2| string::string_contains(&v1, &v2));
        self.register_fn2("String.startsWith", |v1, v2| string::string_starts_with(&v1, &v2));
        self.register_fn2("String.endsWith", |v1, v2| string::string_ends_with(&v1, &v2));

        // Option functions
        self.register_fn1("Option.isSome", |v| option::option_is_some(&v));
        self.register_fn1("Option.isNone", |v| option::option_is_none(&v));
        self.register_fn2("Option.defaultValue", |v1, v2| option::option_default_value(&v1, &v2));
    }

    /// Register a host function with dynamic arity
    ///
    /// # Example
    /// ```no_run
    /// # use fusabi_demo::host_api::FusabiEngine;
    /// # use fusabi_vm::Value;
    /// let mut engine = FusabiEngine::new();
    /// engine.register("sum", |args| {
    ///     let sum: i64 = args.iter()
    ///         .filter_map(|v| v.as_int())
    ///         .sum();
    ///     Ok(Value::Int(sum))
    /// });
    /// ```
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        // Wrap the closure to ignore the VM argument, maintaining backward compatibility
        self.host_registry.borrow_mut().register(name, move |_vm, args| f(args));
    }

    /// Register a raw host function that needs access to the VM context
    pub fn register_raw<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&mut Vm, &[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.borrow_mut().register(name, f);
    }

    /// Register a nullary host function (no arguments)
    pub fn register_fn0<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.borrow_mut().register_fn0(name, move |_vm| f());
    }

    /// Register a unary host function (1 argument)
    pub fn register_fn1<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.borrow_mut().register_fn1(name, move |_vm, arg| f(arg));
    }

    /// Register a binary host function (2 arguments)
    pub fn register_fn2<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.borrow_mut()
            .register_fn2(name, move |_vm, arg1, arg2| f(arg1, arg2));
    }

    /// Register a ternary host function (3 arguments)
    pub fn register_fn3<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.borrow_mut()
            .register_fn3(name, move |_vm, arg1, arg2, arg3| f(arg1, arg2, arg3));
    }

    /// Call a registered host function
    pub fn call_host(&mut self, name: &str, args: &[Value]) -> Result<Value, VmError> {
        // Pass the VM instance to the host function
        let registry = self.host_registry.borrow();
        registry.call(name, &mut self.vm, args)
    }

    /// Check if a host function is registered
    pub fn has_host_function(&self, name: &str) -> bool {
        self.host_registry.borrow().has_function(name)
    }

    /// Get all registered host function names
    pub fn host_function_names(&self) -> Vec<String> {
        self.host_registry.borrow().function_names()
    }

    /// Set a global variable
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.global_bindings.insert(name.to_string(), value);
    }

    /// Get a global variable
    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.global_bindings.get(name)
    }

    /// Execute a host function call (for demonstration purposes)
    /// In a full implementation, this would be integrated with the VM execution
    pub fn execute_host_call(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let registry = self.host_registry.borrow();
        registry.call(name, &mut self.vm, args)
            .map_err(|e| format!("Host function error: {:?}", e))
    }

    /// Convert result to Rust type with better error messages
    pub fn convert_result<T>(value: Value) -> Result<T, String>
    where
        T: TryFrom<Value>,
        T::Error: std::fmt::Debug,
    {
        value
            .try_into()
            .map_err(|e| format!("Type conversion error: {:?}", e))
    }
}

impl Default for FusabiEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for registering host functions with automatic type conversion
///
/// # Example
/// ```ignore
/// register_typed_fn!(engine, "add", |a: i64, b: i64| -> i64 {
///     a + b
/// });
/// ```
#[macro_export]
macro_rules! register_typed_fn {
    ($engine:expr, $name:expr, |$($arg:ident : $arg_ty:ty),*| -> $ret_ty:ty $body:block) => {
        $engine.register($name, |args| {
            let mut arg_iter = args.iter();
            $(
                let $arg: $arg_ty = arg_iter
                    .next()
                    .ok_or_else(|| VmError::Runtime("Missing argument".into()))?
                    .clone()
                    .try_into()
                    .map_err(|e| VmError::Runtime(format!("Type conversion error: {:?}", e)))?;
            )*
            let result: $ret_ty = $body;
            Ok(result.into())
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = FusabiEngine::new();
        assert!(engine.host_registry.borrow().count() >= 20);
    }

    #[test]
    fn test_register_fn1() {
        let mut engine = FusabiEngine::new();
        engine.register_fn1("double", |v| {
            let n = v
                .as_int()
                .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
            Ok(Value::Int(n * 2))
        });

        let result = engine.call_host("double", &[Value::Int(21)]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_register_fn2() {
        let mut engine = FusabiEngine::new();
        engine.register_fn2("add", |a, b| {
            let x = a
                .as_int()
                .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
            let y = b
                .as_int()
                .ok_or_else(|| VmError::Runtime("Expected int".into()))?;
            Ok(Value::Int(x + y))
        });

        let result = engine
            .call_host("add", &[Value::Int(10), Value::Int(32)])
            .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_global_bindings() {
        let mut engine = FusabiEngine::new();
        engine.set_global("x", Value::Int(42));
        assert_eq!(engine.get_global("x"), Some(&Value::Int(42)));
        assert_eq!(engine.get_global("y"), None);
    }

    #[test]
    fn test_has_host_function() {
        let mut engine = FusabiEngine::new();
        assert!(!engine.has_host_function("test"));

        engine.register_fn1("test", |v: Value| -> Result<Value, VmError> { Ok(v) });
        assert!(engine.has_host_function("test"));
    }

    #[test]
    fn test_host_function_names() {
        let mut engine = FusabiEngine::new();
        engine.register_fn1("fn1", |v: Value| -> Result<Value, VmError> { Ok(v) });
        engine.register_fn1("fn2", |v: Value| -> Result<Value, VmError> { Ok(v) });

        let names = engine.host_function_names();
        assert!(names.contains(&"fn1".to_string()));
        assert!(names.contains(&"fn2".to_string()));
    }

    #[test]
    fn test_convert_result_success() {
        let value = Value::Int(42);
        let n: i64 = FusabiEngine::convert_result(value).unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_convert_result_failure() {
        let value = Value::Bool(true);
        let result: Result<i64, String> = FusabiEngine::convert_result(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_function() {
        let mut engine = FusabiEngine::new();
        engine.register_fn1("uppercase", |v| {
            let s = v
                .as_str()
                .ok_or_else(|| VmError::Runtime("Expected string".into()))?;
            Ok(Value::Str(s.to_uppercase()))
        });

        let result = engine
            .call_host("uppercase", &[Value::Str("hello".to_string())])
            .unwrap();
        assert_eq!(result, Value::Str("HELLO".to_string()));
    }

    #[test]
    fn test_list_function() {
        let mut engine = FusabiEngine::new();
        engine.register_fn1("sum_list", |v| {
            let list = v
                .list_to_vec()
                .ok_or_else(|| VmError::Runtime("Expected list".into()))?;
            let sum: i64 = list.iter().filter_map(|v| v.as_int()).sum();
            Ok(Value::Int(sum))
        });

        let list = Value::vec_to_cons(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let result = engine.call_host("sum_list", &[list]).unwrap();
        assert_eq!(result, Value::Int(6));
    }
}
