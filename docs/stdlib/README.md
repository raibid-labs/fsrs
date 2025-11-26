# Fusabi Standard Library

The Fusabi standard library provides essential modules for functional programming with immutable data structures and type-safe operations.

## Quick Reference

| Module | Purpose | Key Functions |
|--------|---------|---------------|
| [List](list.md) | Immutable linked lists | length, head, tail, map, reverse, append |
| [String](string.md) | String operations | length, trim, split, concat, toUpper, toLower |
| [Option](option.md) | Optional values | isSome, isNone, map, bind, defaultValue |
| [Map](map.md) | Key-value dictionaries | add, find, tryFind, containsKey, toList |

## Overview

All standard library modules follow functional programming principles:
- **Immutability**: Operations return new values instead of modifying existing ones
- **Type Safety**: Generic types ensure compile-time correctness
- **Composability**: Functions work seamlessly with the pipeline operator `|>`
- **No Side Effects**: Pure functions (except for I/O operations)

## Module Summaries

### List Module

The List module provides operations for working with immutable cons-based lists. Lists are the fundamental collection type in functional programming.

**Common operations:**
```fsharp
let numbers = [1; 2; 3; 4; 5] in
let count = List.length numbers in           // 5
let doubled = List.map (fun x -> x * 2) numbers in  // [2; 4; 6; 8; 10]
let reversed = List.reverse numbers          // [5; 4; 3; 2; 1]
```

**Learn more:** [List Module Documentation](list.md)

---

### String Module

The String module provides text manipulation functions with full Unicode support. All operations are immutable and return new strings.

**Common operations:**
```fsharp
let text = "  Hello World  " in
let cleaned = text
    |> String.trim
    |> String.toLower in                     // "hello world"

let words = String.split " " cleaned in      // ["hello"; "world"]
let rejoined = String.concat words           // "helloworld"
```

**Learn more:** [String Module Documentation](string.md)

---

### Option Module

The Option module provides type-safe handling of values that may or may not exist, eliminating null reference errors.

**Common operations:**
```fsharp
let someValue = Some(42) in
let noValue = None in

let value = Option.defaultValue 0 someValue in      // 42
let defaulted = Option.defaultValue 0 noValue in    // 0

let doubled = Option.map (fun x -> x * 2) someValue // Some(84)
```

**Learn more:** [Option Module Documentation](option.md)

---

### Map Module

The Map module provides immutable key-value dictionaries with string keys. Maps are efficient for lookups and updates while maintaining immutability.

**Common operations:**
```fsharp
let config = Map.empty()
    |> Map.add "host" "localhost"
    |> Map.add "port" 8080 in

let host = Map.find "host" config in                // "localhost"
let maybePort = Map.tryFind "port" config in        // Some(8080)
let hasDebug = Map.containsKey "debug" config       // false
```

**Learn more:** [Map Module Documentation](map.md)

---

## Getting Started

### Basic Example

```fsharp
// Using multiple stdlib modules together
let processData input =
    input
    |> String.trim
    |> String.split ","
    |> List.map String.trim
    |> List.map String.toUpper
    |> String.concat " | "

let result = processData "  hello,  world,  fusabi  "
// Result: "HELLO | WORLD | FUSABI"
```

### Working with Options

```fsharp
// Safe configuration lookup
let getConfig key configMap =
    configMap
    |> Map.tryFind key
    |> Option.defaultValue "default"

let config = Map.empty()
    |> Map.add "env" "production" in

let env = getConfig "env" config        // "production"
let region = getConfig "region" config  // "default"
```

### Combining Collections

```fsharp
// Building a data pipeline
let names = ["alice"; "bob"; "charlie"] in
let nameMap = names
    |> List.map (fun name -> (name, String.length name))
    |> Map.ofList

let aliceLen = Map.find "alice" nameMap  // 5
```

## Usage Patterns

### Pipeline Composition

The pipeline operator `|>` works seamlessly with all stdlib functions:

```fsharp
let result = data
    |> String.trim
    |> String.toLower
    |> String.split " "
    |> List.map String.toUpper
    |> List.filter (fun s -> String.length s > 3)
    |> String.concat ", "
```

### Option Chaining

Combine Option operations for safe data transformations:

```fsharp
let processUser userId database =
    database
    |> Map.tryFind userId
    |> Option.map (fun user -> user.name)
    |> Option.map String.toUpper
    |> Option.defaultValue "UNKNOWN"
```

### List Transformations

Transform and aggregate list data functionally:

```fsharp
let scores = [85; 92; 78; 95; 88] in
let topScores = scores
    |> List.filter (fun s -> s >= 90)
    |> List.reverse
// Result: [95; 92]
```

## Type Safety

The standard library is fully generic and type-safe:

```fsharp
// Type inference works automatically
let numbers = [1; 2; 3]              // int list
let strings = ["a"; "b"; "c"]        // string list
let mixed = [1; "a"; 2]              // Type error!

// Generic functions preserve types
let doubled = List.map (fun x -> x * 2) numbers  // int list
let lengths = List.map String.length strings     // int list
```

## Performance Characteristics

### List Operations
- Most operations are O(n) where n is the list length
- head and tail are O(1)
- Lists are best for sequential access

### String Operations
- Most operations are O(n) where n is the string length
- Strings are Unicode-aware (character count ≠ byte count)
- All operations create new strings

### Option Operations
- All operations are O(1) plus the cost of the applied function
- Options have minimal memory overhead
- Perfect for representing nullable values

### Map Operations
- Lookups are O(1) average case
- Add/remove are O(n) due to immutability
- Maps are best for frequent lookups

## Common Idioms

### Safe List Access

```fsharp
let safeHead list =
    if List.isEmpty list
    then None
    else Some(List.head list)

let first = safeHead [1; 2; 3]  // Some(1)
let noHead = safeHead []        // None
```

### Building Maps from Data

```fsharp
let buildUserMap users =
    users
    |> List.map (fun user -> (user.id, user))
    |> Map.ofList
```

### Validating Required Fields

```fsharp
let validateRequired fields data =
    let checkField field =
        Map.containsKey field data in
    List.map checkField fields
```

### String Processing Pipeline

```fsharp
let sanitize input =
    input
    |> String.trim
    |> String.toLower
    |> String.split " "
    |> List.map String.trim
    |> List.filter (fun s -> String.length s > 0)
```

## Integration with Language Features

### Pattern Matching

```fsharp
let describe opt =
    match opt with
    | Some(value) -> "Has: " + toString value
    | None -> "Empty"

let msg = describe (Some(42))  // "Has: 42"
```

### Higher-Order Functions

```fsharp
let transform f list =
    List.map f list

let doubled = transform (fun x -> x * 2) [1; 2; 3]
```

### Records and Maps

```fsharp
type User = { name: string; age: int }

let user = { name = "Alice"; age = 30 } in
let userData = Map.empty()
    |> Map.add "name" user.name
    |> Map.add "age" user.age
```

## Best Practices

1. **Use Option instead of null**: Options make absence explicit and type-safe
2. **Leverage pipelines**: Chain operations with `|>` for clarity
3. **Prefer immutability**: Let stdlib functions create new values
4. **Use tryFind for Maps**: Safer than find for unknown keys
5. **Type annotations rarely needed**: Type inference handles most cases
6. **Combine modules**: List + String + Option work great together

## Examples

### Configuration Management

```fsharp
let parseConfig input =
    input
    |> String.trim
    |> String.split "\n"
    |> List.map (String.split "=")
    |> List.map (fun parts ->
        let key = List.head parts in
        let value = List.head (List.tail parts) in
        (key, value))
    |> Map.ofList
```

### Data Validation

```fsharp
let validateUser user =
    let hasName = Map.containsKey "name" user in
    let hasEmail = Map.containsKey "email" user in
    if hasName && hasEmail
    then Some(user)
    else None
```

### Text Analysis

```fsharp
let wordFrequency text =
    text
    |> String.toLower
    |> String.split " "
    |> List.map String.trim
    |> List.filter (fun w -> String.length w > 0)
    |> List.length
```

## See Also

- [Language Specification](../02-language-spec.md) - F# language features
- [Embedding Guide](../embedding-guide.md) - Using Fusabi in Rust
- [Examples Directory](../../examples/) - Working code examples

## Contributing

Found a bug in the stdlib? Have an idea for a new function?

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for how to contribute.

## Version

Current version: 0.15.0

All documented functions are stable and safe to use in production.
