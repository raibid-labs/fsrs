// Simple Currying Example for FSRS
// Demonstrates multi-parameter function syntax and partial application

// Basic two-parameter function
let add x y = x + y

// Partial application creates specialized functions
let add10 = add 10
let add20 = add 20

// Use the partially applied functions
let result1 = add10 5   // Returns 15
let result2 = add20 5   // Returns 25

// Three-parameter function
let addThree x y z = x + y + z

// Multiple levels of partial application
let add10 = addThree 10
let add10_20 = add10 20

// Complete the application
in add10_20 5  // Returns 35
