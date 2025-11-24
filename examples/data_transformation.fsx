// Data Transformation Examples
// Real-world data processing scenarios using Fusabi

// Scenario 1: CSV-like data processing
// Parse and transform comma-separated values

let csvData = "Alice,30,Engineer|Bob,25,Designer|Carol,35,Manager"

// Split into records
let records = String.split "|" csvData

// Process first record
let firstRecord = List.head records
let fields = String.split "," firstRecord
let name = List.head fields

print name  // => "Alice"

// Scenario 2: Building structured data from strings
let parseRecord record =
    let parts = String.split "," record
    let name = List.head parts
    let rest = List.tail parts
    let age = List.head rest
    let remaining = List.tail rest
    let role = List.head remaining
    { name = name; age = age; role = role }

let person = parseRecord "Alice,30,Engineer"
print person.name  // => "Alice"
print person.role  // => "Engineer"

// Scenario 3: Log parsing and filtering
let logEntries = [
    "INFO: Application started";
    "ERROR: Connection failed";
    "INFO: Processing request";
    "ERROR: Invalid input";
    "INFO: Request completed"
]

// Extract error messages
let isError entry = String.startsWith "ERROR:" entry

// Count log entries
let totalLogs = List.length logEntries
print totalLogs  // => 5

// Scenario 4: URL path parsing
let url = "/api/users/123/posts/456"
let segments = String.split "/" url
let cleaned = List.tail segments  // Remove empty first element

let apiVersion = List.head cleaned
let resource = List.head (List.tail cleaned)

print apiVersion  // => "api"
print resource    // => "users"

// Scenario 5: Configuration file parsing
let configLines = [
    "host=localhost";
    "port=8080";
    "debug=true";
    "workers=4"
]

// Parse a single config line
let parseConfigLine line =
    let parts = String.split "=" line
    let key = List.head parts
    let value = List.head (List.tail parts)
    { key = key; value = value }

let firstConfig = parseConfigLine (List.head configLines)
print firstConfig.key    // => "host"
print firstConfig.value  // => "localhost"

// Scenario 6: JSON-like data transformation
// Building nested structures
let user = {
    id = 123;
    profile = {
        name = "Alice";
        email = "alice@example.com"
    };
    settings = {
        theme = "dark";
        notifications = true
    }
}

let userEmail = user.profile.email
let userTheme = user.settings.theme

print userEmail  // => "alice@example.com"
print userTheme  // => "dark"

// Scenario 7: Data aggregation
let prices = [10; 25; 30; 15; 20]

// Calculate total using fold-like pattern
let rec sumList list =
    match list with
    | [] -> 0
    | x :: xs -> x + (sumList xs)

let total = sumList prices
print total  // => 100

// Scenario 8: Building report data
let stats = {
    totalUsers = 1250;
    activeUsers = 980;
    averageAge = 32;
    topCountry = "USA"
}

// Format a simple report
let buildReport stats =
    let report = {
        summary = "User Statistics Report";
        total = stats.totalUsers;
        active = stats.activeUsers;
        country = stats.topCountry
    }
    report

let report = buildReport stats
print report.summary  // => "User Statistics Report"
print report.total    // => 1250

// Real-world use cases:
// - ETL (Extract, Transform, Load) pipelines
// - Log analysis and monitoring
// - API response transformation
// - Configuration management
// - Data validation and cleaning
// - Report generation

"Data transformation examples complete"
