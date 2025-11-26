# Fix for Issue #125: List Literal Comma Syntax

## Problem
List literals with comma-separated syntax `[1, 2, 3]` were not parsing correctly. The parser only supported semicolon-separated syntax `[1; 2; 3]`.

Error encountered:
```
UnexpectedToken { expected: "]", found: Comma }
```

## Solution
Updated the `parse_list()` function in `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/src/parser.rs` to support both comma and semicolon separators for backward compatibility.

### Changes Made

#### 1. Updated Parser Implementation (parser.rs:1447-1480)

**Before:**
```rust
fn parse_list(&mut self) -> Result<Expr> {
    self.expect_token(Token::LBracket)?;

    if self.match_token(&Token::RBracket) {
        return Ok(Expr::List(vec![]));
    }

    let mut elements = vec![];

    loop {
        elements.push(self.parse_expr()?);

        // Check for semicolon separator
        if self.match_token(&Token::Semicolon) {
            if matches!(self.current_token().token, Token::RBracket) {
                break;
            }
        } else {
            break;
        }
    }

    self.expect_token(Token::RBracket)?;
    Ok(Expr::List(elements))
}
```

**After:**
```rust
fn parse_list(&mut self) -> Result<Expr> {
    self.expect_token(Token::LBracket)?;

    if self.match_token(&Token::RBracket) {
        return Ok(Expr::List(vec![]));
    }

    let mut elements = vec![];

    loop {
        elements.push(self.parse_expr()?);

        // Check for comma or semicolon separator
        if self.match_token(&Token::Comma) || self.match_token(&Token::Semicolon) {
            if matches!(self.current_token().token, Token::RBracket) {
                break;
            }
        } else {
            break;
        }
    }

    self.expect_token(Token::RBracket)?;
    Ok(Expr::List(elements))
}
```

Key change: Line 1466 now accepts both `Token::Comma` and `Token::Semicolon` as valid separators.

#### 2. Updated Grammar Documentation

Updated the parser module documentation to reflect the new syntax:

**Before:**
```
//! - Lists: `[1; 2; 3]`, `[]`
```

**After:**
```
//! - Lists: `[1, 2, 3]`, `[1; 2; 3]`, `[]`
```

And in the grammar specification (line 55):
```
//! list       ::= "[" "]" | "[" expr (("," | ";") expr)* ("," | ";")? "]"
```

### Supported Syntax

The parser now supports all of these syntaxes:

1. **Comma-separated:** `[1, 2, 3]`
2. **Comma with trailing comma:** `[1, 2, 3,]`
3. **Semicolon-separated (backward compatible):** `[1; 2; 3]`
4. **Semicolon with trailing separator:** `[1; 2; 3;]`
5. **Empty lists:** `[]`
6. **Nested lists:** `[[1, 2], [3, 4]]`
7. **Mixed separators:** `[[1; 2], [3, 4]]` (outer comma, inner semicolon)

### Tests Added

Created comprehensive test suite in `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/tests/list_literal_syntax_tests.rs`:

- 34 test cases covering:
  - Comma-separated lists
  - Backward compatibility with semicolons
  - Different element types (strings, floats, bools)
  - Lists with expressions
  - Nested lists
  - Lists in different contexts (let bindings, if conditions, function arguments)
  - Cons operator with comma lists
  - Edge cases (whitespace, newlines)
  - Real-world usage examples

All tests pass: **34 passed; 0 failed**

### Verification

#### Parser Tests
```bash
cargo test --package fusabi-frontend
```
Result: **590+ tests passed, 0 failed**

#### Specific Fix Verification
```bash
cargo test --package fusabi-frontend --test list_literal_syntax_tests
```
Result: **34 passed; 0 failed**

#### Real-World Example
The exact syntax from the bytecode API test now parses correctly:
```fsharp
let list = [1, 2, 3] in
let doubled = List.map (fun x -> x * 2) list in
List.head doubled
```

**Status:** Parsing works correctly. (Execution fails due to unrelated runtime issue with `List.map` method dispatch)

## Backward Compatibility

The fix maintains **full backward compatibility**:
- All existing code using semicolon syntax `[1; 2; 3]` continues to work
- No breaking changes to the AST or compiler
- All 590+ existing frontend tests pass

## Files Modified

1. `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/src/parser.rs`
   - Updated `parse_list()` function (lines 1447-1480)
   - Updated module documentation (lines 15, 55)

## Files Created

1. `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/tests/list_literal_syntax_tests.rs`
   - Comprehensive test suite for comma-separated list syntax

2. `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/tests/list_syntax_demo.rs`
   - Demonstration tests showing the fix works for issue #125

## Impact

- **Issue Fixed:** List literals can now use F#-style comma syntax
- **Backward Compatible:** Yes, semicolon syntax still works
- **Breaking Changes:** None
- **Performance Impact:** Negligible (one additional token check)
- **Test Coverage:** 34 new tests added, all existing tests pass

## Next Steps

The parsing issue is resolved. However, the `test_compile_list_operations` bytecode API test still fails with:
```
Runtime(Runtime("Method dispatch not supported for type: record"))
```

This is a separate issue related to runtime method dispatch for `List.map` and `List.head`, which is outside the scope of this parsing fix.
