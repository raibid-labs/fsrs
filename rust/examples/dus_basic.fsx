// Basic Discriminated Union Examples
// Demonstrates simple enum patterns and basic matching

// Simple Direction Enum (no fields)
type Direction =
    | Left
    | Right
    | Up
    | Down

let direction_to_int dir =
    match dir with
    | Left -> -1
    | Right -> 1
    | Up -> 0
    | Down -> 0

let example1 = direction_to_int Left  // Should be -1

// Option Type (classic example)
type Option<'a> =
    | None
    | Some of 'a

let map_option f opt =
    match opt with
    | None -> None
    | Some(x) -> Some(f x)

let example2 = map_option (fun x -> x + 1) (Some 41)  // Should be Some(42)

// Simple Result Type
type Result<'ok, 'err> =
    | Ok of 'ok
    | Error of 'err

let unwrap_or_default result default =
    match result with
    | Ok(value) -> value
    | Error(_) -> default

let example3 = unwrap_or_default (Ok 100) 0  // Should be 100
let example4 = unwrap_or_default (Error "fail") 0  // Should be 0

// Shape with different variants
type Shape =
    | Circle of int  // radius
    | Rectangle of int * int  // width, height
    | Triangle of int * int * int  // sides

let area shape =
    match shape with
    | Circle(r) -> 3 * r * r  // Simplified π
    | Rectangle(w, h) -> w * h
    | Triangle(a, b, c) -> a + b + c  // Simplified

let example5 = area (Rectangle(10, 20))  // Should be 200

// Boolean enum (Yes/No)
type Answer =
    | Yes
    | No
    | Maybe

let is_positive answer =
    match answer with
    | Yes -> true
    | No -> false
    | Maybe -> false

let example6 = is_positive Yes  // Should be true

// Main expression showing all examples
(example1, example2, example3, example4, example5, example6)
