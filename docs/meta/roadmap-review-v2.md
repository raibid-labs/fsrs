# ROADMAP_REVIEW_V2.md

## 1. Executive Summary
**Date:** 2025-11-19
**Auditor:** Principal Compiler Architect
**Scope:** `fsrs-frontend`, `fsrs-vm`, `fsrs-demo`

The FSRS project has made significant progress with the implementation of Records, Discriminated Unions, and a basic Module System. The foundation is stronger, but critical gaps remain for a production-ready language.

**Key Achievements:**
*   ✅ **Advanced Types:** Records and Discriminated Unions are fully implemented across AST, Compiler, and VM.
*   ✅ **Module Foundation:** A registry-based module system exists with support for `open` and qualified names.
*   ✅ **Type Inference Engine:** A robust Hindley-Milner implementation exists in `inference.rs`.

**Critical Weaknesses:**
*   ⚠️ **Optional Safety:** The type inference engine is not mandatorily integrated into the compilation pipeline.
*   ⚠️ **Missing Higher-Order Functions:** The standard library lacks `map`, `filter`, `fold` because the VM interop layer cannot yet call closures from Rust.
*   ⚠️ **Memory Model:** Reliance on `Rc<RefCell<T>>` (verified by Fable comparison) is correct but requires careful handling of recursion to avoid host stack overflows.

## 2. Requests for Discussion (RFDs)

### RFD-004: Mandatory Type Checking Pipeline
**Labels:** `type:rfd`, `status:proposed`, `area:compiler`

#### Context
Currently, `TypeInference` exists but is an optional pass. Scripts can be compiled and run without type checking, leading to runtime errors (or VM panics) that should be caught at compile time.

#### Proposal
Refactor `Compiler::compile` to:
1.  **Mandatory Pass:** Run `TypeInference::infer()` on the AST.
2.  **Annotate:** Produce a `TypedAST` (or annotated AST).
3.  **Generate:** Compile from the typed AST (or use types for optimization).
4.  **Reject:** Fail compilation if inference fails.

#### Context for Agent
*   This requires updating `Compiler::compile` signature to possibly return `TypeError`.
*   Ensure the `ModuleRegistry` propagates type information for imported modules so cross-module calls are type-checked.

---

### RFD-005: VM-Native Closures for Standard Library
**Labels:** `type:rfd`, `status:critical`, `area:vm`

#### Context
The Standard Library (`List.length`, etc.) is implemented as native Rust functions. However, we cannot implement `List.map` or `List.fold` because the native Rust function cannot invoke an FSRS `Value::Closure` passed as an argument.

#### Options
1.  **Option A: VM Callbacks.** Add a `call_closure` method to the `Vm` struct that native functions can invoke. *Problem: Re-entrancy and borrowing rules.*
2.  **Option B: Bytecode Thunks.** Compile `List.map` as a small snippet of FSRS bytecode that calls the user function, rather than a native Rust function.
3.  **Option C: Stackless/Trampoline.** Native functions return a special "RequestCall" result, letting the main VM loop handle the callback execution.

#### Recommendation
**Option B (Bytecode Thunks)** is the cleanest for an interpreter. Implement core HOFs in FSRS itself (once the compiler is stable enough) or hand-write the bytecode instructions.
**Alternative (Option C)** is better if we want high-performance native implementations for the iteration logic.

---

## 3. Epics & Issues

### Epic: Standard Library Completion
**Goal:** Implement `List.map`, `List.filter`, `List.fold`.

#### Task: Enable Native-to-Closure Calls
**Labels:** `priority:high`, `area:vm`
**Description:**
Implement a mechanism for `HostFn` to call a `Value::Closure`.
**Context:**
*   See `fsrs-vm/src/stdlib/list.rs`.
*   Current `list_length` works because it only inspects data.
*   `list_map` needs to execute a function.
*   **Crucial:** As per Fable analysis, ensure `map` is iterative, not recursive, to avoid host stack overflow.

---

### Epic: Module System Hardening
**Goal:** Make modules production-ready.

#### Task: Module Caching & Compilation
**Labels:** `priority:medium`, `area:frontend`
**Description:**
Currently, it seems modules are re-parsed/compiled. Implement a caching mechanism in `ModuleRegistry`.
**Context:**
*   Check `fsrs-frontend/src/compiler.rs` and `modules.rs`.
*   Store compiled `Chunk`s in the registry.

#### Task: Nested Module Compilation
**Labels:** `priority:medium`, `area:frontend`
**Description:**
Ensure `module Outer = module Inner = ...` compiles correctly and symbols are resolved via `Outer.Inner.value`.
