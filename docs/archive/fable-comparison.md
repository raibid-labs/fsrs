# Fable (F# to Rust) vs. FSRS Comparison

## 1. Overview
Fable is a mature transpiler that converts F# source code into other languages (JavaScript, Python, Dart, and recently Rust).
Fusabi (Functional Scripting for Rust) is an embedded bytecode VM written in Rust that executes a subset of F#.

**Core Difference:**
*   **Fable:** Compiles F# to Rust source code, which is then compiled by `rustc` into a native binary.
*   **FSRS:** Compiles F# to custom bytecode, which is executed by the FSRS VM at runtime.

## 2. Memory Management

### Fable-Rust
*   **Strategy:** Translates F#'s Garbage Collection (GC) semantics to Rust's ownership model using smart pointers.
*   **Mechanism:** Heavily relies on `Rc<T>` (Reference Counting) for shared ownership.
*   **Cloning:** Uses "cheap cloning" (incrementing ref count) to simulate F#'s pass-by-reference behavior for immutable data structures.
*   **Cycles:** Like FSRS, Fable-Rust using `Rc` is vulnerable to reference cycles unless manual intervention or specific patterns (like `Weak`) are used, though F# data structures are often acyclic by default (immutable).
*   **Insight for Fusabi:** Fable's extensive use of `Rc` validates FSRS's choice of `Rc` for its Value type. It confirms that for an ML-family language, shared ownership is the natural mapping to Rust without a full GC.

### FSRS
*   **Strategy:** Runtime value representation.
*   **Mechanism:** `Rc<RefCell<T>>` for mutable/complex types (Records, Arrays). `Rc<List<T>>` for immutable lists.
*   **Comparison:** Both use `Rc`. FSRS adds `RefCell` for runtime interior mutability (required for VM), whereas Fable might generate `Mut<T>` or `RefCell<T>` depending on the specific F# construct (mutable vs immutable).

## 3. Currying & Partial Application

### Fable-Rust
*   **Challenge:** Rust functions are not curried by default.
*   **Implementation:** Transforms multi-argument F# functions into Rust functions that return closures.
    ```rust
    // Fable-like generated Rust
    fn add(x: i32) -> impl Fn(i32) -> i32 {
        move |y: i32| x + y
    }
    ```
*   **Optimization:** Fable likely optimizes "known calls" (where all arguments are supplied) to direct function calls to avoid the closure overhead, only falling back to closures for partial application.

### FSRS
*   **Implementation:** Functions are compiled to bytecode.
*   **Currying:** Handled by the VM's `Call` instruction or closure object. If a function expects N args and gets M < N, it returns a new Closure capturing the M args.
*   **Insight:** FSRS's VM approach is actually *simpler* than transpilation here because the VM controls the call stack. However, Fable's "optimize full application" strategy is relevant: FSRS's compiler could detect `add 1 2` and emit a specific `Call2` instruction instead of two `Call1` instructions.

## 4. Standard Library (Lists)

### Fable-Rust
*   **Implementation:** Uses a custom `List<T>` enum in Rust.
    ```rust
    pub enum List<T> {
        Nil,
        Cons(T, Rc<List<T>>),
    }
    ```
*   **Operations:** Implements standard functional operations (`map`, `filter`, `fold`) using Rust iterators or recursion.
*   **Recursion:** Must be careful with stack depth in Rust (no TCO by default). Fable might trampoline or use iterative implementations for `fold`.

### FSRS
*   **Implementation:** `Value::List` variant wrapping `Rc<ListNode>`.
*   **Operations:** Implemented as native Rust functions (host functions) in `stdlib/list.rs`.
*   **Comparison:** Identical structural approach. `Rc` is the correct tool for immutable linked lists in Rust.

## 5. Key Insights & Recommendations for Fusabi

### A. Currying Optimization (Compiler)
Fable likely uncurries functions when possible. FSRS should do the same.
*   **Current:** `add 1 2` -> `Push 1`, `Push 2`, `Call 2` (if `add` is defined as 2-ary).
*   **Improvement:** Ensure the VM supports multi-arg calls natively to avoid creating intermediate closures for every argument.

### B. Recursion Safety
Fable has to deal with Rust's stack limits.
*   **Insight:** FSRS VM has its *own* stack (Vec<Value>), so it is **not** limited by the Rust stack depth for F# recursion, *unless* the interpreter itself is recursive.
*   **Action:** Ensure `vm.run` is an iterative loop (it is), and that standard library functions (like `List.map` implemented in Rust) don't recurse deeply on the Rust stack. FSRS `stdlib` implementation of `map` should use iteration (loops), not recursion, to process lists.

### C. Standard Library Parity
Fable's library is a good reference for "what is essential".
*   **Action:** Review Fable's `fable_library_rust` (if source available or via docs) to prioritize which List/String/Option functions to add next.

### D. Discriminated Unions
Fable maps F# DUs to Rust `enum`. FSRS maps them to `Value::Variant`.
*   **Validation:** This is the correct and most efficient mapping.

## 6. Conclusion
FSRS's architectural choices (Stack VM, `Rc` for values, `List` via `Cons/Rc`) are well-aligned with how Fable translates F# to Rust. FSRS actually has an advantage in handling recursion (virtual stack) and dynamic evaluation (VM) compared to Fable's reliance on Rust's physical stack and compile-time constraints.

**Main actionable item:** Ensure `stdlib` functions in Rust (like `List.map`) are iterative, not recursive, to avoid crashing the host with stack overflow on long lists.
