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

### 1.3 Keywords (initial subset)

`let`, `rec`, `if`, `then`, `else`, `match`, `with`, `type`, `module`, `true`, `false`, `in`, `fun`

Operators and special tokens: `(` `)` `{` `}` `[` `]` `=` `->` `|` `:` `;` `,` `.` `*` `+` `-` `/` `::` `|>` `>>` `<<`

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

### 3.6 Pattern matching

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

### 3.7 Pipelines and composition

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
