// Tuple Equality Examples
// Demonstrates element-wise equality comparison

let t1 = (1, 2, 3) in
let t2 = (1, 2, 3) in
let t3 = (1, 2, 4) in
let equal = t1 == t2 in
let not_equal = t1 == t3 in
let nested1 = ((1, 2), (3, 4)) in
let nested2 = ((1, 2), (3, 4)) in
let nested_equal = nested1 == nested2 in
(equal, not_equal, nested_equal)
