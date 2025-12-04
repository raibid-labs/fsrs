# Fusabi Bytecode Format & Magic Bytes Update

**Date**: November 17, 2025
**Status**: Documentation updated, implementation pending

After comprehensive analysis of the codebase, **no bytecode magic bytes are currently implemented**. The bytecode format with magic bytes exists only as a planned specification in `research-notes.md`.

---

## Current State

### 1. Bytecode Format Specification (Planned)
**Location:** `/home/beengud/fusabi-lang/fusabi/docs/research-notes.md` (lines 1070-1097)

```rust
pub struct FusabiModule {
    /// Magic number: b"FZB\x01" (validation)
    magic: [u8; 4],

    /// Bytecode version (for compatibility)
    version: u16,

    /// Bytecode instructions
    bytecode: Vec<u8>,

    /// Constant pool (literals)
    constants: Vec<Constant>,

    /// Function definitions
    functions: Vec<FunctionDef>,

    /// Type metadata (optional, for validation)
    types: Vec<TypeInfo>,

    /// Debug information (source spans)
    debug_info: Option<DebugInfo>,
}
```

### 2. Current VM Implementation
**Location:** `/home/beengud/fusabi-lang/fusabi/rust/crates/fusabi-vm/src/chunk.rs`

The `Chunk` struct currently has:
- `instructions: Vec<Instruction>`
- `constants: Vec<Value>`
- `name: Option<String>`

**Missing:** No serialization, no magic bytes, no file I/O

### 3. File Extension References

Found `.fsx` references in:
- `docs/01-overview.md:57` - Example file loading
- `docs/research-notes.md:1315` - Fusabi Module (.fzb)
- `docs/host-interop.md` - Multiple examples (lines 32, 57, 64, 96, 442-498, 858-859)

All need updating to `.fzb` or `.fsx` as appropriate.

## Required Changes

### Phase 1: Update Documentation

1. **Update research-notes.md (Line 1076)**
   ```rust
   /// Magic number: b"FZB\x01" (Fusabi Bytecode v1)
   magic: [u8; 4],
   ```

2. **Update file extension references**
   - Source files: `.fsx` (F# Script)
   - Compiled bytecode: `.fzb` (Fusabi Binary)

### Phase 2: Implement Bytecode Format (Future Work)

When implementing the bytecode file format, add to:

**File:** `/home/beengud/fusabi-lang/fusabi/rust/crates/fusabi-vm/src/module.rs` (NEW FILE)

```rust
/// Fusabi Bytecode Module Header
pub const FUSABI_MAGIC: [u8; 4] = *b"FZB\x01";
pub const FUSABI_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct FusabiModule {
    /// Magic bytes for validation: b"FZB\x01"
    pub magic: [u8; 4],
    /// Bytecode format version
    pub version: u16,
    /// Compiled bytecode instructions
    pub bytecode: Vec<u8>,
    /// Constant pool
    pub constants: Vec<Constant>,
    /// Function metadata
    pub functions: Vec<FunctionDef>,
    /// Type information (optional)
    pub types: Vec<TypeInfo>,
    /// Debug info for source mapping
    pub debug_info: Option<DebugInfo>,
}

impl FusabiModule {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ModuleError> {
        // Validate magic bytes
        if &bytes[0..4] != FUSABI_MAGIC {
            return Err(ModuleError::InvalidMagic);
        }
        // Deserialize module...
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Serialize module with magic header...
    }
}
```

### Phase 3: Testing Strategy

**File:** `/home/beengud/fusabi-lang/fusabi/rust/crates/fusabi-vm/tests/module_serialization_test.rs` (NEW FILE)

```rust
#[test]
fn test_module_magic_bytes() {
    let module = FusabiModule::new();
    let bytes = module.to_bytes();

    assert_eq!(&bytes[0..4], b"FZB\x01");
}

#[test]
fn test_module_roundtrip() {
    let original = create_test_module();
    let bytes = original.to_bytes();
    let deserialized = FusabiModule::from_bytes(&bytes).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_invalid_magic_bytes() {
    let bad_bytes = b"XXXX\x00\x00...";
    let result = FusabiModule::from_bytes(bad_bytes);

    assert!(matches!(result, Err(ModuleError::InvalidMagic)));
}
```

## Implementation Timeline

| Phase | Description | Milestone |
|-------|-------------|-----------|
| ✅ Phase 1 | Update documentation references | Complete (Nov 2025) |
| Phase 2 | Implement `FusabiModule` struct | Phase 3, Milestone 3.1 |
| Phase 3 | Add serialization/deserialization | Phase 3, Milestone 3.2 |
| Phase 4 | File I/O for `.fzb` files | Phase 3, Milestone 3.3 |
| Phase 5 | Integration tests | Phase 3, Milestone 3.4 |

## File Naming Conventions

### Source Files (.fsx)
- Human-readable F# scripts
- Parsed by frontend
- Example: `config.fsx`, `plugin.fsx`

### Compiled Bytecode (.fzb)
- Binary format with magic header
- Contains serialized `FusabiModule`
- Example: `config.fzb`, `plugin.fzb`
- Magic bytes: `b"FZB\x01"`

## Documentation Updates

### Files Updated
1. ✅ Update `research-notes.md` magic bytes specification
2. ✅ Update `host-interop.md` examples to use `.fsx`/`.fzb`
3. ✅ Update `01-overview.md` references
4. ✅ Create this file (`bytecode-format-update.md`)

### Files Needing Future Updates (When Implemented)
- `rust/crates/fusabi-vm/src/module.rs` - New file
- `rust/crates/fusabi-vm/src/chunk.rs` - Add serialization
- `rust/crates/fusabi-vm/tests/module_serialization_test.rs` - New tests
- `rust/crates/fusabi-frontend/src/compiler.rs` - Emit `.fzb` files

## References

- **VM Design**: `docs/03-vm-design.md`
- **Research Notes**: `docs/research-notes.md`
- **Host Interop**: `docs/host-interop.md`
- **Roadmap**: `docs/roadmap.md` (Phase 3 details)

---

**Status**: Documentation complete. Implementation deferred to Phase 3.
