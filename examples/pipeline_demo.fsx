// Pipeline Operator Demo
// Demonstrates the power of |> for functional composition
//
// The pipeline operator takes the result of the left side
// and passes it as the last argument to the function on the right.
// This enables readable, top-to-bottom data transformations.

// Example 1: String transformation pipeline
let rawText = "  HELLO FUSABI WORLD  "

let cleaned = rawText
    |> String.trim
    |> String.toLower
    |> String.split " "
    |> List.length

print cleaned  // => 3

// Example 2: List processing pipeline
let numbers = [1; 2; 3; 4; 5]

let result = numbers
    |> List.reverse
    |> List.tail
    |> List.head

print result  // => 4

// Example 3: Complex data pipeline
let csvLine = "Alice,30,Engineer"

let processRecord = csvLine
    |> String.split ","
    |> List.head
    |> String.toUpper

print processRecord  // => "ALICE"

// Example 4: Nested pipeline with multiple operations
let data = ["apple"; "banana"; "cherry"; "date"]

let totalLength = data
    |> List.reverse
    |> List.tail
    |> List.length

print totalLength  // => 3

// Example 5: Function composition via pipeline
let double x = x * 2
let addTen x = x + 10

let pipelineResult = 5
    |> double
    |> addTen
    |> double

print pipelineResult  // => 40

// The pipeline operator makes code read like a recipe:
// "Take 5, double it, add 10, then double again"
// Instead of: double (addTen (double 5))

pipelineResult
