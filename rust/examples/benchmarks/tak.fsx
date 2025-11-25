// Takeuchi Function Benchmark
// Tests deep recursion with multiple recursive calls

let rec tak x y z =
    if y >= x then
        z
    else
        tak (tak (x - 1) y z)
            (tak (y - 1) z x)
            (tak (z - 1) x y)

// Classic Takeuchi test case with deep recursion
tak 18 12 6
