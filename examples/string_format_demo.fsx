// String.format and sprintf demonstration
// Shows printf-style string formatting

// Basic string formatting
let greeting = String.format "Hello, %s!" ["World"] in
printfn greeting;  // Output: "Hello, World!"

// Multiple arguments with different types
let version = String.format "%s version %d.%d" ["MyApp"; 1; 0] in
printfn version;  // Output: "MyApp version 1.0"

// Float formatting
let pi = String.format "Pi is approximately %f" [3.14159] in
printfn pi;  // Output: "Pi is approximately 3.14159"

// Float with precision
let price = String.format "Price: $%.2f" [19.99] in
printfn price;  // Output: "Price: $19.99"

// Using sprintf alias (same functionality)
let message = sprintf "User %s logged in at %.1f seconds" ["Alice"; 12.5] in
printfn message;  // Output: "User Alice logged in at 12.5 seconds"

// Literal percent sign
let progress = String.format "Progress: %d%%" [75] in
printfn progress;  // Output: "Progress: 75%"

// Complex example
let report = String.format "%s: %d items at $%.2f each = $%.2f total"
    ["Product"; 5; 12.99; 64.95] in
printfn report;  // Output: "Product: 5 items at $12.99 each = $64.95 total"

// %s can accept integers and floats too
let mixed = String.format "Values: %s, %s, %s" [42; 3.14; "text"] in
printfn mixed;  // Output: "Values: 42, 3.14, text"

// Note: All arguments must match the format specifiers
// This would error: String.format "%d" ["not a number"]
// This would error: String.format "%s %s" ["only one"]

report
