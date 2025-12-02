# Fusabi Standard Library Reference

This document provides a comprehensive reference for the Fusabi standard library functions.

The standard library is organized into the following modules:

- **Array**: Mutable array operations
- **List**: Immutable cons list operations
- **Map**: Persistent map (dictionary) operations
- **Option**: Option type helpers for working with optional values
- **String**: String manipulation functions
- **JSON**: JSON parsing and serialization (when `json` feature is enabled)

## Table of Contents

- [Array Module](#array-module)
- [List Module](#list-module)
- [Map Module](#map-module)
- [Option Module](#option-module)
- [String Module](#string-module)
- [JSON Module](#json-module)

---

## Array Module

Arrays are mutable, fixed-size collections indexed by integers. Array operations provide efficient random access and in-place mutation.

### `Array.create`

**Type signature:** `int -> 'a -> 'a array`

Creates an array of given length filled with the specified value

---

### `Array.get`

**Type signature:** `int -> 'a array -> 'a`

Safe array indexing - throws error if index is out of bounds

---

### `Array.init`

**Type signature:** `int -> (int -> 'a) -> 'a array`

Creates an array of given length by calling the function for each index Takes (length: int, fn: int -> 'a) and creates array by calling fn for each index from 0 to length-1

---

### `Array.isEmpty`

**Type signature:** `'a array -> bool`

Returns true if the array is empty

---

### `Array.length`

**Type signature:** `'a array -> int`

Returns the number of elements in the array

---

### `Array.ofList`

**Type signature:** `'a list -> 'a array`

Converts a cons list to an array

---

### `Array.set`

**Type signature:** `int -> 'a -> 'a array -> unit`

Mutates array in place by setting the element at the given index Throws error if index is out of bounds

---

### `Array.toList`

**Type signature:** `'a array -> 'a list`

Converts an array to a cons list

---

## List Module

Lists are immutable cons-based linked lists. List operations are functional and never mutate the original list.

### `List.append`

**Type signature:** `'a list -> 'a list -> 'a list`

Concatenates two lists

---

### `List.concat`

**Type signature:** `'a list list -> 'a list`

Concatenates a list of lists into a single list

---

### `List.exists`

**Type signature:** `('a -> bool) -> 'a list -> bool`

Returns true if any element satisfies the predicate

---

### `List.filter`

**Type signature:** `('a -> bool) -> 'a list -> 'a list`

Returns list of elements where predicate returns true

---

### `List.find`

**Type signature:** `('a -> bool) -> 'a list -> 'a`

Returns first element satisfying predicate Throws error if not found

---

### `List.fold`

**Type signature:** `('a -> 'b -> 'a) -> 'a -> 'b list -> 'a`

Applies folder function to accumulator and each element

---

### `List.head`

**Type signature:** `'a list -> 'a`

Returns the first element of a list Throws error if list is empty

---

### `List.isEmpty`

**Type signature:** `'a list -> bool`

Returns true if the list is empty (Nil)

---

### `List.iter`

**Type signature:** `('a -> unit) -> 'a list -> unit`

Calls a function on each element for side effects, returns Unit

---

### `List.length`

**Type signature:** `'a list -> int`

Returns the number of elements in a list

---

### `List.map`

**Type signature:** `('a -> 'b) -> 'a list -> 'b list`

Applies a function to each element of the list

---

### `List.reverse`

**Type signature:** `'a list -> 'a list`

Returns a list with elements in reverse order

---

### `List.tail`

**Type signature:** `'a list -> 'a list`

Returns the list without its first element Throws error if list is empty

---

### `List.tryFind`

**Type signature:** `('a -> bool) -> 'a list -> 'a option`

Returns Some(elem) if found, None otherwise

---

## Map Module

Maps are persistent key-value dictionaries with string keys. Map operations return new maps rather than mutating existing ones.

*No documented functions found for this module.*

## Option Module

The Option type represents optional values. Functions in this module help work with `Some` and `None` variants.

### `Option.bind`

**Type signature:** `('a -> 'b option) -> 'a option -> 'b option`

Monadic bind for Option (also known as flatMap or andThen)

---

### `Option.defaultValue`

**Type signature:** `'a -> 'a option -> 'a`

Returns the value inside Some, or the default value if None

---

### `Option.defaultWith`

**Type signature:** `(unit -> 'a) -> 'a option -> 'a`

Returns the value inside Some, or calls the default function if None

---

### `Option.isNone`

**Type signature:** `'a option -> bool`

Returns true if the option is None, false if Some

---

### `Option.isSome`

**Type signature:** `'a option -> bool`

Returns true if the option is Some, false if None

---

### `Option.iter`

**Type signature:** `('a -> unit) -> 'a option -> unit`

Calls the function with the value if Some, does nothing if None

---

### `Option.map2`

**Type signature:** `('a -> 'b -> 'c) -> 'a option -> 'b option -> 'c option`

Combines two options with a function

---

### `Option.map`

**Type signature:** `('a -> 'b) -> 'a option -> 'b option`

Transforms the value inside Some with the given function

---

### `Option.orElse`

**Type signature:** `'a option -> 'a option -> 'a option`

Returns the first option if Some, otherwise returns the second option

---

## String Module

String operations for text manipulation, searching, and formatting.

### `String.concat`

**Type signature:** `string list -> string`

Concatenates a list of strings into a single string

---

### `String.contains`

**Type signature:** `string -> string -> bool`

Returns true if haystack contains needle

---

### `String.endsWith`

**Type signature:** `string -> string -> bool`

Returns true if string ends with the given suffix

---

### `String.format`

**Type signature:** `string -> any list -> string`

Formats a string using printf-style formatting Supported specifiers: %s (string), %d (int), %f (float), %.Nf (float with precision), %% (literal %) Example: String.format "%s version %d.%d" ["MyApp"; 1; 0] returns "MyApp version 1.0"

---

### `String.length`

**Type signature:** `string -> int`

Returns the length of a string in characters (not bytes)

---

### `String.split`

**Type signature:** `string -> string -> string list`

Splits a string by a delimiter into a list of strings

---

### `String.startsWith`

**Type signature:** `string -> string -> bool`

Returns true if string starts with the given prefix

---

### `String.toLower`

**Type signature:** `string -> string`

Converts string to lowercase

---

### `String.toUpper`

**Type signature:** `string -> string`

Converts string to uppercase

---

### `String.trim`

**Type signature:** `string -> string`

Removes leading and trailing whitespace

---

## JSON Module

JSON parsing and serialization functions. Available when the `json` feature is enabled.

*No documented functions found for this module.*


## Notes

- Type variables like `'a`, `'b`, etc. represent generic types
- Function signatures use OCaml/F#-style syntax with `->` for function types
- Higher-order functions that take functions as arguments are marked with parentheses, e.g., `('a -> 'b)`
- The `unit` type represents no value (similar to `void` in other languages)

## Contributing

This documentation is auto-generated from doc comments in the Rust source code.
To update this documentation, modify the `///` comments in the stdlib source files
located in `rust/crates/fusabi-vm/src/stdlib/` and run:

```bash
./scripts/gen-docs.sh
```

---

*Generated by `scripts/gen-docs.sh`*
