// High-level Host Interop API for FSRS
// Provides ergonomic embedding API for Rust applications

use fsrs_vm::{HostRegistry, Value, Vm, VmError};
use std::collections::HashMap;
use std::convert::TryInto;

/// High-level API for embedding FSRS in Rust applications
///
/// # Example
/// ```no_run
/// use fsrs_demo::host_api::FsrsEngine;
/// use fsrs_vm::Value;
///
/// let mut engine = FsrsEngine::new();
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
pub struct FsrsEngine {
    #[allow(dead_code)]
    vm: Vm,
    host_registry: HostRegistry,
    global_bindings: HashMap<String, Value>,
}

impl FsrsEngine {
    /// Create a new FSRS engine
    pub fn new() -> Self {
        FsrsEngine {
            vm: Vm::new(),
            host_registry: HostRegistry::new(),
            global_bindings: HashMap::new(),
        }
    }

    /// Register a host function with dynamic arity
    ///
    /// # Example
    /// ```no_run
    /// # use fsrs_demo::host_api::FsrsEngine;
    /// # use fsrs_vm::Value;
    /// let mut engine = FsrsEngine::new();
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
        self.host_registry.register(name, f);
    }

    /// Register a nullary host function (no arguments)
    pub fn register_fn0<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.register_fn0(name, f);
    }

    /// Register a unary host function (1 argument)
    pub fn register_fn1<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.register_fn1(name, f);
    }

    /// Register a binary host function (2 arguments)
    pub fn register_fn2<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.register_fn2(name, f);
    }

    /// Register a ternary host function (3 arguments)
    pub fn register_fn3<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry.register_fn3(name, f);
    }

    /// Call a registered host function
    pub fn call_host(&self, name: &str, args: &[Value]) -> Result<Value, VmError> {
        self.host_registry.call(name, args)
    }

    /// Check if a host function is registered
    pub fn has_host_function(&self, name: &str) -> bool {
        self.host_registry.has_function(name)
    }

    /// Get all registered host function names
    pub fn host_function_names(&self) -> Vec<String> {
        self.host_registry.function_names()
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
    pub fn execute_host_call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        self.host_registry
            .call(name, args)
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

impl Default for FsrsEngine {
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
        let engine = FsrsEngine::new();
        assert_eq!(engine.host_registry.count(), 0);
    }

    #[test]
    fn test_register_fn1() {
        let mut engine = FsrsEngine::new();
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
        let mut engine = FsrsEngine::new();
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
        let mut engine = FsrsEngine::new();
        engine.set_global("x", Value::Int(42));
        assert_eq!(engine.get_global("x"), Some(&Value::Int(42)));
        assert_eq!(engine.get_global("y"), None);
    }

    #[test]
    fn test_has_host_function() {
        let mut engine = FsrsEngine::new();
        assert!(!engine.has_host_function("test"));

        engine.register_fn1("test", |v: Value| -> Result<Value, VmError> { Ok(v) });
        assert!(engine.has_host_function("test"));
    }

    #[test]
    fn test_host_function_names() {
        let mut engine = FsrsEngine::new();
        engine.register_fn1("fn1", |v: Value| -> Result<Value, VmError> { Ok(v) });
        engine.register_fn1("fn2", |v: Value| -> Result<Value, VmError> { Ok(v) });

        let mut names = engine.host_function_names();
        names.sort();
        assert_eq!(names, vec!["fn1".to_string(), "fn2".to_string()]);
    }

    #[test]
    fn test_convert_result_success() {
        let value = Value::Int(42);
        let n: i64 = FsrsEngine::convert_result(value).unwrap();
        assert_eq!(n, 42);
    }

    #[test]
    fn test_convert_result_failure() {
        let value = Value::Bool(true);
        let result: Result<i64, String> = FsrsEngine::convert_result(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_function() {
        let mut engine = FsrsEngine::new();
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
        let mut engine = FsrsEngine::new();
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
