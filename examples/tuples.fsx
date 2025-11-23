// Tuple Examples for FSRS
// Demonstrates complete tuple support across all layers
// Each example is a single expression showing a tuple feature

// Basic tuple creation and evaluation
let pair = (1, 2) in
let triple = (42, "hello", true) in
let quintuple = (1, 2, 3, 4, 5) in

// Nested tuples
let nested = (1, (2, 3)) in
let matrix = ((1, 2), (3, 4)) in
let deep = (1, (2, (3, (4, 5)))) in

// Tuples from expressions
let point = (1 + 2, 3 * 4) in
let x = 10 in
let y = 20 in
let coords = (x, y) in

// Computed tuple
let a = 5 in
let b = 3 in
let results = (a + b, a - b, a * b, a == b) in

// Tuple equality
let t1 = (1, 2, 3) in
let t2 = (1, 2, 3) in
let equal = t1 == t2 in

// Nested tuple equality
let n1 = (1, (2, 3)) in
let n2 = (1, (2, 3)) in
let nested_equal = n1 == n2 in

// Mixed types
let mixed = (42, "answer", true, (1, 2)) in

// Conditional returning tuples
let condition = true in
let result = if condition then (1, 2) else (3, 4) in

// Complex nested structure - final result
let base = 10 in
let outer = (base, (base * 2, base + 5)) in
let final = (outer, ((base - 3, base == 10), nested)) in
final
