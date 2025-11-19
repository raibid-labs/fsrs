// Advanced Record Examples for FSRS
// Demonstrates complex record patterns and use cases

// Example 1: Record-based data structures
type Point = { x: int; y: int }
type Circle = { center: Point; radius: int }

let myCircle = {
    center = { x = 0; y = 0 };
    radius = 10
}

// Example 2: Record transformation pipeline
let translatePoint point dx dy =
    { point with x = point.x + dx; y = point.y + dy }

let scalePoint point factor =
    { x = point.x * factor; y = point.y * factor }

let origin = { x = 0; y = 0 }
let translated = translatePoint origin 5 10
let scaled = scalePoint translated 2

// Example 3: Record-based state management
let initialState = {
    count = 0;
    isRunning = false;
    message = "Ready"
}

let incrementState state =
    { state with count = state.count + 1 }

let startState state =
    { state with isRunning = true; message = "Running" }

let state1 = startState initialState
let state2 = incrementState state1

// Example 4: Record lists
let points = [
    { x = 0; y = 0 };
    { x = 1; y = 1 };
    { x = 2; y = 4 }
]

// Example 5: Record with functions (if supported)
// This demonstrates conceptual usage
let calculator = {
    add = fun a b -> a + b;
    subtract = fun a b -> a - b;
    multiply = fun a b -> a * b
}

// Example 6: Configuration records
let config = {
    debug = true;
    maxRetries = 3;
    timeout = 5000;
    apiUrl = "https://api.example.com"
}

// Example 7: Deeply nested records
let company = {
    name = "TechCorp";
    department = {
        name = "Engineering";
        team = {
            name = "Backend";
            lead = {
                name = "Alice";
                age = 35;
                skills = ["Rust"; "F#"; "Go"]
            }
        }
    }
}

let leadName = company.department.team.lead.name

// Example 8: Record builder pattern
let buildPerson name age =
    { name = name; age = age; active = true }

let buildEmployee name age department =
    let person = buildPerson name age
    { person with department = department }

// Example 9: Record validation
let isValidPoint point =
    point.x >= 0 && point.y >= 0

let testPoint = { x = 5; y = 10 }
let valid = isValidPoint testPoint

// Example 10: Record merge/combine
let defaultSettings = {
    theme = "light";
    fontSize = 12;
    autoSave = true
}

let userSettings = {
    theme = "dark";
    fontSize = 14
}

let mergedSettings = {
    defaultSettings with
    theme = userSettings.theme;
    fontSize = userSettings.fontSize
}
