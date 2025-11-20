// Host Function Registry for FSRS
// Enables Rust applications to register native functions callable from FSRS scripts

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;

/// Host function signature - takes a slice of values and returns a value or error
pub type HostFn = Box<dyn Fn(&[Value]) -> Result<Value, VmError> + Send + Sync>;

/// Registry for host functions that can be called from FSRS scripts
pub struct HostRegistry {
    functions: HashMap<String, HostFn>,
}

impl HostRegistry {
    /// Create a new empty host registry
    pub fn new() -> Self {
        HostRegistry {
            functions: HashMap::new(),
        }
    }

    /// Register a host function with dynamic arity
    ///
    /// # Example
    /// ```
    /// # use fsrs_vm::host::HostRegistry;
    /// # use fsrs_vm::value::Value;
    /// # use fsrs_vm::vm::VmError;
    /// let mut registry = HostRegistry::new();
    /// registry.register("add", |args| {
    ///     if args.len() != 2 {
    ///         return Err(VmError::Runtime("add expects 2 arguments".into()));
    ///     }
    ///     let a = args[0].as_int().ok_or_else(|| VmError::Runtime("arg 0 must be int".into()))?;
    ///     let b = args[1].as_int().ok_or_else(|| VmError::Runtime("arg 1 must be int".into()))?;
    ///     Ok(Value::Int(a + b))
    /// });
    /// ```
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Box::new(f));
    }

    /// Register a nullary function (no arguments)
    pub fn register_fn0<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> Result<Value, VmError> + Send + Sync + 'static,
    {
        let name_owned = name.to_string();
        self.register(name, move |args| {
            if !args.is_empty() {
                return Err(VmError::Runtime(format!(
                    "{} expects 0 arguments, got {}",
                    name_owned,
                    args.len()
                )));
            }
            f()
        });
    }

    /// Register a unary function (1 argument)
    pub fn register_fn1<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        let name_owned = name.to_string();
        self.register(name, move |args| {
            if args.len() != 1 {
                return Err(VmError::Runtime(format!(
                    "{} expects 1 argument, got {}",
                    name_owned,
                    args.len()
                )));
            }
            f(args[0].clone())
        });
    }

    /// Register a binary function (2 arguments)
    pub fn register_fn2<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        let name_owned = name.to_string();
        self.register(name, move |args| {
            if args.len() != 2 {
                return Err(VmError::Runtime(format!(
                    "{} expects 2 arguments, got {}",
                    name_owned,
                    args.len()
                )));
            }
            f(args[0].clone(), args[1].clone())
        });
    }

    /// Register a ternary function (3 arguments)
    pub fn register_fn3<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        let name_owned = name.to_string();
        self.register(name, move |args| {
            if args.len() != 3 {
                return Err(VmError::Runtime(format!(
                    "{} expects 3 arguments, got {}",
                    name_owned,
                    args.len()
                )));
            }
            f(args[0].clone(), args[1].clone(), args[2].clone())
        });
    }

    /// Call a registered host function
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, VmError> {
        let f = self
            .functions
            .get(name)
            .ok_or_else(|| VmError::Runtime(format!("Undefined host function: {}", name)))?;
        f(args)
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get the count of registered functions
    pub fn count(&self) -> usize {
        self.functions.len()
    }
}

impl Default for HostRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_call() {
        let mut registry = HostRegistry::new();
        registry.register("double", |args| {
            if args.len() != 1 {
                return Err(VmError::Runtime("double expects 1 argument".into()));
            }
            let n = args[0]
                .as_int()
                .ok_or_else(|| VmError::Runtime("argument must be int".into()))?;
            Ok(Value::Int(n * 2))
        });

        let result = registry.call("double", &[Value::Int(21)]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_register_fn1() {
        let mut registry = HostRegistry::new();
        registry.register_fn1("increment", |v| {
            let n = v
                .as_int()
                .ok_or_else(|| VmError::Runtime("argument must be int".into()))?;
            Ok(Value::Int(n + 1))
        });

        let result = registry.call("increment", &[Value::Int(41)]).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_register_fn2() {
        let mut registry = HostRegistry::new();
        registry.register_fn2("add", |a, b| {
            let x = a
                .as_int()
                .ok_or_else(|| VmError::Runtime("arg 0 must be int".into()))?;
            let y = b
                .as_int()
                .ok_or_else(|| VmError::Runtime("arg 1 must be int".into()))?;
            Ok(Value::Int(x + y))
        });

        let result = registry
            .call("add", &[Value::Int(10), Value::Int(32)])
            .unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_undefined_function() {
        let registry = HostRegistry::new();
        let result = registry.call("undefined", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_has_function() {
        let mut registry = HostRegistry::new();
        assert!(!registry.has_function("test"));

        registry.register_fn1("test", |v: Value| -> Result<Value, VmError> { Ok(v) });
        assert!(registry.has_function("test"));
    }

    #[test]
    fn test_function_names() {
        let mut registry = HostRegistry::new();
        registry.register_fn1("fn1", |v: Value| -> Result<Value, VmError> { Ok(v) });
        registry.register_fn1("fn2", |v: Value| -> Result<Value, VmError> { Ok(v) });

        let mut names = registry.function_names();
        names.sort();
        assert_eq!(names, vec!["fn1".to_string(), "fn2".to_string()]);
    }
}
