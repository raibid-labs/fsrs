// Nested arrays (arrays of arrays)
// Demonstrates matrix-like structures and nested indexing

let matrix = [|[|1; 2|]; [|3; 4|]; [|5; 6|]|] in
let empty_matrix = [|[||]; [||]; [||]|] in
let row0 = matrix.[0] in
let element = matrix.[1].[0] in
let updated = matrix.[0] <- [|99; 88|] in

// More complex nesting
let cube = [|[|[|1; 2|]; [|3; 4|]|]; [|[|5; 6|]; [|7; 8|]|]|] in
let cube_elem = cube.[0].[1].[0] in

// Mixed depth arrays
let mixed = [|[|1; 2; 3|]; [|4; 5|]; [|6|]|] in
let mixed_row0 = mixed.[0] in
let mixed_row1 = mixed.[1] in
let mixed_row2 = mixed.[2] in

// Return results
(matrix, row0, element, updated, cube_elem, mixed_row0, mixed_row1, mixed_row2)
// Expected: ([|[|1; 2|]; [|3; 4|]; [|5; 6|]|], [|1; 2|], 3, [|[|99; 88|]; [|3; 4|]; [|5; 6|]|],
//            3, [|1; 2; 3|], [|4; 5|], [|6|])
