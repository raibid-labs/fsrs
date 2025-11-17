#!/usr/bin/env nu

# FSRS Build Script
# Orchestrates building the workspace with proper error handling

def main [
    --mode: string = "dev"  # Build mode: dev, release
    --crate: string = ""    # Specific crate to build (empty = all)
    --verbose: bool = false # Verbose output
] {
    print $"🚀 Building FSRS ($mode mode)"

    # Validate mode
    if $mode not-in ["dev", "release"] {
        print "❌ Invalid mode. Use 'dev' or 'release'"
        exit 1
    }

    # Check if cargo is available
    if (which cargo | is-empty) {
        print "❌ cargo not found. Please install Rust: https://rustup.rs"
        exit 1
    }

    # Navigate to rust workspace
    cd rust

    # Build command arguments
    let build_args = if $mode == "release" {
        ["build", "--release", "--workspace"]
    } else {
        ["build", "--workspace"]
    }

    # Add crate-specific flag if provided
    let final_args = if ($crate | is-empty) {
        $build_args
    } else {
        $build_args | append ["-p", $crate]
    }

    # Add verbose flag if requested
    let final_args = if $verbose {
        $final_args | append ["--verbose"]
    } else {
        $final_args
    }

    print $"📦 Running: cargo (echo $final_args | str join ' ')"

    # Execute build
    let result = do {
        cargo ...$final_args
    } | complete

    if $result.exit_code == 0 {
        print "✅ Build successful!"

        # Show build artifacts
        if $mode == "release" {
            print "\n📦 Release binaries:"
            ls target/release/fsrs-* | where type == "file" | select name size
        } else {
            print "\n📦 Debug binaries:"
            ls target/debug/fsrs-* | where type == "file" | select name size | first 5
        }
    } else {
        print "❌ Build failed!"
        print $result.stderr
        exit 1
    }
}
