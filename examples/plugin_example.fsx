// Plugin System Example
// Demonstrates how Fusabi can be used as a plugin runtime
//
// This script is designed to be loaded by a Rust host application
// that provides plugin APIs via host functions

// ===== Plugin Metadata =====
let pluginInfo = {
    name = "example-plugin";
    version = 1;
    author = "Fusabi Team";
    description = "Example plugin demonstrating the plugin API"
}

// ===== Command Handlers =====
// These functions would be called by the host application

let handleGreeting name =
    String.concat ["Hello, "; name; " from the plugin!"]

let handleCalculation x y operation =
    match operation with
    | "add" -> x + y
    | "subtract" -> x - y
    | "multiply" -> x * y
    | "divide" ->
        match y with
        | 0 -> 0  // Safe division
        | _ -> x / y
    | _ -> 0

// ===== Data Processing =====
// Process structured data from the host

let processUserData user =
    let name = user.name
    let age = user.age
    let isAdult = age >= 18

    {
        username = name;
        age = age;
        canVote = isAdult;
        status = if isAdult then "adult" else "minor"
    }

// ===== Event Handlers =====
// Respond to events from the host application

let onStartup config =
    let message = String.concat ["Plugin started with config: "; config.environment]
    { success = true; message = message }

let onShutdown =
    { success = true; message = "Plugin shutdown complete" }

let onRequest request =
    match request.method with
    | "GET" -> { status = 200; body = "GET response from plugin" }
    | "POST" -> { status = 201; body = "POST response from plugin" }
    | _ -> { status = 405; body = "Method not allowed" }

// ===== Configuration Processing =====
// Validate and process plugin configuration

let validatePluginConfig cfg =
    let hasName = String.length cfg.name > 0
    let hasValidPort = cfg.port > 0 && cfg.port < 65536
    let hasValidTimeout = cfg.timeout > 0

    hasName && hasValidPort && hasValidTimeout

let processConfig cfg =
    let isValid = validatePluginConfig cfg
    match isValid with
    | true -> Some({
        name = cfg.name;
        port = cfg.port;
        timeout = cfg.timeout;
        enabled = true
    })
    | false -> None

// ===== Utility Functions =====
// Helper functions for the plugin

let formatLog level message =
    String.concat ["["; level; "] "; message]

let parseCommand input =
    let parts = String.split " " input
    let command = List.head parts
    let args = List.tail parts
    { command = command; args = args }

// ===== State Management =====
// Maintain plugin state across calls

let createCounter =
    { count = 0; lastUpdated = "never" }

let incrementCounter counter =
    { count = counter.count + 1; lastUpdated = "now" }

// ===== Testing the Plugin =====
// These would normally be called by the host

print pluginInfo.name
print pluginInfo.version

let greeting = handleGreeting "Alice"
print greeting  // => "Hello, Alice from the plugin!"

let sum = handleCalculation 10 5 "add"
print sum  // => 15

let product = handleCalculation 10 5 "multiply"
print product  // => 50

let user = { name = "Bob"; age = 25 }
let processed = processUserData user
print processed.username  // => "Bob"
print processed.status    // => "adult"

let cfg = { environment = "production" }
let startResult = onStartup cfg
print startResult.success  // => true

let req = { method = "GET"; path = "/api/data" }
let response = onRequest req
print response.status  // => 200

// ===== How This Works with Rust =====
//
// The Rust host application would:
// 1. Load this .fsx or .fzb file
// 2. Register host functions (e.g., log, database, HTTP)
// 3. Call plugin functions like handleGreeting, onStartup, etc.
// 4. Pass data between Rust and Fusabi using Value types
//
// Example Rust code:
//
// ```rust
// let mut engine = FusabiEngine::new();
//
// // Register host functions
// engine.register_fn1("log", |msg| {
//     println!("[LOG] {}", msg);
//     Ok(Value::Unit)
// });
//
// // Load the plugin
// engine.load_plugin("plugin.fsx")?;
//
// // Call plugin function
// let greeting = engine.call("handleGreeting", &[
//     Value::Str("Alice".to_string())
// ])?;
// ```

// ===== Use Cases for Plugin Systems =====
//
// 1. Application extensibility
//    - Allow users to add custom features
//    - Hot-reload plugins without recompiling
//
// 2. Game scripting
//    - AI behavior scripts
//    - Quest logic
//    - Item effects
//
// 3. Data processing pipelines
//    - Custom transformations
//    - Validation rules
//    - Format converters
//
// 4. Configuration management
//    - Dynamic routing rules
//    - Feature flags
//    - Policy enforcement
//
// 5. Testing frameworks
//    - Test case definitions
//    - Mock data generators
//    - Assertion libraries

"Plugin example complete"
