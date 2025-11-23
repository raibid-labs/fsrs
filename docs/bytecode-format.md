# Fusabi Bytecode Format (.fzb)

This document describes the binary format for Fusabi bytecode files (`.fzb`).

## Overview

The `.fzb` format is a serialized representation of a Fusabi `Chunk` (the unit of compilation). It uses a simple header followed by the `bincode` serialization of the chunk structure.

## File Structure

The file consists of a 5-byte header followed by the serialized payload.

| Offset | Length | Value       | Description |
|--------|--------|-------------|-------------|
| 0      | 4      | `FZB\x01`   | Magic Bytes |
| 4      | 1      | `0x01`      | Version     |
| 5      | Var    | `[bytes]`   | Bincode serialized Chunk |

### Magic Bytes

The first 4 bytes are always `b"FZB\x01"`. This identifies the file as a Fusabi Binary.

### Version

The 5th byte is the version number. Currently `1`.
This allows for future changes to the bytecode format or serialization scheme without breaking backward compatibility detection.

### Payload

The rest of the file is the `Chunk` struct serialized using `bincode` (Little Endian).

The `Chunk` struct contains:
1. **Instructions**: A vector of `Instruction` enums.
2. **Constants**: A vector of `Value` enums.
3. **Name**: An optional string (usually the function name or "main").

## Serialization Details

### `Value` Serialization

Fusabi uses `serde` with `bincode` to serialize `Value`s.

- **Supported Values**:
  - `Int`, `Bool`, `Str`, `Unit`
  - `Tuple`, `Cons`, `Nil`
  - `Variant`
  - `NativeFn` (prototypes only, see below)
  - `Closure` (prototypes only)

- **Excluded Values (`serde(skip)`)**:
  - `Array`: Mutable arrays are not serialized (runtime state).
  - `Record`: Mutable records are not serialized (runtime state).
  
  *Note: In the future, immutable records/arrays may be supported if needed for constants.*

### `NativeFn` Serialization

Native functions are serialized by **Name** and **Arity**. The arguments array is serialized (for partial application), but the function pointer itself is resolved at runtime.

When deserializing, the VM does **not** automatically link `NativeFn`s to their implementations. The `HostRegistry` must be populated separately (e.g., via `register_stdlib`). The `NativeFn` value acts as a handle or prototype that the VM uses to look up the implementation when called.

### `Closure` Serialization

Closures are serialized by their underlying `Chunk` and `Upvalue`s.
- `Chunk`: Recursively serialized.
- `Upvalues`: Serialized as their state (Open/Closed) and value.

## Usage

### Compilation (`fus grind`)

The `fus grind` command compiles a source file (`.fsx`) to bytecode (`.fzb`).

```bash
fus grind script.fsx
# Output: script.fzb
```

### Execution (`fus run`)

The `fus run` command detects the `.fzb` extension (or checks magic bytes) and deserializes the chunk instead of compiling source.

```bash
fus run script.fzb
```
