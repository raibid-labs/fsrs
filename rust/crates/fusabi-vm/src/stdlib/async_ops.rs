// Fusabi Async Standard Library
// Provides runtime support for Computation Expressions (Async Builder)

use crate::value::Value;
use crate::vm::{Vm, VmError};
use std::sync::Arc;

// In this initial synchronous implementation:
// Async<'a> is represented as a Closure (unit -> 'a)
// This allows "executing" it by calling it.

/// Async.Return : 'a -> Async<'a>
/// Creates an async computation that returns a value
pub fn async_return(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime(format!(
            "Async.Return expects 1 argument, got {}",
            args.len()
        )));
    }

    let value = args[0].clone();

    // Create a thunk that returns the value
    // For Phase 1, we can't easily create a Closure from Rust code without compiling.
    // So we cheat: we'll use a special NativeFn that wraps the value?
    // Or better: The compiler desugars `async { return 1 }` to `Async.Return 1`.
    // The result of this call MUST be something that looks like an Async.
    // If we use `unit -> 'a` functions, `Async.Return 1` must return a function.

    // Problem: Creating a Fusabi function (Closure) from Rust is hard without compiling AST.
    // Solution: Represents Async as a Variant `Async(Value)`?
    // If we use a Variant `Async(NativeFn)` or `Async(Closure)`, we can distinguish it.

    // Let's define Async<'a> as a Variant:
    // Async.Thunk(closure) - a computation to be run
    // Async.Value(val)     - a completed value (optimization)

    // But `Bind` needs to compose them.
    // If we use `Async.Thunk`, then `Bind` takes a Thunk and a function.
    // `Bind` returns a NEW Thunk.
    // Executing that new Thunk runs the first one, then the second.

    // To make this work natively without compiling helper closures, we might need `Value::Async`.
    // But we don't have that.
    // Let's try to rely on the fact that the compiler can generate the lambdas.
    // But `Async.Return` is a function called AT RUNTIME.

    // Alternative: Since we are in `stdlib`, we can use `NativeFn` which holds a Rust closure.
    // So `Async.Return(x)` returns `Value::NativeFn` which when called returns `x`.

    // Let's define a helper to create a native thunk.
    let thunk = Value::NativeFn {
        name: "async_thunk".to_string(),
        arity: 1, // Takes unit
        args: vec![value], // Store the return value in partial application args
    };
    // Wait, NativeFn logic in VM usually executes a Rust function.
    // We can register a generic "return_thunk" function and return a NativeFn pointing to it,
    // with the value partially applied.

    // Register "Async.impl_return" as a helper?
    // Or just return a Variant wrapping the value, and `RunSynchronously` handles it.
    // Let's go with the Variant approach for inspecting structure, but a Function is more idiomatic for "computation".

    // Let's try the Variant approach for the *Async Data Type*:
    // type Async<'a> =
    //   | Pure of 'a
    //   | Bind of Async<'b> * ('b -> Async<'a>)
    //   | Delay of (unit -> Async<'a>)

    // This is a "Free Monad" structure. It allows the runtime (RunSynchronously) to evaluate it loop-free (trampoline).

    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Pure".to_string(),
        fields: vec![value],
    })
}

/// Async.Bind : Async<'a> -> ('a -> Async<'b>) -> Async<'b>
pub fn async_bind(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime("Async.Bind expects 2 arguments".to_string()));
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
pub fn async_delay(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime("Async.Delay expects 1 argument".to_string()));
    }

    let generator = args[0].clone();

    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Delay".to_string(),
        fields: vec![generator],
    })
}

/// Async.ReturnFrom : Async<'a> -> Async<'a>
pub fn async_return_from(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 1 {
        return Err(VmError::Runtime("Async.ReturnFrom expects 1 argument".to_string()));
    }
    Ok(args[0].clone())
}

/// Async.Zero : unit -> Async<unit>
pub fn async_zero(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::Variant {
        type_name: "Async".to_string(),
        variant_name: "Pure".to_string(),
        fields: vec![Value::Unit],
    })
}

/// Async.Combine : Async<unit> -> Async<'a> -> Async<'a>
/// Used for sequencing: do! a; b
pub fn async_combine(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime("Async.Combine expects 2 arguments".to_string()));
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
        return Err(VmError::Runtime("Async.RunSynchronously expects 1 argument".to_string()));
    }

    let mut current = args[0].clone();

    // We need a stack of continuations to handle Bind/Combine
    // Since Bind(m, f) means "run m, then run f result", we process 'm' and push 'f' to stack.
    let mut continuations: Vec<Value> = Vec::new();

    loop {
        match current {
            Value::Variant { variant_name, fields, .. } => {
                match variant_name.as_str() {
                    "Pure" => {
                        let result = fields[0].clone();
                        if let Some(continuation) = continuations.pop() {
                            // We have a continuation (function) waiting for this result
                            // Execute continuation(result) -> returns new Async
                            // Note: If continuation was from Combine, it might ignore the result (unit)
                            // But Combine is stored as [async1, async2].
                            // Wait, if we use Bind variant for everything, we need consistent handling.

                            // If we use standard Bind: fields = [computation, binder]
                            // We are currently processing "computation". We finished it and got "result".
                            // Now we call binder(result) -> next_computation
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
                            arity: 2, // takes (second, ignored_result)
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
                    _ => return Err(VmError::Runtime(format!("Unknown Async variant: {}", variant_name))),
                }
            }
            _ => return Err(VmError::TypeMismatch {
                expected: "Async variant",
                got: current.type_name(),
            }),
        }
    }
}

/// Helper function for Combine
/// Async.Internal.CombineHelper : Async<'a> -> 'b -> Async<'a>
/// Returns the first argument (the next computation), ignoring the second (the result of previous).
pub fn async_combine_helper(vm: &mut Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() != 2 {
        return Err(VmError::Runtime("CombineHelper expects 2 arguments".to_string()));
    }
    // args[0] is the next computation (captured)
    // args[1] is the result of the previous computation (ignored)
    Ok(args[0].clone())
}
