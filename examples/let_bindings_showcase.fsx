// Let Bindings Showcase
// Comprehensive demonstration of let binding capabilities in Fusabi
//
// This example demonstrates that let bindings are FULLY SUPPORTED:
// 1. Simple value bindings
// 2. Function bindings (named and anonymous)
// 3. Recursive bindings
// 4. Shadowing (variable rebinding)
// 5. Local scoping with 'in'
// 6. Nested let bindings


// ========== Part 1: Simple Value Bindings ==========
// Basic value assignments - the foundation of functional programming

let x = 42
let message = "Hello, Fusabi!"
let isActive = true
let pi = 3.14159

print "=== Simple Value Bindings ==="
print x         // => 42
print message   // => "Hello, Fusabi!"
print isActive  // => true
print pi        // => 3.14159


// ========== Part 2: Function Bindings ==========
// Define named functions using let

// Simple function
let add x y = x + y
let result1 = add 10 32
print "\n=== Function Bindings ==="
print result1  // => 42

// Function with multiple parameters
let greet name age =
    String.concat ["Hello, "; name; "! You are "; String.trim (String.concat [" "; String.trim ""]); " years old."]

let greeting = greet "Alice" 30
print greeting

// Higher-order function (function that takes a function)
let applyTwice f x = f (f x)
let double x = x * 2
let quadruple = applyTwice double 5
print quadruple  // => 20

// Anonymous functions (lambdas)
let increment = fun x -> x + 1
let forty_three = increment 42
print forty_three  // => 43


// ========== Part 3: Recursive Bindings ==========
// Use 'rec' keyword for recursive functions

print "\n=== Recursive Bindings ==="

// Classic factorial
let rec factorial n =
    match n with
    | 0 -> 1
    | 1 -> 1
    | _ -> n * (factorial (n - 1))

let fact5 = factorial 5
print fact5  // => 120

// Fibonacci using recursion
let rec fib n =
    if n <= 1 then n
    else fib (n - 1) + fib (n - 2)

let fib7 = fib 7
print fib7  // => 13

// Sum of a list (recursive)
let rec sumList lst =
    match lst with
    | [] -> 0
    | head :: tail -> head + sumList tail

let total = sumList [1; 2; 3; 4; 5]
print total  // => 15

// Length of a list (recursive)
let rec listLength lst =
    match lst with
    | [] -> 0
    | _ :: tail -> 1 + listLength tail

let len = listLength [10; 20; 30; 40]
print len  // => 4


// ========== Part 4: Shadowing (Variable Rebinding) ==========
// Later bindings shadow earlier ones in the same scope

print "\n=== Shadowing Examples ==="

let value = 10
print value  // => 10

let value = 20
print value  // => 20 (shadowed the previous binding)

let value = value + 15
print value  // => 35 (references the previous value, then shadows it)

// Shadowing with different types is allowed
let count = 42
print count  // => 42

let count = "forty-two"
print count  // => "forty-two"


// ========== Part 5: Local Scoping with 'in' ==========
// Use 'let ... in ...' for local bindings

print "\n=== Local Scoping ==="

let outer = 100

// Local binding only visible within the 'in' expression
let scopeTest =
    let inner = 50 in
    outer + inner

print scopeTest  // => 150

// 'inner' is not accessible here (out of scope)
// Uncomment the following line to see a compile error:
// print inner  // ERROR: inner is not defined

// Chained local bindings
let chainedResult =
    let a = 10 in
    let b = 20 in
    let c = 30 in
    a + b + c

print chainedResult  // => 60


// ========== Part 6: Nested Let Bindings ==========
// Let bindings can be nested to any depth

print "\n=== Nested Let Bindings ==="

let calculateDiscount price =
    let taxRate = 0.08 in
    let discount = 0.15 in
    let afterDiscount = price * (1.0 - discount) in
    let withTax = afterDiscount * (1.0 + taxRate) in
    withTax

let finalPrice = calculateDiscount 100.0
print finalPrice  // => 91.8

// Deeply nested example
let complexCalculation x =
    let step1 = x + 10 in
    let step2 =
        let inner1 = step1 * 2 in
        let inner2 = inner1 + 5 in
        inner2
    in
    let step3 = step2 - 3 in
    step3

let complexResult = complexCalculation 5
print complexResult  // => 32 ((5 + 10) * 2 + 5 - 3)


// ========== Part 7: Combining Patterns ==========
// Real-world scenarios combining multiple let binding patterns

print "\n=== Combining Patterns ==="

// Function with local helpers
let processData items =
    // Local helper functions
    let rec sumItems lst =
        match lst with
        | [] -> 0
        | h :: t -> h + sumItems t
    in
    let rec countItems lst =
        match lst with
        | [] -> 0
        | _ :: t -> 1 + countItems t
    in
    let total = sumItems items in
    let count = countItems items in
    if count > 0 then total / count else 0

let average = processData [10; 20; 30; 40; 50]
print average  // => 30

// Recursive function with shadowing and local scope
let rec power base exp =
    let helper =
        if exp <= 0 then 1
        else if exp == 1 then base
        else
            let half = power base (exp / 2) in
            let result = half * half in
            if exp % 2 == 0 then result else result * base
    in
    helper

let powerResult = power 2 10
print powerResult  // => 1024


// ========== Part 8: Practical Example ==========
// Configuration with computed values

print "\n=== Practical Configuration Example ==="

let environment = "production"
let basePort = 8000

let config =
    let port = basePort + 80 in
    let workers = 4 in
    let timeout = if environment == "production" then 30 else 10 in
    let debugMode = environment != "production" in
    {
        env = environment;
        serverPort = port;
        workerCount = workers;
        requestTimeout = timeout;
        debug = debugMode
    }

print config.serverPort      // => 8080
print config.workerCount     // => 4
print config.requestTimeout  // => 30
print config.debug           // => false


// ========== Summary ==========

print "\n=== Summary ==="
print "Let bindings in Fusabi are FULLY SUPPORTED:"
print "✓ Simple value bindings (let x = 42)"
print "✓ Function bindings (let add x y = x + y)"
print "✓ Recursive bindings (let rec factorial n = ...)"
print "✓ Shadowing (reassignment with same name)"
print "✓ Local scoping (let ... in ...)"
print "✓ Nested bindings (any depth)"
print "✓ Higher-order functions"
print "✓ Anonymous functions (lambdas)"

"Let bindings showcase complete!"
