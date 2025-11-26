# List Module

The `List` module provides core operations for working with cons-based immutable lists in Fusabi. Lists are the fundamental data structure for collections in functional programming.

## Overview

Lists in Fusabi are implemented as cons cells (linked lists) and can be created using the list literal syntax `[1; 2; 3]` or constructed manually with cons cells. All list operations are immutable and return new lists rather than modifying existing ones.

## Functions

### List.length

Returns the number of elements in a list.

**Signature:**
```fsharp
List.length : 'a list -> int
```

**Parameters:**
- `list` - The list to measure

**Returns:**
- An integer representing the number of elements in the list

**Example:**
```fsharp
let numbers = [1; 2; 3; 4; 5] in
let count = List.length numbers  // 5

let empty = [] in
let zero = List.length empty  // 0
```

**See also:**
- [List.isEmpty](#listisempty) - Check if a list is empty

---

### List.head

Returns the first element of a list.

**Signature:**
```fsharp
List.head : 'a list -> 'a
```

**Parameters:**
- `list` - A non-empty list

**Returns:**
- The first element of the list

**Throws:**
- `EmptyList` error if the list is empty

**Example:**
```fsharp
let numbers = [42; 100; 200] in
let first = List.head numbers  // 42

// Error: List.head []  // Throws EmptyList error
```

**See also:**
- [List.tail](#listtail) - Get all elements except the first

---

### List.tail

Returns a list containing all elements except the first.

**Signature:**
```fsharp
List.tail : 'a list -> 'a list
```

**Parameters:**
- `list` - A non-empty list

**Returns:**
- A new list with all elements except the first

**Throws:**
- `EmptyList` error if the list is empty

**Example:**
```fsharp
let numbers = [1; 2; 3; 4] in
let rest = List.tail numbers  // [2; 3; 4]

let single = [42] in
let empty = List.tail single  // []
```

**See also:**
- [List.head](#listhead) - Get the first element

---

### List.reverse

Returns a new list with elements in reverse order.

**Signature:**
```fsharp
List.reverse : 'a list -> 'a list
```

**Parameters:**
- `list` - The list to reverse

**Returns:**
- A new list with elements in reverse order

**Example:**
```fsharp
let numbers = [1; 2; 3; 4; 5] in
let backwards = List.reverse numbers  // [5; 4; 3; 2; 1]

let empty = [] in
let stillEmpty = List.reverse empty  // []
```

**Performance Note:**
- Time complexity: O(n)
- Space complexity: O(n)

---

### List.isEmpty

Checks whether a list is empty.

**Signature:**
```fsharp
List.isEmpty : 'a list -> bool
```

**Parameters:**
- `list` - The list to check

**Returns:**
- `true` if the list is empty (Nil), `false` otherwise

**Example:**
```fsharp
let empty = [] in
let isEmpty = List.isEmpty empty  // true

let numbers = [1; 2; 3] in
let notEmpty = List.isEmpty numbers  // false
```

**See also:**
- [List.length](#listlength) - Get the number of elements

---

### List.append

Concatenates two lists.

**Signature:**
```fsharp
List.append : 'a list -> 'a list -> 'a list
```

**Parameters:**
- `list1` - The first list
- `list2` - The second list to append

**Returns:**
- A new list containing all elements from `list1` followed by all elements from `list2`

**Example:**
```fsharp
let first = [1; 2; 3] in
let second = [4; 5; 6] in
let combined = List.append first second  // [1; 2; 3; 4; 5; 6]

let nums = [1; 2] in
let empty = [] in
let result = List.append nums empty  // [1; 2]
```

**Performance Note:**
- Time complexity: O(n) where n is the length of the first list
- Space complexity: O(n)

**See also:**
- [List.concat](#listconcat) - Concatenate a list of lists

---

### List.concat

Concatenates a list of lists into a single list.

**Signature:**
```fsharp
List.concat : 'a list list -> 'a list
```

**Parameters:**
- `lists` - A list containing multiple lists to concatenate

**Returns:**
- A single flattened list containing all elements from all sublists

**Example:**
```fsharp
let listOfLists = [[1; 2]; [3; 4]; [5]] in
let flattened = List.concat listOfLists  // [1; 2; 3; 4; 5]

let empty = [] in
let result = List.concat empty  // []

let withEmpties = [[1]; []; [2; 3]; []] in
let combined = List.concat withEmpties  // [1; 2; 3]
```

**See also:**
- [List.append](#listappend) - Concatenate two lists

---

### List.map

Applies a function to each element of a list, returning a new list of results.

**Signature:**
```fsharp
List.map : ('a -> 'b) -> 'a list -> 'b list
```

**Parameters:**
- `func` - A function to apply to each element
- `list` - The input list

**Returns:**
- A new list with the function applied to each element

**Example:**
```fsharp
let numbers = [1; 2; 3; 4; 5] in
let doubled = List.map (fun x -> x * 2) numbers  // [2; 4; 6; 8; 10]

let strings = ["hello"; "world"] in
let lengths = List.map String.length strings  // [5; 5]
```

**See also:**
- [Option.map](option.md#optionmap) - Map over optional values

---

## Usage Examples

### Processing CSV Data

```fsharp
let csvRow = "name,age,city" in
let fields = String.split "," csvRow in
let fieldCount = List.length fields  // 3

let processedFields = List.map String.trim fields in
processedFields
```

### Building a List Pipeline

```fsharp
let data = [1; 2; 3; 4; 5] in
let result = data
    |> List.reverse
    |> List.map (fun x -> x * 2)
    |> List.append [0; 0]
// Result: [10; 8; 6; 4; 2; 0; 0]
```

### Safe List Access

```fsharp
let safeHead list =
    if List.isEmpty list
    then None
    else Some(List.head list)

let numbers = [42; 100] in
let first = safeHead numbers  // Some(42)

let empty = [] in
let noValue = safeHead empty  // None
```

### Flattening Nested Structures

```fsharp
let matrix = [
    [1; 2; 3];
    [4; 5; 6];
    [7; 8; 9]
] in
let allNumbers = List.concat matrix
// Result: [1; 2; 3; 4; 5; 6; 7; 8; 9]
```

## Type Safety

All List functions are generic and work with any element type. The type checker ensures that:
- All elements in a list have the same type
- Functions passed to `List.map` have compatible input/output types
- List operations maintain type consistency

## Performance Characteristics

| Function | Time Complexity | Space Complexity |
|----------|----------------|------------------|
| length   | O(n)           | O(1)             |
| head     | O(1)           | O(1)             |
| tail     | O(1)           | O(1)             |
| reverse  | O(n)           | O(n)             |
| isEmpty  | O(1)           | O(1)             |
| append   | O(n)           | O(n)             |
| concat   | O(n*m)         | O(n*m)           |
| map      | O(n)           | O(n)             |

Where:
- n = length of the list
- m = average length of sublists (for concat)

## Notes

- Lists are immutable. All operations return new lists.
- Lists are implemented as cons cells (linked lists).
- Empty list is represented as `[]` or `Nil`.
- List operations that fail (like `head` on empty list) throw errors at runtime.
- For safe operations, combine with Option module pattern matching.
