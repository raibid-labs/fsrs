# Issue 95: [Host] Add Method Dispatch for HostData Instances

**Labels:** `enhancement`, `host-interop`, `ergonomics`

## Context
Working with Rust-backed host objects (`HostData`) currently requires functional-style calls via module functions (e.g., `Database.query(db, "sql")`). To improve ergonomics and provide a more natural API for Rust interop, we want to support object-oriented method calls (e.g., `db.query("sql")`).

## Implementation Plan

### 1. AST & Parser Updates
*   **AST (`ast.rs`):**
    *   Add `Expr::MethodCall { receiver: Box<Expr>, method_name: String, args: Vec<Expr> }`.
*   **Parser (`parser.rs`):**
    *   Update `parse_postfix` to handle `.method_name(...)`.
    *   **Ambiguity:** Distinguish between record field access `x.field` and method call `x.method()`.
        *   *Strategy:* If dot is followed by identifier and `(`, parse as `MethodCall`. If no `(`, parse as `Get`.

### 2. Compiler Updates (`compiler.rs`)
*   **New Instruction:** `Instruction::CallMethod(method_name_idx, arg_count)`.
*   **Compilation Logic:**
    1.  Compile receiver expression (pushes object to stack).
    2.  Compile argument expressions (pushes args to stack).
    3.  Emit `CallMethod`.

### 3. Host API Extensions (`host.rs`)
*   **Method Registry:**
    *   Extend `Module` or `HostContext` to store methods.
    *   Key: `(TypeId, MethodName)` -> `HostFn`.
*   **Registration API:**
    ```rust
    impl Module {
        pub fn register_method<T: HostData>(&mut self, name: &str, func: HostFn) { ... }
    }
    ```

### 4. VM Runtime Support (`vm.rs`)
*   **Dispatch Logic (OpCode `CallMethod`):**
    1.  Peek at stack position `[-argc - 1]` to get the receiver.
    2.  Verify receiver is a `HostData` variant.
    3.  Get `TypeId` from the host data.
    4.  Look up `(TypeId, method_name)` in the global method registry.
    5.  If found, invoke `HostFn` (passing receiver + args).
    6.  If not found, raise runtime error "Method not found on type X".

### 5. Testing & Validation
*   **Host Test:**
    *   Register a `Counter` host type.
    *   Register `increment` method.
    *   Script: `let c = Counter.new() in c.increment(); c.value()`
    *   Assert result.
