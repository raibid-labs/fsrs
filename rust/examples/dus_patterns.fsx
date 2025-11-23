// Advanced Pattern Matching with Discriminated Unions
// Demonstrates complex pattern matching scenarios

// ========================================================================
// Nested Option/Result Types
// ========================================================================

type Option<'a> =
    | None
    | Some of 'a

type Result<'ok, 'err> =
    | Ok of 'ok
    | Error of 'err

// Nested Option matching
let unwrap_nested opt_opt =
    match opt_opt with
    | Some(Some(x)) -> x
    | Some(None) -> 0
    | None -> -1

let example1 = unwrap_nested (Some (Some 42))  // Should be 42

// Nested Result matching
let unwrap_result_option res =
    match res with
    | Ok(Some(x)) -> x
    | Ok(None) -> 0
    | Error(_) -> -1

let example2 = unwrap_result_option (Ok (Some 100))  // Should be 100

// ========================================================================
// List of Variants
// ========================================================================

let filter_options opts =
    match opts with
    | [] -> []
    | Some(x) :: rest -> x :: filter_options rest
    | None :: rest -> filter_options rest

let example3 = filter_options [Some(1); None; Some(2); Some(3)]  
// Should be [1; 2; 3]

// ========================================================================
// Tuple with Variants
// ========================================================================

let combine_options opt1 opt2 =
    match (opt1, opt2) with
    | (Some(a), Some(b)) -> Some(a + b)
    | (Some(a), None) -> Some(a)
    | (None, Some(b)) -> Some(b)
    | (None, None) -> None

let example4 = combine_options (Some 10) (Some 20)  // Should be Some(30)

// ========================================================================
// Complex Shape Matching
// ========================================================================

type Shape =
    | Point
    | Circle of int
    | Rectangle of int * int
    | Polygon of int list

let classify_shape shape =
    match shape with
    | Point -> "zero-dimensional"
    | Circle(_) -> "round"
    | Rectangle(w, h) when w = h -> "square"
    | Rectangle(_, _) -> "rectangular"
    | Polygon([]) -> "invalid"
    | Polygon(_) -> "multi-sided"

let example5 = classify_shape (Rectangle(5, 5))  // Should be "square"

// ========================================================================
// Message Passing Example
// ========================================================================

type Message =
    | Quit
    | Move of int * int
    | Write of string
    | ChangeColor of int * int * int

let handle_message msg =
    match msg with
    | Quit -> 0
    | Move(x, y) -> x + y
    | Write(text) -> String.length text
    | ChangeColor(r, g, b) -> r + g + b

let example6 = handle_message (Move(10, 20))  // Should be 30

// ========================================================================
// Tree Structure
// ========================================================================

type Tree<'a> =
    | Leaf
    | Node of 'a * Tree<'a> * Tree<'a>

let tree_depth tree =
    match tree with
    | Leaf -> 0
    | Node(_, left, right) ->
        let left_depth = tree_depth left in
        let right_depth = tree_depth right in
        1 + max left_depth right_depth

let example_tree = Node(5, Node(3, Leaf, Leaf), Node(7, Leaf, Leaf))
let example7 = tree_depth example_tree  // Should be 2

// ========================================================================
// Expression Evaluator
// ========================================================================

type Expr =
    | Const of int
    | Add of Expr * Expr
    | Mul of Expr * Expr
    | Neg of Expr

let rec eval expr =
    match expr with
    | Const(n) -> n
    | Add(e1, e2) -> eval e1 + eval e2
    | Mul(e1, e2) -> eval e1 * eval e2
    | Neg(e) -> -(eval e)

let example_expr = Add(Const(10), Mul(Const(2), Const(3)))
let example8 = eval example_expr  // Should be 16

// ========================================================================
// Wildcard and Guards
// ========================================================================

let categorize_option opt =
    match opt with
    | Some(x) when x > 0 -> "positive"
    | Some(x) when x < 0 -> "negative"
    | Some(_) -> "zero"
    | None -> "none"

let example9 = categorize_option (Some 42)  // Should be "positive"

// ========================================================================
// Main Result
// ========================================================================

(example1, example2, example3, example4, example5, 
 example6, example7, example8, example9)
