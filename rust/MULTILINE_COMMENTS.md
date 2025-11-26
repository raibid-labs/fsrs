# Multi-line Comment Support (Issue #127)

## Overview

This document describes the implementation of F#-style multi-line comments `(* ... *)` in the Fusabi lexer.

## Syntax

Multi-line comments in Fusabi follow F# conventions:

```fsharp
(* This is a single-line multi-line comment *)

(* This is a multi-line comment
   that spans multiple lines *)

let x = 42 (* inline comment *) in
x
```

## Nested Comments

One of the key features is **nested comment support**, which allows comments to contain other comments:

```fsharp
(* Outer comment
   (* Inner comment *)
   Still in outer comment
*)

(* Level 1
   (* Level 2
      (* Level 3 *)
      Back to level 2
   *)
   Back to level 1
*)
```

## Implementation Details

### Files Modified

- `/home/beengud/tmp/fusabi/rust/crates/fusabi-frontend/src/lexer.rs`

### Changes Made

1. **Added `UnterminatedComment` error variant** to `LexError` enum:
   ```rust
   pub enum LexError {
       // ... existing variants
       UnterminatedComment(Position),
   }
   ```

2. **Modified `skip_whitespace_and_comments()`** to detect and skip multi-line comments:
   ```rust
   '(' if !self.is_at_end_or(1) && self.peek_char() == '*' => {
       // Multi-line comment
       self.skip_multiline_comment()?;
   }
   ```

3. **Added `skip_multiline_comment()` method** with nesting support:
   ```rust
   fn skip_multiline_comment(&mut self) -> Result<(), LexError> {
       let start_pos = self.current_position();
       let mut depth = 1;

       self.advance(); // skip '('
       self.advance(); // skip '*'

       while depth > 0 && !self.is_at_end() {
           let ch = self.current_char();

           if ch == '(' && !self.is_at_end_or(1) && self.peek_char() == '*' {
               // Nested comment start
               depth += 1;
               self.advance();
               self.advance();
           } else if ch == '*' && !self.is_at_end_or(1) && self.peek_char() == ')' {
               // Comment end
               depth -= 1;
               self.advance();
               self.advance();
           } else if ch == '\n' {
               // Track line numbers in comments
               self.line += 1;
               self.column = 0;
               self.advance();
           } else {
               self.advance();
           }
       }

       if depth > 0 {
           return Err(LexError::UnterminatedComment(start_pos));
       }

       Ok(())
   }
   ```

4. **Refactored single-line comment handling** into separate method:
   ```rust
   fn skip_single_line_comment(&mut self) {
       while !self.is_at_end() && self.current_char() != '\n' {
           self.advance();
       }
   }
   ```

5. **Updated documentation** to include multi-line comment support in module docs.

## Test Coverage

Added comprehensive test cases in `lexer.rs`:

1. `test_simple_multiline_comment` - Basic multi-line comment
2. `test_multiline_comment_multiline` - Comments spanning multiple lines
3. `test_nested_multiline_comments` - Simple nesting
4. `test_deeply_nested_comments` - Multiple levels of nesting
5. `test_inline_multiline_comment` - Comments within expressions
6. `test_multiline_comment_with_special_chars` - Special characters in comments
7. `test_multiple_multiline_comments` - Multiple comments in one line
8. `test_unterminated_multiline_comment` - Error handling for unclosed comments
9. `test_unterminated_nested_comment` - Error handling for unclosed nested comments
10. `test_mixed_single_and_multiline_comments` - Mixing `//` and `(* *)` comments
11. `test_comment_does_not_affect_string` - Comments inside strings are preserved
12. `test_empty_multiline_comment` - Empty `(**)` comments
13. `test_multiline_comment_with_asterisks` - Comments with multiple asterisks

### Test Results

All 17 lexer tests pass successfully:

```
test lexer::tests::test_lex_integer ... ok
test lexer::tests::test_tokenize_with_spans ... ok
test lexer::tests::test_lex_anonymous_record_tokens ... ok
test lexer::tests::test_lex_pipe_disambiguation ... ok
test lexer::tests::test_simple_multiline_comment ... ok
test lexer::tests::test_multiline_comment_multiline ... ok
test lexer::tests::test_nested_multiline_comments ... ok
test lexer::tests::test_deeply_nested_comments ... ok
test lexer::tests::test_inline_multiline_comment ... ok
test lexer::tests::test_multiline_comment_with_special_chars ... ok
test lexer::tests::test_multiple_multiline_comments ... ok
test lexer::tests::test_unterminated_multiline_comment ... ok
test lexer::tests::test_unterminated_nested_comment ... ok
test lexer::tests::test_mixed_single_and_multiline_comments ... ok
test lexer::tests::test_comment_does_not_affect_string ... ok
test lexer::tests::test_empty_multiline_comment ... ok
test lexer::tests::test_multiline_comment_with_asterisks ... ok
```

The bytecode API test `test_compile_with_comments` also passes successfully.

## Examples

See `/home/beengud/tmp/fusabi/rust/examples/comments_demo.fsx` for a comprehensive example demonstrating:

- Simple multi-line comments
- Multi-line comments spanning lines
- Nested comments at various depths
- Inline multi-line comments
- Header-style documentation comments
- Mixing single-line and multi-line comments
- Comments with special characters

Running the example:
```bash
cargo run --bin fus -- examples/comments_demo.fsx
```

Output: `204` (demonstrates that all comments are correctly ignored during compilation)

## Edge Cases Handled

1. **Nested Comments**: Full support for arbitrarily deep nesting
2. **Empty Comments**: `(**)` is valid
3. **Multiple Asterisks**: `(* *** *)` is valid
4. **Line Tracking**: Line numbers are correctly tracked within multi-line comments
5. **Error Reporting**: Unterminated comments report the position where they started
6. **String Literals**: Comment syntax inside strings is not treated as comments
7. **Special Characters**: All characters are allowed inside comments

## Compatibility

This implementation is fully compatible with F# multi-line comment syntax and behavior, including the nested comment support that F# provides.

## Performance

The implementation uses a depth counter to track nesting levels, which is O(1) space complexity regardless of nesting depth. The time complexity is O(n) where n is the number of characters in the comment.
