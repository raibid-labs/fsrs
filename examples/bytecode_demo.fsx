// Bytecode Compilation Demo
// This example showcases Fusabi's dual-runtime capability
//
// Run this file in two ways:
// 1. Interpreted:  fus run examples/bytecode_demo.fsx
// 2. Compiled:     fus grind examples/bytecode_demo.fsx && fus run examples/bytecode_demo.fzb
//
// The output will be identical, but performance characteristics differ:
// - .fsx mode: Faster iteration, source code visible
// - .fzb mode: Faster startup, optimized for deployment

// ===== Computation Example =====
// This performs enough work to make timing differences visible

let rec fibonacci n =
    match n with
    | 0 -> 0
    | 1 -> 1
    | _ -> fibonacci (n - 1) + fibonacci (n - 2)

// Calculate several Fibonacci numbers
let fib5 = fibonacci 5
let fib7 = fibonacci 7
let fib10 = fibonacci 10

print fib5   // => 5
print fib7   // => 13
print fib10  // => 55

// ===== String Processing =====
// Multiple string operations
let message = "  Fusabi is FAST  "
let processed = message
    |> String.trim
    |> String.toLower
    |> String.toUpper

print processed  // => "FUSABI IS FAST"

// ===== List Operations =====
// Build and process a list
let numbers = [1; 2; 3; 4; 5; 6; 7; 8; 9; 10]
let reversed = List.reverse numbers
let tail = List.tail reversed
let head = List.head tail

print head  // => 9

// ===== Pattern Matching =====
let classify value =
    match value with
    | v when v < 0 -> "negative"
    | 0 -> "zero"
    | v when v < 10 -> "small"
    | v when v < 100 -> "medium"
    | _ -> "large"

let class1 = classify (-5)
let class2 = classify 5
let class3 = classify 50
let class4 = classify 500

print class1  // => "negative"
print class2  // => "small"
print class3  // => "medium"
print class4  // => "large"

// ===== Nested Records =====
let complexData = {
    metadata = {
        version = 1;
        timestamp = 1234567890
    };
    payload = {
        items = [
            { id = 1; name = "first" };
            { id = 2; name = "second" };
            { id = 3; name = "third" }
        ]
    };
    stats = {
        count = 3;
        processed = true
    }
}

let itemCount = complexData.stats.count
let isProcessed = complexData.stats.processed

print itemCount    // => 3
print isProcessed  // => true

// ===== Performance Tips =====
//
// When to use .fsx (interpreted):
// - Development and debugging
// - Rapid iteration
// - Learning and experimentation
// - One-off scripts
//
// When to use .fzb (compiled):
// - Production deployment
// - CLI tools (faster startup)
// - Embedded scripts (smaller size)
// - Distribution (no source code exposure)
// - Performance-critical paths
//
// Compilation workflow:
// 1. Develop with .fsx for fast iteration
// 2. Test thoroughly
// 3. Compile to .fzb for deployment
// 4. Deploy .fzb files to production

// ===== Benchmarking =====
// Try timing both modes:
//
// $ time fus run examples/bytecode_demo.fsx
// $ fus grind examples/bytecode_demo.fsx
// $ time fus run examples/bytecode_demo.fzb
//
// You should see significantly faster startup time
// for the .fzb version, especially on repeated runs.

"Bytecode demo complete - try compiling this to .fzb!"
