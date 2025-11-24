// Fusabi Quickstart Example
// A comprehensive introduction to Fusabi's key features
//
// This example demonstrates:
// 1. Basic values and expressions
// 2. Let bindings
// 3. Functions
// 4. Lists and pattern matching
// 5. Records
// 6. Standard library usage

// === Part 1: Basic Values ===
let greeting = "Welcome to Fusabi!"
let version = 1
let isReady = true

print greeting

// === Part 2: Simple Functions ===
let add x y = x + y
let square x = x * x

let result = add 5 3
print result  // => 8

let squared = square 4
print squared  // => 16

// === Part 3: Lists ===
let numbers = [1; 2; 3; 4; 5]
let count = List.length numbers
print count  // => 5

let doubled = [2; 4; 6; 8; 10]
let first = List.head doubled
print first  // => 2

// === Part 4: Pattern Matching ===
let classify n =
    match n with
    | 0 -> "zero"
    | 1 -> "one"
    | _ -> "many"

print (classify 0)  // => "zero"
print (classify 5)  // => "many"

// === Part 5: Records ===
let person = {
    name = "Alice";
    age = 30;
    active = true
}

print person.name  // => "Alice"
print person.age   // => 30

// === Part 6: Higher-Order Functions ===
// Functions that work with other functions
let apply f x = f x

let addFive x = x + 5
let result2 = apply addFive 10
print result2  // => 15

// === Part 7: Pipeline Operator ===
// Chain operations left-to-right
let text = "  hello world  "
let processed = text
    |> String.trim
    |> String.toUpper

print processed  // => "HELLO WORLD"

// === Part 8: List Processing ===
let names = ["Alice"; "Bob"; "Carol"]
let nameCount = List.length names
print nameCount  // => 3

let reversed = List.reverse names
let lastName = List.head reversed
print lastName  // => "Carol"

// === Part 9: Recursive Functions ===
let rec factorial n =
    match n with
    | 0 -> 1
    | 1 -> 1
    | _ -> n * (factorial (n - 1))

let fact5 = factorial 5
print fact5  // => 120

// === Part 10: Real-World Example ===
// Configuration with validation
let config = {
    server = { host = "localhost"; port = 8080 };
    database = { name = "myapp"; poolSize = 10 };
    features = { debug = true; logging = true }
}

let serverPort = config.server.port
let dbPool = config.database.poolSize

print serverPort  // => 8080
print dbPool      // => 10

// === Summary ===
// You've now seen:
// - Values and types (strings, integers, booleans)
// - Functions and partial application
// - Lists and list operations
// - Pattern matching for control flow
// - Records for structured data
// - Pipeline operator for composition
// - Recursive functions
// - Real-world configuration modeling

"Quickstart complete!"
