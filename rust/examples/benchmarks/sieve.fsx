// Sieve of Eratosthenes Benchmark
// Tests array/list operations and filtering

let rec filter predicate lst =
    match lst with
    | [] -> []
    | head :: tail ->
        if predicate head then
            head :: filter predicate tail
        else
            filter predicate tail

let rec range start end_val =
    if start > end_val then
        []
    else
        start :: range (start + 1) end_val

let rec sieve lst =
    match lst with
    | [] -> []
    | prime :: rest ->
        let is_not_multiple n = n % prime <> 0
        prime :: sieve (filter is_not_multiple rest)

// Find all primes up to 1000
let numbers = range 2 1000
sieve numbers
