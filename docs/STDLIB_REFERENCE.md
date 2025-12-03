# Fusabi Standard Library Reference

This document provides a comprehensive reference for the Fusabi standard library functions.

The standard library is organized into the following modules:

- **Array**: Mutable array operations
- **List**: Immutable cons list operations
- **Map**: Persistent key-value dictionaries
- **Option**: Optional value handling (Some/None)
- **String**: String manipulation functions
- **Json**: JSON parsing and serialization
- **Result**: Result type for error handling (Ok/Error)
- **Math**: Mathematical functions (trig, logs, rounding, constants)
- **Process**: Process and command execution, environment variables
- **Time**: Date/time operations (now, formatting, parsing)
- **Url**: URL parsing, encoding/decoding
- **Config**: Configuration key-value store
- **Events**: Event emitter pattern
- **TerminalInfo**: Terminal information queries
- **TerminalControl**: Terminal control operations
- **Commands**: Command pattern registry
- **UIFormatting**: UI/text formatting utilities

## Table of Contents

- [Array Module](#array-module)
- [List Module](#list-module)
- [Map Module](#map-module)
- [Option Module](#option-module)
- [String Module](#string-module)
- [Json Module](#json-module)
- [Result Module](#result-module)
- [Math Module](#math-module)
- [Process Module](#process-module)
- [Time Module](#time-module)
- [Url Module](#url-module)
- [Config Module](#config-module)
- [Events Module](#events-module)
- [TerminalInfo Module](#terminalinfo-module)
- [TerminalControl Module](#terminalcontrol-module)
- [Commands Module](#commands-module)
- [UIFormatting Module](#uiformatting-module)

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

### `Map.add`

**Type signature:** `string -> 'a -> 'a map -> 'a map`

Adds a key-value pair to the map, returning a new map

---

### `Map.containsKey`

**Type signature:** `string -> 'a map -> bool`

Returns true if the map contains the given key

---

### `Map.count`

**Type signature:** `'a map -> int`

Returns the number of key-value pairs in the map

---

### `Map.empty`

**Type signature:** `unit -> 'a map`

Creates an empty map

---

### `Map.find`

**Type signature:** `string -> 'a map -> 'a`

Looks up a key in the map, throws error if not found

---

### `Map.isEmpty`

**Type signature:** `'a map -> bool`

Returns true if the map is empty

---

### `Map.iter`

**Type signature:** `(string -> 'a -> unit) -> 'a map -> unit`

Calls a function on each key-value pair for side effects

---

### `Map.map`

**Type signature:** `('a -> 'b) -> 'a map -> 'b map`

Applies a function to each value in the map, returning a new map

---

### `Map.ofList`

**Type signature:** `(string * 'a) list -> 'a map`

Creates a map from a list of key-value tuples

---

### `Map.remove`

**Type signature:** `string -> 'a map -> 'a map`

Removes a key from the map, returning a new map

---

### `Map.toList`

**Type signature:** `'a map -> (string * 'a) list`

Converts a map to a list of key-value tuples (sorted by key)

---

### `Map.tryFind`

**Type signature:** `string -> 'a map -> 'a option`

Looks up a key in the map, returns Some(value) or None

---

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

### `Option.map`

**Type signature:** `('a -> 'b) -> 'a option -> 'b option`

Transforms the value inside Some with the given function

---

### `Option.map2`

**Type signature:** `('a -> 'b -> 'c) -> 'a option -> 'b option -> 'c option`

Combines two options with a function

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

## Json Module

JSON parsing and serialization functions. Available when the `json` feature is enabled.

### `Json.parse`

**Type signature:** `string -> 'a`

Parses a JSON string into a Fusabi value

---

### `Json.stringify`

**Type signature:** `'a -> string`

Converts a Fusabi value to a JSON string

---

### `Json.stringifyPretty`

**Type signature:** `'a -> string`

Converts a Fusabi value to a pretty-printed JSON string

---

## Result Module

The Result type represents computations that may fail. Functions in this module help work with `Ok` and `Error` variants.

### `Result.bind`

**Type signature:** `('a -> Result<'c, 'b>) -> Result<'a, 'b> -> Result<'c, 'b>`

Monadic bind for Result (also known as flatMap or andThen)

---

### `Result.defaultValue`

**Type signature:** `'a -> Result<'a, 'b> -> 'a`

Returns the value inside Ok, or the default value if Error

---

### `Result.defaultWith`

**Type signature:** `('b -> 'a) -> Result<'a, 'b> -> 'a`

Returns the value inside Ok, or calls the default function with the error if Error

---

### `Result.isError`

**Type signature:** `Result<'a, 'b> -> bool`

Returns true if the result is Error, false if Ok

---

### `Result.isOk`

**Type signature:** `Result<'a, 'b> -> bool`

Returns true if the result is Ok, false if Error

---

### `Result.iter`

**Type signature:** `('a -> unit) -> Result<'a, 'b> -> unit`

Calls the function with the Ok value if Ok, does nothing if Error

---

### `Result.map`

**Type signature:** `('a -> 'c) -> Result<'a, 'b> -> Result<'c, 'b>`

Transforms the value inside Ok with the given function, passes through Error

---

### `Result.mapError`

**Type signature:** `('b -> 'c) -> Result<'a, 'b> -> Result<'a, 'c>`

Transforms the error inside Error with the given function, passes through Ok

---

## Math Module

Mathematical operations including trigonometric functions, logarithms, rounding, and mathematical constants.

### `Math.abs`

**Type signature:** `int -> int`

---

### `Math.abs`

**Type signature:** `float -> float`

Returns the absolute value of a number

---

### `Math.acos`

**Type signature:** `float -> float`

Returns the arccosine (inverse cosine) of a value, result in radians

---

### `Math.asin`

**Type signature:** `float -> float`

Returns the arcsine (inverse sine) of a value, result in radians

---

### `Math.atan`

**Type signature:** `float -> float`

Returns the arctangent (inverse tangent) of a value, result in radians

---

### `Math.atan2`

**Type signature:** `float -> float -> float`

Returns the arctangent of y/x in radians, using the signs to determine the quadrant

---

### `Math.ceil`

**Type signature:** `float -> float`

Returns the smallest integer greater than or equal to the number

---

### `Math.cos`

**Type signature:** `float -> float`

Returns the cosine of an angle in radians

---

### `Math.e`

**Type signature:** `unit -> float`

Returns the mathematical constant e (Euler's number)

---

### `Math.exp`

**Type signature:** `float -> float`

Returns e raised to the power of x

---

### `Math.floor`

**Type signature:** `float -> float`

Returns the largest integer less than or equal to the number

---

### `Math.log`

**Type signature:** `float -> float`

Returns the natural logarithm (base e) of a number

---

### `Math.log10`

**Type signature:** `float -> float`

Returns the base-10 logarithm of a number

---

### `Math.max`

**Type signature:** `int -> int -> int`

---

### `Math.max`

**Type signature:** `float -> float -> float`

Returns the maximum of two values

---

### `Math.min`

**Type signature:** `int -> int -> int`

---

### `Math.min`

**Type signature:** `float -> float -> float`

Returns the minimum of two values

---

### `Math.pi`

**Type signature:** `unit -> float`

Returns the mathematical constant π (pi)

---

### `Math.pow`

**Type signature:** `float -> float -> float`

Returns base raised to the power of exponent

---

### `Math.round`

**Type signature:** `float -> float`

Returns the nearest integer, rounding half-way cases away from 0.0

---

### `Math.sin`

**Type signature:** `float -> float`

Returns the sine of an angle in radians

---

### `Math.sqrt`

**Type signature:** `float -> float`

Returns the square root of a number

---

### `Math.tan`

**Type signature:** `float -> float`

Returns the tangent of an angle in radians

---

### `Math.truncate`

**Type signature:** `float -> float`

Returns the integer part of a number, removing any fractional digits

---

## Process Module

Process and system operations including command execution, environment variable access, and process management.

### `Process.cwd`

**Type signature:** `unit -> string`

Gets the current working directory. Example:   Process.cwd ()   // Returns "/home/user/project"

---

### `Process.env`

**Type signature:** `string -> string option`

Gets an environment variable value. Returns Some(value) if the variable exists, None otherwise. Example:   Process.env "PATH"   // Returns Some("/usr/bin:/bin:...")

---

### `Process.run`

**Type signature:** `string -> string list -> ProcessResult`

Runs a command with the given arguments and returns the result. The command is executed directly (not through a shell). Example:   Process.run "echo" ["hello"; "world"]   // Returns { exitCode = 0; stdout = "hello world\n"; stderr = "" }

---

### `Process.runShell`

**Type signature:** `string -> ProcessResult`

Runs a shell command string and returns the result. The command is executed through the system shell (sh on Unix, cmd.exe on Windows). Example:   Process.runShell "echo hello | grep h"   // Returns { exitCode = 0; stdout = "hello\n"; stderr = "" }

---

### `Process.setEnv`

**Type signature:** `string -> string -> unit`

Sets an environment variable for the current process and child processes. Note: This only affects the current process and its children, not the parent process. Example:   Process.setEnv "MY_VAR" "my_value"   // Returns ()

---

## Time Module

Time and date operations for working with timestamps, formatting, and parsing date/time values.

### `Time.format`

**Type signature:** `string -> int -> string`

Formats a Unix timestamp (in milliseconds) according to a format string Supported format specifiers: - %Y - Year (4 digits) - %m - Month (01-12) - %d - Day of month (01-31) - %H - Hour (00-23) - %M - Minute (00-59) - %S - Second (00-59) - %% - Literal '%' Example: Time.format "%Y-%m-%d %H:%M:%S" timestamp

---

### `Time.now`

**Type signature:** `unit -> int`

Returns the current Unix timestamp in milliseconds since the epoch

---

### `Time.nowSeconds`

**Type signature:** `unit -> int`

Returns the current Unix timestamp in seconds since the epoch

---

### `Time.parse`

**Type signature:** `string -> string -> int option`

Parses a time string according to a format string, returning Some timestamp or None Supported format specifiers: - %Y - Year (4 digits) - %m - Month (01-12) - %d - Day of month (01-31) - %H - Hour (00-23) - %M - Minute (00-59) - %S - Second (00-59) Example: Time.parse "%Y-%m-%d" "2024-03-15"

---

## Url Module

URL manipulation functions for parsing, encoding, and decoding URLs and query parameters.

### `Url.decode`

**Type signature:** `string -> string option`

URL-decode a string (percent decoding) Returns None if the string contains invalid percent encoding

---

### `Url.encode`

**Type signature:** `string -> string`

URL-encode a string (percent encoding)

---

### `Url.isValid`

**Type signature:** `string -> bool`

Check if a string is a valid URL

---

### `Url.parse`

**Type signature:** `string -> UrlInfo option`

Parses a URL string into its components Returns None if the URL is invalid

---

## Config Module

Configuration management providing a persistent key-value store for application settings.

### `Config.define`

**Type signature:** `ConfigSchema -> unit`

Register a configuration schema

---

### `Config.get`

**Type signature:** `string -> ConfigValue`

Get a configuration value (throws if not found)

---

### `Config.getOr`

**Type signature:** `string -> ConfigValue -> ConfigValue`

Get a configuration value with a fallback default

---

### `Config.has`

**Type signature:** `string -> bool`

Check if a configuration is defined

---

### `Config.list`

**Type signature:** `unit -> (string * ConfigValue) list`

List all defined configurations with their current values

---

### `Config.reset`

**Type signature:** `string -> unit`

Reset a configuration to its default value

---

### `Config.set`

**Type signature:** `string -> ConfigValue -> unit`

Set a configuration value (validates against schema)

---

## Events Module

Event system for implementing the observer pattern with event emitters and listeners.

### `Events.clear`

**Type signature:** `string -> unit`

Remove all handlers for a specific event. Example:   Events.clear "WindowResized"

---

### `Events.clearAll`

**Type signature:** `unit -> unit`

Remove all event handlers. Example:   Events.clearAll ()

---

### `Events.emit`

**Type signature:** `string -> 'a -> unit`

Emit an event with data, calling all registered handlers. Handlers are called synchronously in registration order. Example:   Events.emit "WindowFocusChanged" true

---

### `Events.emitAsync`

**Type signature:** `string -> 'a -> Async<unit>`

Emit an event asynchronously. Returns immediately while handlers run. Note: In current implementation, this is synchronous but designed for future async support. Example:   Events.emitAsync "Bell" ()

---

### `Events.handlers`

**Type signature:** `string -> int`

Get the count of handlers registered for an event. Example:   let count = Events.handlers "WindowFocusChanged"

---

### `Events.list`

**Type signature:** `unit -> string list`

Get a list of all event names that have handlers registered. Example:   let events = Events.list ()

---

### `Events.off`

**Type signature:** `int -> bool`

Remove a handler by its ID. Returns true if the handler was found and removed. Example:   Events.off handlerId

---

### `Events.on`

**Type signature:** `string -> ('a -> unit) -> int`

Register a handler for an event. Returns a handler ID that can be used to remove the handler. The handler function receives the event data and should return unit. Example:   let handlerId = Events.on "WindowFocusChanged" (fun gained -> printfn (sprintf "Focus: %b" gained))

---

### `Events.once`

**Type signature:** `string -> ('a -> unit) -> int`

Register a one-time handler that automatically removes itself after being called once. Returns a handler ID. Example:   Events.once "Startup" (fun _ -> printfn "App started!")

---

## TerminalInfo Module

Terminal information queries for detecting terminal capabilities and properties.

### `TerminalInfo.getCurrentWorkingDir`

**Type signature:** `unit -> string option`

Returns the current working directory if available

---

### `TerminalInfo.getForegroundProcess`

**Type signature:** `unit -> ProcessInfo option`

Returns information about the foreground process if available

---

### `TerminalInfo.getLine`

**Type signature:** `int -> string option`

Returns the content of a specific line from the scrollback buffer

---

### `TerminalInfo.getLines`

**Type signature:** `int -> int -> string list`

Returns a list of lines from the scrollback buffer between start and end

---

### `TerminalInfo.getTabTitle`

**Type signature:** `unit -> string`

Returns the tab title, or empty string if no provider is registered

---

### `TerminalInfo.getTerminalSize`

**Type signature:** `unit -> (int * int)`

Returns the terminal size as a tuple (columns, rows) Returns (0, 0) if no provider is registered

---

### `TerminalInfo.getWindowTitle`

**Type signature:** `unit -> string`

Returns the window title, or empty string if no provider is registered

---

## TerminalControl Module

Terminal control operations for cursor movement, screen clearing, and terminal state management.

### `TerminalControl.closePane`

**Type signature:** `int -> bool`

Close a pane by ID

---

### `TerminalControl.closeTab`

**Type signature:** `int -> bool`

Close a tab by ID

---

### `TerminalControl.createTab`

**Type signature:** `unit -> int option`

Create a new tab, returning the tab ID

---

### `TerminalControl.focusPane`

**Type signature:** `int -> bool`

Focus a pane by ID

---

### `TerminalControl.sendKeys`

**Type signature:** `string list -> unit`

Send key sequences to the active pane

---

### `TerminalControl.sendText`

**Type signature:** `string -> unit`

Send text to the active pane

---

### `TerminalControl.setTabTitle`

**Type signature:** `int -> string -> bool`

Set the title of a tab

---

### `TerminalControl.showToast`

**Type signature:** `string -> unit`

Show a toast notification

---

### `TerminalControl.splitHorizontal`

**Type signature:** `unit -> int option`

Split the active pane horizontally, returning the new pane ID

---

### `TerminalControl.splitVertical`

**Type signature:** `unit -> int option`

Split the active pane vertically, returning the new pane ID

---

## Commands Module

Command pattern implementation with a registry for managing and executing named commands.

### `Commands.getById`

**Type signature:** `string -> CommandInfo option`

Gets a command by its string ID

---

### `Commands.invoke`

**Type signature:** `string -> unit`

Invokes a command by its string ID

---

### `Commands.list`

**Type signature:** `unit -> CommandInfo list`

Returns a list of all registered commands

---

### `Commands.register`

**Type signature:** `CommandInfo -> int`

Registers a command and returns its numeric ID

---

### `Commands.registerMany`

**Type signature:** `CommandInfo list -> int list`

Registers multiple commands and returns their numeric IDs

---

### `Commands.unregister`

**Type signature:** `int -> bool`

Unregisters a command by its string ID (despite the type signature suggesting int) Returns true if a command was found and removed, false otherwise

---

## UIFormatting Module

UI and text formatting utilities for styling console output with colors, styles, and formatting.

### `UIFormatting.clearFormatters`

**Type signature:** `unit -> unit`

Removes all registered formatters

---

### `UIFormatting.onFormatStatusLeft`

**Type signature:** `(StatusInfo -> StatusSegment list) -> int`

Registers a formatter callback for left status area Returns a handler ID that can be used to remove the formatter

---

### `UIFormatting.onFormatStatusRight`

**Type signature:** `(StatusInfo -> StatusSegment list) -> int`

Registers a formatter callback for right status area Returns a handler ID that can be used to remove the formatter

---

### `UIFormatting.onFormatTab`

**Type signature:** `(TabInfo -> StatusSegment list) -> int`

Registers a formatter callback for tab rendering Returns a handler ID that can be used to remove the formatter

---

### `UIFormatting.removeFormatter`

**Type signature:** `int -> bool`

Removes a formatter by its handler ID Returns true if a formatter was removed, false if not found

---


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
nu scripts/gen-docs.nu
```

---

*Generated by `scripts/gen-docs.nu`*
