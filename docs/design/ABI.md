# Fusabi ABI Specification

This document describes the Application Binary Interface (ABI) for Fusabi, including the internal runtime value representation and the bytecode file format.

## Table of Contents

- [Runtime Value Representation](#runtime-value-representation)
- [Bytecode File Format (.fzb)](#bytecode-file-format-fzb)
- [Instruction Set Reference](#instruction-set-reference)
- [Serialization Details](#serialization-details)
- [Host Interop](#host-interop)

## Runtime Value Representation

### The `Value` Enum

All runtime values in Fusabi are represented by the `Value` enum:

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
    Variant { type_name: String, variant_name: String, fields: Vec<Value> },
    Closure(Rc<Closure>),
    NativeFn { name: String, arity: u8, args: Vec<Value> },
    HostData(HostData),
}
```

### Value Types

#### Primitive Types

| Type | Description | Example | Memory Layout |
|------|-------------|---------|---------------|
| `Int(i64)` | 64-bit signed integer | `42`, `-100` | Stack-allocated |
| `Bool(bool)` | Boolean value | `true`, `false` | Stack-allocated |
| `Str(String)` | Heap-allocated UTF-8 string | `"hello"` | Heap pointer |
| `Unit` | Unit type (void) | `()` | Zero-sized |

#### Structured Types

##### Tuple

Heterogeneous fixed-size sequence of values.

```rust
Tuple(Vec<Value>)
```

**Examples**:
- `(1, 2)` → `Tuple(vec![Int(1), Int(2)])`
- `(true, "hello", 42)` → `Tuple(vec![Bool(true), Str("hello"), Int(42)])`

**Operations**:
- Access: `GetTupleField(index)`
- Construction: `MakeTuple(size)`
- Pattern matching: `CheckTupleLen`, `GetTupleElem`

##### List (Cons Cells)

Immutable singly-linked list using cons cells.

```rust
Cons { head: Box<Value>, tail: Box<Value> }
Nil
```

**Structure**:
```
[1; 2; 3] →
Cons { head: Int(1), tail:
  Cons { head: Int(2), tail:
    Cons { head: Int(3), tail: Nil }
  }
}
```

**Operations**:
- Construction: `Cons`, `MakeList(count)`
- Deconstruction: `ListHead`, `ListTail`
- Check: `IsNil`

##### Array

Mutable, indexed collection with `Rc<RefCell<_>>` for shared ownership.

```rust
Array(Rc<RefCell<Vec<Value>>>)
```

**Examples**:
- `[|1; 2; 3|]` → `Array(Rc::new(RefCell::new(vec![Int(1), Int(2), Int(3)])))`

**Operations**:
- Construction: `MakeArray(size)`
- Access: `ArrayGet` (pops index, array; pushes value)
- Mutation: `ArraySet` (pops value, index, array; mutates in-place)
- Length: `ArrayLength`
- Immutable update: `ArrayUpdate` (creates new array)

**Mutability**: Arrays use interior mutability via `RefCell`. Mutations are checked at runtime.

##### Record

Mutable key-value store using `Rc<RefCell<HashMap<String, Value>>>`.

```rust
Record(Rc<RefCell<HashMap<String, Value>>>)
```

**Examples**:
- `{ x = 1; y = 2 }` → `Record(Rc::new(RefCell::new(map! { "x" => Int(1), "y" => Int(2) })))`

**Operations**:
- Construction: `MakeRecord(field_count)` (pops 2*N values: field names and values)
- Access: `GetRecordField` (pops field_name, record; pushes value)
- Update: `UpdateRecord(field_count)` (creates new record with updated fields)

**Mutability**: Records are immutable from the VM's perspective. Updates create new instances.

##### Discriminated Union (Variant)

Tagged union with a type name, variant name, and field values.

```rust
Variant {
    type_name: String,     // e.g., "Option"
    variant_name: String,  // e.g., "Some"
    fields: Vec<Value>,    // e.g., [Int(42)]
}
```

**Examples**:
- `Some(42)` → `Variant { type_name: "Option", variant_name: "Some", fields: vec![Int(42)] }`
- `None` → `Variant { type_name: "Option", variant_name: "None", fields: vec![] }`

**Operations**:
- Construction: `MakeVariant(field_count)`
- Pattern matching: `CheckVariantTag(variant_name)`
- Field access: `GetVariantField(index)`

#### Function Types

##### Closure

Function with captured upvalues (lexically scoped variables).

```rust
Closure(Rc<Closure>)

pub struct Closure {
    pub chunk: Chunk,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

pub enum Upvalue {
    Open(usize),      // Stack index (not yet closed)
    Closed(Value),    // Captured value
}
```

**Creation**: `MakeClosure(chunk_idx, upvalue_count)`
**Calling**: `Call(argc)`, `TailCall(argc)`

##### Native Function

Host-provided function with optional partial application.

```rust
NativeFn {
    name: String,       // Qualified name (e.g., "List.map")
    arity: u8,          // Total parameter count
    args: Vec<Value>,   // Partially applied arguments
}
```

**Partial Application**:
```
List.map        → NativeFn { name: "List.map", arity: 2, args: [] }
List.map f      → NativeFn { name: "List.map", arity: 2, args: [f] }
List.map f list → (calls the function)
```

#### Host Interop

##### HostData

Opaque wrapper for Rust objects exposed to scripts.

```rust
HostData {
    data: Rc<RefCell<dyn Any>>,
    type_name: String,
}
```

**Usage**:
```rust
// In Rust host:
let config = MyConfig { port: 8080 };
let host_data = HostData::new(config, "MyConfig");
vm.globals.insert("config", Value::HostData(host_data));

// In F# script:
// config is accessible but opaque
// Host functions can downcast and use it
```

**Important**: `HostData` is **not serializable**. It cannot appear in `.fzb` bytecode files.

### Type Name Mapping

| F# Type | Value Type | Type Name String |
|---------|------------|------------------|
| `int` | `Int(i64)` | `"int"` |
| `bool` | `Bool(bool)` | `"bool"` |
| `string` | `Str(String)` | `"string"` |
| `unit` | `Unit` | `"unit"` |
| `'a * 'b` | `Tuple(Vec<Value>)` | `"tuple"` |
| `'a list` | `Cons` / `Nil` | `"list"` |
| `'a[]` | `Array(...)` | `"array"` |
| `{ field: 'a }` | `Record(...)` | `"record"` |
| `Option<'a>`, etc. | `Variant {...}` | `"variant"` |
| `'a -> 'b` | `Closure` / `NativeFn` | `"function"` |

## Bytecode File Format (.fzb)

### File Structure

```
+-------------------+
| Magic Bytes (4)   |  "FZB\x01"
+-------------------+
| Version (1)       |  0x01
+-------------------+
| Payload (var)     |  Bincode-serialized Chunk
+-------------------+
```

### Header Format

| Offset | Size | Field | Value | Description |
|--------|------|-------|-------|-------------|
| 0 | 4 | Magic | `FZB\x01` | File type identifier |
| 4 | 1 | Version | `0x01` | Bytecode format version |
| 5 | var | Payload | Bincode | Serialized `Chunk` struct |

### Magic Bytes

The first 4 bytes must be exactly:
```
0x46 0x5A 0x42 0x01
"F"  "Z"  "B"  0x01
```

This identifies the file as a Fusabi Binary.

### Version

Current version: **1**

Future versions may change:
- Instruction encoding
- Value serialization
- Chunk structure

The VM checks this byte and rejects incompatible versions.

### Payload

The payload is the `Chunk` struct serialized with `bincode` (little-endian):

```rust
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub name: Option<String>,
}
```

#### Bincode Encoding

- **Vec**: Length (varint) + elements
- **Option**: 0 (None) or 1 (Some) + value
- **String**: Length (varint) + UTF-8 bytes
- **Enum**: Discriminant (u32) + fields

See [Bincode Spec](https://github.com/bincode-org/bincode) for details.

## Instruction Set Reference

### Stack Operations

| Instruction | Arguments | Description | Stack Effect |
|-------------|-----------|-------------|--------------|
| `LoadConst` | `u16` idx | Push constant from pool | `[] → [value]` |
| `LoadLocal` | `u8` idx | Push local variable | `[] → [value]` |
| `StoreLocal` | `u8` idx | Pop and store to local | `[value] → []` |
| `LoadUpvalue` | `u8` idx | Push captured upvalue | `[] → [value]` |
| `StoreUpvalue` | `u8` idx | Pop and store to upvalue | `[value] → []` |
| `LoadGlobal` | `u16` idx | Push global by name | `[] → [value]` |
| `Pop` | - | Discard top of stack | `[value] → []` |
| `Dup` | - | Duplicate top of stack | `[value] → [value, value]` |

### Arithmetic Operations

| Instruction | Stack Effect | Description |
|-------------|--------------|-------------|
| `Add` | `[a, b] → [a+b]` | Integer addition |
| `Sub` | `[a, b] → [a-b]` | Integer subtraction |
| `Mul` | `[a, b] → [a*b]` | Integer multiplication |
| `Div` | `[a, b] → [a/b]` | Integer division (error if b=0) |

### Comparison Operations

| Instruction | Stack Effect | Description |
|-------------|--------------|-------------|
| `Eq` | `[a, b] → [a==b]` | Equality |
| `Neq` | `[a, b] → [a!=b]` | Inequality |
| `Lt` | `[a, b] → [a<b]` | Less than |
| `Lte` | `[a, b] → [a<=b]` | Less than or equal |
| `Gt` | `[a, b] → [a>b]` | Greater than |
| `Gte` | `[a, b] → [a>=b]` | Greater than or equal |

### Logical Operations

| Instruction | Stack Effect | Description |
|-------------|--------------|-------------|
| `And` | `[a, b] → [a&&b]` | Logical AND |
| `Or` | `[a, b] → [a||b]` | Logical OR |
| `Not` | `[a] → [!a]` | Logical NOT |

### Control Flow

| Instruction | Arguments | Description | Stack Effect |
|-------------|-----------|-------------|--------------|
| `Jump` | `i16` offset | Unconditional jump | - |
| `JumpIfFalse` | `i16` offset | Jump if top is falsy | `[cond] → []` |
| `Call` | `u8` argc | Call function with N args | `[func, arg1, ..., argN] → [result]` |
| `TailCall` | `u8` argc | Tail-recursive call | Same as `Call` |
| `Return` | - | Return from function | `[result] → (caller's stack)` |

### Tuple Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeTuple` | `u16` size | `[v1, ..., vN] → [tuple]` | Create tuple from N values |
| `GetTupleField` | `u8` index | `[tuple] → [value]` | Extract field by index |
| `GetTupleElem` | `u8` index | `[tuple] → [tuple, value]` | Get element (keeps tuple) |
| `CheckTupleLen` | `u8` len | `[tuple] → [tuple, bool]` | Check tuple length |

### List Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeList` | `u16` count | `[v1, ..., vN] → [list]` | Build list from N values |
| `Cons` | - | `[head, tail] → [list]` | Create cons cell |
| `ListHead` | - | `[list] → [head]` | Get list head (error if empty) |
| `ListTail` | - | `[list] → [tail]` | Get list tail (error if empty) |
| `IsNil` | - | `[list] → [bool]` | Check if list is empty |

### Array Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeArray` | `u16` size | `[v1, ..., vN] → [array]` | Create array from N values |
| `ArrayGet` | - | `[array, index] → [value]` | Get element by index |
| `ArraySet` | - | `[array, index, value] → [unit]` | Set element (mutable) |
| `ArrayLength` | - | `[array] → [length]` | Get array length |
| `ArrayUpdate` | - | `[array, index, value] → [new_array]` | Update (immutable) |

### Record Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeRecord` | `u16` count | `[k1, v1, ..., kN, vN] → [record]` | Create record from N pairs |
| `GetRecordField` | - | `[record, field_name] → [value]` | Get field by name |
| `UpdateRecord` | `u16` count | `[record, k1, v1, ..., kN, vN] → [record]` | Update record (immutable) |

### Discriminated Union Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeVariant` | `u16` field_count | `[type, variant, f1, ..., fN] → [variant]` | Create variant |
| `CheckVariantTag` | `String` tag | `[variant] → [bool]` | Check variant tag |
| `GetVariantField` | `u8` index | `[variant] → [field]` | Get field by index |

### Closure Operations

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `MakeClosure` | `u16` idx, `u8` count | `[upval1, ..., upvalN] → [closure]` | Create closure |
| `CloseUpvalue` | `u8` slot | - | Close upvalue at stack slot |

### Pattern Matching Support

| Instruction | Arguments | Stack Effect | Description |
|-------------|-----------|--------------|-------------|
| `CheckInt` | `i64` value | `[v] → [v, bool]` | Check if v equals int |
| `CheckBool` | `bool` value | `[v] → [v, bool]` | Check if v equals bool |
| `CheckString` | `String` value | `[v] → [v, bool]` | Check if v equals string |

## Serialization Details

### Serializable Values

The following `Value` types can be serialized to `.fzb` files:

- ✅ `Int`, `Bool`, `Str`, `Unit`
- ✅ `Tuple`, `Cons`, `Nil`
- ✅ `Variant`
- ✅ `Closure` (with upvalues)
- ✅ `NativeFn` (as prototype/name only)

### Non-Serializable Values

The following types **cannot** be serialized:

- ❌ `Array` - Runtime mutable state
- ❌ `Record` - Runtime mutable state
- ❌ `HostData` - Rust object references

**Rationale**: Arrays and records contain `Rc<RefCell<_>>` for runtime mutation. Serializing pointers is unsafe and non-portable.

**Future Work**: Immutable variants may be supported if needed for constants.

### Native Function Serialization

`NativeFn` values serialize as **prototypes**:

```rust
NativeFn {
    name: "List.map",  // ✅ Serialized
    arity: 2,          // ✅ Serialized
    args: [closure],   // ✅ Serialized (partially applied args)
}
```

**Important**: The function pointer is **not** serialized. At load time:

1. VM deserializes the prototype
2. Host must register the function via `register_stdlib()` or custom registry
3. VM looks up implementation by name when called

**Example**:

```rust
// Save bytecode
let chunk = compile("List.map (fun x -> x + 1) [1; 2; 3]")?;
let bytes = serialize_chunk(&chunk)?;
fs::write("script.fzb", bytes)?;

// Load bytecode (different process)
let bytes = fs::read("script.fzb")?;
let chunk = deserialize_chunk(&bytes)?;

let mut vm = Vm::new();
register_stdlib(&mut vm);  // ⚠️ Must register before execution
let result = vm.execute(chunk)?;
```

### Closure Serialization

Closures serialize their chunk and captured upvalues:

```rust
Closure {
    chunk: Chunk,                        // ✅ Recursively serialized
    upvalues: Vec<Rc<RefCell<Upvalue>>>, // ✅ Serialized (values only)
}
```

**Upvalue States**:
- `Open(stack_idx)` → Serialized as stack reference
- `Closed(value)` → Serialized as captured value

## Host Interop

### Registering Host Functions

Host functions are registered via the `HostRegistry`:

```rust
use fusabi_vm::{Vm, Value, VmError};

let mut vm = Vm::new();

// Nullary function (0 args)
vm.host_registry.borrow_mut().register_fn0("getTime", |_vm| {
    Ok(Value::Int(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64))
});

// Unary function (1 arg)
vm.host_registry.borrow_mut().register_fn1("square", |_vm, x| {
    let n = x.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    Ok(Value::Int(n * n))
});

// Binary function (2 args)
vm.host_registry.borrow_mut().register_fn2("add", |_vm, a, b| {
    let x = a.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    let y = b.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    Ok(Value::Int(x + y))
});

// Variable arity
vm.host_registry.borrow_mut().register("sum", |_vm, args| {
    let mut total = 0i64;
    for arg in args {
        total += arg.as_int().ok_or_else(|| VmError::Runtime("Expected int".into()))?;
    }
    Ok(Value::Int(total))
});
```

### Re-entrant Functions

Host functions can call back into the VM:

```rust
// List.map implementation (simplified)
vm.host_registry.borrow_mut().register("List.map", |vm, args| {
    if args.len() != 2 {
        return Err(VmError::Runtime("List.map expects 2 arguments".into()));
    }

    let func = args[0].clone();
    let list = args[1].clone();

    let elements = list.list_to_vec()
        .ok_or_else(|| VmError::Runtime("Expected list".into()))?;

    let mut results = Vec::new();
    for elem in elements {
        // Re-enter VM to call the function
        let result = vm.call_function(func.clone(), &[elem])?;
        results.push(result);
    }

    Ok(Value::vec_to_cons(results))
});
```

### Value Conversion Helpers

```rust
// Extract primitives
let n: i64 = value.as_int().ok_or(VmError::TypeMismatch { ... })?;
let b: bool = value.as_bool().ok_or(...)?;
let s: &str = value.as_str().ok_or(...)?;

// Extract structured types
let tuple: &Vec<Value> = value.as_tuple().ok_or(...)?;
let array: Rc<RefCell<Vec<Value>>> = value.as_array().ok_or(...)?;
let record: Rc<RefCell<HashMap<String, Value>>> = value.as_record().ok_or(...)?;

// Pattern matching
match value {
    Value::Int(n) => println!("Integer: {}", n),
    Value::Variant { variant_name, fields, .. } if variant_name == "Some" => {
        println!("Some({:?})", fields);
    }
    Value::Cons { head, tail } => println!("List: {} :: {:?}", head, tail),
    _ => println!("Other"),
}

// List conversion
let elements: Vec<Value> = value.list_to_vec().ok_or(...)?;
let list_value: Value = Value::vec_to_cons(elements);
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2024 | Initial ABI specification |

## References

- [Bytecode Format Details](bytecode-format.md)
- [VM Design](03-vm-design.md)
- [Host Interop Guide](host-interop.md)
- [Contributing Guide](../CONTRIBUTING.md)
