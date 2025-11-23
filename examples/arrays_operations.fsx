// Array operations and built-in functions
// Demonstrates Array.length and working with array sizes

let numbers = [|1; 2; 3; 4; 5|] in
let len = Array.length numbers in
let empty = [||] in
let empty_len = Array.length empty in
let single = [|42|] in
let single_len = Array.length single in

// Nested array lengths
let matrix = [|[|1; 2; 3|]; [|4; 5|]; [|6|]|] in
let outer_len = Array.length matrix in
let first_len = Array.length matrix.[0] in
let second_len = Array.length matrix.[1] in
let third_len = Array.length matrix.[2] in

// Length after updates
let arr = [|10; 20; 30|] in
let updated = arr.[1] <- 99 in
let arr_len = Array.length arr in
let updated_len = Array.length updated in

// Array with mixed types
let mixed = [|1; true; "hello"|] in
let mixed_len = Array.length mixed in

// Using length in computations
let data = [|100; 200; 300; 400|] in
let size = Array.length data in
let doubled = size + size in

// Return results
(len, empty_len, single_len, outer_len, first_len, second_len, third_len,
 arr_len, updated_len, mixed_len, doubled)
// Expected: (5, 0, 1, 3, 3, 2, 1, 3, 3, 3, 8)
