// Pattern Matching in Functions
// Demonstrates using pattern matching to define function behavior

// Absolute value function
let abs n =
  match n with
  | x -> if x < 0 then 0 - x else x

print (abs 5)    // => 5
print (abs (-5)) // => 5
print (abs 0)    // => 0

// Sign function
let sign n =
  match n with
  | 0 -> 0
  | x -> if x < 0 then (-1) else 1

print (sign 42)  // => 1
print (sign (-10)) // => -1
print (sign 0)   // => 0

// Factorial (pattern match + recursion)
let rec factorial n =
  match n with
  | 0 -> 1
  | 1 -> 1
  | n -> n * factorial (n - 1)

print (factorial 0)  // => 1
print (factorial 1)  // => 1
print (factorial 5)  // => 120

// Distance between two points
let distance p1 p2 =
  match (p1, p2) with
  | ((x1, y1), (x2, y2)) ->
      let dx = x2 - x1 in
      let dy = y2 - y1 in
      dx * dx + dy * dy  // squared distance for simplicity

print (distance (0, 0) (3, 4))  // => 25 (3^2 + 4^2)

// Classify based on multiple conditions
let classify_value v =
  match v with
  | 0 -> "zero"
  | n -> if n > 0 then "positive" else "negative"

print (classify_value 0)    // => "zero"
print (classify_value 10)   // => "positive"
print (classify_value (-5)) // => "negative"
