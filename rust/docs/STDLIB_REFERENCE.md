# Fusabi Standard Library Reference

*Auto-generated from source code on 2025-12-02*

This document provides a comprehensive reference for all functions in the Fusabi standard library.

## Table of Contents

- [List]
- [Array]
- [Map]
- [Option]
- [Result]
- [String]
- [Math]
- [Config]
- [Time]
- [Url]
- [Process]
- [Json]
- [Print]

---

## List

Functions for working with cons-based linked lists

| Function | Description |
|----------|-------------|
| `List.length : 'a list -> int` | Returns the number of elements in a list |
| `List.head : 'a list -> 'a` | Returns the first element of a list Throws error if list is empty |
| `List.tail : 'a list -> 'a list` | Returns the list without its first element Throws error if list is empty |
| `List.reverse : 'a list -> 'a list` | Returns a list with elements in reverse order |
| `List.isEmpty : 'a list -> bool` | Returns true if the list is empty (Nil) |
| `List.append : 'a list -> 'a list -> 'a list` | Concatenates two lists |
| `List.concat : 'a list list -> 'a list` | Concatenates a list of lists into a single list |
| `List.map : ('a -> 'b) -> 'a list -> 'b list` | Applies a function to each element of the list |
| `List.iter : ('a -> unit) -> 'a list -> unit` | Calls a function on each element for side effects, returns Unit |
| `List.filter : ('a -> bool) -> 'a list -> 'a list` | Returns list of elements where predicate returns true |
| `List.fold : ('a -> 'b -> 'a) -> 'a -> 'b list -> 'a` | Applies folder function to accumulator and each element |
| `List.exists : ('a -> bool) -> 'a list -> bool` | Returns true if any element satisfies the predicate |
| `List.find : ('a -> bool) -> 'a list -> 'a` | Returns first element satisfying predicate Throws error if not found |
| `List.tryFind : ('a -> bool) -> 'a list -> 'a option` | Returns Some(elem) if found, None otherwise |

## Array

Functions for mutable fixed-size arrays

| Function | Description |
|----------|-------------|
| `Array.length : 'a array -> int` | Returns the number of elements in the array |
| `Array.isEmpty : 'a array -> bool` | Returns true if the array is empty |
| `Array.get : int -> 'a array -> 'a` | Safe array indexing - throws error if index is out of bounds |
| `Array.set : int -> 'a -> 'a array -> unit` | Mutates array in place by setting the element at the given index Throws error if index is out of bounds |
| `Array.ofList : 'a list -> 'a array` | Converts a cons list to an array |
| `Array.toList : 'a array -> 'a list` | Converts an array to a cons list |
| `Array.init : int -> (int -> 'a) -> 'a array` | Creates an array of given length by calling the function for each index Takes (length: int, fn: int -> 'a) and creates array by calling fn for each index from 0 to length-1 |
| `Array.create : int -> 'a -> 'a array` | Creates an array of given length filled with the specified value |

## Map

Functions for string-keyed hash maps

| Function | Description |
|----------|-------------|
| `Map.empty : unit -> 'a map` | Creates an empty map |
| `Map.add : string -> 'a -> 'a map -> 'a map` | Adds a key-value pair to the map, returning a new map |
| `Map.remove : string -> 'a map -> 'a map` | Removes a key from the map, returning a new map |
| `Map.find : string -> 'a map -> 'a` | Looks up a key in the map, throws error if not found |
| `Map.tryFind : string -> 'a map -> 'a option` | Looks up a key in the map, returns Some(value) or None |
| `Map.containsKey : string -> 'a map -> bool` | Returns true if the map contains the given key |
| `Map.isEmpty : 'a map -> bool` | Returns true if the map is empty |
| `Map.count : 'a map -> int` | Returns the number of key-value pairs in the map |
| `Map.ofList : (string * 'a) list -> 'a map` | Creates a map from a list of key-value tuples |
| `Map.toList : 'a map -> (string * 'a) list` | Converts a map to a list of key-value tuples (sorted by key) |
| `Map.map : ('a -> 'b) -> 'a map -> 'b map` | Applies a function to each value in the map, returning a new map |
| `Map.iter : (string -> 'a -> unit) -> 'a map -> unit` | Calls a function on each key-value pair for side effects |

## Option

Functions for optional values (Some/None)

| Function | Description |
|----------|-------------|
| `Option.isSome : 'a option -> bool` | Returns true if the option is Some, false if None |
| `Option.isNone : 'a option -> bool` | Returns true if the option is None, false if Some |
| `Option.defaultValue : 'a -> 'a option -> 'a` | Returns the value inside Some, or the default value if None |
| `Option.defaultWith : (unit -> 'a) -> 'a option -> 'a` | Returns the value inside Some, or calls the default function if None |
| `Option.map : ('a -> 'b) -> 'a option -> 'b option` | Transforms the value inside Some with the given function |
| `Option.bind : ('a -> 'b option) -> 'a option -> 'b option` | Monadic bind for Option (also known as flatMap or andThen) |
| `Option.iter : ('a -> unit) -> 'a option -> unit` | Calls the function with the value if Some, does nothing if None |
| `Option.map2 : ('a -> 'b -> 'c) -> 'a option -> 'b option -> 'c option` | Combines two options with a function |
| `Option.orElse : 'a option -> 'a option -> 'a option` | Returns the first option if Some, otherwise returns the second option |

## Result

Functions for error handling (Ok/Error)

| Function | Description |
|----------|-------------|
| `Result.isOk : Result<'a, 'b> -> bool` | Returns true if the result is Ok, false if Error |
| `Result.isError : Result<'a, 'b> -> bool` | Returns true if the result is Error, false if Ok |
| `Result.defaultValue : 'a -> Result<'a, 'b> -> 'a` | Returns the value inside Ok, or the default value if Error |
| `Result.defaultWith : ('b -> 'a) -> Result<'a, 'b> -> 'a` | Returns the value inside Ok, or calls the default function with the error if Error |
| `Result.map : ('a -> 'c) -> Result<'a, 'b> -> Result<'c, 'b>` | Transforms the value inside Ok with the given function, passes through Error |
| `Result.mapError : ('b -> 'c) -> Result<'a, 'b> -> Result<'a, 'c>` | Transforms the error inside Error with the given function, passes through Ok |
| `Result.bind : ('a -> Result<'c, 'b>) -> Result<'a, 'b> -> Result<'c, 'b>` | Monadic bind for Result (also known as flatMap or andThen) |
| `Result.iter : ('a -> unit) -> Result<'a, 'b> -> unit` | Calls the function with the Ok value if Ok, does nothing if Error |

## String

Functions for string manipulation

| Function | Description |
|----------|-------------|
| `String.length : string -> int` | Returns the length of a string in characters (not bytes) |
| `String.trim : string -> string` | Removes leading and trailing whitespace |
| `String.toLower : string -> string` | Converts string to lowercase |
| `String.toUpper : string -> string` | Converts string to uppercase |
| `String.split : string -> string -> string list` | Splits a string by a delimiter into a list of strings |
| `String.concat : string list -> string` | Concatenates a list of strings into a single string |
| `String.contains : string -> string -> bool` | Returns true if haystack contains needle |
| `String.startsWith : string -> string -> bool` | Returns true if string starts with the given prefix |
| `String.endsWith : string -> string -> bool` | Returns true if string ends with the given suffix |
| `String.format : string -> any list -> string` | Formats a string using printf-style formatting Supported specifiers: %s (string), %d (int), %f (float), %.Nf (float with precision), %% (literal %) Example: String.format "%s version %d.%d" ["MyApp"; 1; 0] returns "MyApp version 1.0" |

## Math

Mathematical constants and functions

| Function | Description |
|----------|-------------|
| `Math.pi : unit -> float` | Returns the mathematical constant π (pi) |
| `Math.e : unit -> float` | Returns the mathematical constant e (Euler's number) |
| `Math.abs : int -> int` | Math.abs : float -> float Returns the absolute value of a number |
| `Math.sqrt : float -> float` | Returns the square root of a number |
| `Math.pow : float -> float -> float` | Returns base raised to the power of exponent |
| `Math.max : int -> int -> int` | Math.max : float -> float -> float Returns the maximum of two values |
| `Math.min : int -> int -> int` | Math.min : float -> float -> float Returns the minimum of two values |
| `Math.sin : float -> float` | Returns the sine of an angle in radians |
| `Math.cos : float -> float` | Returns the cosine of an angle in radians |
| `Math.tan : float -> float` | Returns the tangent of an angle in radians |
| `Math.asin : float -> float` | Returns the arcsine (inverse sine) of a value, result in radians |
| `Math.acos : float -> float` | Returns the arccosine (inverse cosine) of a value, result in radians |
| `Math.atan : float -> float` | Returns the arctangent (inverse tangent) of a value, result in radians |
| `Math.atan2 : float -> float -> float` | Returns the arctangent of y/x in radians, using the signs to determine the quadrant |
| `Math.log : float -> float` | Returns the natural logarithm (base e) of a number |
| `Math.log10 : float -> float` | Returns the base-10 logarithm of a number |
| `Math.exp : float -> float` | Returns e raised to the power of x |
| `Math.floor : float -> float` | Returns the largest integer less than or equal to the number |
| `Math.ceil : float -> float` | Returns the smallest integer greater than or equal to the number |
| `Math.round : float -> float` | Returns the nearest integer, rounding half-way cases away from 0.0 |
| `Math.truncate : float -> float` | Returns the integer part of a number, removing any fractional digits |

## Config

Typed configuration with schema validation

| Function | Description |
|----------|-------------|
| `Config.define : ConfigSchema -> unit` | Register a configuration schema |
| `Config.get : string -> ConfigValue` | Get a configuration value (throws if not found) |
| `Config.getOr : string -> ConfigValue -> ConfigValue` | Get a configuration value with a fallback default |
| `Config.set : string -> ConfigValue -> unit` | Set a configuration value (validates against schema) |
| `Config.has : string -> bool` | Check if a configuration is defined |
| `Config.list : unit -> (string * ConfigValue) list` | List all defined configurations with their current values |
| `Config.reset : string -> unit` | Reset a configuration to its default value |

## Time

Time and date functions

| Function | Description |
|----------|-------------|
| `Time.now : unit -> int` | Returns the current Unix timestamp in milliseconds since the epoch |
| `Time.nowSeconds : unit -> int` | Returns the current Unix timestamp in seconds since the epoch |
| `Time.format : string -> int -> string` | Formats a Unix timestamp (in milliseconds) according to a format string  Supported format specifiers: - %Y - Year (4 digits) - %m - Month (01-12) - %d - Day of month (01-31) - %H - Hour (00-23) - %M - Minute (00-59) - %S - Second (00-59) - %% - Literal '%'  Example: Time.format "%Y-%m-%d %H:%M:%S" timestamp |
| `Time.parse : string -> string -> int option` | Parses a time string according to a format string, returning Some timestamp or None  Supported format specifiers: - %Y - Year (4 digits) - %m - Month (01-12) - %d - Day of month (01-31) - %H - Hour (00-23) - %M - Minute (00-59) - %S - Second (00-59)  Example: Time.parse "%Y-%m-%d" "2024-03-15" |

## Url

URL parsing and manipulation

| Function | Description |
|----------|-------------|
| `Url.parse : string -> UrlInfo option` | Parses a URL string into its components Returns None if the URL is invalid |
| `Url.isValid : string -> bool` | Check if a string is a valid URL |
| `Url.encode : string -> string` | URL-encode a string (percent encoding) |
| `Url.decode : string -> string option` | URL-decode a string (percent decoding) Returns None if the string contains invalid percent encoding |

## Process

Process execution and environment

| Function | Description |
|----------|-------------|
| `Process.run : string -> string list -> ProcessResult` | Runs a command with the given arguments and returns the result. The command is executed directly (not through a shell).  Example:   Process.run "echo" ["hello"; "world"]   // Returns { exitCode = 0; stdout = "hello world\n"; stderr = "" } |
| `Process.runShell : string -> ProcessResult` | Runs a shell command string and returns the result. The command is executed through the system shell (sh on Unix, cmd.exe on Windows).  Example:   Process.runShell "echo hello \| grep h"   // Returns { exitCode = 0; stdout = "hello\n"; stderr = "" } |
| `Process.env : string -> string option` | Gets an environment variable value. Returns Some(value) if the variable exists, None otherwise.  Example:   Process.env "PATH"   // Returns Some("/usr/bin:/bin:...") |
| `Process.setEnv : string -> string -> unit` | Sets an environment variable for the current process and child processes. Note: This only affects the current process and its children, not the parent process.  Example:   Process.setEnv "MY_VAR" "my_value"   // Returns () |
| `Process.cwd : unit -> string` | Gets the current working directory.  Example:   Process.cwd ()   // Returns "/home/user/project" |

## Json

JSON parsing and serialization

| Function | Description |
|----------|-------------|
| `Json.parse : string -> 'a` | Parses a JSON string into a Fusabi value |
| `Json.stringify : 'a -> string` | Converts a Fusabi value to a JSON string |
| `Json.stringifyPretty : 'a -> string` | Converts a Fusabi value to a pretty-printed JSON string |

## Print

Output functions

| Function | Description |
|----------|-------------|
| `print : 'a -> unit` | Prints a value to stdout without a trailing newline |
| `printfn : 'a -> unit` | Prints a value to stdout with a trailing newline |

## Usage Examples

### List Operations
```fsharp
let nums = [1; 2; 3; 4; 5]
let doubled = List.map (fun x -> x * 2) nums
let sum = List.fold (fun acc x -> acc + x) 0 nums
```

### Option Handling
```fsharp
let maybeValue = Some 42
let value = Option.defaultValue 0 maybeValue  // 42
let mapped = Option.map (fun x -> x * 2) maybeValue  // Some 84
```

### Result Error Handling
```fsharp
let result = Ok 100
let value = Result.defaultValue 0 result  // 100
let mapped = Result.map (fun x -> x / 2) result  // Ok 50
```

### Math Functions
```fsharp
let pi = Math.pi ()
let sqrt2 = Math.sqrt 2.0
let angle = Math.atan2 1.0 1.0  // pi/4
```

---

*For more examples, see the `examples/` directory in the repository.*
