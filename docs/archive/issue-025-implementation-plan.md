# Issue #25: Complete List Support Implementation Plan

## Overview
Complete implementation of list support with cons-cell representation across all compiler layers.

## Changes Required

### 1. AST (ast.rs) - COMPLETED
- Added `List(Vec<Expr>)` variant
- Added `Cons { head: Box<Expr>, tail: Box<Expr> }` variant
- Added helper methods: `is_list()`, `is_cons()`
- Added Display implementation

### 2. Lexer (lexer.rs)
- Add `LBracket` token ([)
- Add `RBracket` token (])
- Add `Semicolon` token (;)
- Add `ColonColon` token (::)

### 3. Parser (parser.rs)
- Parse list literals: `[e1; e2; e3]`
- Parse cons operator: `e1 :: e2` (right-associative)
- Update precedence: cons between app and add

### 4. Value (value.rs)
- Add cons-cell representation with Rc for shared structure
- Add `ListNode { head: Value, tail: Option<Rc<ListNode>> }`
- Add `Value::List(Option<Rc<ListNode>>)`

### 5. Instructions (instruction.rs)
- `MakeList(u16)` - Create list from N stack values
- `Cons` - Create cons cell from head and tail
- `ListHead` - Get head of list
- `ListTail` - Get tail of list
- `ListIsEmpty` - Check if list is empty

### 6. Compiler (compiler.rs)
- Compile `Expr::List` to bytecode
- Compile `Expr::Cons` to bytecode

### 7. VM (vm.rs)
- Execute list instructions
- Handle list operations

### 8. Tests (tests/list_tests.rs)
- 25+ comprehensive tests
- Empty list, single element, multiple elements
- Cons operations
- List operations (head, tail, isEmpty)
- Pattern matching over lists

## Implementation Order
1. Lexer tokens
2. Value representation
3. Instructions
4. Parser
5. Compiler
6. VM
7. Tests
