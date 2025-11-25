// Test method dispatch for HostData instances

use fusabi_frontend::ast::{Expr, Literal};
use fusabi_frontend::compiler::Compiler;
use fusabi_vm::value::{HostData, Value};
use fusabi_vm::vm::{Vm, VmError};

/// Sample host type - a counter
#[derive(Debug, Clone)]
struct Counter {
    value: i64,
}

impl Counter {
    fn new(initial: i64) -> Self {
        Counter { value: initial }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i64) {
        self.value += n;
    }

    fn get_value(&self) -> i64 {
        self.value
    }
}

#[test]
fn test_method_dispatch_basic() {
    // Create a counter instance
    let counter = Counter::new(0);
    let host_data = HostData::new(counter, "Counter");

    // Register the increment method
    let mut vm = Vm::new();
    {
        let mut registry = vm.host_registry.borrow_mut();
        registry.register_method::<Counter, _>("increment", |_vm, args| {
            // First arg is receiver
            if args.is_empty() {
                return Err(VmError::Runtime("Expected receiver".into()));
            }

            let receiver = &args[0];
            match receiver {
                Value::HostData(hd) => {
                    if let Some(mut counter) = hd.try_borrow_mut::<Counter>() {
                        counter.increment();
                        Ok(Value::Unit)
                    } else {
                        Err(VmError::Runtime("Type mismatch".into()))
                    }
                }
                _ => Err(VmError::Runtime("Expected HostData".into())),
            }
        });

        registry.register_method::<Counter, _>("value", |_vm, args| {
            if args.is_empty() {
                return Err(VmError::Runtime("Expected receiver".into()));
            }

            let receiver = &args[0];
            match receiver {
                Value::HostData(hd) => {
                    if let Some(counter) = hd.try_borrow::<Counter>() {
                        Ok(Value::Int(counter.get_value()))
                    } else {
                        Err(VmError::Runtime("Type mismatch".into()))
                    }
                }
                _ => Err(VmError::Runtime("Expected HostData".into())),
            }
        });
    }

    // Store counter in a global variable
    vm.globals
        .insert("counter".to_string(), Value::HostData(host_data));

    // Test: counter.increment()
    let source = "counter.increment()";
    let expr = parse_simple_method_call("counter", "increment", vec![]);

    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Unit);

    // Test: counter.value()
    let expr = parse_simple_method_call("counter", "value", vec![]);
    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(1));

    // Call increment again
    let expr = parse_simple_method_call("counter", "increment", vec![]);
    let chunk = Compiler::compile(&expr).unwrap();
    vm.execute(chunk).unwrap();

    // Check value again
    let expr = parse_simple_method_call("counter", "value", vec![]);
    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(2));
}

#[test]
fn test_method_dispatch_with_args() {
    let counter = Counter::new(10);
    let host_data = HostData::new(counter, "Counter");

    let mut vm = Vm::new();
    {
        let mut registry = vm.host_registry.borrow_mut();
        registry.register_method::<Counter, _>("add", |_vm, args| {
            if args.len() < 2 {
                return Err(VmError::Runtime("Expected receiver and argument".into()));
            }

            let receiver = &args[0];
            let arg = &args[1];

            match (receiver, arg) {
                (Value::HostData(hd), Value::Int(n)) => {
                    if let Some(mut counter) = hd.try_borrow_mut::<Counter>() {
                        counter.add(*n);
                        Ok(Value::Unit)
                    } else {
                        Err(VmError::Runtime("Type mismatch".into()))
                    }
                }
                _ => Err(VmError::Runtime("Type mismatch".into())),
            }
        });

        registry.register_method::<Counter, _>("value", |_vm, args| {
            if args.is_empty() {
                return Err(VmError::Runtime("Expected receiver".into()));
            }

            let receiver = &args[0];
            match receiver {
                Value::HostData(hd) => {
                    if let Some(counter) = hd.try_borrow::<Counter>() {
                        Ok(Value::Int(counter.get_value()))
                    } else {
                        Err(VmError::Runtime("Type mismatch".into()))
                    }
                }
                _ => Err(VmError::Runtime("Expected HostData".into())),
            }
        });
    }

    vm.globals
        .insert("counter".to_string(), Value::HostData(host_data));

    // Test: counter.add(5)
    let expr = parse_simple_method_call("counter", "add", vec![Expr::Lit(Literal::Int(5))]);
    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Unit);

    // Check value
    let expr = parse_simple_method_call("counter", "value", vec![]);
    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_method_not_found() {
    let counter = Counter::new(0);
    let host_data = HostData::new(counter, "Counter");

    let mut vm = Vm::new();
    vm.globals
        .insert("counter".to_string(), Value::HostData(host_data));

    // Try to call a method that doesn't exist
    let expr = parse_simple_method_call("counter", "nonexistent", vec![]);
    let chunk = Compiler::compile(&expr).unwrap();
    let result = vm.execute(chunk);

    assert!(result.is_err());
    match result {
        Err(VmError::Runtime(msg)) => {
            assert!(msg.contains("Method not found"));
        }
        _ => panic!("Expected Runtime error"),
    }
}

// Helper function to parse a simple method call expression
fn parse_simple_method_call(receiver_name: &str, method_name: &str, args: Vec<Expr>) -> Expr {
    Expr::MethodCall {
        receiver: Box::new(Expr::Var(receiver_name.to_string())),
        method_name: method_name.to_string(),
        args,
    }
}
