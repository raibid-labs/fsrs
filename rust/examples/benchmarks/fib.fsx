// Recursive Fibonacci Benchmark
// Tests recursion and function call performance

let rec fib n =
    if n <= 1 then
        n
    else
        fib (n - 1) + fib (n - 2)

// Calculate fib(25) which generates significant recursion
fib 25
