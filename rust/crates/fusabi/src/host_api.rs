// High-level Host Interop API for Fusabi
// Provides ergonomic embedding API for Rust applications

use fusabi_frontend::compiler::CompileOptions;
use fusabi_frontend::{Compiler, Lexer, Parser};
use fusabi_vm::{HostData, HostRegistry, Value, Vm, VmError};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

/// Type alias for a raw host function
pub type HostFunction = Box<dyn Fn(&mut Vm, &[Value]) -> Result<Value, VmError> + Send + Sync>;

/// Module builder for grouping related host functions
pub struct Module {
    name: String,
    functions: Vec<(String, HostFunction)>,
}

impl Module {
    /// Create a new module
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: Vec::new(),
        }
    }

    /// Get the module name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Register a function with dynamic arity in this module
    pub fn register<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(&[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions
            .push((name.to_string(), Box::new(move |_vm, args| f(args))));
        self
    }

    /// Register a raw function that needs VM access
    pub fn register_raw<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(&mut Vm, &[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.push((name.to_string(), Box::new(f)));
        self
    }

    /// Register a nullary function (no arguments)
    pub fn register_fn0<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn() -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.push((
            name.to_string(),
            Box::new(move |_vm, args| {
                if !args.is_empty() {
                    return Err(VmError::Runtime(format!(
                        "Expected 0 arguments, got {}",
                        args.len()
                    )));
                }
                f()
            }),
        ));
        self
    }

    /// Register a unary function (1 argument)
    pub fn register_fn1<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.push((
            name.to_string(),
            Box::new(move |_vm, args| {
                if args.len() != 1 {
                    return Err(VmError::Runtime(format!(
                        "Expected 1 argument, got {}",
                        args.len()
                    )));
                }
                f(args[0].clone())
            }),
        ));
        self
    }

    /// Register a binary function (2 arguments)
    pub fn register_fn2<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.push((
            name.to_string(),
            Box::new(move |_vm, args| {
                if args.len() != 2 {
                    return Err(VmError::Runtime(format!(
                        "Expected 2 arguments, got {}",
                        args.len()
                    )));
                }
                f(args[0].clone(), args[1].clone())
            }),
        ));
        self
    }

    /// Register a ternary function (3 arguments)
    pub fn register_fn3<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(Value, Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.functions.push((
            name.to_string(),
            Box::new(move |_vm, args| {
                if args.len() != 3 {
                    return Err(VmError::Runtime(format!(
                        "Expected 3 arguments, got {}",
                        args.len()
                    )));
                }
                f(args[0].clone(), args[1].clone(), args[2].clone())
            }),
        ));
        self
    }

    /// Get the list of functions in this module
    pub(crate) fn functions(self) -> Vec<(String, HostFunction)> {
        self.functions
    }
}

/// High-level API for embedding Fusabi in Rust applications
///
/// # Example
/// ```no_run
/// use fusabi::Engine;
/// use fusabi::Value;
///
/// let mut engine = Engine::new();
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
    vm: Vm,
    host_registry: Rc<RefCell<HostRegistry>>,
    global_bindings: HashMap<String, Value>,
}

impl FusabiEngine {
    /// Create a new Fusabi engine
    pub fn new() -> Self {
        let mut vm = Vm::new();

        // Register standard library functions and modules
        fusabi_vm::stdlib::register_stdlib(&mut vm);

        let host_registry = vm.host_registry.clone();

        FusabiEngine {
            vm,
            host_registry,
            global_bindings: HashMap::new(),
        }
    }

    /// Evaluate a Fusabi script and return the result
    ///
    /// # Example
    /// ```no_run
    /// use fusabi::Engine;
    ///
    /// let mut engine = Engine::new();
    /// let result = engine.eval("let x = 42 in x * 2").unwrap();
    /// assert_eq!(result.as_int(), Some(84));
    /// ```
    pub fn eval(&mut self, source: &str) -> Result<Value, crate::FusabiError> {
        self.eval_with_options(source, crate::RunOptions::default())
    }

    /// Evaluate a Fusabi script with type checking enabled
    ///
    /// # Example
    /// ```no_run
    /// use fusabi::Engine;
    ///
    /// let mut engine = Engine::new();
    /// let result = engine.eval_checked("let x: int = 42 in x * 2").unwrap();
    /// assert_eq!(result.as_int(), Some(84));
    /// ```
    pub fn eval_checked(&mut self, source: &str) -> Result<Value, crate::FusabiError> {
        let options = crate::RunOptions {
            enable_type_checking: true,
            ..Default::default()
        };
        self.eval_with_options(source, options)
    }

    /// Evaluate a Fusabi script with custom run options
    ///
    /// # Example
    /// ```no_run
    /// use fusabi::{Engine, RunOptions};
    ///
    /// let mut engine = Engine::new();
    /// let options = RunOptions {
    ///     enable_type_checking: true,
    ///     verbose: false,
    ///     strict_mode: true,
    /// };
    /// let result = engine.eval_with_options("let x = 42 in x * 2", options).unwrap();
    /// assert_eq!(result.as_int(), Some(84));
    /// ```
    pub fn eval_with_options(
        &mut self,
        source: &str,
        options: crate::RunOptions,
    ) -> Result<Value, crate::FusabiError> {
        // Stage 1: Lexical Analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        // Stage 2: Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;

        // Stage 3: Compilation (with optional type checking)
        let compile_options = CompileOptions {
            enable_type_checking: options.enable_type_checking,
            strict_mode: options.strict_mode,
            allow_warnings: !options.strict_mode,
        };
        let chunk = Compiler::compile_with_options(&ast, compile_options)?;

        // Stage 4: Execution
        let result = self.vm.execute(chunk)?;

        Ok(result)
    }

    /// Register a host function with dynamic arity
    ///
    /// # Example
    /// ```no_run
    /// # use fusabi::Engine;
    /// # use fusabi::Value;
    /// # use fusabi_vm::VmError;
    /// let mut engine = Engine::new();
    /// engine.register("sum", |args: &[Value]| -> Result<Value, VmError> {
    ///     let sum: i64 = args.iter()
    ///         .filter_map(|v: &Value| v.as_int())
    ///         .sum();
    ///     Ok(Value::Int(sum))
    /// });
    /// ```
    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&[Value]) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        // Wrap the closure to ignore the VM argument, maintaining backward compatibility
        self.host_registry
            .borrow_mut()
            .register(name, move |_vm, args| f(args));
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
        self.host_registry
            .borrow_mut()
            .register_fn0(name, move |_vm| f());
    }

    /// Register a unary host function (1 argument)
    pub fn register_fn1<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry
            .borrow_mut()
            .register_fn1(name, move |_vm, arg| f(arg));
    }

    /// Register a binary host function (2 arguments)
    pub fn register_fn2<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry
            .borrow_mut()
            .register_fn2(name, move |_vm, arg1, arg2| f(arg1, arg2));
    }

    /// Register a ternary host function (3 arguments)
    pub fn register_fn3<F>(&mut self, name: &str, f: F)
    where
        F: Fn(Value, Value, Value) -> Result<Value, VmError> + Send + Sync + 'static,
    {
        self.host_registry
            .borrow_mut()
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

    /// Register a module with namespaced functions
    ///
    /// # Example
    /// ```no_run
    /// use fusabi::{Engine, Module};
    /// use fusabi::Value;
    ///
    /// let mut engine = Engine::new();
    ///
    /// let fs_module = Module::new("fs")
    ///     .register_fn1("read", |path: Value| {
    ///         // Implementation here
    ///         Ok(Value::Str("file contents".to_string()))
    ///     })
    ///     .register_fn2("write", |path: Value, contents: Value| {
    ///         // Implementation here
    ///         Ok(Value::Unit)
    ///     });
    ///
    /// engine.register_module(fs_module);
    /// ```
    pub fn register_module(&mut self, module: Module) {
        let module_name = module.name().to_string();
        for (fn_name, f) in module.functions() {
            let full_name = format!("{}.{}", module_name, fn_name);
            self.host_registry.borrow_mut().register(&full_name, f);
        }
    }

    /// Create and return a host data value
    ///
    /// # Example
    /// ```no_run
    /// use fusabi::Engine;
    /// use std::sync::Mutex;
    ///
    /// struct EventStore {
    ///     events: Vec<String>,
    /// }
    ///
    /// let mut engine = Engine::new();
    /// let store = EventStore { events: vec![] };
    /// let store_value = engine.create_host_data(store, "EventStore");
    /// ```
    pub fn create_host_data<T: Any + 'static>(&self, data: T, type_name: &str) -> Value {
        Value::HostData(HostData::new(data, type_name))
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
        registry
            .call(name, &mut self.vm, args)
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
