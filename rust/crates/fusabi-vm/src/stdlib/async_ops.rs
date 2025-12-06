// Fusabi Async Standard Library
// Provides runtime support for Computation Expressions (Async Builder)

use crate::value::Value;
use crate::vm::{Vm, VmError};

// In this initial synchronous implementation:
// Async<'a> is represented as a Closure (unit -> 'a)
// This allows "executing" it by calling it.

/// Async.Return : 'a -> Async<'a>
/// Creates an async computation that returns a value
pub fn async_return(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Async.Return expects 1 argument, got {}",
            args.len()
        )));
    }

    let value = args[0].clone();

    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Pure".to_string(),
        fields: vec![value],
    })
}

/// Async.Bind : Async<'a> -> ('a -> Async<'b>) -> Async<'b>
pub fn async_bind(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(
            "Async.Bind expects 2 arguments".to_string(),
        ));
    }

    let computation = args[0].clone();
    let binder = args[1].clone();

    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Bind".to_string(),
        fields: vec![computation, binder],
    })
}

/// Async.Delay : (unit -> Async<'a>) -> Async<'a>
pub fn async_delay(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(
            "Async.Delay expects 1 argument".to_string(),
        ));
    }

    let generator = args[0].clone();

    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Delay".to_string(),
        fields: vec![generator],
    })
}

/// Async.ReturnFrom : Async<'a> -> Async<'a>
pub fn async_return_from(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(
            "Async.ReturnFrom expects 1 argument".to_string(),
        ));
    }
    Ok(args[0].clone())
}

/// Async.Zero : unit -> Async<unit>
pub fn async_zero(_vm: &mut Vm, _args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Pure".to_string(),
        fields: vec![Value::Unit],
    })
}

/// Async.Combine : Async<unit> -> Async<'a> -> Async<'a>
/// Used for sequencing: do! a; b
pub fn async_combine(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(
            "Async.Combine expects 2 arguments".to_string(),
        ));
    }

    let first = args[0].clone();
    let second = args[1].clone();

    // Bind(first, fun () -> second)
    // But we can't easily create the lambda "fun () -> second" from Rust.
    // However, if we define Combine in terms of the variant structure:
    // We can introduce a specific "Combine" variant, or just use Bind if we can create the lambda.

    // Simpler: Let's introduce a Combine variant to the Free Monad.
    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Combine".to_string(),
        fields: vec![first, second],
    })
}

/// Async.RunSynchronously : Async<'a> -> 'a
/// Interprets the Async Free Monad
pub fn async_run_synchronously(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(
            "Async.RunSynchronously expects 1 argument".to_string(),
        ));
    }

    let mut current = args[0].clone();

    // We need a stack of continuations to handle Bind/Combine
    // Since Bind(m, f) means "run m, then run f result", we process 'm' and push 'f' to stack.
    let mut continuations: Vec<Value> = Vec::new();

    loop {
        match current {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                match variant_name.as_str() {
                    "Pure" => {
                        let result = fields[0].clone();
                        if let Some(continuation) = continuations.pop() {
                            // We have a continuation (function) waiting for this result
                            // Execute continuation(result) -> returns new Async
                            current = vm.call_value(continuation, &[result])?;
                        } else {
                            // No more continuations, this is the final result
                            return Ok(result);
                        }
                    }
                    "Bind" => {
                        let computation = fields[0].clone();
                        let binder = fields[1].clone();
                        // Push binder to continuations
                        continuations.push(binder);
                        // Process inner computation
                        current = computation;
                    }
                    "Combine" => {
                        let first = fields[0].clone();
                        let second = fields[1].clone();
                        // Combine(a, b) is semantically Bind(a, fun _ -> b)
                        // We need to construct a "const b" function?
                        // Or handle "Combine" specially in the continuation stack?
                        // Let's make continuations support specific actions.
                        // But Value::NativeFn is hard to synthesize.

                        // Hack: Register a "CombineContinuation" variant in our internal logic?
                        // No, let's just optimize:
                        // We push 'second' as a "computation to run next", ignoring the previous result.
                        // But 'continuations' stores FUNCTIONS.
                        // We need to wrap 'second' in a function `fun _ -> second`.

                        // Since we are in Rust, we can cheat again.
                        // Let's create a native function closure that captures 'second' and returns it.
                        // Warning: This requires `vm` reference which we have.
                        // BUT `Value` cannot hold a Rust closure directly unless it's `HostData`.
                        // `Value::NativeFn` holds name+args. We can't create anonymous ones easily.

                        // Alternative: Push a special marker to `continuations`?
                        // But `vm.call_value` expects a function.

                        // Let's define `Async.Combine` as `Bind` in the compiler desugaring?
                        // The compiler currently desugars `stmt; rest` to `Combine`.
                        // If we change `Async.Combine` to return `Bind(first, fun _ -> second)`...
                        // We still have the problem of creating `fun _ -> second`.

                        // Okay, let's handle "Combine" explicitly in the interpreter loop.
                        // We need a stack of "Tasks".
                        // Frame: (Binder(func) | Next(async))
                        // This loop structure is getting complex for a simple `impl`.

                        // RESTARTING STRATEGY:
                        // Desugar `Combine(a, b)` at the COMPILER level to `Bind(a, fun _ -> b)`?
                        // Yes, `compiler.rs` already does `Delay(fun _ -> rest)`.
                        // So `Combine(a, b)` is `Combine(a, Delay(fun _ -> b))` in the compiler.
                        // The `Async.Combine` implementation here just needs to handle that.
                        // Signature: Combine : Async<unit> -> Async<'a> -> Async<'a>
                        // But wait, if the second arg is ALREADY a Delay (which is Async),
                        // then Combine just takes two Asyncs.

                        // Let's treat Combine as Bind where we discard the result.
                        // We need to create a binder function that returns `second`.
                        // `async_combiner_helper(ignored_val, second_computation)`
                        // We can register a native helper `Async.Internal.CombineHelper`
                        // and return `Bind(first, NativeFn("CombineHelper", [second]))`.

                        let helper = Value::NativeFn {
                            name: "Async.Internal.CombineHelper".to_string(),
                            arity: 2,           // takes (second, ignored_result)
                            args: vec![second], // Partially applied 'second'
                        };

                        // Return Bind(first, helper)
                        current = Value::Variant {
                            type_name: "Async".to_string(),
                            variant_name: "Bind".to_string(),
                            fields: vec![first, helper],
                        };
                    }
                    "Delay" => {
                        let generator = fields[0].clone();
                        // Call generator(unit) -> async_computation
                        current = vm.call_value(generator, &[Value::Unit])?;
                    }
                    _ => {
                        return Err(VmError::Runtime(format!(
                            "Unknown Async variant: {}",
                            variant_name
                        )))
                    }
                }
            }
            _ => {
                return Err(VmError::TypeMismatch {
                    expected: "Async variant",
                    got: current.type_name(),
                })
            }
        }
    }
}

/// Helper function for Combine
/// Async.Internal.CombineHelper : Async<'a> -> 'b -> Async<'a>
/// Returns the first argument (the next computation), ignoring the second (the result of previous).
pub fn async_combine_helper(_vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime(
            "CombineHelper expects 2 arguments".to_string(),
        ));
    }
    // args[0] is the next computation (captured)
    // args[1] is the result of the previous computation (ignored)
    Ok(args[0].clone())
}
