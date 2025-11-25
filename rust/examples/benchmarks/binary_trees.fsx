// Binary Trees Benchmark
// Tests allocation pressure and GC performance

type Tree =
    | Leaf of int
    | Node of Tree * int * Tree

let rec make_tree depth value =
    if depth = 0 then
        Leaf value
    else
        Node (
            make_tree (depth - 1) (value * 2),
            value,
            make_tree (depth - 1) (value * 2 + 1)
        )

let rec check_tree tree =
    match tree with
    | Leaf value -> value
    | Node (left, value, right) ->
        value + check_tree left + check_tree right

let rec iterate n depth =
    if n = 0 then
        0
    else
        let tree = make_tree depth n
        let result = check_tree tree
        result + iterate (n - 1) depth

// Create and traverse binary trees of depth 10
// This creates lots of allocations
let max_depth = 10
iterate 100 max_depth
