# Mini‑F# Dialect Specification (Draft)

This document defines the initial Mini‑F# dialect that `fsrs` will support.

The language is intentionally small and expression‑oriented.

## 1. Lexical elements

### 1.1 Identifiers

- Start with a letter or `_`, followed by letters, digits, or `_`.
- Examples: `foo`, `fooBar`, `_internal`, `Pane`, `Direction`.

### 1.2 Literals

- Integers: `0`, `42`, `-3`
- Floats: `3.14`, `-0.5`
- Booleans: `true`, `false`
- Strings: `"hello"`, `"tab: " + name`
- Unit: `()` (used rarely; mostly for host interop)
- Tuples: `(1, 2)`, `(x, "hello", true)`
- Lists: `[]`, `[1; 2; 3]`, `[[1; 2]; [3; 4]]`
- Arrays: `[||]`, `[|1; 2; 3|]`, `[|[|1; 2|]; [|3; 4|]|]`

### 1.3 Keywords (initial subset)

`let`, `rec`, `if`, `then`, `else`, `match`, `with`, `type`, `module`, `true`, `false`, `in`, `fun`

Operators and special tokens: `(` `)` `{` `}` `[` `]` `[|` `|]` `=` `->` `|` `:` `;` `,` `.` `*` `+` `-` `/` `::` `|>` `>>` `<<` `<-`

## 2. Types

Built‑in primitive types:

- `int`, `float`, `bool`, `string`, `unit`

Composite:

- Tuple: `int * string`
- List: `int list`
- Array: `int array`
- Option: `'a option` (syntactic sugar for a DU `None | Some of 'a`)

User‑defined:

- Record types:

  ```fsharp
  type TabInfo =
    { Title: string
      Index: int
      ProcessName: string
      Cwd: string }
  ```

- Discriminated unions:

  ```fsharp
  type Direction = Left | Right | Up | Down

  type Action =
    | Split of Direction
    | MoveFocus of Direction
    | SendKeys of string
    | RenameTab of string
  ```

## 3. Expressions

### 3.1 Let bindings

Top‑level and local:

```fsharp
let x = 1

let y =
  let z = x + 2
  z * 2
```

Recursive:

```fsharp
let rec fact n =
  if n <= 1 then 1
  else n * fact (n - 1)
```

### 3.2 Functions and application

```fsharp
let add x y = x + y

let inc = add 1

let result = inc 41
```

Curried functions only (no tupled arguments yet, for simplicity).

### 3.3 Conditionals

```fsharp
let describe n =
  if n < 0 then "neg"
  else if n = 0 then "zero"
  else "pos"
```

### 3.4 Tuples

Tuples are heterogeneous, fixed-size collections of values:

```fsharp
// Creating tuples
let pair = (1, 2)
let triple = (42, "hello", true)
let nested = ((1, 2), (3, 4))

// Tuples in bindings
let coordinates = (10, 20) in
let point3d = (x, y, z) in
```

Tuples are displayed with comma separators: `(1, 2, 3)`

### 3.5 Lists

Lists are homogeneous, immutable sequences. FSRS supports:

#### List literals

```fsharp
// Empty list
let empty = []

// List with elements (semicolon-separated)
let numbers = [1; 2; 3; 4; 5]

// Nested lists
let matrix = [[1; 2]; [3; 4]; [5; 6]]
```

#### Cons operator (`::`）

The cons operator prepends an element to a list. It is right-associative.

```fsharp
// Build list with cons
let list1 = 1 :: 2 :: 3 :: []        // [1; 2; 3]

// Prepend to existing list
let numbers = [2; 3; 4] in
let extended = 1 :: numbers          // [1; 2; 3; 4]

// Nested list construction
let nested = [1; 2] :: [3; 4] :: []  // [[1; 2]; [3; 4]]
```

#### List properties

- Lists are printed with semicolon separators: `[1; 2; 3]`
- Empty list is `[]` (also called `Nil`)
- Lists can be nested: `[[1; 2]; [3; 4]]`
- Lists support structural equality: `[1; 2] = [1; 2]` is `true`

### 3.6 Arrays

Arrays are mutable, indexed collections with F# syntax. FSRS provides full array support with immutable update semantics.

#### Array Literals

Arrays use `[|...|]` delimiters to distinguish them from lists `[...]`:

```fsharp
// Empty array
let empty = [||]

// Array with elements (semicolon-separated)
let numbers = [|1; 2; 3; 4; 5|]

// Nested arrays (matrices)
let matrix = [|[|1; 2|]; [|3; 4|]; [|5; 6|]|]
```

#### Array Indexing

Arrays support zero-based indexing using the `.[index]` syntax:

```fsharp
let arr = [|10; 20; 30; 40; 50|]
let first = arr.[0]   // 10
let third = arr.[2]   // 30
let last = arr.[4]    // 50

// Nested array indexing
let matrix = [|[|1; 2|]; [|3; 4|]|]
let element = matrix.[1].[0]  // 3
```

**Index bounds checking**:
- Valid indices: `0` to `length - 1`
- Out-of-bounds access results in a runtime error
- Negative indices are not supported

#### Array Updates

Array updates use the `<-` operator and follow **immutable semantics**. Each update creates a new array, preserving the original:

```fsharp
let arr = [|1; 2; 3; 4; 5|]

// Simple update (creates new array)
let arr2 = arr.[1] <- 99  // [|1; 99; 3; 4; 5|]

// Original array is unchanged
print arr   // [|1; 2; 3; 4; 5|]
print arr2  // [|1; 99; 3; 4; 5|]

// Chained updates (left-to-right evaluation)
let result = arr.[0] <- 10.[1] <- 20.[2] <- 30
// Result: [|10; 20; 30; 4; 5|]

// Update nested arrays
let matrix = [|[|1; 2|]; [|3; 4|]|]
let updated = matrix.[0] <- [|99; 88|]
// updated: [|[|99; 88|]; [|3; 4|]|]
```

**Update semantics**:
- `arr.[i] <- value` returns a **new array** with updated value at index `i`
- Original array remains unchanged (immutable semantics)
- Updates can be chained: `arr.[0] <- 10.[1] <- 20`
- Out-of-bounds updates result in runtime errors

#### Array.length

The `Array.length` function returns the number of elements in an array:

```fsharp
let numbers = [|1; 2; 3; 4; 5|]
let len = Array.length numbers  // 5

// Empty array
let empty = [||]
Array.length empty  // 0

// Nested arrays
let matrix = [|[|1; 2; 3|]; [|4; 5|]; [|6|]|]
Array.length matrix          // 3 (outer array)
Array.length matrix.[0]      // 3 (first row)
Array.length matrix.[1]      // 2 (second row)
```

#### Array Properties

- Arrays are printed with `[|...|]` delimiters and semicolon separators: `[|1; 2; 3|]`
- Empty array is `[||]`
- Arrays can be nested: `[|[|1; 2|]; [|3; 4|]|]`
- Arrays support structural equality: `[|1; 2|] = [|1; 2|]` is `true`
- Arrays can contain mixed types: `[|1; true; "hello"|]`
- Array updates are immutable (create new arrays)

#### Example Usage

```fsharp
// Basic array operations
let empty = [||]
let numbers = [|1; 2; 3; 4; 5|]
let first = numbers.[0]
let len = Array.length numbers

// Immutable updates
let arr = [|10; 20; 30|]
let updated = arr.[1] <- 99
// arr is still [|10; 20; 30|]
// updated is [|10; 99; 30|]

// Nested arrays (matrices)
let matrix = [|[|1; 2|]; [|3; 4|]; [|5; 6|]|]
let row = matrix.[0]      // [|1; 2|]
let elem = matrix.[1].[0]  // 3
```

### 3.7 Pattern matching

Over literals, tuples, records, and DUs:

```fsharp
let describeDirection d =
  match d with
  | Left -> "left"
  | Right -> "right"
  | Up -> "up"
  | Down -> "down"

let describeTab tab =
  match tab with
  | { ProcessName = "cargo" } -> "build"
  | { ProcessName = "npm" } -> "node"
  | _ -> "other"
```

### 3.8 Pipelines and composition

```fsharp
let normalizeTitle (title: string) =
  title
  |> String.trim
  |> String.toLower

let f = g >> h   // f x = h (g x)
```

In the core AST, these become ordinary function calls.

## 4. Modules

Single file, multiple modules:

```fsharp
module Layouts =
  let default = // ...

module Keys =
  let bindings = // ...

module Config =
  type Config = { Layout : Layout; KeyBindings : KeyBinding list }
  let config : Config = { Layout = Layouts.default; KeyBindings = Keys.bindings }
```

The front‑end maintains a symbol table keyed by `ModuleName.identifier`.

## 5. Computation expressions (CEs)

We support a minimal subset of F# CEs for domain‑specific DSLs. The CE support is implemented via **desugaring only**; the VM sees only function calls and lambdas.

Examples:

```fsharp
layout {
  row {
    pane { cmd "htop"; width 30 }
    column {
      pane { cmd "cargo watch -x test" }
      pane { cmd "cargo watch -x run" }
    }
  }
}

keys {
  bind "Ctrl-Shift-H" (MoveFocus Left)
  bind "Ctrl-Shift-L" (MoveFocus Right)
}
```

Rough desugaring strategy is described in `03-vm-design.md`.

## 6. Omitted features (for v1)

The initial dialect intentionally omits:

- Classes, interfaces, inheritance.
- Type providers.
- Units of measure.
- Active patterns (can be added later).
- Overloads and operator customisation beyond a fixed set.
- Full `do`/`while` loops (these can be emulated with recursion and higher‑order functions).

The idea is to converge quickly on a compact core that is easy to embed and optimize.
