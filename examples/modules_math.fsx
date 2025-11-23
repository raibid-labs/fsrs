// Math library module example
//
// This example demonstrates:
// - Multiple functions in a module
// - Recursive functions in modules
// - Using modules like a library

module MathLib =
    let abs x = if x < 0 then -x else x

    let max x y = if x > y then x else y

    let min x y = if x < y then x else y

    let rec factorial n =
        if n <= 1 then 1
        else n * factorial (n - 1)

    let rec fibonacci n =
        if n <= 1 then n
        else fibonacci (n - 1) + fibonacci (n - 2)

    let pow x n =
        let rec helper x n acc =
            if n = 0 then acc
            else helper x (n - 1) (acc * x)
        helper x n 1

// Import all MathLib functions
open MathLib

// Use library functions
let a = abs (-42)
let b = max 10 20
let c = factorial 5
let d = fibonacci 7
let e = pow 2 8

// Result: 2^8 = 256
e
