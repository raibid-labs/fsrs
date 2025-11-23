// FSRS Standard Library Demonstration
// This file shows how to use the built-in List, String, and Option modules

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


// ========== Option Operations ==========

// Option type definition syntax is not yet stable or fully supported for generics.
// Using explicit constructors for now.
// For type-safe Option handling, it's recommended to register Option.Some and Option.None
// as host functions, or use built-in VM variants where appropriate.

// Create option values (assuming Some/None are in scope or registered as variant constructors)
// The compiler/VM already supports Discriminated Unions and their variants.
// If not defined here, 'Some' and 'None' need to be defined as variants of a type.
// Example: type MyOption = Some of any | None
// Or rely on host-defined variant constructors.
// For this demo, we'll assume Some and None are available as variant constructors.
let someValue = Some(42) in
let noValue = None in

// Check option variants
let isSome = Option.isSome someValue in   // true
let isNone = Option.isNone noValue in     // true

// Get value with default
let value = Option.defaultValue 0 someValue in  // 42
let defaulted = Option.defaultValue 0 noValue in  // 0


// ========== Real-World Examples ==========

// Example 1: Process a CSV-like string
let csvData = "name,age,city" in
let fields = String.split "," csvData in
let fieldCount = List.length fields in  // 3
let upperFields = List.map String.toUpper fields in  // Not implemented yet

// Example 2: Safe string processing with options
let maybeName = Some("Alice") in
let defaultName = Option.defaultValue "Unknown" maybeName in
let greeting = String.concat ["Hello, "; defaultName; "!"] in

// Example 3: List transformations
let data = [1; 2; 3; 4; 5] in
let reversed = List.reverse data in
let doubled = List.map (fun x -> x * 2) data in  // Not implemented yet

// Example 4: String cleaning pipeline
let rawInput = "  HELLO WORLD  " in
let cleaned = rawInput
    |> String.trim
    |> String.toLower in  // "hello world"

// Example 5: Combining list and string operations
let paths = ["/home"; "/user"; "/docs"] in
let fullPath = String.concat (List.append paths ["/file.txt"]) in
// Result: "/home/user/docs/file.txt"


// ========== Notes ==========

// The following higher-order functions are referenced but not yet implemented:
// - List.map : ('a -> 'b) -> 'a list -> 'b list
// - List.filter : ('a -> bool) -> 'a list -> 'a list
// - List.fold : ('a -> 'b -> 'a) -> 'a -> 'b list -> 'a
// - Option.map : ('a -> 'b) -> 'a option -> 'b option
// - Option.bind : ('a -> 'b option) -> 'a option -> 'b option

// These will be added in future iterations when closures and
// first-class functions are fully integrated with the VM.
fullPath // Return the last calculated value to be printed by the CLI
