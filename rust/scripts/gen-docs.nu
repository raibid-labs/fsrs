#!/usr/bin/env nu
# Fusabi Standard Library Documentation Generator
# Parses /// doc comments from stdlib/*.rs and generates STDLIB_REFERENCE.md

# Configuration
let stdlib_path = "crates/fusabi-vm/src/stdlib"
let output_path = "docs/STDLIB_REFERENCE.md"

# Module display order and metadata
let modules = [
    { name: "List", file: "list.rs", description: "Functions for working with cons-based linked lists" }
    { name: "Array", file: "array.rs", description: "Functions for mutable fixed-size arrays" }
    { name: "Map", file: "map.rs", description: "Functions for string-keyed hash maps" }
    { name: "Option", file: "option.rs", description: "Functions for optional values (Some/None)" }
    { name: "Result", file: "result.rs", description: "Functions for error handling (Ok/Error)" }
    { name: "String", file: "string.rs", description: "Functions for string manipulation" }
    { name: "Math", file: "math.rs", description: "Mathematical constants and functions" }
    { name: "Json", file: "json.rs", description: "JSON parsing and serialization" }
    { name: "Print", file: "print.rs", description: "Output functions" }
]

# Parse doc comments from a Rust file
def parse_doc_comments [file_path: string]: nothing -> list<any> {
    let content = open $file_path | lines

    mut functions = []
    mut current_docs = []
    mut in_doc_block = false

    for line in $content {
        if ($line | str starts-with "///") {
            # Extract doc comment content (remove /// prefix)
            let doc_line = $line | str replace "^///\\s?" "" --regex
            $current_docs = ($current_docs | append $doc_line)
            $in_doc_block = true
        } else if $in_doc_block and ($line | str starts-with "pub fn") {
            # Found a public function after doc comments
            if ($current_docs | length) > 0 {
                # First line is typically the signature
                let signature = $current_docs | first
                # Remaining lines are the description
                let description = $current_docs | skip 1 | str join " " | str trim

                $functions = ($functions | append {
                    signature: $signature
                    description: $description
                })
            }
            $current_docs = []
            $in_doc_block = false
        } else if not ($line | str starts-with "///") {
            # Reset if we hit a non-doc, non-function line
            if not ($line | str starts-with "pub fn") {
                $current_docs = []
                $in_doc_block = false
            }
        }
    }

    $functions
}

# Generate markdown for a module
def generate_module_markdown [module: record, functions: list]: nothing -> string {
    mut md = $"## ($module.name)\n\n"
    $md = $md + $"($module.description)\n\n"

    if ($functions | length) == 0 {
        $md = $md + "*No documented functions*\n\n"
        return $md
    }

    $md = $md + "| Function | Description |\n"
    $md = $md + "|----------|-------------|\n"

    for func in $functions {
        let sig = $func.signature | str replace "|" "\\|" --all
        let desc = $func.description | str replace "|" "\\|" --all
        $md = $md + $"| `($sig)` | ($desc) |\n"
    }

    $md = $md + "\n"
    $md
}

# Main script
def main [] {
    print "Generating Fusabi Standard Library Reference..."

    # Header
    mut output = "# Fusabi Standard Library Reference\n\n"
    $output = $output + $"*Auto-generated from source code on (date now | format date '%Y-%m-%d')*\n\n"
    $output = $output + "This document provides a comprehensive reference for all functions in the Fusabi standard library.\n\n"
    $output = $output + "## Table of Contents\n\n"

    # Generate TOC
    for module in $modules {
        $output = $output + $"- [($module.name)](#($module.name | str downcase))\n"
    }
    $output = $output + "\n---\n\n"

    # Process each module
    for module in $modules {
        let file_path = $"($stdlib_path)/($module.file)"

        if ($file_path | path exists) {
            print $"  Processing ($module.name) module..."
            let functions = parse_doc_comments $file_path
            let module_md = generate_module_markdown $module $functions
            $output = $output + $module_md
        } else {
            print $"  Warning: ($file_path) not found, skipping..."
        }
    }

    # Add usage examples section
    $output = $output + "## Usage Examples\n\n"
    $output = $output + "### List Operations\n"
    $output = $output + "```fsharp\n"
    $output = $output + "let nums = [1; 2; 3; 4; 5]\n"
    $output = $output + "let doubled = List.map (fun x -> x * 2) nums\n"
    $output = $output + "let sum = List.fold (fun acc x -> acc + x) 0 nums\n"
    $output = $output + "```\n\n"

    $output = $output + "### Option Handling\n"
    $output = $output + "```fsharp\n"
    $output = $output + "let maybeValue = Some 42\n"
    $output = $output + "let value = Option.defaultValue 0 maybeValue  // 42\n"
    $output = $output + "let mapped = Option.map (fun x -> x * 2) maybeValue  // Some 84\n"
    $output = $output + "```\n\n"

    $output = $output + "### Result Error Handling\n"
    $output = $output + "```fsharp\n"
    $output = $output + "let result = Ok 100\n"
    $output = $output + "let value = Result.defaultValue 0 result  // 100\n"
    $output = $output + "let mapped = Result.map (fun x -> x / 2) result  // Ok 50\n"
    $output = $output + "```\n\n"

    $output = $output + "### Math Functions\n"
    $output = $output + "```fsharp\n"
    $output = $output + "let pi = Math.pi ()\n"
    $output = $output + "let sqrt2 = Math.sqrt 2.0\n"
    $output = $output + "let angle = Math.atan2 1.0 1.0  // pi/4\n"
    $output = $output + "```\n\n"

    # Footer
    $output = $output + "---\n\n"
    $output = $output + "*For more examples, see the `examples/` directory in the repository.*\n"

    # Write output
    $output | save -f $output_path
    print $"Documentation generated: ($output_path)"
}

# Entry point - main is called automatically when script runs
