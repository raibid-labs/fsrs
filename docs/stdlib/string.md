# String Module

The `String` module provides common string operations including case conversion, trimming, splitting, and searching. All operations are Unicode-aware and return new strings rather than modifying existing ones.

## Overview

Strings in Fusabi are immutable UTF-8 encoded text sequences. The String module provides functional operations for string manipulation that integrate seamlessly with the pipeline operator `|>`.

## Functions

### String.length

Returns the length of a string in characters (not bytes).

**Signature:**
```fsharp
String.length : string -> int
```

**Parameters:**
- `str` - The string to measure

**Returns:**
- An integer representing the number of Unicode characters (not bytes)

**Example:**
```fsharp
let message = "Hello, World!" in
let len = String.length message  // 13

let unicode = "Hello 世界" in
let unicodeLen = String.length unicode  // 8 (6 ASCII + 1 space + 2 Chinese chars)

let empty = "" in
let zero = String.length empty  // 0
```

**Note:**
- Counts Unicode characters, not bytes
- Handles multi-byte UTF-8 characters correctly

---

### String.trim

Removes leading and trailing whitespace from a string.

**Signature:**
```fsharp
String.trim : string -> string
```

**Parameters:**
- `str` - The string to trim

**Returns:**
- A new string with leading and trailing whitespace removed

**Example:**
```fsharp
let messy = "  hello world  " in
let clean = String.trim messy  // "hello world"

let tabs = "\t\nhello\n\t" in
let trimmed = String.trim tabs  // "hello"

let noSpace = "hello" in
let unchanged = String.trim noSpace  // "hello"
```

**Note:**
- Removes all Unicode whitespace characters (spaces, tabs, newlines, etc.)
- Does not remove whitespace from the middle of the string

---

### String.toUpper

Converts a string to uppercase.

**Signature:**
```fsharp
String.toUpper : string -> string
```

**Parameters:**
- `str` - The string to convert

**Returns:**
- A new string with all characters converted to uppercase

**Example:**
```fsharp
let greeting = "hello world" in
let loud = String.toUpper greeting  // "HELLO WORLD"

let mixed = "Hello World" in
let upper = String.toUpper mixed  // "HELLO WORLD"
```

**See also:**
- [String.toLower](#stringtolower) - Convert to lowercase

---

### String.toLower

Converts a string to lowercase.

**Signature:**
```fsharp
String.toLower : string -> string
```

**Parameters:**
- `str` - The string to convert

**Returns:**
- A new string with all characters converted to lowercase

**Example:**
```fsharp
let greeting = "HELLO WORLD" in
let quiet = String.toLower greeting  // "hello world"

let mixed = "Hello World" in
let lower = String.toLower mixed  // "hello world"
```

**See also:**
- [String.toUpper](#stringtoupper) - Convert to uppercase

---

### String.split

Splits a string by a delimiter into a list of strings.

**Signature:**
```fsharp
String.split : string -> string -> string list
```

**Parameters:**
- `delimiter` - The delimiter string to split on
- `str` - The string to split

**Returns:**
- A list of strings separated by the delimiter

**Example:**
```fsharp
let sentence = "hello world foo bar" in
let words = String.split " " sentence  // ["hello"; "world"; "foo"; "bar"]

let csv = "name,age,city" in
let fields = String.split "," csv  // ["name"; "age"; "city"]

let path = "/home/user/docs" in
let parts = String.split "/" path  // [""; "home"; "user"; "docs"]

// Empty parts are preserved
let data = "a,,b" in
let parts = String.split "," data  // ["a"; ""; "b"]
```

**Note:**
- Empty strings between consecutive delimiters are preserved
- If delimiter is not found, returns a single-element list with the original string

**See also:**
- [String.concat](#stringconcat) - Join strings together

---

### String.concat

Concatenates a list of strings into a single string.

**Signature:**
```fsharp
String.concat : string list -> string
```

**Parameters:**
- `strings` - A list of strings to concatenate

**Returns:**
- A single string containing all elements joined together

**Example:**
```fsharp
let parts = ["hello"; " "; "world"] in
let message = String.concat parts  // "hello world"

let empty = [] in
let result = String.concat empty  // ""

let paths = ["/home"; "/user"; "/docs"] in
let fullPath = String.concat paths  // "/home/user/docs"
```

**Note:**
- Does not insert any separator between strings
- For joining with a separator, build the separator into the list

**See also:**
- [String.split](#stringsplit) - Split strings into lists

---

### String.contains

Checks if a string contains a substring.

**Signature:**
```fsharp
String.contains : string -> string -> bool
```

**Parameters:**
- `needle` - The substring to search for
- `haystack` - The string to search in

**Returns:**
- `true` if the haystack contains the needle, `false` otherwise

**Example:**
```fsharp
let text = "hello world" in
let hasWorld = String.contains "world" text  // true
let hasGoodbye = String.contains "goodbye" text  // false

let empty = "" in
let hasEmpty = String.contains "" text  // true (empty string is in all strings)
```

**See also:**
- [String.startsWith](#stringstartswith) - Check prefix
- [String.endsWith](#stringendswith) - Check suffix

---

### String.startsWith

Checks if a string starts with a given prefix.

**Signature:**
```fsharp
String.startsWith : string -> string -> bool
```

**Parameters:**
- `prefix` - The prefix to check for
- `str` - The string to check

**Returns:**
- `true` if the string starts with the prefix, `false` otherwise

**Example:**
```fsharp
let text = "hello world" in
let startsHello = String.startsWith "hello" text  // true
let startsWorld = String.startsWith "world" text  // false

let empty = "" in
let startsEmpty = String.startsWith "" text  // true
```

**See also:**
- [String.endsWith](#stringendswith) - Check suffix
- [String.contains](#stringcontains) - Check substring

---

### String.endsWith

Checks if a string ends with a given suffix.

**Signature:**
```fsharp
String.endsWith : string -> string -> bool
```

**Parameters:**
- `suffix` - The suffix to check for
- `str` - The string to check

**Returns:**
- `true` if the string ends with the suffix, `false` otherwise

**Example:**
```fsharp
let text = "hello world" in
let endsWorld = String.endsWith "world" text  // true
let endsHello = String.endsWith "hello" text  // false

let empty = "" in
let endsEmpty = String.endsWith "" text  // true
```

**See also:**
- [String.startsWith](#stringstartswith) - Check prefix
- [String.contains](#stringcontains) - Check substring

---

## Usage Examples

### String Cleaning Pipeline

```fsharp
let rawInput = "  HELLO WORLD  " in
let cleaned = rawInput
    |> String.trim
    |> String.toLower
// Result: "hello world"
```

### Processing CSV Data

```fsharp
let csvLine = "  John Doe  ,  42  ,  New York  " in
let fields = String.split "," csvLine in
let cleanFields = List.map String.trim fields
// Result: ["John Doe"; "42"; "New York"]
```

### Building a Path

```fsharp
let home = "/home" in
let user = "alice" in
let file = "document.txt" in
let path = String.concat [home; "/"; user; "/"; file]
// Result: "/home/alice/document.txt"
```

### Case-Insensitive Comparison

```fsharp
let normalize str = str |> String.trim |> String.toLower in

let input1 = "  Hello World  " in
let input2 = "hello world" in
let equal = normalize input1 = normalize input2  // true
```

### Validating Input

```fsharp
let validateEmail email =
    if String.contains "@" email && String.contains "." email
    then Some(email)
    else None

let valid = validateEmail "user@example.com"  // Some("user@example.com")
let invalid = validateEmail "notanemail"  // None
```

### Text Search

```fsharp
let searchText query text =
    let normalizedQuery = String.toLower query in
    let normalizedText = String.toLower text in
    String.contains normalizedQuery normalizedText

let found = searchText "WORLD" "Hello world"  // true
```

### URL Parsing

```fsharp
let url = "https://example.com/api/users" in
let hasHttps = String.startsWith "https://" url  // true
let isApi = String.contains "/api/" url  // true
let isUsers = String.endsWith "/users" url  // true
```

### Building Sentences

```fsharp
let buildGreeting name =
    let words = ["Hello"; ","; " "; name; "!"] in
    String.concat words

let greeting = buildGreeting "Alice"
// Result: "Hello, Alice!"
```

## Type Safety

All String functions are strongly typed:
- String operations only accept string arguments
- Type mismatches are caught at compile time
- String lists must contain only strings

## Unicode Support

String operations are fully Unicode-aware:
- `String.length` counts Unicode characters, not bytes
- Case conversion handles international characters
- All operations preserve valid UTF-8 encoding

## Performance Characteristics

| Function    | Time Complexity | Space Complexity |
|-------------|----------------|------------------|
| length      | O(n)           | O(1)             |
| trim        | O(n)           | O(n)             |
| toUpper     | O(n)           | O(n)             |
| toLower     | O(n)           | O(n)             |
| split       | O(n)           | O(n)             |
| concat      | O(n)           | O(n)             |
| contains    | O(n*m)         | O(1)             |
| startsWith  | O(m)           | O(1)             |
| endsWith    | O(m)           | O(1)             |

Where:
- n = length of the string
- m = length of the search pattern

## Notes

- All string operations are immutable
- Strings are UTF-8 encoded
- Empty string is represented as `""`
- String operations integrate with the pipeline operator `|>`
- Case conversion is Unicode-aware (handles international characters)
