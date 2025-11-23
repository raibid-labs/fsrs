// Basic module example - module definition with let bindings
//
// This example demonstrates:
// - Module definition syntax
// - Open imports
// - Qualified name access

module Math =
    let add x y = x + y
    let multiply x y = x * y
    let square x = multiply x x

// Import Math module bindings
open Math

// Use imported function (unqualified)
let result1 = square (add 3 4)

// Use qualified access
let result2 = Math.add 10 20

result1
