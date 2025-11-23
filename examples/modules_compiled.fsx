// Example demonstrating module-aware compilation in FSRS
//
// This script shows how to define modules, import bindings, and use
// both qualified and unqualified names for module access.

// Define a Math module with arithmetic operations
module Math =
    let add x y = x + y
    let multiply x y = x * y
    let square x = multiply x x
    let double x = add x x

// Define a Utils module with utility functions
module Utils =
    let identity x = x
    let const x y = x
    let flip f x y = f y x

// Define a Constants module
module Constants =
    let pi = 3
    let e = 2
    let maxValue = 100

// Example 1: Use qualified names without import
let result1 = Math.square (Math.add 3 4)  // Should be (3 + 4)^2 = 49

// Example 2: Import Math module and use unqualified names
open Math

let result2 = square (add 5 10)  // Should be (5 + 10)^2 = 225

// Example 3: Use multiple modules with qualified names
let result3 = multiply (Utils.const 5 10) (add 2 3)  // (5) * (2 + 3) = 25

// Example 4: Import multiple modules
open Utils

let result4 = identity (const 42 99)  // Should be 42

// Example 5: Mix qualified and unqualified access
let result5 = add (Constants.pi) (Constants.e)  // 3 + 2 = 5

// Example 6: Nested function application with modules
let result6 = flip multiply 2 (double 3)  // multiply 3 2 flipped -> 3 * (2 * 2) = 12

// Final result - combine all examples
let finalResult = add result1 (add result2 (add result3 (add result4 (add result5 result6))))

finalResult
