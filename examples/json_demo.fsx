// JSON Parsing and Serialization Demo
// Demonstrates the Json standard library module
// Part of RFD-001: Unified MCP DSL

open Json

// ===== JSON Parsing =====

printfn "=== JSON Parsing Demo ==="

// Parse simple values
let nullVal = Json.parse "null"
let boolVal = Json.parse "true"
let numVal = Json.parse "42"
let strVal = Json.parse "\"hello\""

printfn "Parsed null: %A" nullVal
printfn "Parsed bool: %A" boolVal
printfn "Parsed number: %A" numVal
printfn "Parsed string: %A" strVal

// Parse array
let arrayJson = "[1, 2, 3, 4, 5]"
let array = Json.parse arrayJson
printfn "\nParsed array: %A" array

// Parse complex object
let personJson = "{\"name\": \"Alice\", \"age\": 30, \"active\": true}"
let person = Json.parse personJson
printfn "\nParsed person object: %A" person

// Parse nested structure
let dataJson = "{
  \"users\": [
    {\"id\": 1, \"name\": \"Alice\"},
    {\"id\": 2, \"name\": \"Bob\"}
  ],
  \"total\": 2
}"
let data = Json.parse dataJson
printfn "\nParsed nested data: %A" data

// ===== JSON Serialization =====

printfn "\n\n=== JSON Serialization Demo ==="

// Stringify simple values
let s1 = Json.stringify ()
let s2 = Json.stringify true
let s3 = Json.stringify 123
let s4 = Json.stringify "test"

printfn "Stringified unit: %s" s1
printfn "Stringified bool: %s" s2
printfn "Stringified int: %s" s3
printfn "Stringified string: %s" s4

// Stringify lists (converted to arrays)
let list = [1; 2; 3; 4; 5]
let listJson = Json.stringify list
printfn "\nStringified list: %s" listJson

// Stringify record (object)
// Note: In actual Fusabi, you'd create a record type
// For this demo, we're showing the concept

// ===== Pretty Printing =====

printfn "\n\n=== Pretty Print Demo ==="

let complexData = Json.parse "{\"name\":\"Project\",\"items\":[{\"id\":1,\"value\":100},{\"id\":2,\"value\":200}]}"
let pretty = Json.stringifyPretty complexData
printfn "Pretty printed JSON:\n%s" pretty

// ===== Real-World Example: API Response =====

printfn "\n\n=== Real-World Example: API Response ==="

let apiResponse = "{
  \"status\": \"success\",
  \"data\": {
    \"tracks\": [
      {\"id\": 1, \"name\": \"Intro\", \"duration\": 45},
      {\"id\": 2, \"name\": \"Verse 1\", \"duration\": 90},
      {\"id\": 3, \"name\": \"Chorus\", \"duration\": 60}
    ],
    \"tempo\": 120,
    \"key\": \"C major\"
  }
}"

let response = Json.parse apiResponse
printfn "Parsed API response: %A" response

// In a real application, you would extract specific fields:
// let status = response.status
// let tracks = response.data.tracks
// etc.

printfn "\n✅ JSON demo complete!"
printfn "This demonstrates Json.parse, Json.stringify, and Json.stringifyPretty"
printfn "for working with JSON data in Fusabi scripts."
