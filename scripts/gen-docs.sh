#!/usr/bin/env bash
#
# gen-docs.sh - Generate Fusabi Standard Library Reference Documentation
#
# This script parses doc comments from Rust stdlib source files and generates
# a comprehensive markdown reference document.
#
# Usage: ./scripts/gen-docs.sh
#

set -euo pipefail

# Repository root directory
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STDLIB_DIR="${REPO_ROOT}/rust/crates/fusabi-vm/src/stdlib"
OUTPUT_FILE="${REPO_ROOT}/docs/STDLIB_REFERENCE.md"

# Temporary file for collecting functions
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Module names
MODULES=("Array" "List" "Map" "Option" "String" "JSON")

echo "Generating Fusabi Standard Library Reference..."
echo "Source directory: ${STDLIB_DIR}"
echo "Output file: ${OUTPUT_FILE}"

# Function to extract doc comments and function info from a file
# Args: $1 = module name, $2 = file path
extract_functions() {
    local module="$1"
    local file="$2"
    local output="${TMP_DIR}/${module}.txt"

    if [ ! -f "$file" ]; then
        echo "Warning: File not found: $file" >&2
        return
    fi

    # Use awk to parse the file
    awk -v module="$module" '
    BEGIN {
        in_doc_comment = 0
        signature = ""
        description = ""
        func_name = ""
    }

    # Match doc comment lines with Module.function : signature
    /^\/\/\/ [A-Z][a-z]+\.[a-z]/ {
        # Extract the full signature line
        line = $0
        sub(/^\/\/\/ /, "", line)

        # Split on " : " to get function name and type signature
        if (match(line, /^([A-Za-z]+\.[a-z][A-Za-z0-9]*) : (.+)$/, arr)) {
            # Only process if it matches our module
            if (index(arr[1], module ".") == 1) {
                if (func_name != "") {
                    # Output previous function
                    print func_name "|" signature "|" description
                }
                func_name = arr[1]
                signature = arr[2]
                description = ""
                in_doc_comment = 1
            }
        }
        next
    }

    # Match other doc comment lines (descriptions)
    /^\/\/\/ / && in_doc_comment {
        line = $0
        sub(/^\/\/\/ /, "", line)
        if (description == "") {
            description = line
        } else {
            description = description " " line
        }
        next
    }

    # End of doc comments
    /^[^\/]/ && in_doc_comment {
        if (func_name != "") {
            print func_name "|" signature "|" description
            func_name = ""
            signature = ""
            description = ""
        }
        in_doc_comment = 0
    }

    END {
        if (func_name != "") {
            print func_name "|" signature "|" description
        }
    }
    ' "$file" | sort > "$output"
}

# Extract functions from each module
echo "Extracting function documentation..."

extract_functions "Array" "${STDLIB_DIR}/array.rs"
extract_functions "List" "${STDLIB_DIR}/list.rs"
extract_functions "Map" "${STDLIB_DIR}/map.rs"
extract_functions "Option" "${STDLIB_DIR}/option.rs"
extract_functions "String" "${STDLIB_DIR}/string.rs"
extract_functions "JSON" "${STDLIB_DIR}/json.rs"

# Generate the markdown file
echo "Generating markdown documentation..."

cat > "$OUTPUT_FILE" <<'HEADER'
# Fusabi Standard Library Reference

This document provides a comprehensive reference for the Fusabi standard library functions.

The standard library is organized into the following modules:

- **Array**: Mutable array operations
- **List**: Immutable cons list operations
- **Map**: Persistent map (dictionary) operations
- **Option**: Option type helpers for working with optional values
- **String**: String manipulation functions
- **JSON**: JSON parsing and serialization (when `json` feature is enabled)

## Table of Contents

- [Array Module](#array-module)
- [List Module](#list-module)
- [Map Module](#map-module)
- [Option Module](#option-module)
- [String Module](#string-module)
- [JSON Module](#json-module)

---

HEADER

# Function to generate a module section
generate_module_section() {
    local module="$1"
    local module_lower=$(echo "$module" | tr '[:upper:]' '[:lower:]')
    local func_file="${TMP_DIR}/${module}.txt"

    echo "## ${module} Module" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"

    # Add module description
    case "$module" in
        "Array")
            echo "Arrays are mutable, fixed-size collections indexed by integers. Array operations provide efficient random access and in-place mutation." >> "$OUTPUT_FILE"
            ;;
        "List")
            echo "Lists are immutable cons-based linked lists. List operations are functional and never mutate the original list." >> "$OUTPUT_FILE"
            ;;
        "Map")
            echo "Maps are persistent key-value dictionaries with string keys. Map operations return new maps rather than mutating existing ones." >> "$OUTPUT_FILE"
            ;;
        "Option")
            echo "The Option type represents optional values. Functions in this module help work with \`Some\` and \`None\` variants." >> "$OUTPUT_FILE"
            ;;
        "String")
            echo "String operations for text manipulation, searching, and formatting." >> "$OUTPUT_FILE"
            ;;
        "JSON")
            echo "JSON parsing and serialization functions. Available when the \`json\` feature is enabled." >> "$OUTPUT_FILE"
            ;;
    esac

    echo "" >> "$OUTPUT_FILE"

    if [ ! -f "$func_file" ] || [ ! -s "$func_file" ]; then
        echo "*No documented functions found for this module.*" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        return
    fi

    # Count functions
    local func_count=$(wc -l < "$func_file")

    if [ "$func_count" -eq 0 ]; then
        echo "*No documented functions found for this module.*" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        return
    fi

    # Generate function entries
    while IFS='|' read -r name signature description; do
        echo "### \`${name}\`" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
        echo "**Type signature:** \`${signature}\`" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"

        if [ -n "$description" ]; then
            echo "${description}" >> "$OUTPUT_FILE"
            echo "" >> "$OUTPUT_FILE"
        fi

        echo "---" >> "$OUTPUT_FILE"
        echo "" >> "$OUTPUT_FILE"
    done < "$func_file"
}

# Generate sections for each module
for module in "${MODULES[@]}"; do
    echo "Processing ${module} module..."
    generate_module_section "$module"
done

# Add footer
cat >> "$OUTPUT_FILE" <<'FOOTER'

## Notes

- Type variables like `'a`, `'b`, etc. represent generic types
- Function signatures use OCaml/F#-style syntax with `->` for function types
- Higher-order functions that take functions as arguments are marked with parentheses, e.g., `('a -> 'b)`
- The `unit` type represents no value (similar to `void` in other languages)

## Contributing

This documentation is auto-generated from doc comments in the Rust source code.
To update this documentation, modify the `///` comments in the stdlib source files
located in `rust/crates/fusabi-vm/src/stdlib/` and run:

```bash
./scripts/gen-docs.sh
```

---

*Generated by `scripts/gen-docs.sh`*
FOOTER

echo "Documentation generated successfully!"
echo "Output: ${OUTPUT_FILE}"
echo ""
echo "Summary:"
for module in "${MODULES[@]}"; do
    func_file="${TMP_DIR}/${module}.txt"
    if [ -f "$func_file" ]; then
        count=$(wc -l < "$func_file" 2>/dev/null || echo 0)
        printf "  %-10s %2d functions\n" "${module}:" "$count"
    else
        printf "  %-10s %2d functions\n" "${module}:" "0"
    fi
done
