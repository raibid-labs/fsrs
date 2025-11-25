# Issue 100: [Language] Add `while` Loops with `break` and `continue`

**Labels:** `enhancement`, `compiler`, `vm`

## Context
Fusabi currently lacks imperative looping constructs (`while`, `for`), relying instead on recursion. To improve ergonomics for scripting and performance for hot loops, we need to implement `while` loops. Additionally, unlike standard F#, we will support `break` and `continue` to provide familiar control flow for users coming from other scripting languages.

## Implementation Plan

### 1. Lexer & AST Updates
*   **New Keywords:** Add `while`, `for` (reserved for future), `break`, `continue` to `lexer.rs`.
    *   `Token::While`, `Token::For`, `Token::Break`, `Token::Continue`.
*   **AST Nodes:** Update `Expr` enum in `ast.rs`.
    *   `Expr::While { cond: Box<Expr>, body: Box<Expr> }`
    *   `Expr::Break`
    *   `Expr::Continue`

### 2. Parser Implementation
*   **Parsing Logic:** Implement `parse_while` in `parser.rs`.
    *   Syntax: `while <expr> do <expr>`
    *   Note: `do` token already exists.
*   **Statement Parsing:** Add support for `break` and `continue` as expressions (returning `unit`).

### 3. Compiler & Bytecode
*   **Loop Stack:** Add a `loop_stack: Vec<LoopState>` to `Compiler` struct in `compiler.rs`.
    *   `struct LoopState { start_label: usize, end_label: usize }`
*   **Compiling `while`:**
    1.  Emit `LABEL_START`.
    2.  Compile condition.
    3.  Emit `JUMP_IF_FALSE LABEL_END`.
    4.  Push `LoopState { start: LABEL_START, end: LABEL_END }`.
    5.  Compile body.
    6.  Emit `POP` (discard body result, loops return unit).
    7.  Emit `JUMP LABEL_START`.
    8.  Emit `LABEL_END`.
    9.  Push `Unit` (result of the loop expression).
    10. Pop `LoopState`.
*   **Compiling `break`:**
    *   Peek `loop_stack`.
    *   Emit `JUMP loop_stack.last().end_label`.
*   **Compiling `continue`:**
    *   Peek `loop_stack`.
    *   Emit `JUMP loop_stack.last().start_label`.

### 4. Validation
*   **Scope Check:** `break` and `continue` must fail to compile if `loop_stack` is empty (used outside a loop).

### 5. Testing
*   **Integration Tests:**
    *   Basic `while` loop (counting up/down).
    *   `break` escaping an infinite loop.
    *   `continue` skipping an iteration.
    *   Nested loops (ensure `break` affects inner loop only).
