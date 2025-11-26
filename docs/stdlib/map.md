# Map Module

The `Map` module provides an immutable key-value dictionary data structure. Maps use string keys and can store any value type. All operations return new maps rather than modifying existing ones.

## Overview

Maps in Fusabi are immutable hash maps that associate string keys with values. They provide efficient lookup and insertion operations while maintaining functional immutability guarantees.

## Functions

### Map.empty

Creates an empty map.

**Signature:**
```fsharp
Map.empty : unit -> Map<string, 'a>
```

**Parameters:**
- `unit` - Unit value (typically `()`)

**Returns:**
- An empty map

**Example:**
```fsharp
let emptyMap = Map.empty() in
let isEmpty = Map.isEmpty emptyMap  // true
```

---

### Map.add

Adds or updates a key-value pair in a map.

**Signature:**
```fsharp
Map.add : string -> 'a -> Map<string, 'a> -> Map<string, 'a>
```

**Parameters:**
- `key` - The key (must be a string)
- `value` - The value to associate with the key
- `map` - The map to add to

**Returns:**
- A new map with the key-value pair added (or updated if key exists)

**Example:**
```fsharp
let empty = Map.empty() in
let map1 = Map.add "name" "Alice" empty in
let map2 = Map.add "age" 30 map1 in
let map3 = Map.add "name" "Bob" map2  // Replaces "Alice" with "Bob"
```

**Note:**
- Keys must be strings
- If the key already exists, the value is replaced
- Returns a new map; does not modify the original

---

### Map.remove

Removes a key-value pair from a map.

**Signature:**
```fsharp
Map.remove : string -> Map<string, 'a> -> Map<string, 'a>
```

**Parameters:**
- `key` - The key to remove
- `map` - The map to remove from

**Returns:**
- A new map with the key-value pair removed

**Example:**
```fsharp
let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30 in

let withoutAge = Map.remove "age" map
let count = Map.count withoutAge  // 1
```

**Note:**
- If the key doesn't exist, returns the map unchanged
- Returns a new map; does not modify the original

---

### Map.find

Looks up a value by key, throws an error if not found.

**Signature:**
```fsharp
Map.find : string -> Map<string, 'a> -> 'a
```

**Parameters:**
- `key` - The key to look up
- `map` - The map to search

**Returns:**
- The value associated with the key

**Throws:**
- Runtime error if the key is not found

**Example:**
```fsharp
let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30 in

let name = Map.find "name" map  // "Alice"
// let error = Map.find "missing" map  // Error: Map key not found: missing
```

**See also:**
- [Map.tryFind](#mapttryfind) - Safe lookup returning Option

---

### Map.tryFind

Safely looks up a value by key, returning an Option.

**Signature:**
```fsharp
Map.tryFind : string -> Map<string, 'a> -> 'a option
```

**Parameters:**
- `key` - The key to look up
- `map` - The map to search

**Returns:**
- `Some(value)` if the key exists
- `None` if the key is not found

**Example:**
```fsharp
let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30 in

let maybeName = Map.tryFind "name" map  // Some("Alice")
let maybeCity = Map.tryFind "city" map  // None

let name = Option.defaultValue "Unknown" maybeName  // "Alice"
```

**See also:**
- [Map.find](#mapfind) - Lookup that throws on missing key
- [Option Module](option.md) - Working with Option values

---

### Map.containsKey

Checks if a map contains a given key.

**Signature:**
```fsharp
Map.containsKey : string -> Map<string, 'a> -> bool
```

**Parameters:**
- `key` - The key to check for
- `map` - The map to search

**Returns:**
- `true` if the key exists, `false` otherwise

**Example:**
```fsharp
let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30 in

let hasName = Map.containsKey "name" map  // true
let hasCity = Map.containsKey "city" map  // false
```

---

### Map.isEmpty

Checks if a map is empty.

**Signature:**
```fsharp
Map.isEmpty : Map<string, 'a> -> bool
```

**Parameters:**
- `map` - The map to check

**Returns:**
- `true` if the map contains no entries, `false` otherwise

**Example:**
```fsharp
let empty = Map.empty() in
let isEmpty = Map.isEmpty empty  // true

let map = Map.add "key" "value" empty in
let notEmpty = Map.isEmpty map  // false
```

**See also:**
- [Map.count](#mapcount) - Get the number of entries

---

### Map.count

Returns the number of key-value pairs in a map.

**Signature:**
```fsharp
Map.count : Map<string, 'a> -> int
```

**Parameters:**
- `map` - The map to count

**Returns:**
- The number of entries in the map

**Example:**
```fsharp
let empty = Map.empty() in
let zero = Map.count empty  // 0

let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30
    |> Map.add "city" "NYC" in
let count = Map.count map  // 3
```

**See also:**
- [Map.isEmpty](#mapisempty) - Check if empty

---

### Map.ofList

Creates a map from a list of key-value tuples.

**Signature:**
```fsharp
Map.ofList : (string * 'a) list -> Map<string, 'a>
```

**Parameters:**
- `list` - A list of 2-tuples where the first element is a string key

**Returns:**
- A map containing all the key-value pairs from the list

**Example:**
```fsharp
let entries = [
    ("name", "Alice");
    ("age", "30");
    ("city", "NYC")
] in
let map = Map.ofList entries

let name = Map.find "name" map  // "Alice"
```

**Note:**
- If duplicate keys exist, the last value wins
- All keys must be strings
- All tuples must be 2-tuples

**Throws:**
- Runtime error if list contains non-tuples
- Runtime error if tuples are not 2-tuples
- Type error if keys are not strings

**See also:**
- [Map.toList](#maptolist) - Convert map to list

---

### Map.toList

Converts a map to a list of key-value tuples.

**Signature:**
```fsharp
Map.toList : Map<string, 'a> -> (string * 'a) list
```

**Parameters:**
- `map` - The map to convert

**Returns:**
- A list of 2-tuples containing all key-value pairs, sorted by key

**Example:**
```fsharp
let map = Map.empty()
    |> Map.add "name" "Alice"
    |> Map.add "age" 30
    |> Map.add "city" "NYC" in

let entries = Map.toList map
// [("age", 30); ("city", "NYC"); ("name", "Alice")]
```

**Note:**
- Entries are sorted alphabetically by key
- Returns a list that can be processed with List module functions

**See also:**
- [Map.ofList](#mapoflist) - Create map from list

---

## Usage Examples

### Building a Configuration Map

```fsharp
let config = Map.empty()
    |> Map.add "host" "localhost"
    |> Map.add "port" 8080
    |> Map.add "timeout" 30
    |> Map.add "debug" true
```

### Safe Lookup with Defaults

```fsharp
let getConfigValue key defaultValue map =
    map
    |> Map.tryFind key
    |> Option.defaultValue defaultValue

let host = getConfigValue "host" "0.0.0.0" config
let port = getConfigValue "port" 3000 config
```

### Updating Nested Values

```fsharp
let updateIfExists key updater map =
    match Map.tryFind key map with
    | Some(value) -> Map.add key (updater value) map
    | None -> map

let incrementPort map =
    updateIfExists "port" (fun p -> p + 1) map

let newConfig = incrementPort config
```

### Building from Data

```fsharp
let userData = [
    ("username", "alice");
    ("email", "alice@example.com");
    ("role", "admin")
] in

let userMap = Map.ofList userData

let isAdmin =
    Map.tryFind "role" userMap
    |> Option.map (fun role -> role = "admin")
    |> Option.defaultValue false
```

### Merging Maps

```fsharp
let merge map1 map2 =
    let entries1 = Map.toList map1 in
    let entries2 = Map.toList map2 in
    let allEntries = List.append entries1 entries2 in
    Map.ofList allEntries

let defaults = Map.empty()
    |> Map.add "theme" "dark"
    |> Map.add "lang" "en" in

let userPrefs = Map.empty()
    |> Map.add "theme" "light" in

let finalPrefs = merge defaults userPrefs
// userPrefs values override defaults
```

### Filtering Map Entries

```fsharp
let filterMap predicate map =
    map
    |> Map.toList
    |> List.filter (fun (k, v) -> predicate k v)
    |> Map.ofList

let onlyNumbers = filterMap
    (fun key value -> String.contains "num" key)
    config
```

### Validating Required Keys

```fsharp
let validateKeys required map =
    let hasAllKeys = List.map
        (fun key -> Map.containsKey key map)
        required in
    List.fold (fun acc x -> acc && x) true hasAllKeys

let requiredFields = ["name"; "email"; "password"] in
let isValid = validateKeys requiredFields userMap
```

## Type Safety

Map operations enforce type safety:
- Keys must always be strings
- All values in a map must have the same type
- Type mismatches are caught at compile time
- Generic type parameter preserves value types

## Performance Characteristics

| Function     | Time Complexity | Space Complexity |
|-------------|----------------|------------------|
| empty       | O(1)           | O(1)             |
| add         | O(n)*          | O(n)             |
| remove      | O(n)*          | O(n)             |
| find        | O(1) avg       | O(1)             |
| tryFind     | O(1) avg       | O(1)             |
| containsKey | O(1) avg       | O(1)             |
| isEmpty     | O(1)           | O(1)             |
| count       | O(1)           | O(1)             |
| ofList      | O(n)           | O(n)             |
| toList      | O(n log n)     | O(n)             |

*Maps are immutable, so add/remove create copies. Time complexity is O(n) for copying.

Where:
- n = number of entries in the map

## Implementation Notes

- Maps are implemented using Rust's HashMap internally
- Maps are immutable; operations return new maps
- Reference counting (Rc) is used for efficient copying
- Keys are always strings
- Values can be any type, but all values in a map must have the same type
- toList returns entries sorted alphabetically by key

## Comparison with F# Map

Fusabi's Map is similar to F# Map but with some differences:
- Keys must be strings (F# Map supports any comparable key)
- Implementation uses HashMap (F# uses balanced trees)
- Average O(1) lookup instead of O(log n)
- Less memory efficient for small maps
- Order is not guaranteed (except in toList which sorts)

## Notes

- All map operations are immutable
- Keys are case-sensitive
- Empty string is a valid key
- Maps integrate well with the pipeline operator `|>`
- Combine with Option module for safe lookups
- Use toList/ofList for functional transformations
