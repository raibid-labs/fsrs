(*
 * Comments Demo - Multi-line Comment Support
 *
 * This file demonstrates F#-style multi-line comments (* ... *)
 * with full nesting support, as implemented in issue #127
 *)

(* Simple multi-line comment *)
let x = 42 in

(*
   Multi-line comment
   spanning multiple lines
   with proper indentation
*)
let y = 10 in

(* Nested comments are supported!
   (* This is a nested comment *)
   (* You can have multiple nested comments
      (* Even deeply nested ones! *)
      at different levels
   *)
   All properly handled
*)
let z = x + y in

(* Inline comments *) let result = z * 2 (* can appear anywhere *) in

(*
 * Header-style comments
 * =====================
 *
 * Features:
 * - Single-line comments with //
 * - Multi-line comments with (* *)
 * - Nested multi-line comments
 * - Comments can contain special chars: !@#$%^&*()[]{}:;"'<>,.?/\|`~
 *)

// You can also mix single-line comments
let final_value = result + 100 in  // with inline single-line comments

(* Final calculation *)
final_value
