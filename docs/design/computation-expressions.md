# Computation Expressions Design Specification

**Status**: Proposed
**Author**: Fusabi Core Team
**Date**: 2025-12-03
**Target Version**: 0.14.0+

---

## Table of Contents

1. [Introduction](#introduction)
2. [Motivation](#motivation)
3. [Proposed Syntax](#proposed-syntax)
4. [Desugaring Rules](#desugaring-rules)
5. [Builder Interface](#builder-interface)
6. [Standard Builders](#standard-builders)
7. [Implementation Roadmap](#implementation-roadmap)
8. [Examples](#examples)
9. [Open Questions](#open-questions)
10. [References](#references)

---

## 1. Introduction

Computation expressions (CEs) are a powerful syntactic feature from F# that provide a unified way to work with different computational contexts such as asynchronous operations, error handling, sequences, and custom DSLs. They are sometimes called "monadic syntax" in other functional languages.

A computation expression consists of:
- A **builder object** that defines the computational context
- A **special syntax** using the builder name followed by `{ }` braces
- **Keywords** like `let!`, `do!`, `return`, `yield` that desugar to builder method calls

### Example from F#

```fsharp
// F# async computation expression
async {
    let! data = fetchDataAsync url
    let processed = processData data
    return processed
}
```

This desugars to method calls on the `async` builder object, hiding the complexity of asynchronous programming behind clean, sequential-looking code.

---

## 2. Motivation

### Why Fusabi Needs Computation Expressions

Fusabi currently has excellent support for:
- Records, discriminated unions, and pattern matching
- Option types for safe error handling
- Higher-order functions and pipelines

However, certain patterns become verbose without CEs:

#### Problem 1: Nested Option Handling

```fusabi
// Current approach - deeply nested
let getUserInfo userId =
    match findUser userId with
    | None -> None
    | Some(user) ->
        match findAccount user.accountId with
        | None -> None
        | Some(account) ->
            match getPermissions account.id with
            | None -> None
            | Some(perms) ->
                Some({ user = user; account = account; perms = perms })
```

#### Problem 2: Error Propagation with Result Types

```fusabi
// Result type: type Result<'a, 'e> = Ok of 'a | Err of 'e

let validateAndProcess request =
    match validateEmail request.email with
    | Err(e) -> Err(e)
    | Ok(email) ->
        match validateAge request.age with
        | Err(e) -> Err(e)
        | Ok(age) ->
            match createUser email age with
            | Err(e) -> Err(e)
            | Ok(user) -> Ok(user)
```

#### Problem 3: Async Operations (Future Enhancement)

Once Fusabi adds async support, we'll need clean syntax for sequential async operations:

```fusabi
// Future: async operations
let processWorkflow url =
    fetchData url
    |> bindAsync (fun data -> transformAsync data)
    |> bindAsync (fun result -> saveAsync result)
```

### Solution: Computation Expressions

With CEs, the above examples become:

```fusabi
// Option builder
option {
    let! user = findUser userId
    let! account = findAccount user.accountId
    let! perms = getPermissions account.id
    return { user = user; account = account; perms = perms }
}

// Result builder
result {
    let! email = validateEmail request.email
    let! age = validateAge request.age
    let! user = createUser email age
    return user
}

// Future: async builder
async {
    let! data = fetchData url
    let! result = transformAsync data
    do! saveAsync result
    return result
}
```

### Benefits

1. **Readability**: Sequential-looking code that's actually composing operations
2. **Error Handling**: Automatic propagation of None/Err without manual matching
3. **Composability**: Different builders for different contexts (option, result, async, list)
4. **DSL Creation**: Custom builders enable domain-specific languages
5. **F# Compatibility**: Maintains Fusabi's goal of F# dialect consistency

---

## 3. Proposed Syntax

### Basic Structure

```fusabi
builderName {
    // computation expression body
}
```

### Supported Keywords

#### `let!` - Bind Operation
Unwraps a value from the computational context:

```fusabi
option {
    let! x = someOption  // If None, short-circuits; if Some(v), binds v to x
    let! y = anotherOption
    return x + y
}
```

#### `let` - Regular Let Binding
Normal let binding within the CE (no unwrapping):

```fusabi
option {
    let! x = someOption
    let doubled = x * 2  // Regular computation, not unwrapped
    return doubled
}
```

#### `do!` - Bind with Unit Result
Like `let!` but discards the result (used for side effects):

```fusabi
async {
    do! delay 1000       // Wait but don't bind the result
    let! data = fetchData url
    return data
}
```

#### `return` - Wrap Result
Wraps a value back into the computational context:

```fusabi
option {
    let! x = someValue
    return x + 1         // Returns Some(x + 1)
}
```

#### `return!` - Return Existing Wrapped Value
Returns an already-wrapped value without double-wrapping:

```fusabi
option {
    let! x = getValue1
    if x > 10 then
        return! getValue2  // getValue2 already returns Option
    else
        return x
}
```

#### `yield` - Sequence Builder Keyword
Used in sequence/list builders to produce elements:

```fusabi
seq {
    yield 1
    yield 2
    yield 3
}  // Produces sequence [1; 2; 3]
```

#### `yield!` - Yield Sequence
Yields all elements from a sequence:

```fusabi
seq {
    yield 1
    yield! [2; 3; 4]  // Yields each element
    yield 5
}  // Produces [1; 2; 3; 4; 5]
```

### Complete Example

```fusabi
result {
    // Regular let binding
    let threshold = 100

    // Bind unwraps Result
    let! user = findUser userId
    let! account = findAccount user.accountId

    // Regular computation
    let isValid = account.balance > threshold

    // Conditional return
    if isValid then
        return { user = user; account = account }
    else
        return! Err("Insufficient balance")  // Already a Result
}
```

---

## 4. Desugaring Rules

Computation expressions are syntactic sugar that desugar into method calls on builder objects. Here are the core transformation rules:

### 4.1 Simple Return

```fusabi
builder {
    return expr
}
```

Desugars to:

```fusabi
builder.return expr
```

### 4.2 Let-Bang (Bind)

```fusabi
builder {
    let! x = expr1
    expr2
}
```

Desugars to:

```fusabi
builder.bind expr1 (fun x ->
    // expr2 gets wrapped in builder context
    builder { expr2 }
)
```

### 4.3 Multiple Let-Bang

```fusabi
builder {
    let! x = expr1
    let! y = expr2
    return x + y
}
```

Desugars to:

```fusabi
builder.bind expr1 (fun x ->
    builder.bind expr2 (fun y ->
        builder.return (x + y)
    )
)
```

### 4.4 Regular Let Inside CE

```fusabi
builder {
    let x = expr1
    let! y = expr2
    return x + y
}
```

Desugars to:

```fusabi
let x = expr1 in
builder.bind expr2 (fun y ->
    builder.return (x + y)
)
```

### 4.5 Do-Bang

```fusabi
builder {
    do! expr1
    expr2
}
```

Desugars to:

```fusabi
builder.bind expr1 (fun () ->
    builder { expr2 }
)
```

### 4.6 Return-Bang

```fusabi
builder {
    return! expr
}
```

Desugars to:

```fusabi
builder.returnFrom expr
```

### 4.7 If-Then-Else Inside CE

```fusabi
builder {
    let! x = expr1
    if condition then
        return x
    else
        return! expr2
}
```

Desugars to:

```fusabi
builder.bind expr1 (fun x ->
    if condition then
        builder.return x
    else
        builder.returnFrom expr2
)
```

### 4.8 Yield (for sequence builders)

```fusabi
builder {
    yield expr
}
```

Desugars to:

```fusabi
builder.yield expr
```

### 4.9 Zero (Empty Body)

```fusabi
builder {
    if condition then
        return x
    // else branch is empty
}
```

Desugars to:

```fusabi
if condition then
    builder.return x
else
    builder.zero ()
```

---

## 5. Builder Interface

A computation expression builder is a record containing methods that implement the computational context. Different builders implement different subsets of these methods.

### 5.1 Core Builder Type

```fusabi
// Minimal builder interface
type Builder<'T> = {
    bind : 'T -> ('a -> 'T) -> 'T
    return : 'a -> 'T
}

// Extended builder interface
type ExtendedBuilder<'T> = {
    bind : 'T -> ('a -> 'T) -> 'T
    return : 'a -> 'T
    returnFrom : 'T -> 'T
    zero : unit -> 'T
    combine : 'T -> 'T -> 'T
    delay : (unit -> 'T) -> 'T
    run : 'T -> 'T
}
```

### 5.2 Method Descriptions

#### `bind : M<'a> -> ('a -> M<'b>) -> M<'b>`
The core monadic bind operation. Unwraps a value from context and passes it to a continuation.

**Used for**: `let!` and `do!` keywords

```fusabi
// Usage in CE:
let! x = someValue

// Desugars to:
builder.bind someValue (fun x -> ...)
```

#### `return : 'a -> M<'a>`
Wraps a plain value into the computational context.

**Used for**: `return` keyword

```fusabi
// Usage in CE:
return 42

// Desugars to:
builder.return 42
```

#### `returnFrom : M<'a> -> M<'a>`
Returns an already-wrapped value without double-wrapping.

**Used for**: `return!` keyword

```fusabi
// Usage in CE:
return! existingOption

// Desugars to:
builder.returnFrom existingOption
```

#### `zero : unit -> M<unit>`
Provides a "do-nothing" value for empty branches.

**Used for**: Missing else branches, empty sequences

```fusabi
// Usage in CE:
if condition then
    return x
// else is implicitly: builder.zero ()
```

#### `yield : 'a -> M<'a>`
Produces a single element in a sequence builder.

**Used for**: `yield` keyword (sequence builders only)

```fusabi
// Usage in CE:
yield 42

// Desugars to:
builder.yield 42
```

#### `combine : M<'a> -> M<'a> -> M<'a>`
Combines two computations sequentially.

**Used for**: Sequential statements in CE body

#### `delay : (unit -> M<'a>) -> M<'a>`
Delays evaluation of a computation.

**Used for**: Lazy evaluation control

#### `run : M<'a> -> M<'a>`
Final transformation when CE exits (optional).

**Used for**: Post-processing the entire computation

---

## 6. Standard Builders

Fusabi will ship with these standard computation expression builders in the standard library.

### 6.1 Option Builder

Handles `Option<'a>` types, short-circuiting on `None`.

```fusabi
// Definition (in stdlib)
module Option =
    let builder = {
        bind = fun opt f ->
            match opt with
            | None -> None
            | Some(x) -> f x

        return = fun x -> Some(x)

        returnFrom = fun opt -> opt

        zero = fun () -> None
    }

// Usage
let option = Option.builder

let getUserData userId =
    option {
        let! user = findUser userId
        let! profile = findProfile user.profileId
        return { user = user; profile = profile }
    }
```

### 6.2 Result Builder

Handles `Result<'a, 'e>` types, propagating errors.

```fusabi
// Definition (in stdlib)
type Result<'a, 'e> = Ok of 'a | Err of 'e

module Result =
    let builder = {
        bind = fun result f ->
            match result with
            | Err(e) -> Err(e)
            | Ok(x) -> f x

        return = fun x -> Ok(x)

        returnFrom = fun result -> result

        zero = fun () -> Ok(())
    }

// Usage
let result = Result.builder

let validateUser request =
    result {
        let! email = validateEmail request.email
        let! age = validateAge request.age
        let! password = validatePassword request.password
        return createUser email age password
    }
```

### 6.3 List Builder (Sequence)

Produces lists/sequences with `yield`.

```fusabi
// Definition (in stdlib)
module List =
    let builder = {
        bind = fun list f ->
            List.concat (List.map f list)

        return = fun x -> [x]

        returnFrom = fun list -> list

        yield = fun x -> [x]

        yieldFrom = fun list -> list

        combine = fun list1 list2 ->
            List.append list1 list2

        zero = fun () -> []
    }

// Usage
let list = List.builder

let numbers =
    list {
        yield 1
        yield 2
        yield! [3; 4; 5]
        yield 6
    }
// Result: [1; 2; 3; 4; 5; 6]
```

### 6.4 Async Builder (Future Enhancement)

For asynchronous operations (requires async runtime support).

```fusabi
// Definition (future - requires async support)
module Async =
    let builder = {
        bind = fun asyncOp f ->
            // Implementation depends on async runtime
            Async.bind asyncOp f

        return = fun x ->
            Async.return x

        returnFrom = fun asyncOp -> asyncOp

        zero = fun () ->
            Async.return ()
    }

// Usage (future)
let async = Async.builder

let fetchAndProcess url =
    async {
        let! data = Http.getAsync url
        let! parsed = Json.parseAsync data
        do! Logger.logAsync "Processing complete"
        return parsed
    }
```

---

## 7. Implementation Roadmap

Implementation is divided into four phases, building on Fusabi's existing infrastructure.

### Phase 1: Parser Changes (Week 1-2)

**Goal**: Extend parser to recognize CE syntax and create new AST nodes.

#### New AST Nodes

```rust
// In fusabi-frontend/src/ast.rs

pub enum Expr {
    // ... existing variants

    // New: Computation expression
    ComputationExpr {
        builder: Box<Expr>,      // The builder object expression
        body: Vec<CompStmt>,     // CE body statements
        span: Span,
    },
}

pub enum CompStmt {
    // let! x = expr
    LetBang {
        ident: String,
        value: Box<Expr>,
        span: Span,
    },

    // let x = expr
    Let {
        ident: String,
        value: Box<Expr>,
        span: Span,
    },

    // do! expr
    DoBang {
        expr: Box<Expr>,
        span: Span,
    },

    // return expr
    Return {
        expr: Box<Expr>,
        span: Span,
    },

    // return! expr
    ReturnBang {
        expr: Box<Expr>,
        span: Span,
    },

    // yield expr
    Yield {
        expr: Box<Expr>,
        span: Span,
    },

    // if-then-else inside CE
    IfThenElse {
        condition: Box<Expr>,
        then_stmts: Vec<CompStmt>,
        else_stmts: Option<Vec<CompStmt>>,
        span: Span,
    },
}
```

#### Parser Grammar Extensions

```
computation_expr :=
    expr "{" comp_stmt* "}"

comp_stmt :=
    | "let" "!" ident "=" expr
    | "let" ident "=" expr
    | "do" "!" expr
    | "return" "!" expr
    | "return" expr
    | "yield" "!" expr
    | "yield" expr
    | "if" expr "then" comp_stmt* ("else" comp_stmt*)?
    | expr
```

#### Tasks

- [ ] Add `CompStmt` and `ComputationExpr` to AST
- [ ] Implement parser for `builder { ... }` syntax
- [ ] Add parsing for `let!`, `do!`, `return`, `return!`, `yield`
- [ ] Handle if-then-else inside CE bodies
- [ ] Add error recovery for malformed CEs
- [ ] Write parser tests for all CE forms

**Estimated Time**: 10-15 days

---

### Phase 2: Type Checker Integration (Week 3-4)

**Goal**: Ensure CEs type-check correctly and builder methods have proper signatures.

#### Type Checking Rules

1. **Builder Object Type**: Must be a record with specific method signatures
2. **Let-Bang Type**: `let! x = expr` requires `expr : M<'a>` and `x : 'a`
3. **Return Type**: CE return type matches builder's wrapped type
4. **Method Presence**: Check that used keywords have corresponding builder methods

#### Builder Type Constraints

```fusabi
// Type checker validates:
// - If CE uses `let!`, builder must have `bind` field
// - If CE uses `return`, builder must have `return` field
// - If CE uses `yield`, builder must have `yield` field
// etc.

// Example builder type signature:
type OptionBuilder = {
    bind : Option<'a> -> ('a -> Option<'b>) -> Option<'b>
    return : 'a -> Option<'a>
    returnFrom : Option<'a> -> Option<'a>
    zero : unit -> Option<'a>
}
```

#### Tasks

- [ ] Add type checking for `ComputationExpr` nodes
- [ ] Validate builder object has required method fields
- [ ] Type-check each `CompStmt` variant
- [ ] Ensure proper type unification for bind operations
- [ ] Add helpful error messages for missing builder methods
- [ ] Write type checker tests for CEs

**Estimated Time**: 12-18 days

---

### Phase 3: Desugaring Pass (Week 5-6)

**Goal**: Transform CE AST nodes into regular function calls before codegen.

#### Desugaring Algorithm

The desugaring pass happens after type checking but before bytecode generation. It transforms `ComputationExpr` nodes into nested function applications.

```rust
// Pseudocode for desugaring

fn desugar_ce(builder: Expr, stmts: Vec<CompStmt>) -> Expr {
    desugar_stmts(builder, stmts, None)
}

fn desugar_stmts(
    builder: Expr,
    stmts: Vec<CompStmt>,
    cont: Option<Expr>
) -> Expr {
    match stmts {
        [] => cont.unwrap_or_else(|| {
            // Empty body: builder.zero()
            MethodCall(builder, "zero", [])
        }),

        [LetBang { ident, value }, ..rest] => {
            // let! x = value
            // rest
            //
            // Desugars to:
            // builder.bind value (fun x ->
            //     desugar_stmts(builder, rest, cont)
            // )
            MethodCall(
                builder.clone(),
                "bind",
                [
                    value,
                    Lambda {
                        params: [ident],
                        body: desugar_stmts(builder, rest, cont)
                    }
                ]
            )
        },

        [Return { expr }, ..rest] => {
            // return expr
            // Desugars to: builder.return expr
            MethodCall(builder, "return", [expr])
        },

        [ReturnBang { expr }, ..rest] => {
            // return! expr
            // Desugars to: builder.returnFrom expr
            MethodCall(builder, "returnFrom", [expr])
        },

        // ... handle other statement types
    }
}
```

#### Desugar Examples

**Example 1**: Simple Option CE

```fusabi
// Before desugaring:
option {
    let! x = findUser 1
    return x.name
}

// After desugaring:
option.bind (findUser 1) (fun x ->
    option.return x.name
)
```

**Example 2**: Multiple Binds

```fusabi
// Before desugaring:
result {
    let! x = getValue1
    let! y = getValue2 x
    return x + y
}

// After desugaring:
result.bind (getValue1) (fun x ->
    result.bind (getValue2 x) (fun y ->
        result.return (x + y)
    )
)
```

**Example 3**: If-Then-Else

```fusabi
// Before desugaring:
option {
    let! x = getX
    if x > 10 then
        return x
    else
        return! getDefault
}

// After desugaring:
option.bind (getX) (fun x ->
    if x > 10 then
        option.return x
    else
        option.returnFrom getDefault
)
```

#### Tasks

- [ ] Implement desugaring pass in compiler pipeline
- [ ] Handle all `CompStmt` variants
- [ ] Preserve spans for error messages
- [ ] Add desugaring tests with before/after AST comparison
- [ ] Optimize trivial cases (e.g., `return! expr` with no binds)

**Estimated Time**: 10-14 days

---

### Phase 4: Standard Builders & Documentation (Week 7-8)

**Goal**: Ship built-in builders and comprehensive examples.

#### Standard Library Additions

**File**: `stdlib/option.fsx` (or integrated into core)

```fusabi
module Option =
    // ... existing Option module functions

    let builder = {
        bind = fun opt f ->
            match opt with
            | None -> None
            | Some(x) -> f x

        return = fun x -> Some(x)

        returnFrom = fun opt -> opt

        zero = fun () -> None
    }
```

**File**: `stdlib/result.fsx`

```fusabi
type Result<'a, 'e> = Ok of 'a | Err of 'e

module Result =
    let bind result f =
        match result with
        | Err(e) -> Err(e)
        | Ok(x) -> f x

    let return x = Ok(x)

    let builder = {
        bind = bind
        return = return
        returnFrom = fun r -> r
        zero = fun () -> Ok(())
    }
```

**File**: `stdlib/list.fsx`

```fusabi
module List =
    // ... existing List functions

    let builder = {
        bind = fun list f ->
            List.concat (List.map f list)

        return = fun x -> [x]

        returnFrom = fun list -> list

        yield = fun x -> [x]

        yieldFrom = fun list -> list

        combine = List.append

        zero = fun () -> []
    }
```

#### Documentation

**File**: `docs/computation-expressions.md` (user guide)

- Introduction to CEs
- When to use CEs vs manual matching
- Built-in builders reference
- Custom builder creation guide
- Common patterns and idioms

**File**: `examples/ce_option.fsx`

```fusabi
// examples/ce_option.fsx
// Demonstrates option computation expressions

let option = Option.builder

// Example: Nested option handling
let findUserProfile userId =
    option {
        let! user = Database.findUser userId
        let! profile = Database.findProfile user.profileId
        let! avatar = Storage.loadAvatar profile.avatarId
        return {
            userName = user.name
            profileBio = profile.bio
            avatarUrl = avatar.url
        }
    }

// Without CE (for comparison)
let findUserProfile_manual userId =
    match Database.findUser userId with
    | None -> None
    | Some(user) ->
        match Database.findProfile user.profileId with
        | None -> None
        | Some(profile) ->
            match Storage.loadAvatar profile.avatarId with
            | None -> None
            | Some(avatar) ->
                Some({
                    userName = user.name
                    profileBio = profile.bio
                    avatarUrl = avatar.url
                })
```

#### Tasks

- [ ] Implement `Option.builder` in stdlib
- [ ] Implement `Result` type and `Result.builder`
- [ ] Implement `List.builder` with yield support
- [ ] Write comprehensive examples for each builder
- [ ] Add API documentation for builder methods
- [ ] Create migration guide from manual matching to CEs
- [ ] Add performance notes (CE overhead)

**Estimated Time**: 12-16 days

---

### Implementation Timeline Summary

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| **Phase 1**: Parser | 2 weeks | None |
| **Phase 2**: Type Checker | 2-3 weeks | Phase 1 complete |
| **Phase 3**: Desugaring | 2 weeks | Phase 2 complete |
| **Phase 4**: Stdlib & Docs | 2 weeks | Phase 3 complete |
| **Total** | **8-9 weeks** | Sequential |

---

## 8. Examples

### 8.1 Option Builder - Database Queries

```fusabi
let option = Option.builder

// Scenario: Fetch user with related data
let getUserDetails userId =
    option {
        let! user = findUser userId
        let! account = findAccount user.accountId
        let! settings = loadSettings user.id

        return {
            user = user
            account = account
            settings = settings
        }
    }

// Usage
match getUserDetails 42 with
| None -> print "User not found"
| Some(details) -> print details.user.name
```

### 8.2 Result Builder - Validation Pipeline

```fusabi
type ValidationError =
    | InvalidEmail of string
    | InvalidAge of string
    | InvalidPassword of string

type User = {
    email : string
    age : int
    password : string
}

let result = Result.builder

let validateEmail email =
    if String.contains "@" email then
        Ok(email)
    else
        Err(InvalidEmail("Email must contain @"))

let validateAge age =
    if age >= 18 && age <= 120 then
        Ok(age)
    else
        Err(InvalidAge("Age must be between 18 and 120"))

let validatePassword pwd =
    if String.length pwd >= 8 then
        Ok(pwd)
    else
        Err(InvalidPassword("Password must be at least 8 characters"))

// Validation pipeline using result CE
let createUser email age password =
    result {
        let! validEmail = validateEmail email
        let! validAge = validateAge age
        let! validPwd = validatePassword password

        return {
            email = validEmail
            age = validAge
            password = validPwd
        }
    }

// Usage
match createUser "test@example.com" 25 "secret123" with
| Ok(user) -> print ("Created user: " + user.email)
| Err(InvalidEmail(msg)) -> print ("Email error: " + msg)
| Err(InvalidAge(msg)) -> print ("Age error: " + msg)
| Err(InvalidPassword(msg)) -> print ("Password error: " + msg)
```

### 8.3 List Builder - Sequence Generation

```fusabi
let list = List.builder

// Generate a range with conditions
let evenRange start end =
    list {
        let mutable i = start
        while i <= end do
            if i % 2 == 0 then
                yield i
            i <- i + 1
    }

// Cartesian product
let cartesian list1 list2 =
    list {
        for x in list1 do
            for y in list2 do
                yield (x, y)
    }

// Usage
let evens = evenRange 1 10
// Result: [2; 4; 6; 8; 10]

let pairs = cartesian [1; 2] ["a"; "b"]
// Result: [(1, "a"); (1, "b"); (2, "a"); (2, "b")]
```

### 8.4 Custom Builder - Maybe with Logging

```fusabi
// Custom builder that logs each bind operation
type LoggedOption<'a> = {
    value : Option<'a>
    logs : string list
}

let loggedOption = {
    bind = fun logged f ->
        match logged.value with
        | None ->
            { value = None
              logs = logged.logs @ ["Bind: short-circuited on None"] }
        | Some(x) ->
            let result = f x
            { value = result.value
              logs = logged.logs @ ["Bind: continued with " + toString x] @ result.logs }

    return = fun x ->
        { value = Some(x)
          logs = ["Return: " + toString x] }

    returnFrom = fun logged -> logged

    zero = fun () ->
        { value = None
          logs = ["Zero: empty branch"] }
}

// Usage
let computation =
    loggedOption {
        let! x = { value = Some(10); logs = [] }
        let! y = { value = Some(20); logs = [] }
        return x + y
    }

print computation.value  // Some(30)
print computation.logs
// ["Bind: continued with 10", "Bind: continued with 20", "Return: 30"]
```

### 8.5 Before/After Comparison

#### Without Computation Expressions

```fusabi
// Deeply nested, hard to read
let processOrder orderId =
    match getOrder orderId with
    | None -> None
    | Some(order) ->
        match validateOrder order with
        | None -> None
        | Some(validated) ->
            match applyDiscount validated with
            | None -> None
            | Some(discounted) ->
                match chargePayment discounted with
                | None -> None
                | Some(charged) ->
                    match shipOrder charged with
                    | None -> None
                    | Some(shipped) -> Some(shipped)
```

#### With Computation Expressions

```fusabi
// Clean, sequential, readable
let processOrder orderId =
    option {
        let! order = getOrder orderId
        let! validated = validateOrder order
        let! discounted = applyDiscount validated
        let! charged = chargePayment discounted
        let! shipped = shipOrder charged
        return shipped
    }
```

---

## 9. Open Questions

### 9.1 Syntax Questions

**Q1**: Should we support applicative syntax (`and!`)?

F# has `let! x = ... and! y = ...` for parallel/independent operations.

```fusabi
// F# syntax
async {
    let! x = fetch1 ()
    and! y = fetch2 ()  // Can run in parallel
    return x + y
}
```

**Decision**: Defer to post-MVP. Requires more complex desugaring.

---

**Q2**: Should we support `use!` for resource management?

F# has `use!` for automatic disposal (IDisposable).

```fsharp
// F# syntax
async {
    use! file = openFileAsync path
    let! contents = readAsync file
    return contents
}  // file automatically closed
```

**Decision**: Not in initial implementation. Fusabi doesn't have resource management yet.

---

**Q3**: Should we allow custom keywords in builders?

Some F# libraries define custom keywords (e.g., `where`, `orderby` in query expressions).

**Decision**: Not in initial implementation. Standard keywords only.

---

### 9.2 Implementation Questions

**Q1**: Where does desugaring happen in the pipeline?

Options:
- A) During parsing (immediate desugaring to function calls)
- B) After type checking but before codegen
- C) During codegen

**Recommendation**: Option B. Allows type checker to understand CE structure for better error messages, but keeps codegen simple.

---

**Q2**: Should builders be first-class values or special syntax?

Options:
- A) Builders are regular record values (current proposal)
- B) Builders are a special language construct

**Recommendation**: Option A. More flexible, allows custom builders as library code.

---

**Q3**: Performance implications?

CEs desugar to nested function calls. Potential overhead:
- Lambda allocations
- Function call overhead

**Mitigation**:
- Inline builder methods during optimization passes
- Specialize common patterns (e.g., option chaining)
- Document performance characteristics

---

### 9.3 Compatibility Questions

**Q1**: How closely should we match F# semantics?

Trade-offs:
- Perfect F# compatibility makes porting code easier
- Fusabi-specific design could be simpler

**Recommendation**: Match F# as closely as possible for core features. Diverge only where necessary for implementation simplicity.

---

**Q2**: Should we support custom operators inside CEs?

F# allows custom operators like `>>=`, `<*>` to be defined for builders.

**Decision**: Not in initial implementation. Standard method names only.

---

## 10. References

### F# Documentation
- [F# Computation Expressions](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/computation-expressions)
- [F# Language Spec - Computation Expressions](https://fsharp.org/specs/language-spec/4.1/FSharpSpec-4.1-latest.pdf)

### Academic Papers
- "Monads for Functional Programming" - Philip Wadler (1992)
- "Composing Monads Using Coproducts" - Luth & Ghani (2002)

### Related Languages
- Haskell `do` notation
- Scala `for` comprehensions
- Idris Effects system

### Fusabi Internals
- [Language Spec](../02-language-spec.md)
- [VM Design](../03-vm-design.md)
- [Embedding Guide](../embedding-guide.md)

---

## Appendix A: Grammar Specification

Full BNF grammar for computation expressions:

```bnf
<computation-expr> ::= <expr> "{" <comp-stmt>* "}"

<comp-stmt> ::= <let-bang-stmt>
              | <let-stmt>
              | <do-bang-stmt>
              | <return-stmt>
              | <return-bang-stmt>
              | <yield-stmt>
              | <yield-bang-stmt>
              | <if-stmt>
              | <expr>

<let-bang-stmt> ::= "let" "!" <ident> "=" <expr>

<let-stmt> ::= "let" <ident> "=" <expr>

<do-bang-stmt> ::= "do" "!" <expr>

<return-stmt> ::= "return" <expr>

<return-bang-stmt> ::= "return" "!" <expr>

<yield-stmt> ::= "yield" <expr>

<yield-bang-stmt> ::= "yield" "!" <expr>

<if-stmt> ::= "if" <expr> "then" <comp-stmt>* ("else" <comp-stmt>*)?
```

---

## Appendix B: Type System Extensions

Type inference rules for computation expressions:

```
Γ ⊢ builder : Builder<M>
Γ ⊢ builder.bind : M<'a> -> ('a -> M<'b>) -> M<'b>
Γ ⊢ expr₁ : M<'a>
Γ, x : 'a ⊢ expr₂ : M<'b>
────────────────────────────────────────────────────
Γ ⊢ builder { let! x = expr₁; expr₂ } : M<'b>


Γ ⊢ builder : Builder<M>
Γ ⊢ builder.return : 'a -> M<'a>
Γ ⊢ expr : 'a
────────────────────────────────────────────────────
Γ ⊢ builder { return expr } : M<'a>
```

---

## Appendix C: Alternative Approaches Considered

### Approach 1: Macro-based CEs

Use a macro system to transform CE syntax.

**Pros**: More flexible, could support custom keywords
**Cons**: Fusabi doesn't have macros yet, adds complexity

**Decision**: Rejected. Direct AST transformation is simpler.

---

### Approach 2: Builder Methods as Functions

Instead of record fields, use module functions.

```fusabi
module Option =
    let bind opt f = ...
    let return x = ...

// Usage:
Option {  // Special syntax looks up module
    let! x = ...
}
```

**Pros**: More idiomatic for module-heavy code
**Cons**: Less flexible, harder to pass builders as values

**Decision**: Rejected. Record-based builders are more general.

---

### Approach 3: Type-class based Builders

Use type-classes/traits to define builder interface.

```fusabi
// Hypothetical type-class syntax
type Monad M =
    bind : M<'a> -> ('a -> M<'b>) -> M<'b>
    return : 'a -> M<'a>

instance Monad Option =
    bind = ...
    return = ...
```

**Pros**: More principled, better type inference
**Cons**: Fusabi doesn't have type-classes

**Decision**: Deferred. Could be future enhancement.

---

**End of Specification**
