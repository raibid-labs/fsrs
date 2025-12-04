# Fusabi Research Notes: Bytecode VM Implementation & Embedding Patterns

**Research Date:** 2025-11-17
**Purpose:** Inform FSRS architecture decisions for F#-to-Rust bytecode VM with excellent embedding characteristics

---

## Table of Contents

1. [F# Bytecode VM Implementations](#1-f-bytecode-vm-implementations)
2. [OCaml ZINC Machine Architecture](#2-ocaml-zinc-machine-architecture)
3. [Erlang BEAM VM](#3-erlang-beam-vm)
4. [Lua Embedding Patterns](#4-lua-embedding-patterns)
5. [Rust Embedding Examples](#5-rust-embedding-examples)
6. [Bytecode VM Design Best Practices](#6-bytecode-vm-design-best-practices)
7. [Key Takeaways for Fusabi](#7-key-takeaways-for-fsrs)
8. [Recommended Architecture](#8-recommended-architecture)
9. [Resources & References](#9-resources--references)

---

## 1. F# Bytecode VM Implementations

### 1.1 FSharp.Compiler.Service

**Overview:**
FSharp.Compiler.Service is a component derived from the F# compiler source code that exposes functionality for implementing F# language bindings and embedding F# scripting into applications.

**Key Features:**
- F# Interactive service for runtime evaluation
- Dynamic compilation to System.Reflection.Assembly
- Support for F# 9.0 as of 2024
- Can be used for on-the-fly F# code execution

**Limitations for Fusabi:**
- Requires full .NET runtime
- Heavy dependency footprint
- Not suitable for lightweight embedding
- Generates IL bytecode, not custom VM bytecode

**Links:**
- [FSharp.Compiler.Service Guide](https://fsharp.github.io/fsharp-compiler-docs/fcs/)
- [NuGet Package](https://www.nuget.org/packages/FSharp.Compiler.Service)

### 1.2 FSharp.Compiler.PortaCode

**Overview:**
PortaCode is an F# code format and corresponding interpreter used by Fabulous and DiffSharp for device-side F# execution.

**Architecture:**
- Derives from FSharp.Compiler.Service expressions
- Interprets intermediate representation directly
- No IL generation or Reflection.Emit required
- Library calls implemented via reflection invoke

**Execution Modes:**
1. **Live Checking** - Selective code execution with `LiveCheck` attributes
2. **Observed Execution** - Runtime data collection for development insights
3. **Symbolic Execution** - Inject symbolic variables for analysis
4. **Reflection.Emit-Free Execution** - Runs on constrained platforms

**Use Cases:**
- Fabulous "LiveUpdate" feature for on-device development
- Elmish model/view/update code interpretation
- Platforms without full .NET capabilities

**Key Insight for Fusabi:**
PortaCode demonstrates that F# can be interpreted from compiler service expressions without full IL compilation. This validates our approach of transpiling F# to a custom bytecode format.

**Links:**
- [GitHub Repository](https://github.com/fsprojects/FSharp.Compiler.PortaCode)

### 1.3 Key Takeaway

**No existing lightweight F# bytecode VM exists** that's suitable for Rust embedding. Current solutions either:
- Require full .NET runtime (too heavy)
- Use Fable for transpilation (generates source code, not bytecode)
- Are platform-specific (PortaCode for mobile)

**This validates FSRS's unique position** as a Fable-to-Rust-bytecode engine.

---

## 2. OCaml ZINC Machine Architecture

### 2.1 Overview

The ZINC (ZINC Is Not Caml) machine was designed by Xavier Leroy in 1990 as an "Economical Implementation of the ML Language." Modern OCaml's bytecode interpreter is still based on the ZINC model.

### 2.2 Architecture Details

**Type:** Stack-based virtual machine

**Registers (only 7 total):**
- Program counter
- Stack pointer
- Exception pointer
- Argument pointer
- Accumulator
- Environment pointer
- Global data pointer

**Memory Model:**
- Extremely uniform - values are single words (32 or 64 bits)
- Three value types:
  - `long` values (OCaml int types)
  - `block` values (header + indexed fields)
  - `code offset` values (relative addresses)

**Instruction Set:**
- ~140 instructions total
- Many are arity-specific variants (e.g., function application at different arities)
- Examples: `branch`, `branchifnot`, `closure`, `makeblock`, `const`, `return`

### 2.3 Design Philosophy

**Key Principles:**
1. **High-level operations** - Works with constructed datatypes, not just primitives
2. **Safety through prior validation** - Type checking happens before bytecode generation
3. **Simplicity** - Minimal register set reduces VM complexity
4. **Functional-first** - Designed specifically for strict functional evaluation

### 2.4 Performance Characteristics

- Much slower than native code compilation
- Remarkably performant for an interpreter without JIT
- Different calling convention than native (stack-based vs register-based)

### 2.5 Relevance to FSRS

**Positive:**
- Proves stack-based VMs work well for ML-family languages
- Minimal register set reduces implementation complexity
- High-level instructions simplify compiler backend
- Time-tested architecture (35+ years)

**Considerations:**
- Stack-based might not be optimal for Rust embedding
- ~140 instructions is substantial (consider smaller set)
- Lack of JIT limits performance ceiling

**Links:**
- [Real World OCaml: Compiler Backend](https://dev.realworldocaml.org/compiler-backend.html)
- [HardCaml ZINC Implementation](https://github.com/ujamjar/hardcaml-zinc)

---

## 3. Erlang BEAM VM

### 3.1 Overview

BEAM (Bogdan/Björn's Erlang Abstract Machine) is the virtual machine at the core of Erlang OTP. Multiple languages run on BEAM including Erlang, Elixir, and Gleam.

### 3.2 Architecture Characteristics

**Type:** Register-based virtual machine

**Key Design:**
- All instructions operate on named registers
- Each register can contain any Erlang term
- No statements, only expressions (functional)
- Instruction combining/superinstructions for optimization

### 3.3 Concurrency Model

**Revolutionary Design:**
- Erlang processes are VM-level, not OS-level
- Immutable messages between processes (no locking needed)
- Linear scaling on multi-core/distributed systems
- Same API for local and distributed processes
- One OS process runs entire BEAM VM

**Process Management:**
- Preemptive scheduling via reduction counting
- Lightweight green threads
- Isolated heaps per process
- Efficient message passing

### 3.4 Multi-Language Support

Languages on BEAM share the VM infrastructure:
- **Erlang** - Dynamic, functional, concurrent
- **Elixir** - Modern syntax, metaprogramming, same VM semantics
- **Gleam** - Static typing, compile-time safety

### 3.5 Relevance to FSRS

**Positive:**
- Register-based design may be faster than stack-based
- Multi-language support validates VM as compilation target
- Concurrency model could inspire future features
- Process isolation reduces GC complexity

**Considerations:**
- BEAM is massive and complex (not lightweight)
- Concurrency focus diverges from FSRS goals
- Register allocation adds compiler complexity

**Links:**
- [BEAM Primer](https://www.erlang.org/blog/a-brief-beam-primer/)
- [Wikipedia: BEAM](https://en.wikipedia.org/wiki/BEAM_(Erlang_virtual_machine))
- [The BEAM Book](https://blog.stenmans.org/theBeamBook/)

---

## 4. Lua Embedding Patterns

### 4.1 The Virtual Stack Design

**Core Insight:**
Lua's C API uses an "omnipresent virtual stack" as the sole communication channel between C and Lua. All data exchange flows through this stack.

**How It Works:**
- Stack is where data is temporarily stored between calls
- Put data on stack to pass to Lua
- Get data from stack when Lua returns values
- Stack indices can be positive (from bottom) or negative (from top)

**Benefits:**
1. Solves **GC impedance mismatch** - C uses manual memory, Lua is GC'd
2. Solves **type impedance mismatch** - C is static, Lua is dynamic
3. Simple, uniform API - most functions just manipulate the stack
4. Language-agnostic design

### 4.2 Type Independence

**API Design:**
- Functions are decoupled from type system
- Operations based on stack position, not types
- Example: `lua_setglobal(L, name)` - no type parameter needed
- Stack value determines behavior, not function signature

### 4.3 Value Representation

**lua_State:**
- Opaque pointer to Lua environment
- Contains all global values, closures, coroutines, modules
- Multiple states can coexist independently

**Registry:**
- Special table for C to store values
- `luaL_ref` stores value and returns integer key
- Prevents GC of C-referenced objects
- Avoids exposing Lua objects to C directly

### 4.4 Safety and Control

**Philosophy:**
- C API prioritizes flexibility over ease of use
- No argument validation in most functions
- Programmer responsible for correctness
- Mistakes can cause segfaults
- Manual memory management required

**Trade-off:**
Power and control vs. safety and convenience.

### 4.5 GC Integration

**Key Mechanisms:**

**Write Barriers:**
- Required for incremental GC
- VM must track pointer changes
- Marks new or old pointers based on collector state

**Finalizers:**
- `__gc` metamethod for cleanup
- Can be problematic - no GC during finalizer
- Some implementations (Luau) use tag-based destructors instead

**Host Control:**
- Manual GC triggering (`lua_gc`)
- Incremental collection with configurable steps
- Game engines often run GC per frame

### 4.6 Relevance to FSRS

**Positive:**
- Virtual stack is proven, simple abstraction
- Type-independent API is powerful
- Registry pattern solves GC reference issues
- Clear separation of concerns

**Considerations:**
- Stack manipulation can be verbose
- Manual GC control adds complexity
- No automatic type marshaling
- C-style error handling (error codes, not Rust Result)

**Links:**
- [Lua 5.0 C API Overview](https://www.lua.org/pil/24.html)
- [Embedding Lua in C](https://lucasklassmann.com/blog/2019-02-02-embedding-lua-in-c/)

---

## 5. Rust Embedding Examples

### 5.1 Rhai - The Clear Winner

**Overview:**
Rhai is an embedded scripting language for Rust with JavaScript+Rust-like syntax and dynamic typing.

**Key Features:**
- **Zero boilerplate** - Functions register directly without wrappers
- **Any clonable Rust type** - No special traits required
- **Never panics** - "Don't Panic" guarantee
- **Fast** - 1 million iterations in 0.14 sec (single-core 2.6 GHz)
- **Sandboxed** - Cannot mutate host unless explicitly permitted

**API Design Excellence:**
```rust
let engine = Engine::new();
let script = "print('Hello, World!');";
engine.execute(script).unwrap();
```

**Type Integration:**
- Pass Rust values via `Scope` - automatic serialization
- Runtime type introspection handles dynamic typing
- Getter/setter/method/indexer support
- No manual type marshaling needed

**Safety Features:**
- Stack overflow protection
- Oversized data limits
- Infinite loop detection
- Progress tracking for manual termination
- Passes Miri verification

**Customization:**
- Disable features surgically (loops, keywords)
- Custom operators and syntax extensions
- Use as DSL or full scripting language

**Platform Support:**
- All Rust-supported platforms
- WebAssembly
- no_std environments
- Minimal dependencies

**Community Verdict:**
"Use rhai if you just want a good scripting language."

**Links:**
- [GitHub Repository](https://github.com/rhaiscript/rhai)
- [Official Documentation](https://rhai.rs/)
- [Crates.io](https://crates.io/crates/rhai)

### 5.2 Gluon - Statically Typed Functional

**Overview:**
A static, type-inferred, embeddable functional language written in Rust.

**Key Features:**
- **Hindley-Milner type system** with extensions
- **Thread-safe** - Multiple programs run in parallel
- **Separate heaps** - Per-thread GC reduces overhead
- **Near-zero marshaling** - Rust functions pass directly to Gluon

**Design Philosophy:**
- Emphasizes type safety and functional programming
- More complex than Rhai but more expressive
- Better for domain-specific strongly-typed languages

**Challenges:**
- Algebraic data types can cause stack overflows (reported issues)
- Smaller community than Rhai
- Steeper learning curve

**Links:**
- [GitHub Repository](https://github.com/gluon-lang/gluon)

### 5.3 Dyon - Game-Focused Dynamic

**Overview:**
Dynamically-typed language designed for game engines and interactive applications.

**Key Features:**
- **No garbage collector** - Works around limited memory model
- **4D vector support** - Built-in for game programming
- **Coroutines** - Similar to Go's goroutines
- **Piston integration** - Made by Piston game engine team

**Limitations:**
- Niche use case (game scripting)
- Smaller ecosystem
- Dynamic typing only

**Links:**
- [GitHub Repository](https://github.com/PistonDevelopers/dyon)

### 5.4 Comparison Summary

| Language | Type System | Best For | Strengths | Weaknesses |
|----------|-------------|----------|-----------|------------|
| **Rhai** | Dynamic | General-purpose embedding | Zero boilerplate, excellent docs, fast | Dynamic typing only |
| **Gluon** | Static (HM) | Typed DSLs, functional | Type safety, parallel execution | Complexity, bugs reported |
| **Dyon** | Dynamic | Game scripting | 4D vectors, coroutines, no GC | Niche focus, smaller community |
| **rlua** | Dynamic | Lua familiarity | Known language, portable | Type marshaling overhead |
| **Rune** | Dynamic | Pattern matching | Clean syntax, modern | VM cloning per call |

### 5.5 Relevance to FSRS

**Key Lessons:**

1. **API Simplicity Wins** - Rhai's success comes from minimal boilerplate
2. **Safety Guarantees Matter** - "Don't Panic" is a selling point
3. **Type Marshaling is Pain Point** - Automatic conversion is crucial
4. **Sandboxing is Expected** - Scripts shouldn't mutate host by default
5. **Performance Baseline** - 1M iterations in ~0.14s is competitive
6. **Static Typing Niche** - Gluon shows demand for typed embedding

**For FSRS:**
- Aim for Rhai-level API simplicity
- Leverage F#'s type system (unlike Rhai's dynamic types)
- Provide automatic Rust<->F# type conversion
- Implement resource limits and sandboxing
- Target similar or better performance than Rhai

**Links:**
- [Survey of Rust Embeddable Languages](https://www.boringcactus.com/2020/09/16/survey-of-rust-embeddable-scripting-languages.html)
- [LogRocket: Comparing Rust Scripting Languages](https://blog.logrocket.com/comparing-rust-scripting-language-game-development/)

---

## 6. Bytecode VM Design Best Practices

### 6.1 Stack-Based vs Register-Based VMs

#### Stack-Based Architecture

**How It Works:**
- Operands addressed implicitly by stack pointer
- Operations PUSH/POP from stack
- Example: `ADD` pops two values, pushes result

**Advantages:**
- Easier code generation - no register allocation
- Smaller opcodes (15-20% smaller bytecode)
- Simpler VM implementation
- Easier transfer to both register and stack machines

**Disadvantages:**
- Excessive PUSH/POP operations
- More VM instructions executed (~47% more)
- Longer execution time (~32% slower)
- Value-swapping operations needed
- Cache pressure from stack access

**Examples:** JVM, .NET CLR, Python bytecode

#### Register-Based Architecture

**How It Works:**
- Instructions explicitly name registers
- Operations: `ADD R1, R2, R3` (R1 = R2 + R3)
- Finite set of registers (virtual, not CPU)

**Advantages:**
- 47% fewer executed instructions on average
- 32% faster execution (non-JIT benchmarks)
- No PUSH/POP overhead
- Better for optimizations (common subexpressions)
- Register caching improves performance

**Disadvantages:**
- Harder code generation - register allocation required
- 25% larger bytecode (must encode register addresses)
- More complex VM implementation

**Examples:** Lua VM, Dalvik VM, BEAM

#### Hybrid Approach: Accumulator

**Design:**
- Single special register (accumulator)
- Implicit operand for arithmetic
- No opcode needs more than 1 explicit argument

**Benefits:**
- Simpler than full register machine
- Better than pure stack for common operations
- Reduced instruction encoding size

### 6.2 Instruction Set Design

#### High-Level vs Low-Level Instructions

**High-Level Instructions:**
- Operate on constructed datatypes
- Example: `CALL_FUNCTION(arity)`, `MAKE_CLOSURE(upvalues)`
- Advantages: Simpler compiler, safer VM, fewer instructions
- Disadvantages: Less flexible, larger VM code

**Low-Level Instructions:**
- Primitive operations only
- Example: `LOAD`, `STORE`, `ADD`, `JUMP`
- Advantages: Flexible, smaller VM, easier optimization
- Disadvantages: More instructions executed, complex compiler

**Best Practice:**
ZINC/OCaml approach - high-level instructions with common variants (e.g., arity-specific calls)

#### Redundancy and Convenience

**Redundancy is okay:**
- Some effects can be composed from simpler instructions
- Common patterns get dedicated opcodes
- Trade-off: larger VM for faster execution

**Examples:**
- `PUSH_CONST_0`, `PUSH_CONST_1` vs. `PUSH_CONST(n)`
- `CALL_0`, `CALL_1`, `CALL_2` vs. `CALL(arity)`

#### Encoding Strategy

**Variable-Length Encoding:**
- Common instructions (ADD, GOTO) are shorter
- Uncommon instructions (function call, exception) are longer
- First bits determine instruction family
- Later bits specify variant and operands

**Benefits:**
- Smaller bytecode size
- Better cache utilization
- Faster instruction fetch

### 6.3 Bytecode Validation

**Best Practice:**
Single validation pass before execution.

**What to Check:**
- Well-formed instruction encoding
- Valid jump targets (within bounds)
- Stack depth consistency
- Type compatibility (if statically checkable)

**Cost:**
- Proportional to program size
- One-time overhead
- Prevents runtime errors

**Failure Strategy:**
- Detect invalid bytecode
- Fail with clear error message
- Never execute malformed code

### 6.4 Dispatch Optimization

#### Switch-Based Dispatch (Traditional)

```rust
loop {
    match bytecode[pc] {
        Opcode::Add => { /* add */ }
        Opcode::Sub => { /* sub */ }
        // ...
    }
    pc += 1;
}
```

**Characteristics:**
- Simple, portable
- Single master jump
- Bounds checking overhead
- Poor branch prediction

#### Computed Goto / Threaded Dispatch

**GCC Extension:**
```c
void* dispatch_table[] = {
    &&add_handler,
    &&sub_handler,
    // ...
};

goto *dispatch_table[bytecode[pc]];

add_handler:
    /* add */
    pc += 1;
    goto *dispatch_table[bytecode[pc]];
```

**Performance:**
- **25% faster** than switch on average
- CPython saw 15-20% improvement
- Ruby 1.9 (YARV) uses this
- Better branch prediction (separate jump per opcode)
- No bounds checking overhead

**Requirements:**
- GCC or Clang compiler
- Non-portable (compiler-specific)

**Rust Equivalent:**
- Not directly supported in safe Rust
- Can use with inline assembly or external C
- Consider trade-off: portability vs. performance

### 6.5 Closure Representation

#### Two-Tier Variable Strategy

**Stack-Allocated Locals:**
- Variables NOT captured by closures
- Fast access, automatic cleanup
- No heap allocation

**Heap-Allocated Upvalues:**
- Variables captured by closures
- Outlive creating function
- Managed by GC

#### ObjClosure Architecture

**Components:**
1. **ObjFunction** - Compile-time function definition
2. **ObjClosure** - Runtime closure instance
3. **Upvalue Array** - References to captured variables

**Wrapping Strategy:**
- Every function wrapped in ObjClosure at runtime
- Simplifies VM - always calls closures
- Multiple closures from same function share ObjFunction

#### Upvalue System

**Open Upvalues:**
- Reference stack-allocated variables
- Stored as linked list sorted by stack position
- Shared across closures capturing same variable

**Closed Upvalues:**
- Variables moved to heap when function returns
- `OP_CLOSE_UPVALUE` instruction triggers this
- `closed` field in ObjUpvalue contains heap copy

**Compiler Support:**
- Three-tier identifier lookup: local, upvalue, global
- `isCaptured` flag on locals
- Emit `OP_CLOSE_UPVALUE` instead of `OP_POP` at scope exit

#### Key Instructions

- `OP_CLOSURE(function)` - Create closure with upvalue spec
- `OP_GET_UPVALUE(index)` - Read captured variable
- `OP_SET_UPVALUE(index)` - Write captured variable
- `OP_CLOSE_UPVALUE(index)` - Move to heap

### 6.6 Tail Call Optimization

**What It Is:**
Replace current stack frame with tail call's frame (no stack growth).

**Why It Matters:**
- Required by functional language standards (Scheme, ML)
- Enables unbounded recursion
- Essential for loop-free functional style

#### Recognition Strategies

**Compiler Detection:**
- Emit special `TAIL_CALL` instruction
- Compiler knows call is in tail position
- VM doesn't need to detect

**Runtime Detection:**
- Check if next instruction is RETURN
- If yes, reuse current frame
- Simpler compiler, more VM logic

#### Implementation

**Process:**
1. Pop current function's locals
2. Preserve essential data in registers
3. Push arguments for tail call
4. Jump to function start (or call normally)

**JVM Approach:**
- No native tail call support
- Replace recursive tail calls with `goto` to function start
- Limited to self-recursion

### 6.7 Value Representation

#### NaN Boxing

**Concept:**
Store type tag and value in single 64-bit word by exploiting IEEE 754 NaN encodings.

**How It Works:**
- Valid doubles: stored directly (no masking)
- NaN space: exponent all 1s, mantissa non-zero
- 51+ payload bits available for type encoding
- Pointers in lower 48 bits (sufficient on x64)
- Integers, booleans, null in NaN payload

**Bit Layout:**
```
0xffff000000000000 = NaN mask
- Sign bit: pointer type indicator
- Exponent: 11 bits (all 1s for NaN)
- Mantissa: 52 bits (payload space)
```

**Advantages:**
- Minimal memory (8 bytes)
- Direct double storage (no heap indirection)
- Fast type checking (bitwise operations)
- Used by LuaJIT, SpiderMonkey

**Disadvantages:**
- Implementation complexity
- Platform assumptions (48-bit pointers)
- Not suitable for 32-bit systems
- Future compatibility concerns

#### Tagged Pointers

**Concept:**
Use low bits of pointers for type tags (pointers are aligned).

**How It Works:**
- Pointers aligned to 4 or 8 bytes
- Low 2-3 bits always zero
- Use those bits for type tags

**Advantages:**
- Simpler than NaN boxing
- Works on 32-bit systems
- Portable

**Disadvantages:**
- Doubles require heap allocation
- Larger memory footprint
- Pointer masking overhead

**Used By:** V8, many Lisp implementations

#### Comparison

| Technique | Memory | Doubles | Complexity | Portability |
|-----------|--------|---------|------------|-------------|
| NaN Boxing | 8 bytes | Direct | High | 64-bit only |
| Tagged Pointers | 8-16 bytes | Heap | Medium | All platforms |
| Separate Tag+Value | 16 bytes | Direct | Low | All platforms |

### 6.8 Relevance to FSRS

**Recommendations:**

1. **Start with Stack-Based VM**
   - Simpler implementation
   - Easier Fable integration
   - Can migrate to register-based later

2. **Use High-Level Instructions**
   - Align with F# semantics
   - Simpler compiler backend
   - Example: `CALL_CLOSURE(arity)`, `MAKE_LIST(size)`

3. **Implement Bytecode Validation**
   - Critical for untrusted scripts
   - Single-pass before execution
   - Clear error messages

4. **Consider Computed Goto**
   - 25% performance gain is significant
   - GCC/Clang already used in Rust toolchain
   - Can be gated behind feature flag

5. **Upvalue-Based Closures**
   - Well-proven approach
   - Efficient for most programs
   - Aligns with F# semantics

6. **Mandatory Tail Call Optimization**
   - Required for F# semantics
   - Compiler-detected tail position
   - Emit `TAIL_CALL` instruction

7. **Value Representation:**
   - Start simple (tagged enum in Rust)
   - Consider NaN boxing for performance
   - Benchmark before optimizing

**Links:**
- [Stack vs Register VMs](https://langdev.stackexchange.com/questions/1450)
- [Crafting Interpreters: Closures](https://craftinginterpreters.com/closures.html)
- [Computed Goto Optimization](https://eli.thegreenplace.net/2012/07/12/computed-goto-for-efficient-dispatch-tables)
- [NaN Boxing Explained](https://piotrduperas.com/posts/nan-boxing/)

---

## 7. Key Takeaways for Fusabi

### 7.1 VM Architecture Decisions

#### Stack-Based with Accumulator Hybrid

**Rationale:**
- ZINC proves stack-based works well for ML languages
- Simpler Fable integration (expression stack maps naturally)
- Accumulator reduces instruction count for common patterns
- Can evolve to register-based if needed

**Implementation:**
- 7 registers similar to ZINC (minimal set)
- Stack for expression evaluation
- Accumulator for arithmetic/comparison results
- Environment register for closure captures

#### Instruction Set Philosophy

**High-Level, F#-Aware Instructions:**
```
MAKE_CLOSURE(upvalues)
CALL_FUNCTION(arity)
TAIL_CALL(arity)
MAKE_RECORD(fields)
MAKE_LIST(size)
PATTERN_MATCH(cases)
```

**Benefits:**
- Simpler Fable backend (maps F# constructs directly)
- Safer VM (type-aware operations)
- Easier optimization (high-level analysis)

**Size Target:**
- 50-80 core instructions
- Variants for common arities
- Balance between ZINC (~140) and minimal (~30)

### 7.2 Embedding API Design

#### Learn from Rhai's Success

**Key Principles:**
1. **Zero Boilerplate** - Register Rust functions directly
2. **Automatic Type Conversion** - Any `Clone` type works
3. **Never Panic** - Result-based error handling
4. **Sandboxed by Default** - Explicit opt-in for host mutation
5. **Resource Limits** - Instruction count, memory, stack depth

**FSRS API Sketch:**
```rust
use fsrs::{Engine, Scope};

// Create engine
let engine = Engine::new();

// Register Rust function
engine.register_fn("print", |s: String| {
    println!("{}", s);
});

// Create scope with Rust values
let mut scope = Scope::new();
scope.push("x", 42);

// Execute F# script
let script = r#"
    let doubled = x * 2
    print (sprintf "Result: %d" doubled)
"#;

engine.run_with_scope(&mut scope, script)?;
```

#### Virtual Stack Pattern (Like Lua)

**For Advanced Users:**
```rust
// Low-level API for custom integration
let vm = engine.vm();
vm.push_string("hello");
vm.push_int(42);
vm.call_function("processData", 2)?; // 2 arguments
let result = vm.pop_int()?;
```

**Benefits:**
- Power users get full control
- Simple API for common cases
- Familiar to Lua embedders

### 7.3 GC Integration Strategies

#### Piccolo's Stackless Approach

**Key Innovation:**
Separate mutation from collection by using trampoline/polling pattern.

**Advantages for Fusabi:**
- Rust-Lua interop without stack overflow
- Pauseable execution (fuel-based limits)
- GC can run between steps
- Enables preemptive concurrency

**Implementation:**
```rust
pub struct Executor {
    // VM state reified as data
}

impl Executor {
    pub fn step(&mut self, fuel: u64) -> StepResult {
        // Execute up to `fuel` instructions
        // Return Paused or Completed
    }
}

// Host code
loop {
    match executor.step(1000) {
        StepResult::Paused => { /* yield, run GC, etc. */ }
        StepResult::Completed(value) => { /* done */ }
        StepResult::Error(e) => { /* handle error */ }
    }
}
```

#### GC Arena Crate

**Leverage Existing Solution:**
- Use `gc-arena` crate (same as Piccolo)
- Brandable, generative lifetimes
- Zero-cost GC pointer semantics
- Incremental collection support

**Benefits:**
- Battle-tested by Piccolo
- Prevents cross-arena pointer bugs
- Good ergonomics with procedural macros

### 7.4 Performance Targets

#### Competitive Baseline

**Rhai Performance:**
- 1 million iterations: 0.14s (single-core 2.6 GHz)
- ~7M iterations/second

**FSRS Targets:**
- **Acceptable:** 5M iterations/second (30% slower than Rhai)
- **Good:** 7M iterations/second (match Rhai)
- **Excellent:** 10M+ iterations/second (faster than Rhai)

**How to Achieve:**
1. Computed goto dispatch (25% speedup)
2. NaN boxing values (reduce heap allocations)
3. Inline caching for polymorphism
4. Superinstructions for common patterns

### 7.5 Type System Leverage

#### F#'s Advantage Over Dynamic Languages

**Static Typing Benefits:**
- Compile-time type checking (safer than Rhai/Lua)
- Better optimization opportunities
- No runtime type checks for monomorphic code

**Fable Integration:**
- Fable already handles F# type system
- Emit type metadata in bytecode
- Runtime can validate or skip checks

**Hybrid Approach:**
```
Script Code: Type-checked by F# compiler
Host Interop: Runtime type conversion
```

### 7.6 Hot Reload Architecture

#### Component Isolation

**Design:**
```
Module {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
    functions: Vec<FunctionDef>,
    metadata: ModuleMetadata,
}
```

**Hot Reload Process:**
1. Detect F# source change
2. Fable recompiles to new bytecode
3. Load new Module
4. Swap in VM (preserve state if possible)
5. Call initialization code

**State Preservation:**
- Serialize VM state to JSON/binary
- Load into new module version
- Type compatibility checks required

### 7.7 Developer Experience

#### Clear Error Messages

**Like Rust:**
- Span information in bytecode
- Source file + line + column
- Helpful suggestions
- Color-coded terminal output

**Runtime Errors:**
```
Error: Type mismatch in function call
  --> examples/hello.fsx:12:5
   |
12 |     printNumbers("not a list")
   |                  ^^^^^^^^^^^^
   |                  expected list<int>, found string
   |
help: Did you mean to call `printString` instead?
```

#### Debug Mode

**Features:**
- Instruction trace
- Stack inspection
- Breakpoint support
- Step-through execution

### 7.8 Security and Sandboxing

#### Resource Limits

**Mandatory Controls:**
```rust
let limits = ResourceLimits {
    max_instructions: 1_000_000,  // Prevent infinite loops
    max_memory: 10 * 1024 * 1024, // 10 MB
    max_stack_depth: 1000,        // Prevent stack overflow
    max_allocations: 100_000,     // Prevent memory DoS
};

engine.set_limits(limits);
```

#### Capability-Based Security

**No Ambient Authority:**
```rust
// Script can't access filesystem by default
engine.run(untrusted_script)?; // Safe

// Explicit capability grant
engine.allow_io(PathBuf::from("./data"))?;
engine.run(trusted_script)?; // Can read ./data
```

---

## 8. Recommended Architecture

### 8.1 Fusabi Bytecode Format

#### Module Structure

```rust
pub struct FusabiModule {
    /// Magic number: b"FZB\x01" (Fusabi Bytecode v1) (validation)
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

#### Instruction Set (Initial 50 Instructions)

**Stack Operations:**
```
PUSH_CONST(index)       // Push constant from pool
POP                     // Discard top
DUP                     // Duplicate top
SWAP                    // Swap top two
```

**Variables:**
```
LOAD_LOCAL(index)       // Load local variable
STORE_LOCAL(index)      // Store to local variable
LOAD_UPVALUE(index)     // Load captured variable
STORE_UPVALUE(index)    // Store to captured variable
LOAD_GLOBAL(name)       // Load global
STORE_GLOBAL(name)      // Store global
```

**Arithmetic:**
```
ADD, SUB, MUL, DIV, MOD
NEG                     // Negate
```

**Comparison:**
```
EQ, NEQ, LT, LTE, GT, GTE
```

**Logic:**
```
AND, OR, NOT
```

**Control Flow:**
```
JUMP(offset)            // Unconditional jump
JUMP_IF_FALSE(offset)   // Conditional jump
JUMP_IF_TRUE(offset)    // Conditional jump
RETURN                  // Return from function
TAIL_CALL(arity)        // Tail call optimization
```

**Functions:**
```
CALL(arity)             // Call function
MAKE_CLOSURE(func_idx, upvalue_count)
CLOSE_UPVALUE(index)    // Move upvalue to heap
```

**Data Structures:**
```
MAKE_LIST(size)         // Create list from stack
MAKE_RECORD(field_count)// Create record
LIST_GET                // Index into list
LIST_SET                // Update list element
RECORD_GET(field)       // Get record field
```

**Pattern Matching:**
```
MATCH_START             // Begin pattern match
MATCH_VARIANT(tag)      // Test discriminated union tag
MATCH_END               // End pattern match
```

### 8.2 VM Architecture

```rust
pub struct VirtualMachine {
    /// Program counter
    pc: usize,

    /// Evaluation stack
    stack: Vec<Value>,

    /// Call frames
    frames: Vec<CallFrame>,

    /// Accumulator (for arithmetic)
    acc: Value,

    /// Global variables
    globals: HashMap<String, Value>,

    /// GC arena
    gc: GcArena,

    /// Open upvalues (sorted by stack index)
    open_upvalues: Vec<Gc<Upvalue>>,
}

pub struct CallFrame {
    /// Return address
    return_pc: usize,

    /// Stack base pointer
    base_pointer: usize,

    /// Closure being executed
    closure: Gc<Closure>,
}
```

### 8.3 Value Representation

#### Start Simple, Optimize Later

**Phase 1: Rust Enum**
```rust
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Gc<String>),
    List(Gc<Vec<Value>>),
    Record(Gc<HashMap<String, Value>>),
    Closure(Gc<Closure>),
    NativeFunction(fn(&[Value]) -> Result<Value>),
}
```

**Phase 2: NaN Boxing (if needed)**
```rust
#[repr(transparent)]
pub struct Value(u64);

impl Value {
    // Direct double storage
    fn from_float(f: f64) -> Self { ... }

    // Encoded types in NaN space
    fn from_int(i: i64) -> Self { ... }
    fn from_bool(b: bool) -> Self { ... }
    fn from_ptr<T>(ptr: Gc<T>) -> Self { ... }
}
```

### 8.4 Embedding API

#### High-Level API (Like Rhai)

```rust
pub struct Engine {
    vm: VirtualMachine,
    module_cache: HashMap<PathBuf, Module>,
    limits: ResourceLimits,
}

impl Engine {
    pub fn new() -> Self { ... }

    pub fn register_fn<F, Args, Ret>(&mut self, name: &str, f: F)
    where
        F: Fn(Args) -> Ret,
        Args: FromFsrsArgs,
        Ret: IntoFsrsValue,
    { ... }

    pub fn run(&mut self, script: &str) -> Result<Value> { ... }

    pub fn run_with_scope(&mut self, scope: &mut Scope, script: &str)
        -> Result<Value> { ... }

    pub fn compile(&mut self, script: &str) -> Result<Module> { ... }

    pub fn load_module(&mut self, path: &Path) -> Result<Module> { ... }
}

pub struct Scope {
    vars: HashMap<String, Value>,
}

impl Scope {
    pub fn push<T: IntoFsrsValue>(&mut self, name: &str, value: T) { ... }
    pub fn get<T: FromFsrsValue>(&self, name: &str) -> Option<T> { ... }
}
```

#### Low-Level API (Like Lua)

```rust
pub struct Vm {
    // Direct VM access
}

impl Vm {
    pub fn stack_top(&self) -> usize { ... }
    pub fn push_value(&mut self, value: Value) { ... }
    pub fn pop_value(&mut self) -> Result<Value> { ... }
    pub fn call(&mut self, arity: usize) -> Result<()> { ... }
    pub fn get_global(&mut self, name: &str) -> Result<Value> { ... }
    pub fn set_global(&mut self, name: &str, value: Value) { ... }
}
```

### 8.5 Fable Integration

#### Compilation Pipeline

```
F# Source (.fsx)
    |
    v
Fable Compiler (--lang rust)
    |
    v
Rust Source Code
    |
    v
FSRS Bytecode Compiler (new component)
    |
    v
Fusabi Module (.fzb)
    |
    v
FSRS VM Execution
```

#### Bytecode Compiler Component

```rust
pub struct BytecodeCompiler {
    // Analyzes Fable-generated Rust AST
    // Emits FSRS bytecode
}

impl BytecodeCompiler {
    pub fn compile_file(&mut self, path: &Path) -> Result<FsrsModule> {
        let rust_ast = parse_rust_file(path)?;
        let bytecode = self.emit_bytecode(&rust_ast)?;
        Ok(FsrsModule { bytecode, ... })
    }
}
```

### 8.6 Hot Reload System

```rust
pub struct HotReloadWatcher {
    watcher: notify::RecommendedWatcher,
    engine: Engine,
}

impl HotReloadWatcher {
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        // Watch F# source file
        // On change:
        //   1. Recompile via Fable
        //   2. Compile to bytecode
        //   3. Reload module in engine
        //   4. Preserve state if possible
    }
}
```

---

## 9. Resources & References

### 9.1 F# and ML Language VMs

- [FSharp.Compiler.Service Guide](https://fsharp.github.io/fsharp-compiler-docs/fcs/)
- [FSharp.Compiler.PortaCode](https://github.com/fsprojects/FSharp.Compiler.PortaCode)
- [Real World OCaml: Compiler Backend](https://dev.realworldocaml.org/compiler-backend.html)
- [OCaml Bytecode VM (HardCaml ZINC)](https://github.com/ujamjar/hardcaml-zinc)
- [OCaml JIT Compilation Paper](https://arxiv.org/pdf/1011.6223)

### 9.2 Embedding Patterns and APIs

- [Lua 5.0 C API](https://www.lua.org/pil/24.html)
- [Embedding Lua in C](https://lucasklassmann.com/blog/2019-02-02-embedding-lua-in-c/)
- [Rhai Official Documentation](https://rhai.rs/)
- [Rhai GitHub](https://github.com/rhaiscript/rhai)
- [Survey of Rust Embeddable Languages](https://www.boringcactus.com/2020/09/16/survey-of-rust-embeddable-scripting-languages.html)

### 9.3 Bytecode VM Design

- [Crafting Interpreters (Free Book)](https://craftinginterpreters.com/)
  - [Closures Chapter](https://craftinginterpreters.com/closures.html)
  - [Chunks of Bytecode](https://craftinginterpreters.com/chunks-of-bytecode.html)
- [Stack vs Register VMs Discussion](https://langdev.stackexchange.com/questions/1450)
- [Computed Goto Optimization](https://eli.thegreenplace.net/2012/07/12/computed-goto-for-efficient-dispatch-tables)
- [VM Dispatch Experiments in Rust](https://pliniker.github.io/post/dispatchers/)
- [NaN Boxing Explained](https://piotrduperas.com/posts/nan-boxing/)
- [Value Representation in JS VMs](https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations)

### 9.4 Advanced VM Techniques

- [Piccolo: Stackless Lua in Rust](https://github.com/kyren/piccolo)
- [Piccolo Blog Post](https://kyju.org/blog/piccolo-a-stackless-lua-interpreter/)
- [Erlang BEAM VM Primer](https://www.erlang.org/blog/a-brief-beam-primer/)
- [The BEAM Book](https://blog.stenmans.org/theBeamBook/)
- [Lua VM Implementation Notes](https://poga.github.io/lua53-notes/print.html)

### 9.5 GC and Memory Management

- [gc-arena crate](https://crates.io/crates/gc-arena)
- [Piccolo GC Arena Integration](https://github.com/kyren/piccolo#design)
- [Lua GC in Real-Time Games](http://lua-users.org/wiki/GarbageCollectionInRealTimeGames)

### 9.6 Rust Scripting Languages

- [Rhai](https://github.com/rhaiscript/rhai)
- [Gluon](https://github.com/gluon-lang/gluon)
- [Dyon](https://github.com/PistonDevelopers/dyon)
- [rlua](https://github.com/amethyst/rlua)
- [Rune](https://github.com/rune-rs/rune)
- [Are We Game Yet: Scripting](https://arewegameyet.rs/ecosystem/scripting/)

### 9.7 Performance and Optimization

- [CPython Computed Goto](https://bugs.python.org/issue4753)
- [Ruby YARV VM](https://www.jstorimer.com/blogs/workingwithcode/7766081-how-does-ruby-execute-your-code)
- [V8 Value Representation](https://v8.dev/blog/pointer-compression)

### 9.8 WebAssembly (Inspiration)

- [Wasm3: Lightweight Interpreter](https://github.com/wasm3/wasm3)
- [wasmi: Wasm Interpreter in Rust](https://github.com/wasmi-labs/wasmi)
- [WASM C API](https://github.com/WebAssembly/wasm-c-api)

---

## Conclusion

This research provides a solid foundation for Fusabi's architecture. Key decisions:

1. **Stack-based VM** with accumulator (like ZINC)
2. **High-level, F#-aware instruction set** (~50-80 instructions)
3. **Rhai-inspired embedding API** (zero boilerplate)
4. **Piccolo-style stackless execution** (pauseable, GC-friendly)
5. **gc-arena for memory management**
6. **Computed goto dispatch** (when available)
7. **Upvalue-based closures** (proven approach)
8. **Mandatory tail call optimization** (F# requirement)

Next steps:
1. Design detailed bytecode format
2. Implement minimal VM (10-20 instructions)
3. Create Fable-to-bytecode compiler
4. Build embedding API
5. Benchmark and optimize

Fusabi is positioned to be the **first lightweight, embeddable F# VM** with Rust-native integration.
