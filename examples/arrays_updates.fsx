// Array updates with immutable semantics
// Updates create new arrays, preserving the original

let arr = [|1; 2; 3; 4; 5|] in
let arr2 = arr.[0] <- 99 in
let arr3 = arr.[2] <- 77 in
let arr4 = arr.[4] <- 55 in

// Chained updates (left-to-right evaluation with parentheses)
let result = ((arr.[0] <- 10).[1] <- 20).[2] <- 30 in

// Update with expressions
let base = [|10; 20; 30|] in
let computed = base.[1] <- 100 in

// Return tuple showing immutability and updates
(arr, arr2, arr3, arr4, result, computed)
// Expected: ([|1; 2; 3; 4; 5|], [|99; 2; 3; 4; 5|], [|1; 2; 77; 4; 5|],
//            [|1; 2; 3; 4; 55|], [|10; 20; 30; 4; 5|], [|10; 100; 30|])
