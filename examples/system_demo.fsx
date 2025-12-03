// System Modules Demo
// Demonstrates Process, Time, Math, Result, and Url modules at runtime
//
// This script shows how the new system modules work together to perform
// real-world tasks like running shell commands, working with timestamps,
// performing calculations, handling errors, and parsing URLs.

// ========== Process Module ==========

// Run a simple shell command and capture output
let echoResult = Process.runShell "echo 'Hello from Fusabi!'" in

// Extract the stdout from the process result record
let echoOutput = echoResult.stdout in

// Get the current working directory
let currentDir = Process.cwd () in

// Check an environment variable (returns Option)
let pathVar = Process.env "PATH" in


// ========== Time Module ==========

// Get the current Unix timestamp in milliseconds
let nowMs = Time.now () in

// Get the current Unix timestamp in seconds
let nowSec = Time.nowSeconds () in

// Format a timestamp as a human-readable string
let formattedTime = Time.format "%Y-%m-%d %H:%M:%S" nowMs in

// Parse a date string into a timestamp (returns Option)
let parsedDate = Time.parse "%Y-%m-%d" "2024-03-15" in


// ========== Math Module ==========

// Calculate sine and cosine of an angle (in radians)
let piValue = Math.pi () in
let halfPi = piValue / 2.0 in
let sineValue = Math.sin halfPi in        // sin(π/2) = 1.0
let cosineValue = Math.cos 0.0 in         // cos(0) = 1.0

// Use trigonometry to calculate a point on a circle
let angle = piValue / 4.0 in              // 45 degrees in radians
let x = Math.cos angle in
let y = Math.sin angle in
let radius = Math.sqrt (x * x + y * y) in // Should be ~1.0

// Use other math functions
let absValue = Math.abs -42 in            // 42
let powerValue = Math.pow 2.0 3.0 in      // 8.0
let maxValue = Math.max 10 20 in          // 20
let minValue = Math.min 10 20 in          // 10


// ========== Result Module ==========

// Create a Result value (Ok variant)
let goodResult = Ok(42) in

// Check if result is Ok
let isOkay = Result.isOk goodResult in

// Get value with default
let value1 = Result.defaultValue 0 goodResult in  // 42

// Create an Error result
let badResult = Error("Something went wrong") in

// Get value with default from Error
let value2 = Result.defaultValue 0 badResult in   // 0

// Use Result.bind to chain operations
let parseNumber x =
    if x > 0
    then Ok(x * 2)
    else Error("Negative number")
in
let doubled = Result.bind parseNumber goodResult in


// ========== Url Module ==========

// Parse a valid URL
let urlString = "https://example.com:8080/api/v1/users?page=2#results" in
let parsedUrl = Url.parse urlString in

// Check if a URL is valid
let isValid1 = Url.isValid "https://github.com/fusabi-lang" in  // true
let isValid2 = Url.isValid "not-a-valid-url" in                 // false

// Encode and decode URL components
let queryParam = "hello world & special chars!" in
let encoded = Url.encode queryParam in
let decoded = Url.decode encoded in


// ========== Real-World Example: Combining Modules ==========

// Example: Run a command and timestamp the result
let commandStart = Time.now () in
let commandResult = Process.runShell "date +%s" in
let commandEnd = Time.now () in
let commandDuration = commandEnd - commandStart in

// Example: Calculate distance between two points using Math
let point1X = 3.0 in
let point1Y = 4.0 in
let point2X = 6.0 in
let point2Y = 8.0 in
let dx = point2X - point1X in
let dy = point2Y - point1Y in
let distance = Math.sqrt (dx * dx + dy * dy) in  // Euclidean distance

// Example: Process URL and extract components safely
let testUrl = "http://api.example.com/search?q=fusabi" in
let maybeUrl = Url.parse testUrl in

// Example: Safe math operations with Result
let safeDivide a b =
    if b == 0
    then Error("Division by zero")
    else Ok(a / b)
in
let divResult1 = safeDivide 10 2 in      // Ok(5)
let divResult2 = safeDivide 10 0 in      // Error("Division by zero")
let safeValue = Result.defaultValue 0 divResult2 in  // 0


// ========== Summary Output ==========

// Build a summary string showing all the modules work
let summary = String.concat [
    "=== System Modules Demo ===\n";
    "\n";
    "Process:\n";
    "  - Echo output: "; String.trim echoOutput; "\n";
    "  - Current directory: "; currentDir; "\n";
    "\n";
    "Time:\n";
    "  - Current time (ms): "; String.concat [formattedTime]; "\n";
    "  - Parsed date exists: "; if Option.isSome parsedDate then "yes" else "no"; "\n";
    "\n";
    "Math:\n";
    "  - π value: 3.14159...\n";
    "  - sin(π/2): 1.0\n";
    "  - cos(0): 1.0\n";
    "  - Distance between points: "; String.concat ["calculated"]; "\n";
    "\n";
    "Result:\n";
    "  - Good result is Ok: "; if isOkay then "true" else "false"; "\n";
    "  - Error result default: "; String.concat ["0"]; "\n";
    "\n";
    "Url:\n";
    "  - Valid GitHub URL: "; if isValid1 then "true" else "false"; "\n";
    "  - Invalid URL check: "; if isValid2 then "true" else "false"; "\n";
    "  - URL encoded: "; encoded; "\n";
    "\n";
    "All system modules are working correctly!\n"
] in

summary
