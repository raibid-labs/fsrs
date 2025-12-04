# ROADMAP_REVIEW.md

## 1. Executive Summary
**Date:** 2025-11-19
**Auditor:** Principal Compiler Architect
**Scope:** `fsrs-frontend`, `fsrs-vm`

The **FSRS** (F# Rust Scripting) project demonstrates a mature understanding of compiler design, with a clean separation between the frontend (AST/Compilation) and the runtime (VM). The adherence to F# syntax is commendable, and the planned features (Type Inference) are architecturally sound.

However, the current **Memory Model** (`Rc<RefCell<T>>`) presents a critical risk for a functional language that encourages recursive data structures. Without a cycle collector or a move to a Garbage Collector (GC), the language is unsafe for long-running embedded scenarios.

## 2. Requests for Discussion (RFDs)

### RFD-001: Memory Management Strategy & Cycle Detection
**Labels:** `type:rfd`, `status:critical`, `area:vm`

#### Context
The current `Value` implementation relies on `Rc<RefCell<T>>` for complex types (Records, Arrays, Functions).
While `Rc` handles shared ownership efficiently, it is vulnerable to reference cycles (e.g., a closure capturing itself, or two records referencing each other). In an embedded scripting language, these leaks are unacceptable as they will eventually exhaust the host's memory.

#### Options
1.  **Option A: Cycle Collector (Python-style).** Keep `Rc`, but implement a periodic "Trace and Collect" phase that detects and breaks cycles in `RefCell` types.
    *   *Pros:* Keeps current architecture mostly intact.
    *   *Cons:* Runtime overhead; complex to implement correctly.
2.  **Option B: Arena Allocation (mark-and-sweep).** Move all heap objects to a centralized `GcArena`. `Value` becomes a `Copy` handle (e.g., `u32` index).
    *   *Pros:* Solves cycles completely; strictly controls memory usage; trivial "Reset" of the VM.
    *   *Cons:* Major refactor of `Value` and `Vm`; requires passing `&mut Arena` everywhere.
3.  **Option C: Weak References.** Expose `Weak` references to the user and force manual cycle breaking.
    *   *Pros:* Zero runtime cost.
    *   *Cons:* Terrible UX for a high-level language.

#### Recommendation
**Option B (Arena/GC).** For an embedded language, predictability and safety are paramount. An Arena-based approach allows the Host to strictly limit the script's memory usage and guarantees cleanup when the VM is destroyed.

#### Context for Agent
*If implementing Option B, look at the `generational-arena` crate or a simple `Vec<Entry>` implementation. The `Value` enum will shrink significantly (8-16 bytes), which is also a cache-locality win.*

---

### RFD-002: Tail Call Optimization (TCO)
**Labels:** `type:rfd`, `status:proposed`, `area:compiler`

#### Context
FSRS is a functional language where recursion is the primary iteration mechanism. Without TCO, deep recursion will cause a `StackOverflow` in the VM (or Host).

#### Options
1.  **Option A: Trampoline.** The VM loop detects a "TailCall" return status and immediately re-executes the function without pushing a new stack frame.
2.  **Option B: Compiler Unrolling.** The compiler transforms simple tail recursion into `Loop` instructions.

#### Recommendation
**Option A.** It is more general and supports mutually recursive tail calls.

#### Context for Agent
*This requires a new `Instruction::TailCall` or a modification to `Call` that checks if it's in a tail position. The VM loop needs to handle this by replacing the *current* frame instead of pushing a new one.*

---

## 3. Epics & Issues

### Epic: Type System Maturity
**Goal:** Complete the Hindley-Milner type inference and ensure all F# constructs are type-checked before bytecode generation.

#### Task: Verify and Stabilize Type Inference
**Labels:** `priority:high`, `area:frontend`

**Description:**
The `inference.rs` module exists but needs rigorous testing against edge cases (generic functions, complex pattern matching).

**Context for Agent:**
*   Create a test suite in `tests/inference_tests.rs`.
*   Focus on "Let-Polymorphism" (e.g., `let id x = x` used with both `int` and `string`).
*   Ensure error messages are human-readable (e.g., "Expected int, but got string" vs "Type mismatch").

---

### Epic: Host Interop Hardening
**Goal:** Ensure the Host (Rust) is protected from the Guest (Script).

#### Task: Panic Safety for Host Functions
**Labels:** `priority:medium`, `area:vm`

**Description:**
Currently, if a `RefCell` borrows fails (e.g., double mutable borrow), it panics. If this happens inside a Host call or VM loop, it crashes the application.

**Context for Agent:**
*   Audit all `borrow_mut()` calls in `vm.rs` and `value.rs`.
*   Replace them with `try_borrow_mut()` and return `VmError::Runtime` on failure.
*   Wrap user-provided `HostFn` executions in `std::panic::catch_unwind` (if possible/desirable) or enforce strict `Result` return types.

---

### Epic: Standard Library
**Goal:** Provide a "Batteries Included" experience.

#### Task: Implement List Module
**Labels:** `priority:medium`, `area:stdlib`

**Description:**
F# relies heavily on the `List` module. We need to implement core functions: `List.map`, `List.filter`, `List.fold`, `List.rev`.

**Context for Agent:**
*   These can be implemented *in Rust* as `HostFn`s for performance, or *in FSRS* for portability.
*   **Recommendation:** Implement `List.map` and `List.fold` in Rust for speed, and build others on top of them in FSRS (if mixed-mode stdlib is supported).

## 4. Standard Library & Prelude

### RFD-003: Implicit Prelude & Operators
**Labels:** `type:rfd`, `status:proposed`, `area:stdlib`

#### Context
F# developers expect a rich set of operators and functions to be available without imports (`Microsoft.FSharp.Core`).
Currently, FSRS lacks:
1.  **Pipeline Operators:** `|>` and `<|` are not defined as tokens or functions.
2.  **Composition:** `>>` and `<<`.
3.  **Core Functions:** `ignore`, `id`, `fst`, `snd`, `failwith`, `print/printfn`.
4.  **Type Conversions:** `int()`, `string()`, `float()`.

#### Recommendation
Implement a **Core Prelude** that is implicitly opened in every script.
1.  **Operators:** Add `|>` (`PipeRight`) and others to `Lexer` and `Parser` as first-class operators (for precedence handling), or define them as infix functions in the Core module. *Recommendation: First-class operators in Parser for better error messages.*
2.  **Core Module:** Create a `src/stdlib/core.rs` (or `.fsrs`) that defines `ignore`, `id`, etc.
3.  **Auto-Open:** The compiler should implicitly inject `open Core` at the start of every compilation unit.

#### Context for Agent
*   Check `lexer.rs`: `|>` is parsed as `Pipe` + `Gt`. Needs a new `PipeRight` token.
*   Check `parser.rs`: Add infix parsing rule for `|>` with precedence lower than function application but higher than assignment.
*   Define `print` and `printfn` as host functions wrapping `println!`.

### Epic: Core Library Expansion
**Goal:** Align FSRS standard library with F# expectations.

#### Task: Implement Core Operators (`|>`, `>>`, etc.)
**Labels:** `priority:high`, `area:frontend`

**Description:**
The pipeline operator `|>` is idiomatic F#. It must be supported either syntactically or as an operator.

**Context for Agent:**
*   **Lexer:** Add `Token::PipeRight` (`|>`) and `Token::PipeLeft` (`<|`).
*   **Parser:** Add infix parsing logic. `left |> right` desugars to `right left` (function application).
*   **AST:** Optional `Expr::Pipeline` or just desugar immediately to `Expr::App`.

#### Task: Add 'AutoOpen' Core Module
**Labels:** `priority:medium`, `area:vm`

**Description:**
Functions like `int`, `string`, `failwith`, `ignore` should be available globally.

**Context for Agent:**
*   Create `StdlibRegistry::register_core_functions()`.
*   Register `print`, `string` (conversion), `int` (conversion).
*   Ensure these are added to the global scope of the VM upon initialization.
