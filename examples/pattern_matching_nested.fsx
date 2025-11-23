// Pattern Matching with Nested Tuples
// Demonstrates nested destructuring and complex patterns

// Match nested tuples
let describe_nested t =
  match t with
  | (0, (0, 0)) -> "zero with origin"
  | (x, (0, 0)) -> "non-zero with origin"
  | (0, (x, y)) -> "zero with point"
  | (a, (b, c)) -> "general case"

print (describe_nested (0, (0, 0)))  // => "zero with origin"
print (describe_nested (5, (0, 0)))  // => "non-zero with origin"
print (describe_nested (0, (3, 4)))  // => "zero with point"
print (describe_nested (1, (2, 3)))  // => "general case"

// Extract from nested tuple
let get_nested_value t =
  match t with
  | (x, (y, z)) -> y

print (get_nested_value (1, (2, 3)))  // => 2

// Calculate with nested tuple
let sum_nested t =
  match t with
  | (a, (b, c)) -> a + b + c

print (sum_nested (1, (2, 3)))  // => 6

// Triple nested
let triple_sum t =
  match t with
  | (a, b, c) -> a + b + c

print (triple_sum (10, 20, 30))  // => 60
