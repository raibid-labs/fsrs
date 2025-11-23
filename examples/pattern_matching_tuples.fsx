// Pattern Matching with Tuples
// Demonstrates destructuring tuples and mixed patterns

// Classify points on a 2D plane
let classify_point p =
  match p with
  | (0, 0) -> "origin"
  | (0, y) -> "y-axis"
  | (x, 0) -> "x-axis"
  | (x, y) -> "quadrant"

print (classify_point (0, 0))  // => "origin"
print (classify_point (0, 5))  // => "y-axis"
print (classify_point (3, 0))  // => "x-axis"
print (classify_point (3, 4))  // => "quadrant"

// Extract tuple elements
let first p =
  match p with
  | (x, _) -> x

print (first (10, 20))  // => 10

let second p =
  match p with
  | (_, y) -> y

print (second (10, 20))  // => 20

// Swap tuple elements
let swap p =
  match p with
  | (x, y) -> (y, x)

print (swap (1, 2))  // => (2, 1)

// Add tuple elements
let add_pair p =
  match p with
  | (a, b) -> a + b

print (add_pair (3, 4))  // => 7
