// Basic list operations
// Demonstrates list literal syntax and cons construction

// Empty list
let empty = [] in

// Single element list
let single = [42] in

// Multiple element list
let numbers = [1; 2; 3; 4; 5] in

// Cons construction: building lists with :: operator
// 1 :: 2 :: [] creates [1; 2]
let one_two = 1 :: 2 :: [] in

// Prepend element to existing list
let prepend = 0 :: numbers in

// Return prepend for display
// Expected output: [0; 1; 2; 3; 4; 5]
prepend
