---
layout: home

hero:
  name: Fusabi
  text: Mini-F# for Rust
  tagline: A small F# dialect with a Lua-style bytecode VM, designed for embedded scripting in Rust applications
  actions:
    - theme: brand
      text: Get Started
      link: /01-overview
    - theme: alt
      text: View on GitHub
      link: https://github.com/fusabi-lang/fusabi

features:
  - icon: 🦀
    title: Rust-Powered VM
    details: Lua-style bytecode interpreter with mark-and-sweep GC, optimized for embedded use cases
  - icon: 🔷
    title: F# Syntax
    details: Records, discriminated unions, pattern matching, pipelines, and computation expressions
  - icon: ⚡
    title: Fast Embedding
    details: Replace Lua in your Rust application with a strongly-typed scripting language
  - icon: 🛠️
    title: WezTerm Ready
    details: Designed as a Lua replacement for terminal emulators and similar applications
---

## Quick Example

```fsharp
// Define a record type
type Person = { name: string; age: int }

// Create and use records
let alice = { name = "Alice"; age = 30 }

// Pattern matching
let greet person =
    match person.age with
    | age when age < 18 -> "Hello, young " + person.name
    | _ -> "Hello, " + person.name

// Pipelines
alice |> greet |> print
```

## Installation

Add Fusabi to your Rust project:

```toml
[dependencies]
fusabi = "0.1"
```

Then embed it in your application:

```rust
use fusabi::{Vm, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = Vm::new();
    vm.eval_file("config.fsx")?;
    Ok(())
}
```
