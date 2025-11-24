// Error Handling with Option Types
// Demonstrates safe error handling using Option<'a>
//
// In Fusabi, we handle errors functionally using the Option type:
// - Some(value) represents success with a value
// - None represents absence of a value or failure

// Example 1: Safe division
let safeDivide x y =
    match y with
    | 0 -> None
    | _ -> Some(x / y)

let result1 = safeDivide 10 2   // Some(5)
let result2 = safeDivide 10 0   // None

print (Option.isSome result1)   // => true
print (Option.isNone result2)   // => true

// Example 2: Safe list operations
let safeHead list =
    match list with
    | [] -> None
    | x :: _ -> Some(x)

let list1 = [1; 2; 3]
let list2 = []

let head1 = safeHead list1  // Some(1)
let head2 = safeHead list2  // None

// Example 3: Default values with Option
let getWithDefault opt defaultValue =
    match opt with
    | Some(v) -> v
    | None -> defaultValue

let value1 = getWithDefault result1 0   // => 5
let value2 = getWithDefault result2 0   // => 0

print value1
print value2

// Example 4: Chaining optional operations
let parseNumber str =
    match str with
    | "0" -> Some(0)
    | "1" -> Some(1)
    | "2" -> Some(2)
    | _ -> None

let process input =
    match parseNumber input with
    | Some(n) -> Some(n * 2)
    | None -> None

let parsed1 = process "1"   // Some(2)
let parsed2 = process "x"   // None

print (Option.defaultValue 0 parsed1)  // => 2
print (Option.defaultValue 0 parsed2)  // => 0

// Example 5: Real-world use case - configuration lookup
let getConfig key =
    match key with
    | "host" -> Some("localhost")
    | "port" -> Some("8080")
    | "debug" -> Some("true")
    | _ -> None

let host = getConfig "host" |> Option.defaultValue "0.0.0.0"
let port = getConfig "port" |> Option.defaultValue "3000"
let timeout = getConfig "timeout" |> Option.defaultValue "30"

print host      // => "localhost"
print port      // => "8080"
print timeout   // => "30" (fallback value)

// Best Practice: Always use Option for operations that might fail
// - Safer than exceptions
// - Forces callers to handle both success and failure cases
// - Makes error handling explicit in the type system

"Error handling complete"
