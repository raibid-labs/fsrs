// REPL Example - Read-Eval-Print Loop
//
// This example demonstrates a simple REPL (Read-Eval-Print Loop) for Fusabi.
// It showcases:
// - Interactive command-line interface using Console module
// - Pattern matching for command processing
// - Recursive loop implementation
// - String manipulation for input parsing
//
// Note: Full Script.eval integration requires host-level support from the frontend crate.
// This example shows the REPL structure and uses Script.evalToString which currently
// returns an error message, but demonstrates how a complete REPL would work.

// ============================================================================
// Configuration
// ============================================================================

let version = "0.1.0"
let prompt = "> "

// ============================================================================
// Display Functions
// ============================================================================

let showBanner () =
    Console.writeLine "========================================="
    Console.writeLine "  Fusabi REPL v0.1.0"
    Console.writeLine "  Interactive F#-like Scripting Shell"
    Console.writeLine "========================================="
    Console.writeLine ""
    Console.writeLine "Type :help or :h for available commands"
    Console.writeLine ""

let showHelp () =
    Console.writeLine ""
    Console.writeLine "Available Commands:"
    Console.writeLine "  :help, :h      - Show this help message"
    Console.writeLine "  :clear, :c     - Clear the screen"
    Console.writeLine "  :version, :v   - Show version information"
    Console.writeLine "  :quit, :q      - Exit the REPL"
    Console.writeLine "  :exit          - Exit the REPL"
    Console.writeLine ""
    Console.writeLine "Usage:"
    Console.writeLine "  Enter any Fusabi expression to evaluate it"
    Console.writeLine "  Example: let x = 42"
    Console.writeLine "  Example: Console.writeLine \"Hello, World!\""
    Console.writeLine ""

let showVersion () =
    Console.writeLine ""
    Console.writeLine "Fusabi REPL Version: 0.1.0"
    Console.writeLine "F#-like Scripting Language"
    Console.writeLine ""

let showGoodbye () =
    Console.writeLine ""
    Console.writeLine "Thank you for using Fusabi REPL!"
    Console.writeLine "Goodbye!"

// ============================================================================
// Command Processing
// ============================================================================

// Normalize command input by trimming and converting to lowercase
let normalizeCommand input =
    let trimmed = String.trim input
    String.toLower trimmed

// Check if input is a REPL command (starts with :)
let isCommand input =
    let trimmed = String.trim input
    String.startsWith ":" trimmed

// Evaluate user code using Script.evalToString
// Note: Currently returns error message as Script.eval requires frontend integration
let evalInput code =
    Console.writeLine ""
    let result = Script.evalToString code
    Console.writeLine result
    Console.writeLine ""

// Process a REPL command and return whether to continue the loop
let processCommand cmd =
    let normalized = normalizeCommand cmd
    match normalized with
    | ":help" ->
        showHelp ()
        true
    | ":h" ->
        showHelp ()
        true
    | ":clear" ->
        Console.clear ()
        showBanner ()
        true
    | ":c" ->
        Console.clear ()
        showBanner ()
        true
    | ":version" ->
        showVersion ()
        true
    | ":v" ->
        showVersion ()
        true
    | ":quit" ->
        false
    | ":q" ->
        false
    | ":exit" ->
        false
    | _ ->
        Console.writeLine ""
        Console.writeLine "Unknown command. Type :help for available commands."
        Console.writeLine ""
        true

// ============================================================================
// REPL Loop
// ============================================================================

// Main REPL loop - recursively reads input and processes commands
let rec replLoop () =
    // Display prompt (no newline)
    Console.write prompt

    // Read user input
    let input = Console.readLine ()
    let trimmed = String.trim input

    // Skip empty lines
    if String.length trimmed == 0 then
        replLoop ()
    else
        // Check if it's a command or code to evaluate
        let shouldContinue =
            if isCommand input then
                processCommand input
            else
                evalInput input
                true

        // Continue loop if requested
        if shouldContinue then
            replLoop ()
        else
            showGoodbye ()

// ============================================================================
// Entry Point
// ============================================================================

let main () =
    showBanner ()
    replLoop ()

// Start the REPL
main ()
