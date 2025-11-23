# VM Design (Draft)

This document outlines the core VM for the Mini‑F# dialect.

The VM is:

- A **stack‑based bytecode interpreter**.
- Designed for **embedding** in Rust applications.
- Intended to be **good enough** performance‑wise to replace Lua for configs + callbacks.

## 1. Runtime value representation

A single `Value` enum represents all runtime values:

```rust
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
    Tuple(Vec<Value>),
    Cons { head: Box<Value>, tail: Box<Value> },
    Nil,
    Array(Rc<RefCell<Vec<Value>>>),
    Record(Rc<RefCell<HashMap<String, Value>>>),
    Variant {
        type_name: String,
        variant_name: String,
        fields: Vec<Value>,
    },
    Closure(Rc<Closure>),
    NativeFn {
        name: String,
        arity: u8,
        args: Vec<Value>, // Partially applied arguments
    },
}
```

Where:

- `Rc<RefCell<T>>` is used for mutable/shared heap objects (Phase 1 GC).
- `Closure` contains the `Chunk` and captured `Upvalue`s.
- `NativeFn` supports partial application natively.

## 2. Bytecode

Bytecode is organised in **chunks**, one per function:

```rust
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>, // literals, function refs, etc.
    pub name: Option<String>,
}
```

### 2.1 Instructions (current set)

```rust
pub enum Instruction {
    LoadConst(u16),        // push constants[idx]
    LoadLocal(u16),        // push locals[idx]
    StoreLocal(u16),       // pop -> locals[idx]
    LoadGlobal(u16),       // push globals[name_idx] (name in constants)
    LoadUpvalue(u16),      // push captured upvalue
    StoreUpvalue(u16),     // pop -> captured upvalue
    Pop,                   // pop1

    // Arithmetic & Logic
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, Lte, Gt, Gte,
    And, Or, Not,

    // Construction
    MakeTuple(u16),        // pop N -> tuple
    MakeList(u16),         // pop N -> list (from stack elements)
    Cons,                  // pop 2 -> Cons cell
    MakeArray(u16),        // pop N -> array
    MakeRecord(u16),       // pop 2*N -> record (key, value pairs)
    MakeClosure(u16, u16), // const_idx (proto), upvalue_count

    // Access
    GetField(u16),         // pop record, push field (name in constants)
    ArrayGet,              // pop array, index -> push value
    ArraySet,              // pop array, index, value -> update
    ListHead,
    ListTail,
    IsNil,

    // Control Flow
    Jump(i16),
    JumpIfFalse(i16),
    Call(u8),              // arg_count
    Return,
}
```

## 3. Call frames and VM loop

A `Frame` holds:

```rust
pub struct Frame {
    pub chunk: ChunkId,      // which function
    pub ip: usize,           // instruction pointer
    pub base: usize,         // base index into the VM value stack
}
```

The VM maintains:

```rust
pub struct Vm {
    pub stack: Vec<Value>,
    pub frames: Vec<Frame>,
    pub globals: Vec<Value>,
    pub heap: Heap,          // GC arena, depending on implementation
}
```

Pseudo‑code for the interpreter loop:

```rust
loop {
    let instr = current_frame.fetch_next();
    match instr {
        Instruction::LoadConst(idx) => { push(constants[idx]); }
        Instruction::Add => { let b = pop_int(); let a = pop_int(); push_int(a + b); }
        Instruction::Call(arg_count) => { setup_new_frame(func_value, arg_count); }
        Instruction::Return => { tear_down_frame(); if frames.is_empty() { break; } }
        // ...
    }
}
```

Start simple; optimise later.

## 4. Compilation pipeline

Front‑end steps:

1. Parse source into AST.
2. Typecheck (HM) and annotate AST with concrete types.
3. Desugar:
   - Pipelines → nested function calls.
   - CEs → builder calls.
   - Pattern matches → decision trees.
4. Compile typed AST to bytecode:
   - Map each function to a `Chunk`.
   - Build constant pools.
   - Emit instruction sequences with jumps.

### 4.1 Pattern match compilation

Compile:

```fsharp
match v with
| Left -> e1
| Right -> e2
| _ -> e3
```

Rough strategy:

1. Evaluate `v` → stack top.
2. Emit:

   ```text
   MatchTag(Direction, Left, L1)
   // matched Left:
   drop scrutinee or bind variables
   code for e1
   Jump L_end

   L1:
   MatchTag(Direction, Right, L2)
   // matched Right:
   code for e2
   Jump L_end

   L2:
   // default:
   code for e3

   L_end:
   ```

For records and tuples, you add `GetField` / `DESTRUCT` instructions as needed.

## 5. GC plan

Phase 1: **simplest thing that works**:

- Use `Rc<RefCell<T>>` and accept non‑moving semantics and minor leaks when cycles happen.
- This is enough to get correctness while you develop front‑end and VM.

Phase 2: introduce a proper arena + mark‑and‑sweep:

- Heap stores objects in vectors keyed by an index type.
- On GC:
  - Start from stack, globals, upvalues as roots.
  - Mark reachable heap objects.
  - Sweep unreached objects.

Tuning will depend on benchmarks (e.g., config size, callback frequency).

## 6. Host interop and built‑ins

The VM exposes built‑ins as `Value::NativeFn`:

```rust
pub type HostFn = dyn Fn(&mut Vm, &[Value]) -> Result<Value, VmError>;
```

The host registers built‑ins via `HostRegistry` (shared via `Rc<RefCell<HostRegistry>>`):

```rust
engine.register_fn1("print", |v| {
    println!("{}", v);
    Ok(Value::Unit)
});
```

Features:
- **Re-entrancy**: Host functions receive `&mut Vm`, allowing them to call back into the VM (e.g., `List.map` taking a closure).
- **Partial Application**: `NativeFn` values store partially applied arguments, allowing standard F# currying.
- **Type Safety**: Helper methods `as_int`, `as_str`, etc. on `Value` simplify argument extraction.

## 7. Performance notes

To approach Lua‑class performance:

- Avoid heavy dynamic features:
  - No reflection.
  - No dynamic field lookup by string; use `FieldId` indices.
- After functionality is stable:
  - Consider NaN‑boxing for numeric/bool/ptr values.
  - Inline hot opcodes (e.g., arithmetic, comparisons).
  - Add simple bytecode peephole optimisations.

However, for the intended use (configs + callbacks), a straightforward Rust interpreter should already be sufficient.
