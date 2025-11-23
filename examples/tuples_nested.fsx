// Nested Tuple Examples
// Demonstrates arbitrary nesting depth

let simple = (1, (2, 3)) in
let matrix = ((1, 2), (3, 4)) in
let deep = (1, (2, (3, (4, (5, 6))))) in
let mixed = ((1, "a"), (true, (2, "b"))) in
(simple, matrix, deep, mixed)
