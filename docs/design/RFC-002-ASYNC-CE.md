# RFC-002: Async Computation Expressions

**Status**: Draft
**Author**: Claude (AI Assistant)
**Created**: 2025-12-03

## Summary

This RFC proposes the design for async computation expressions in Fusabi, enabling non-blocking I/O operations using familiar F#-style syntax.

## Motivation

Currently, Fusabi operations are synchronous. For TUI applications and network operations, async support is essential for responsive user interfaces.

### Problems Solved
- Blocking I/O freezes the application
- No way to run concurrent operations
- Callback-based async is verbose and error-prone

## Proposed Syntax

### Basic Async Block
```fusabi
async {
    let! data = fetchData url
    let! processed = processData data
    return processed
}
```

### Keywords
- `async { ... }` - Creates an async computation
- `let!` - Awaits an async operation and binds result
- `do!` - Awaits an async operation, discards result
- `return` - Wraps value in completed async
- `return!` - Returns existing async directly

### Async Module Signature
```fusabi
module Async =
    // Core builder methods
    val bind : Async<'a> -> ('a -> Async<'b>) -> Async<'b>
    val return : 'a -> Async<'a>
    val returnFrom : Async<'a> -> Async<'a>
    val zero : unit -> Async<unit>
    val delay : (unit -> Async<'a>) -> Async<'a>
    val combine : Async<unit> -> Async<'a> -> Async<'a>

    // Execution
    val runSynchronously : Async<'a> -> 'a
    val start : Async<unit> -> unit
    val startWithContinuation : Async<'a> -> ('a -> unit) -> (exn -> unit) -> unit

    // Utilities
    val sleep : int -> Async<unit>
    val parallel : Async<'a> list -> Async<'a list>
    val sequential : Async<'a> list -> Async<'a list>
    val catch : Async<'a> -> Async<Result<'a, exn>>
    val ignore : Async<'a> -> Async<unit>
```

## Desugaring Rules

### let! Desugaring
```fusabi
// Source
async {
    let! x = asyncOp
    body
}

// Desugars to
Async.bind asyncOp (fun x -> body)
```

### do! Desugaring
```fusabi
// Source
async {
    do! asyncOp
    body
}

// Desugars to
Async.bind asyncOp (fun () -> body)
```

### return Desugaring
```fusabi
// Source
async { return value }

// Desugars to
Async.return value
```

### Nested let!
```fusabi
// Source
async {
    let! x = op1
    let! y = op2 x
    return x + y
}

// Desugars to
Async.bind op1 (fun x ->
    Async.bind (op2 x) (fun y ->
        Async.return (x + y)))
```

## Implementation Strategy

### Phase 1: Runtime Support
1. Define `Async<'a>` type in VM
2. Implement core Async module functions
3. Add task scheduler to VM

### Phase 2: Syntax Support
1. Add `async` keyword to lexer
2. Add `let!`, `do!` to parser
3. Implement CE desugaring in compiler

### Phase 3: Standard Library
1. Async versions of I/O functions
2. Timer and sleep support
3. Parallel execution utilities

## Examples

### HTTP Fetch
```fusabi
let fetchJson url = async {
    let! response = Http.getAsync url
    let! body = response.readBodyAsync ()
    return Json.parse body
}
```

### Parallel Operations
```fusabi
let fetchAll urls = async {
    let! results = Async.parallel (List.map fetchJson urls)
    return results
}
```

### Error Handling
```fusabi
let safeFetch url = async {
    let! result = Async.catch (fetchJson url)
    match result with
    | Ok data -> return Some data
    | Error _ -> return None
}
```

### TUI Event Loop
```fusabi
let rec eventLoop model = async {
    let! event = Events.nextAsync ()
    let newModel = update event model
    do! render newModel
    if newModel.running then
        return! eventLoop newModel
    else
        return ()
}
```

## Open Questions

1. **Cancellation**: Should we support CancellationToken?
2. **Exception Propagation**: How to handle errors in parallel operations?
3. **Tail Recursion**: Can async loops be tail-recursive?
4. **Interop**: How to integrate with host async runtimes (Tokio)?

## Alternatives Considered

### Promises/Futures
- Pros: Familiar to JS developers
- Cons: Not idiomatic F#, less composable

### Callbacks
- Pros: Simple implementation
- Cons: Callback hell, hard to reason about

### Go-style Channels
- Pros: Simple concurrency model
- Cons: Different paradigm, harder migration from F#

## References

- [F# Async Programming](https://docs.microsoft.com/en-us/dotnet/fsharp/tutorials/async)
- [Computation Expressions](https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/computation-expressions)
- [RFC-001: Computation Expressions](./computation-expressions.md)
