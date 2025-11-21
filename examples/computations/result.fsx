// Fusabi Computation Expressions Example
// Demonstrates custom DSL with computation expressions (manually desugared)

// ============================================
// Result Type - For Error Handling
// ============================================

type Result<'T, 'E> =
    | Ok of 'T
    | Error of 'E

// Result computation builder
// In full F#, this would enable 'result { ... }' syntax
// Here we show the desugared version
let resultBuilder = {
    // bind: M<'a> -> ('a -> M<'b>) -> M<'b>
    bind = fun (result, f) ->
        match result with
        | Ok value -> f value
        | Error err -> Error err

    // return: 'a -> M<'a>
    return = fun value -> Ok value

    // returnFrom: M<'a> -> M<'a>
    returnFrom = fun result -> result

    // zero: unit -> M<'a>
    zero = fun () -> Error "No value"

    // combine: M<'a> -> M<'a> -> M<'a>
    combine = fun (r1, r2) ->
        match r1 with
        | Error _ -> r2
        | Ok _ -> r1
}

// ============================================
// Example: Division with Error Handling
// ============================================

let divide x y =
    if y = 0 then
        Error "Division by zero"
    else
        Ok (x / y)

let divideFloat x y =
    if y = 0.0 then
        Error "Division by zero"
    else
        Ok (x / y)

// Using computation expression (desugared manually)
// In full F#, this would be:
// result {
//     let! a = divide 100 10
//     let! b = divide a 2
//     return b
// }

let compute1 x y z =
    resultBuilder.bind(
        divide x y,
        fun a ->
            resultBuilder.bind(
                divide a z,
                fun b ->
                    resultBuilder.return b
            )
    )

// More complex example with multiple operations
let compute2 x y z w =
    resultBuilder.bind(
        divide x y,
        fun a ->
            resultBuilder.bind(
                divide a z,
                fun b ->
                    resultBuilder.bind(
                        divide b w,
                        fun c ->
                            resultBuilder.return c
                    )
            )
    )

// ============================================
// Option Type - For Nullable Values
// ============================================

type Option<'T> =
    | Some of 'T
    | None

// Option computation builder
let optionBuilder = {
    bind = fun (opt, f) ->
        match opt with
        | Some value -> f value
        | None -> None

    return = fun value -> Some value

    returnFrom = fun opt -> opt

    zero = fun () -> None
}

// Example: Safe list operations
let tryHead list =
    match list with
    | [] -> None
    | h :: _ -> Some h

let tryTail list =
    match list with
    | [] -> None
    | _ :: t -> Some t

// Using option computation (desugared)
// option {
//     let! head = tryHead list
//     let! tail = tryTail list
//     let! secondHead = tryHead tail
//     return secondHead
// }

let getSecondElement list =
    optionBuilder.bind(
        tryHead list,
        fun _ ->
            optionBuilder.bind(
                tryTail list,
                fun tail ->
                    optionBuilder.bind(
                        tryHead tail,
                        fun secondHead ->
                            optionBuilder.return secondHead
                    )
            )
    )

// ============================================
// List Computation - For Multiple Values
// ============================================

// List monad operations
let listBind list f =
    List.concat (List.map f list)

let listReturn value = [value]

let listBuilder = {
    bind = listBind
    return = listReturn
    zero = fun () -> []
}

// Example: Cartesian product
// list {
//     let! x = [1; 2; 3]
//     let! y = [4; 5; 6]
//     return (x, y)
// }

let cartesianProduct xs ys =
    listBuilder.bind(
        xs,
        fun x ->
            listBuilder.bind(
                ys,
                fun y ->
                    listBuilder.return (x, y)
            )
    )

// ============================================
// State Computation - For Stateful Operations
// ============================================

type State<'S, 'A> = State of ('S -> ('A * 'S))

let runState (State f) s = f s

let stateBuilder = {
    bind = fun (State m, f) ->
        State (fun s ->
            let (a, s') = m s
            let (State m') = f a
            m' s'
        )

    return = fun a -> State (fun s -> (a, s))

    get = State (fun s -> (s, s))

    put = fun s -> State (fun _ -> ((), s))
}

// Example: Counter with state
let increment =
    State (fun count -> (count, count + 1))

let decrement =
    State (fun count -> (count, count - 1))

// Using state computation (desugared)
let counterProgram =
    stateBuilder.bind(
        increment,
        fun x ->
            stateBuilder.bind(
                increment,
                fun y ->
                    stateBuilder.bind(
                        decrement,
                        fun z ->
                            stateBuilder.return (x, y, z)
                    )
            )
    )

// ============================================
// Async Computation - For Asynchronous Operations
// ============================================

// Simplified async type (real implementation would use promises/futures)
type Async<'T> = Async of (unit -> 'T)

let runAsync (Async f) = f ()

let asyncBuilder = {
    bind = fun (Async m, f) ->
        Async (fun () ->
            let a = m ()
            let (Async m') = f a
            m' ()
        )

    return = fun value -> Async (fun () -> value)

    delay = fun f -> Async f
}

// Example: Simulated async operations
let asyncOperation1 () =
    Async (fun () ->
        // Simulate delay
        42
    )

let asyncOperation2 x =
    Async (fun () ->
        // Simulate delay
        x * 2
    )

// Using async computation (desugared)
let asyncProgram =
    asyncBuilder.bind(
        asyncOperation1 (),
        fun x ->
            asyncBuilder.bind(
                asyncOperation2 x,
                fun y ->
                    asyncBuilder.return y
            )
    )

// ============================================
// Tests and Examples
// ============================================

// Test Result computation
let test1 = compute1 100 10 2    // Ok 5
let test2 = compute1 100 0 2     // Error "Division by zero"
let test3 = compute1 100 10 0    // Error "Division by zero"

printfn "Result Tests:"
printfn "  compute1 100 10 2 = %A" test1
printfn "  compute1 100 0 2 = %A" test2
printfn "  compute1 100 10 0 = %A" test3
printfn ""

// Test Option computation
let list1 = [1; 2; 3; 4]
let list2 = [42]
let list3 = []

let test4 = getSecondElement list1  // Some 2
let test5 = getSecondElement list2  // None
let test6 = getSecondElement list3  // None

printfn "Option Tests:"
printfn "  getSecondElement [1;2;3;4] = %A" test4
printfn "  getSecondElement [42] = %A" test5
printfn "  getSecondElement [] = %A" test6
printfn ""

// Test List computation
let test7 = cartesianProduct [1; 2] [3; 4]
// [(1,3); (1,4); (2,3); (2,4)]

printfn "List Tests:"
printfn "  cartesianProduct [1;2] [3;4] = %A" test7
printfn ""

// Test State computation
let (result, finalState) = runState counterProgram 0
// result = (0, 1, 2), finalState = 1

printfn "State Tests:"
printfn "  counterProgram from 0 = result: %A, final: %d" result finalState
printfn ""

// Test Async computation
let asyncResult = runAsync asyncProgram  // 84

printfn "Async Tests:"
printfn "  asyncProgram = %d" asyncResult