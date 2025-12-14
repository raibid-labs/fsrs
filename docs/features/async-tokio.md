# Async Tokio Integration

Fusabi provides real non-blocking async I/O backed by the Tokio runtime.

## Overview

When the `async` feature is enabled, Fusabi VM integrates with Tokio for:

- **Non-blocking I/O**: Sleep, HTTP, file operations without blocking
- **Parallel Execution**: Run multiple async tasks concurrently
- **Timeout Handling**: Cancel tasks that exceed time limits
- **Error Handling**: Catch and handle async failures

## Enabling Async

Add the `async` feature to your Cargo.toml:

```toml
[dependencies]
fusabi-vm = { version = "0.35.0", features = ["async"] }
```

## Standard Library Functions

### Async.sleep

Non-blocking sleep for specified milliseconds:

```fsharp
// Sleep for 1 second without blocking
let task = Async.sleep 1000
let result = Async.await task
```

### Async.parallel

Run multiple async tasks concurrently:

```fsharp
let task1 = Async.sleep 500
let task2 = Async.sleep 300
let task3 = Async.sleep 400

let tasks = [task1; task2; task3]
let parallel_task = Async.parallel tasks

// Wait for all tasks to complete
let results = Async.await parallel_task
```

### Async.withTimeout

Apply a timeout to an async task:

```fsharp
let long_task = Async.sleep 5000

// Timeout after 1 second
let timeout_task = Async.withTimeout 1000 long_task
let result = Async.await timeout_task

match result with
| Some(v) -> printfn "Task completed: %A" [v]
| None -> printfn "Task timed out!"
```

### Async.catch

Handle errors from async tasks:

```fsharp
let risky_task = doSomethingAsync()
let safe_task = Async.catch risky_task
let result = Async.await safe_task

match result with
| Ok(v) -> printfn "Success: %A" [v]
| Error(e) -> printfn "Error: %s" [e]
```

### Async.cancel

Cancel a running async task:

```fsharp
let task = Async.sleep 3000
let status = Async.poll task  // "Pending"

Async.cancel task

let status2 = Async.poll task  // "Cancelled"
```

### Async.poll

Non-blocking check of task status:

```fsharp
let task = Async.sleep 1000
let status = Async.poll task

match status with
| "Pending" -> printfn "Still running..."
| ("Ready", value) -> printfn "Completed: %A" [value]
| ("Failed", error) -> printfn "Failed: %s" [error]
| "Cancelled" -> printfn "Task was cancelled"
```

## API Usage (Rust)

### Enabling Async Runtime

```rust
use fusabi_vm::Vm;

let mut vm = Vm::new();
vm.enable_async()?;  // Initialize Tokio runtime
```

### Spawning Async Tasks

```rust
use fusabi_vm::{Vm, Value, VmError};

let task_id = vm.exec_async(|| {
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(Value::Int(42))
})?;

// Non-blocking poll
let state = vm.poll_async(task_id);

// Blocking wait
let result = vm.await_async(task_id)?;
```

### Task States

```rust
use fusabi_vm::AsyncState;

match state {
    AsyncState::Pending => { /* still running */ }
    AsyncState::Ready(value) => { /* completed with value */ }
    AsyncState::Failed(error) => { /* failed with error */ }
    AsyncState::Cancelled => { /* task was cancelled */ }
}
```

## Backward Compatibility

The async feature is additive. When disabled:

- Free-monad based async (`Async.Return`, `Async.Bind`, `Async.RunSynchronously`) continues to work
- Computation expression syntax unchanged
- No breaking changes to existing code

## Example

See the [async demo example](../../examples/async_tokio_demo.fsx) for a complete demonstration.

```fsharp
// Simple async sleep
let example1 () =
    let task = Async.sleep 1000
    Async.await task

// Parallel execution
let example2 () =
    let task1 = Async.sleep 500
    let task2 = Async.sleep 300
    let tasks = [task1; task2]
    let parallel = Async.parallel tasks
    Async.await parallel

// Run examples
let _ = example1()
let _ = example2()
```

## See Also

- [RFC-004: Async Tokio Integration](../design/RFC-004-ASYNC-TOKIO.md)
- [Example: async_tokio_demo.fsx](../../examples/async_tokio_demo.fsx)
