# Security Model

This document describes the current security model of Fusabi and outlines proposed future enhancements for sandboxing and resource limits.

## Table of Contents

- [Current Security Model](#current-security-model)
- [Threat Model](#threat-model)
- [Known Limitations](#known-limitations)
- [Safe Usage Guidelines](#safe-usage-guidelines)
- [Proposed Future Enhancements](#proposed-future-enhancements)
- [Reporting Security Issues](#reporting-security-issues)

## Current Security Model

### Trust Model

Fusabi is designed as an **embedded scripting engine for trusted scripts**. The current security model assumes:

- ✅ Scripts are written by trusted developers
- ✅ Scripts are vetted before execution
- ✅ Host application controls what capabilities are exposed
- ❌ Scripts are **not sandboxed** from the host
- ❌ No protection against resource exhaustion attacks
- ❌ No isolation between concurrent script executions

### Trust Boundaries

```
┌─────────────────────────────────────────┐
│ Rust Host Application (Trusted)         │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │ Fusabi VM                          │ │
│  │                                    │ │
│  │  ┌──────────────────────────────┐ │ │
│  │  │ F# Script (Trusted)          │ │ │
│  │  │                              │ │ │
│  │  │  - Access to all registered  │ │ │
│  │  │    host functions            │ │ │
│  │  │  - Can call back into VM     │ │ │
│  │  │  - Full access to globals    │ │ │
│  │  └──────────────────────────────┘ │ │
│  │                                    │ │
│  └────────────────────────────────────┘ │
│                                          │
└─────────────────────────────────────────┘

No sandboxing boundary exists between script and host
```

### What Is Protected

#### Memory Safety

Fusabi inherits Rust's memory safety guarantees:

- ✅ **No buffer overflows**: All array accesses are bounds-checked at runtime
- ✅ **No use-after-free**: Rust's ownership system prevents dangling pointers
- ✅ **No data races**: `Rc<RefCell<_>>` provides runtime borrow checking
- ✅ **Type safety**: Values are tagged with their runtime type

Example of safe array access:

```fsharp
let arr = [|1; 2; 3|]
let x = arr.[10]  // ❌ Runtime error: Array index out of bounds
```

#### Type Safety

The VM enforces type safety at runtime:

- ✅ Operations on wrong types fail gracefully (e.g., `1 + "hello"` → error)
- ✅ Pattern matching validates discriminated union tags
- ✅ Native functions validate argument types

```fsharp
let x = 1 + "hello"  // ❌ VmError::TypeMismatch
```

#### Stack Safety

- ✅ Stack overflow protection via frame depth limits (not currently enforced - see [Limitations](#known-limitations))
- ✅ Stack underflow detection (attempting to pop from empty stack → error)

### What Is NOT Protected

#### Resource Exhaustion

Scripts can currently exhaust resources without limits:

- ❌ **Infinite loops**: No execution time limits
- ❌ **Memory bombs**: Can allocate unbounded memory
- ❌ **Call stack depth**: No recursion depth limits enforced
- ❌ **I/O abuse**: If host functions provide I/O, no rate limiting

Example unbounded computation:

```fsharp
// Infinite loop - will hang forever
let rec loop () = loop ()
loop ()
```

Example memory exhaustion:

```fsharp
// Allocates increasingly large lists
let rec exhaust n =
    if n > 0 then
        let list = List.init n (fun i -> i)
        exhaust (n + 1000000)
exhaust 1
```

#### Capability-Based Access

Scripts have access to **all** registered host functions:

- ❌ No per-script capability restrictions
- ❌ Scripts cannot be isolated from each other's globals
- ❌ All scripts share the same `HostRegistry`

If you register a dangerous function like file I/O, **all scripts** can use it:

```rust
// ⚠️ All scripts can now access the filesystem!
vm.host_registry.borrow_mut().register_fn1("readFile", |_vm, path| {
    let path = path.as_str().ok_or(VmError::Runtime("Expected string".into()))?;
    let content = std::fs::read_to_string(path)?;
    Ok(Value::Str(content))
});
```

#### Cryptographic Security

- ❌ No cryptographic verification of bytecode
- ❌ `.fzb` files are not signed or authenticated
- ❌ Scripts can be modified without detection

## Threat Model

### In Scope

Fusabi aims to protect against:

- ✅ **Accidental memory corruption** by untrusted scripts
- ✅ **Type confusion** attacks
- ✅ **Basic implementation bugs** (covered by Rust's safety)

### Out of Scope (Current)

Fusabi does **not** currently protect against:

- ❌ **Malicious scripts** trying to DoS the host
- ❌ **Resource exhaustion** attacks
- ❌ **Side-channel attacks**
- ❌ **Code injection** via bytecode tampering
- ❌ **Sandbox escapes** (no sandbox exists)

## Known Limitations

### 1. No Execution Time Limits

**Issue**: Scripts can run forever.

**Impact**: A malicious or buggy script can hang the application.

**Example**:

```fsharp
let rec infinite () = infinite ()
infinite ()
```

**Mitigation** (current): Run scripts in a separate thread with a timeout:

```rust
use std::time::Duration;
use std::thread;

let handle = thread::spawn(move || {
    let mut vm = Vm::new();
    register_stdlib(&mut vm);
    vm.execute(chunk)
});

match handle.join_timeout(Duration::from_secs(5)) {
    Ok(Ok(result)) => println!("Success: {:?}", result),
    Ok(Err(e)) => eprintln!("VM error: {}", e),
    Err(_) => eprintln!("Timeout: script took too long"),
}
```

### 2. No Memory Limits

**Issue**: Scripts can allocate unbounded memory.

**Impact**: Can cause OOM and crash the host.

**Example**:

```fsharp
// Allocate a huge array
let bomb = Array.init 1000000000 (fun i -> i)
```

**Mitigation** (current): Monitor memory usage externally or use OS-level limits (`ulimit`, cgroups).

### 3. No Call Stack Depth Limits

**Issue**: Deep recursion can overflow the call stack.

**Impact**: Stack overflow crashes the process.

**Example**:

```fsharp
let rec deep n =
    if n > 0 then deep (n - 1)
    else ()
deep 1000000  // ❌ Stack overflow
```

**Mitigation** (current): Use tail-call optimization where possible (`TailCall` instruction), but not all recursion is optimized.

### 4. No Capability Isolation

**Issue**: All scripts share the same set of host functions.

**Impact**: Cannot restrict what different scripts can do.

**Mitigation** (current): Only register safe functions. Avoid exposing file I/O, network, or other dangerous capabilities unless needed.

### 5. Reference Counting GC (Cycle Leaks)

**Issue**: `Rc<RefCell<_>>` cannot collect reference cycles.

**Impact**: Circular data structures leak memory.

**Example**:

```fsharp
type Node = { mutable next: Node option }
let rec node = { next = Some node }  // ❌ Memory leak
```

**Mitigation** (current): Avoid circular structures. Future: Implement mark-and-sweep GC.

### 6. No Bytecode Verification

**Issue**: `.fzb` files are not validated before execution.

**Impact**: Maliciously crafted bytecode could trigger undefined behavior.

**Example**: Bytecode with out-of-bounds constant indices, invalid instruction pointers, etc.

**Mitigation** (current): Only load `.fzb` files from trusted sources. Future: Add bytecode verifier.

## Safe Usage Guidelines

To use Fusabi safely in production:

### 1. Trust Your Scripts

- ✅ Only run scripts you control or have reviewed
- ✅ Use version control and code review for script changes
- ✅ Consider signing `.fzb` files with external tooling

### 2. Limit Exposed Capabilities

Only register the host functions you need:

```rust
let mut vm = Vm::new();

// ✅ Good: Register only safe, pure functions
vm.host_registry.borrow_mut().register_fn1("sqrt", |_vm, x| {
    let n = x.as_int()?;
    Ok(Value::Int((n as f64).sqrt() as i64))
});

// ❌ Bad: Exposing filesystem access to all scripts
vm.host_registry.borrow_mut().register_fn1("deleteFile", |_vm, path| {
    std::fs::remove_file(path.as_str()?)?;
    Ok(Value::Unit)
});
```

### 3. Run Scripts in Separate Threads

Use threads with timeouts to prevent hangs:

```rust
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn run_script_with_timeout(chunk: Chunk, timeout: Duration) -> Result<Value, String> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        let mut vm = Vm::new();
        register_stdlib(&mut vm);
        let result = vm.execute(chunk);
        let _ = tx.send(result);
    });

    rx.recv_timeout(timeout)
        .map_err(|_| "Script timeout".to_string())?
        .map_err(|e| format!("VM error: {}", e))
}
```

### 4. Monitor Resource Usage

Track memory and CPU usage externally:

```rust
use sysinfo::{System, SystemExt};

let mut system = System::new_all();
system.refresh_all();

let before_mem = system.used_memory();

// Run script
let result = vm.execute(chunk)?;

system.refresh_all();
let after_mem = system.used_memory();
let memory_used = after_mem - before_mem;

if memory_used > MAX_ALLOWED_MEMORY {
    return Err("Script used too much memory");
}
```

### 5. Use OS-Level Sandboxing

Deploy Fusabi in containers or VMs with resource limits:

```bash
# Docker example with memory and CPU limits
docker run --memory=512m --cpus=1.0 my-fusabi-app

# Linux cgroups
cgcreate -g memory,cpu:fusabi
echo 512M > /sys/fs/cgroup/memory/fusabi/memory.limit_in_bytes
cgexec -g memory,cpu:fusabi ./my-fusabi-app
```

### 6. Validate Scripts Before Deployment

- ✅ Run static analysis on F# source
- ✅ Test scripts in a staging environment
- ✅ Monitor production usage for anomalies

## Proposed Future Enhancements

### Phase 1: Resource Limits (High Priority)

Add configurable limits to the VM:

```rust
pub struct VmLimits {
    pub max_stack_depth: usize,      // Max call frames (default: 1000)
    pub max_instructions: u64,       // Max instructions executed (default: 1_000_000)
    pub max_memory_bytes: usize,     // Max heap memory (default: 100 MB)
    pub max_execution_time_ms: u64,  // Timeout (default: 5000 ms)
}

let mut vm = Vm::with_limits(VmLimits {
    max_stack_depth: 500,
    max_instructions: 100_000,
    max_memory_bytes: 10 * 1024 * 1024,  // 10 MB
    max_execution_time_ms: 1000,
});
```

**Implementation**:

- **Instruction counter**: Increment on every instruction, check against limit
- **Stack depth**: Check `frames.len()` on `Call`, fail if exceeded
- **Memory tracking**: Track allocations in `Value::Array`, `Value::Record`, etc.
- **Time limit**: Store start time, check elapsed time periodically

**API**:

```rust
impl Vm {
    pub fn with_limits(limits: VmLimits) -> Self;
    pub fn set_limits(&mut self, limits: VmLimits);
    pub fn get_stats(&self) -> VmStats;  // Current usage
}

pub struct VmStats {
    pub instructions_executed: u64,
    pub stack_depth: usize,
    pub memory_allocated: usize,
    pub execution_time_ms: u64,
}
```

**New Error Types**:

```rust
pub enum VmError {
    // ... existing errors
    InstructionLimitExceeded,
    StackDepthLimitExceeded,
    MemoryLimitExceeded,
    ExecutionTimeoutExceeded,
}
```

### Phase 2: Capability-Based Security

Allow per-script capability restrictions:

```rust
pub struct ScriptCapabilities {
    pub allowed_functions: HashSet<String>,  // Whitelist of host functions
    pub allow_globals: bool,                 // Can access globals?
    pub allow_native_modules: bool,          // Can use stdlib?
}

let mut vm = Vm::new();

// Register functions globally
register_stdlib(&mut vm);
vm.host_registry.borrow_mut().register_fn1("readFile", read_file_impl);
vm.host_registry.borrow_mut().register_fn1("writeFile", write_file_impl);

// Execute script with restricted capabilities
let capabilities = ScriptCapabilities {
    allowed_functions: vec!["List.map", "String.length"].into_iter().collect(),
    allow_globals: false,
    allow_native_modules: true,
};

let result = vm.execute_with_capabilities(chunk, capabilities)?;
```

**Implementation**:

- Store `capabilities` in the VM context
- Check against whitelist before calling native functions
- Fail with `VmError::CapabilityDenied` if not allowed

### Phase 3: Bytecode Verification

Add a verification pass before execution:

```rust
pub struct BytecodeVerifier {
    chunk: Chunk,
}

impl BytecodeVerifier {
    pub fn verify(&self) -> Result<(), VerificationError> {
        self.verify_constants()?;
        self.verify_instructions()?;
        self.verify_jumps()?;
        self.verify_locals()?;
        Ok(())
    }

    fn verify_constants(&self) -> Result<(), VerificationError> {
        // Ensure all LoadConst indices are valid
    }

    fn verify_instructions(&self) -> Result<(), VerificationError> {
        // Check instruction sequence validity
    }

    fn verify_jumps(&self) -> Result<(), VerificationError> {
        // Ensure all jumps land on valid instruction boundaries
    }

    fn verify_locals(&self) -> Result<(), VerificationError> {
        // Check local variable indices
    }
}

// Usage
let chunk = deserialize_chunk(&bytes)?;
BytecodeVerifier::new(chunk.clone()).verify()?;
let result = vm.execute(chunk)?;
```

### Phase 4: Proper Garbage Collection

Replace `Rc<RefCell<_>>` with a mark-and-sweep or generational GC:

**Benefits**:

- ✅ Collect reference cycles
- ✅ Better memory control
- ✅ Predictable memory usage

**Tradeoffs**:

- ❌ More complex implementation
- ❌ GC pauses (stop-the-world)

**Design**:

```rust
pub struct Heap {
    objects: Vec<HeapObject>,
    free_list: Vec<ObjectId>,
}

pub enum HeapObject {
    Array(Vec<Value>),
    Record(HashMap<String, Value>),
    Closure(ClosureData),
}

impl Heap {
    pub fn allocate(&mut self, obj: HeapObject) -> ObjectId;
    pub fn collect_garbage(&mut self, roots: &[Value]);
}
```

### Phase 5: Signed Bytecode

Support cryptographic signing of `.fzb` files:

```
+-------------------+
| Magic Bytes (4)   |  "FZB\x01"
+-------------------+
| Version (1)       |  0x02 (new version)
+-------------------+
| Signature (var)   |  Ed25519 signature
+-------------------+
| Payload (var)     |  Bincode-serialized Chunk
+-------------------+
```

**Workflow**:

1. **Compile** script → bytecode
2. **Sign** bytecode with private key → signature
3. **Distribute** signed `.fzb` file
4. **Verify** signature with public key before execution

**API**:

```rust
pub struct SignedChunk {
    chunk: Chunk,
    signature: Vec<u8>,
}

impl SignedChunk {
    pub fn verify(&self, public_key: &PublicKey) -> Result<(), SignatureError>;
}

let signed = deserialize_signed_chunk(&bytes)?;
signed.verify(&trusted_key)?;
let result = vm.execute(signed.chunk)?;
```

## Reporting Security Issues

If you discover a security vulnerability in Fusabi:

- **Do not** open a public issue
- **Do not** disclose details publicly
- **Do** email security@fusabi-lang.org (or repository maintainers directly)
- **Do** provide:
  - Description of the vulnerability
  - Steps to reproduce
  - Potential impact
  - Suggested fix (if applicable)

We will:

- Acknowledge receipt within 48 hours
- Investigate and develop a fix
- Release a security advisory and patched version
- Credit you in the advisory (if desired)

## References

- [ABI Specification](ABI.md)
- [VM Design](03-vm-design.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [Rust Security Best Practices](https://anssi-fr.github.io/rust-guide/)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024 | Initial security model documentation |
