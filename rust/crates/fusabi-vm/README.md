# Fusabi VM

The high-performance bytecode virtual machine for the **Fusabi** scripting engine.

## Features

- **Stack-based**: Efficient execution model.
- **Garbage Collection**: Simple reference counting (Phase 1), moving to Mark-and-Sweep (Phase 4).
- **Host Interop**: Safe, re-entrant API for calling Rust from scripts and vice-versa.
- **Serialization**: Load pre-compiled `.fzb` bytecode.

## Usage

```rust
use fusabi_vm::{Vm, Value};

let mut vm = Vm::new();
// Register standard library
fusabi_vm::stdlib::register_stdlib(&mut vm);

// ... load chunk ...
// vm.execute(&chunk)?;
```

## License

MIT
