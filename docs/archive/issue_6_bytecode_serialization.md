# Issue 6: [Architecture] Implement Bytecode Serialization (.fzb)

**Labels:** `feature`, `area:vm`

## Context
To support faster startup times and caching, we need to serialize compiled `Chunk`s to disk.

## Implementation Plan
**Objective:** Implement `serde::Serialize` for the VM structures.

1.  **Derive Serde** (`fusabi-vm/src/`):
    * Add `serde` feature to `fusabi-vm`.
    * Derive `Serialize, Deserialize` for `Chunk`, `Instruction`, and `Value`.
    * *Challenge:* `Value` contains `Rc`. You cannot serialize `Rc` directly easily. You may need a custom serializer that serializes the *data* structure, not the pointers, or refuse to serialize Chunks containing runtime-generated values (closures/native fns). Focus only on serializing *code chunks* (constants + instructions), not heap state.

2.  **Magic Bytes**:
    * Define `const FZB_MAGIC: &[u8] = b"FZB\x01";` in `lib.rs`.

3.  **CLI Update** (`fusabi/src/main.rs`):
    * Implement `fus grind <file.fsx>`.
        * Compile to `Chunk`.
        * `bincode::serialize` the Chunk.
        * Prepend Magic Bytes.
        * Write to `<file.fzb>`.
    * Update `fus run` to detect magic bytes. If present, skip parsing and deserialize directly.
