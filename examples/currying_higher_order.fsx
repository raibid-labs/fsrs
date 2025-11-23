// Higher-Order Functions with Currying
// Demonstrates function composition and curried predicates

// Function composition
let compose f g = fun x -> f (g x)

// Simple transformations
let add10 x = x + 10
let mul2 x = x * 2

// Compose functions to create pipelines
let transform = compose add10 mul2

// Apply the composed function
let result = transform 5  // (5 * 2) + 10 = 20

// Curried comparison functions
let greaterThan threshold value = value > threshold

// Create specialized predicates
let isPositive = greaterThan 0

in result
