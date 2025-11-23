// Record Pattern Matching Examples for Fusabi
// Demonstrates pattern matching over record types

// Example 1: Simple pattern matching
let classifyPoint point =
    match point with
    | { x = 0; y = 0 } -> "Origin"
    | { x = 0; y = _ } -> "Y-axis"
    | { x = _; y = 0 } -> "X-axis"
    | _ -> "General point"

let origin = { x = 0; y = 0 }
let xAxis = { x = 5; y = 0 }
let classification = classifyPoint origin

// Example 2: Partial pattern matching
let isAtOrigin point =
    match point with
    | { x = 0; y = 0 } -> true
    | _ -> false

// Example 3: Pattern matching with guards
let quadrant point =
    match point with
    | { x = x; y = y } when x > 0 && y > 0 -> "Q1"
    | { x = x; y = y } when x < 0 && y > 0 -> "Q2"
    | { x = x; y = y } when x < 0 && y < 0 -> "Q3"
    | { x = x; y = y } when x > 0 && y < 0 -> "Q4"
    | _ -> "On axis"

// Example 4: Nested record patterns
let describeCircle circle =
    match circle with
    | { center = { x = 0; y = 0 }; radius = r } ->
        "Circle at origin with radius " + string r
    | { center = _; radius = r } when r > 10 -> "Large circle"
    | { center = _; radius = r } when r > 5 -> "Medium circle"
    | _ -> "Small circle"

// Example 5: Record destructuring
let getCoordinates point =
    let { x = x; y = y } = point
    (x, y)

let coords = getCoordinates { x = 10; y = 20 }

// Example 6: Conditional record updates
let updatePersonAge person newAge =
    if newAge >= 0 && newAge <= 150 then
        { person with age = newAge }
    else
        person

let person = { name = "Alice"; age = 30 }
let validUpdate = updatePersonAge person 31
let invalidUpdate = updatePersonAge person 200

// Example 7: Record field extraction
let extractName record =
    record.name

let extractAge record =
    record.age

let person2 = { name = "Bob"; age = 25 }
let name = extractName person2

// Example 8: Record comparison patterns
let comparePoints p1 p2 =
    match (p1, p2) with
    | ({ x = x1; y = y1 }, { x = x2; y = y2 }) when x1 = x2 && y1 = y2 ->
        "Equal points"
    | ({ x = x1; y = _ }, { x = x2; y = _ }) when x1 = x2 ->
        "Same X coordinate"
    | ({ x = _; y = y1 }, { x = _; y = y2 }) when y1 = y2 ->
        "Same Y coordinate"
    | _ -> "Different points"

// Example 9: Option record patterns
let tryGetName record =
    if record.name <> "" then
        Some record.name
    else
        None

let result1 = tryGetName { name = "Alice"; age = 30 }
let result2 = tryGetName { name = ""; age = 30 }

// Example 10: List of records pattern matching
let countOrigins points =
    let rec helper pts count =
        match pts with
        | [] -> count
        | { x = 0; y = 0 } :: rest -> helper rest (count + 1)
        | _ :: rest -> helper rest count
    helper points 0

let pointList = [
    { x = 0; y = 0 };
    { x = 1; y = 1 };
    { x = 0; y = 0 };
    { x = 2; y = 2 }
]

let originCount = countOrigins pointList
