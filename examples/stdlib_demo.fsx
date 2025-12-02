// FSRS Standard Library Demonstration
// This file shows how to use the built-in List, String, Array, Map, and Option modules

// ========== List Operations ==========

// Create a simple list
let numbers = [1; 2; 3; 4; 5] in

// Get the length of a list
let count = List.length numbers in  // 5

// Access first element
let first = List.head numbers in    // 1

// Get remaining elements
let rest = List.tail numbers in     // [2; 3; 4; 5]

// Reverse a list
let backwards = List.reverse numbers in  // [5; 4; 3; 2; 1]

// Check if list is empty
let isEmpty = List.isEmpty [] in    // true
let notEmpty = List.isEmpty numbers in  // false

// Append two lists
let moreNumbers = [6; 7; 8] in
let combined = List.append numbers moreNumbers in  // [1; 2; 3; 4; 5; 6; 7; 8]

// Concatenate a list of lists
let listOfLists = [[1; 2]; [3; 4]; [5]] in
let flattened = List.concat listOfLists in  // [1; 2; 3; 4; 5]

// Map a function over a list
let doubled = List.map (fun x -> x * 2) numbers in  // [2; 4; 6; 8; 10]

// Filter elements based on a predicate
let evens = List.filter (fun x -> x > 2) numbers in  // [3; 4; 5]

// Fold (reduce) a list to a single value
// NOTE: List.fold requires closure upvalue capture (not yet implemented)
// let sum = List.fold (fun acc x -> acc + x) 0 numbers in  // 15
let sum = 15 in  // Placeholder until upvalue capture is implemented

// Check if any element satisfies a predicate
let hasLarge = List.exists (fun x -> x > 3) numbers in  // true

// Find the first element matching a predicate
let firstLarge = List.find (fun x -> x > 3) numbers in  // 4

// Safely find (returns Option)
let maybeEven = List.tryFind (fun x -> x > 10) numbers in  // None

// Iterate over a list for side effects
let iterResult = List.iter (fun x -> x) numbers in  // Returns unit


// ========== Array Operations ==========

// Create arrays from lists
let arr = Array.ofList [1; 2; 3; 4; 5] in

// Get array length
let arrLen = Array.length arr in  // 5

// Check if array is empty
let arrEmpty = Array.isEmpty arr in  // false

// Get element by index (0-based)
let third = Array.get 2 arr in  // 3

// Set element by index (mutates in place)
let setResult1 = Array.set 0 100 arr in  // arr[0] is now 100

// Create array filled with a value
let zeros = Array.create 5 0 in  // [|0; 0; 0; 0; 0|]

// Create array using initializer function
let squares = Array.init 5 (fun i -> i * i) in  // [|0; 1; 4; 9; 16|]

// Convert array back to list
let arrList = Array.toList squares in  // [0; 1; 4; 9; 16]


// ========== String Operations ==========

// String length (character count, not bytes)
let message = "Hello, World!" in
let msgLength = String.length message in  // 13

// Trim whitespace
let messy = "  hello  " in
let clean = String.trim messy in  // "hello"

// Case conversion
let upper = String.toUpper "hello" in  // "HELLO"
let lower = String.toLower "WORLD" in  // "world"

// Split strings into lists
let sentence = "hello world foo bar" in
let words = String.split " " sentence in  // ["hello"; "world"; "foo"; "bar"]

// Concatenate string lists
let wordList = ["hello"; " "; "world"] in
let joined = String.concat wordList in  // "hello world"

// String predicates
let text = "hello world" in
let hasWorld = String.contains "world" text in     // true
let startsHello = String.startsWith "hello" text in  // true
let endsWorld = String.endsWith "world" text in    // true


// ========== Map Operations ==========

// Create an empty map
let emptyMap = Map.empty () in

// Add entries to a map
// NOTE: Nested calls have parsing issues; use sequential bindings instead
let mapStep1 = Map.add "city" "NYC" emptyMap in
let myMap = Map.add "name" "Alice" mapStep1 in

// Find a value by key
let name = Map.find "name" myMap in  // "Alice"

// Safely find (returns Option)
let maybeName = Map.tryFind "age" myMap in  // None

// Check if key exists
let hasName = Map.containsKey "name" myMap in  // true

// Get map size
let mapSize = Map.count myMap in  // 2

// Transform all values in a map
let upperMap = Map.map String.toUpper myMap in

// Iterate over map entries
// NOTE: Map.iter requires 2-arg closure with upvalue capture (not yet implemented)
// let mapIterResult = Map.iter (fun k v -> ()) myMap in  // Returns unit
let mapIterResult = () in  // Placeholder until upvalue capture is implemented


// ========== Option Operations ==========

// Create option values
let someValue = Some(42) in
let noValue = None in

// Check option variants
let isSome = Option.isSome someValue in   // true
let isNone = Option.isNone noValue in     // true

// Get value with default
let value = Option.defaultValue 0 someValue in  // 42
let defaulted = Option.defaultValue 0 noValue in  // 0

// Map over options
let doubledOpt = Option.map (fun x -> x * 2) someValue in  // Some(84)

// Bind (flatMap) options
let bound = Option.bind (fun x -> Some(x + 1)) someValue in  // Some(43)

// Iterate over option for side effects
let optIterResult = Option.iter (fun x -> ()) someValue in  // Returns unit


// ========== Real-World Examples ==========

// Example 1: Process a CSV-like string
let csvData = "name,age,city" in
let fields = String.split "," csvData in
let fieldCount = List.length fields in  // 3
let upperFields = List.map String.toUpper fields in  // ["NAME"; "AGE"; "CITY"]

// Example 2: Safe string processing with options
let maybeName = Some("Alice") in
let defaultName = Option.defaultValue "Unknown" maybeName in
let greeting = String.concat ["Hello, "; defaultName; "!"] in

// Example 3: List transformations with higher-order functions
let data = [1; 2; 3; 4; 5] in
// NOTE: Chained pipelines have parsing issues; use sequential bindings
// Full example: data |> List.filter (...) |> List.map (...) = [6; 8; 10]
let filtered = data |> List.filter (fun x -> x > 2) in
let processed = filtered |> List.map (fun x -> x * 2) in  // [6; 8; 10]

// Example 4: String cleaning pipeline
let rawInput = "  HELLO WORLD  " in
let trimmed = rawInput |> String.trim in
let cleaned = trimmed |> String.toLower in  // "hello world"

// Example 5: Array manipulation
let arr = Array.init 10 (fun i -> i + 1) in  // [|1; 2; 3; ...; 10|]
let setResult2 = Array.set 0 999 arr in  // Mutate first element
let firstElem = Array.get 0 arr in  // 999

// Example 6: Combining list and string operations
let paths = ["/home"; "/user"; "/docs"] in
let allPaths = List.append paths ["/file.txt"] in
let fullPath = String.concat allPaths in
// Result: "/home/user/docs/file.txt"

fullPath // Return the last calculated value to be printed by the CLI
