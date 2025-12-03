#!/usr/bin/env nu
#
# gen-docs.nu - Generate Fusabi Standard Library Reference Documentation
#
# This script parses doc comments from Rust stdlib source files and generates
# a comprehensive markdown reference document.
#
# Usage: nu scripts/gen-docs.nu (from repository root)
#        or: ./scripts/gen-docs.nu (from repository root)
#

# Get repository root directory (assume script is in scripts/ subdirectory)
let script_dir = $env.CURRENT_FILE | path dirname
let repo_root = $script_dir | path dirname
let stdlib_dir = $"($repo_root)/rust/crates/fusabi-vm/src/stdlib"
let output_file = $"($repo_root)/docs/STDLIB_REFERENCE.md"

# Module configuration
let modules = [
    {name: "Array", file: "array.rs", description: "Mutable array operations"},
    {name: "List", file: "list.rs", description: "Immutable cons list operations"},
    {name: "Map", file: "map.rs", description: "Persistent key-value dictionaries"},
    {name: "Option", file: "option.rs", description: "Optional value handling (Some/None)"},
    {name: "String", file: "string.rs", description: "String manipulation functions"},
    {name: "Json", file: "json.rs", description: "JSON parsing and serialization"},
    {name: "Result", file: "result.rs", description: "Result type for error handling (Ok/Error)"},
    {name: "Math", file: "math.rs", description: "Mathematical functions (trig, logs, rounding, constants)"},
    {name: "Process", file: "process.rs", description: "Process and command execution, environment variables"},
    {name: "Time", file: "time.rs", description: "Date/time operations (now, formatting, parsing)"},
    {name: "Url", file: "url.rs", description: "URL parsing, encoding/decoding"},
    {name: "Config", file: "config.rs", description: "Configuration key-value store"},
    {name: "Events", file: "events.rs", description: "Event emitter pattern"},
    {name: "TerminalInfo", file: "terminal_info.rs", description: "Terminal information queries"},
    {name: "TerminalControl", file: "terminal_control.rs", description: "Terminal control operations"},
    {name: "Commands", file: "commands.rs", description: "Command pattern registry"},
    {name: "UIFormatting", file: "ui_formatting.rs", description: "UI/text formatting utilities"}
]

print "Generating Fusabi Standard Library Reference..."
print $"Source directory: ($stdlib_dir)"
print $"Output file: ($output_file)"

# Function to extract doc comments from a single file
def extract_functions [module: string, file_path: string] {
    if not ($file_path | path exists) {
        print $"Warning: File not found: ($file_path)"
        return []
    }

    let content = open $file_path | lines | enumerate

    mut functions = []
    mut current_name = ""
    mut current_sig = ""
    mut current_desc = ""
    mut in_doc = false

    for line_item in $content {
        let line = $line_item.item

        # Check if this is a signature line (Module.function : signature)
        if ($line | str starts-with $"/// ($module).") {
            # Save previous function if exists
            if $in_doc and ($current_name | is-not-empty) {
                $functions = ($functions | append {
                    name: $current_name,
                    signature: $current_sig,
                    description: $current_desc
                })
            }

            # Parse new signature
            let clean_line = $line | str substring 4..
            let parts = $clean_line | split row " : "

            if ($parts | length) >= 2 {
                $current_name = $parts.0
                $current_sig = $parts | skip 1 | str join " : "
                $current_desc = ""
                $in_doc = true
            }
        } else if $in_doc and ($line | str starts-with "/// ") {
            # This is a description line
            let desc_line = $line | str substring 4..
            if ($current_desc | is-empty) {
                $current_desc = $desc_line
            } else {
                $current_desc = $"($current_desc) ($desc_line)"
            }
        } else if $in_doc and not ($line | str starts-with "///") {
            # End of doc comment block
            if ($current_name | is-not-empty) {
                $functions = ($functions | append {
                    name: $current_name,
                    signature: $current_sig,
                    description: $current_desc
                })
                $current_name = ""
                $current_sig = ""
                $current_desc = ""
            }
            $in_doc = false
        }
    }

    # Don't forget the last function
    if $in_doc and ($current_name | is-not-empty) {
        $functions = ($functions | append {
            name: $current_name,
            signature: $current_sig,
            description: $current_desc
        })
    }

    $functions | sort-by name
}

# Extract functions from all modules
print "Extracting function documentation..."
let all_functions = $modules | each {|mod|
    let file_path = $"($stdlib_dir)/($mod.file)"
    print $"  Processing ($mod.name)..."
    {
        module: $mod.name,
        description: $mod.description,
        functions: (extract_functions $mod.name $file_path)
    }
}

# Generate the markdown documentation
print "Generating markdown documentation..."

# Build header
mut doc_content = "# Fusabi Standard Library Reference\n\n"
$doc_content = $doc_content + "This document provides a comprehensive reference for the Fusabi standard library functions.\n\n"
$doc_content = $doc_content + "The standard library is organized into the following modules:\n\n"

for mod in $modules {
    $doc_content = $doc_content + $"- **($mod.name)**: ($mod.description)\n"
}

$doc_content = $doc_content + "\n## Table of Contents\n\n"

for mod in $modules {
    let anchor = $mod.name | str downcase
    $doc_content = $doc_content + $"- [($mod.name) Module]\(#($anchor)-module\)\n"
}

$doc_content = $doc_content + "\n---\n\n"

# Generate module sections
for mod_data in $all_functions {
    let mod_name = $mod_data.module
    $doc_content = $doc_content + $"## ($mod_name) Module\n\n"

    # Add module-specific description
    let long_desc = match $mod_name {
        "Array" => "Arrays are mutable, fixed-size collections indexed by integers. Array operations provide efficient random access and in-place mutation.",
        "List" => "Lists are immutable cons-based linked lists. List operations are functional and never mutate the original list.",
        "Map" => "Maps are persistent key-value dictionaries with string keys. Map operations return new maps rather than mutating existing ones.",
        "Option" => "The Option type represents optional values. Functions in this module help work with `Some` and `None` variants.",
        "String" => "String operations for text manipulation, searching, and formatting.",
        "Json" => "JSON parsing and serialization functions. Available when the `json` feature is enabled.",
        "Result" => "The Result type represents computations that may fail. Functions in this module help work with `Ok` and `Error` variants.",
        "Math" => "Mathematical operations including trigonometric functions, logarithms, rounding, and mathematical constants.",
        "Process" => "Process and system operations including command execution, environment variable access, and process management.",
        "Time" => "Time and date operations for working with timestamps, formatting, and parsing date/time values.",
        "Url" => "URL manipulation functions for parsing, encoding, and decoding URLs and query parameters.",
        "Config" => "Configuration management providing a persistent key-value store for application settings.",
        "Events" => "Event system for implementing the observer pattern with event emitters and listeners.",
        "TerminalInfo" => "Terminal information queries for detecting terminal capabilities and properties.",
        "TerminalControl" => "Terminal control operations for cursor movement, screen clearing, and terminal state management.",
        "Commands" => "Command pattern implementation with a registry for managing and executing named commands.",
        "UIFormatting" => "UI and text formatting utilities for styling console output with colors, styles, and formatting.",
        _ => $mod_data.description
    }

    $doc_content = $doc_content + $"($long_desc)\n\n"

    if ($mod_data.functions | length) == 0 {
        $doc_content = $doc_content + "*No documented functions found for this module.*\n\n"
    } else {
        # Generate function entries
        for func in $mod_data.functions {
            $doc_content = $doc_content + $"### `($func.name)`\n\n"
            $doc_content = $doc_content + $"**Type signature:** `($func.signature)`\n\n"

            if not ($func.description | is-empty) {
                $doc_content = $doc_content + $"($func.description)\n\n"
            }

            $doc_content = $doc_content + "---\n\n"
        }
    }
}

# Add footer
$doc_content = $doc_content + "\n## Notes\n\n"
$doc_content = $doc_content + "- Type variables like `'a`, `'b`, etc. represent generic types\n"
$doc_content = $doc_content + "- Function signatures use OCaml/F#-style syntax with `->` for function types\n"
$doc_content = $doc_content + "- Higher-order functions that take functions as arguments are marked with parentheses, e.g., `('a -> 'b)`\n"
$doc_content = $doc_content + "- The `unit` type represents no value (similar to `void` in other languages)\n\n"
$doc_content = $doc_content + "## Contributing\n\n"
$doc_content = $doc_content + "This documentation is auto-generated from doc comments in the Rust source code.\n"
$doc_content = $doc_content + "To update this documentation, modify the `///` comments in the stdlib source files\n"
$doc_content = $doc_content + "located in `rust/crates/fusabi-vm/src/stdlib/` and run:\n\n"
$doc_content = $doc_content + "```bash\n"
$doc_content = $doc_content + "nu scripts/gen-docs.nu\n"
$doc_content = $doc_content + "```\n\n"
$doc_content = $doc_content + "---\n\n"
$doc_content = $doc_content + "*Generated by `scripts/gen-docs.nu`*\n"

# Write the complete documentation
$doc_content | save -f $output_file

print "Documentation generated successfully!"
print $"Output: ($output_file)"
print ""
print "Summary:"

for mod in $all_functions {
    let count = $mod.functions | length
    let padded_name = $mod.module | fill -a right -w 18
    print $"  ($padded_name): ($count) functions"
}
