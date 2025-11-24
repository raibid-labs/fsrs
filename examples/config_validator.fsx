// Configuration Validator Example
// Demonstrates practical validation patterns
//
// This example shows how to validate configuration data
// using pattern matching and the Option type

// Define a configuration structure
let config = {
    host = "localhost";
    port = 8080;
    maxConnections = 100;
    timeout = 30;
    enableLogging = true
}

// Validation functions
let validatePort port =
    match port with
    | p when p > 0 && p < 65536 -> Some(port)
    | _ -> None

let validateTimeout timeout =
    match timeout with
    | t when t > 0 && t < 3600 -> Some(timeout)
    | _ -> None

let validateConnections conns =
    match conns with
    | c when c > 0 && c < 10000 -> Some(conns)
    | _ -> None

// Validate port
let portResult = validatePort config.port
let isValidPort = Option.isSome portResult
print isValidPort  // => true

// Validate timeout
let timeoutResult = validateTimeout config.timeout
let validTimeout = Option.defaultValue 60 timeoutResult
print validTimeout  // => 30

// Validate max connections
let connsResult = validateConnections config.maxConnections
let validConns = Option.defaultValue 50 connsResult
print validConns  // => 100

// Build a validated configuration
let validateConfig cfg =
    let vPort = validatePort cfg.port
    let vTimeout = validateTimeout cfg.timeout
    let vConns = validateConnections cfg.maxConnections

    match (vPort, vTimeout, vConns) with
    | (Some(p), Some(t), Some(c)) ->
        Some({
            host = cfg.host;
            port = p;
            maxConnections = c;
            timeout = t;
            enableLogging = cfg.enableLogging
        })
    | _ -> None

let validatedConfig = validateConfig config
let configIsValid = Option.isSome validatedConfig

print configIsValid  // => true

// Example: Invalid configuration
let badConfig = {
    host = "localhost";
    port = 99999;  // Invalid port
    maxConnections = 100;
    timeout = 30;
    enableLogging = true
}

let badResult = validateConfig badConfig
let badConfigIsValid = Option.isSome badResult
print badConfigIsValid  // => false

// Practical use cases:
// 1. API request validation
// 2. Configuration file validation
// 3. User input validation
// 4. Database connection parameters
// 5. Environment variable checking

// Pattern: Always return Option for validation
// - Some(value) = valid
// - None = invalid

"Configuration validation complete"
