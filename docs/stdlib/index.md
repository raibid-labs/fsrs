# Standard Library

Fusabi's standard library provides essential modules for common programming tasks.

## Modules

| Module | Description |
|--------|-------------|
| [List](./list) | Immutable list operations |
| [Map](./map) | Key-value dictionary operations |
| [Option](./option) | Safe handling of optional values |
| [String](./string) | String manipulation utilities |

## Usage

Standard library modules are available by default:

```fsharp
// List operations
let nums = [1; 2; 3; 4; 5]
let doubled = List.map (fun x -> x * 2) nums

// Option handling
let maybeValue = Some 42
let result = Option.map (fun x -> x + 1) maybeValue

// String operations
let greeting = String.concat " " ["Hello"; "World"]
```

See [Stdlib Reference](/STDLIB_REFERENCE) for complete API documentation.
