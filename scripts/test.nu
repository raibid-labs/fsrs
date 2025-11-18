#!/usr/bin/env nu

# FSRS Test Runner
# Runs tests with proper filtering and reporting

def main [
    --mode: string = "all"  # Test mode: all, unit, integration, crate
    --crate: string = ""    # Specific crate to test
    --verbose              # Show test output
    --nocapture            # Don't capture stdout
] {
    print $"🧪 Running FSRS tests \(($mode) mode\)"

    # Check if cargo is available
    if (which cargo | is-empty) {
        print "❌ cargo not found. Please install Rust: https://rustup.rs"
        exit 1
    }

    # Navigate to rust workspace
    cd rust

    # Build test command based on mode
    let test_args = match $mode {
        "all" => ["test", "--workspace"],
        "unit" => ["test", "--workspace", "--lib"],
        "integration" => ["test", "--workspace", "--test", "*"],
        "crate" => {
            if ($crate | is-empty) {
                print "❌ --crate flag required when using --mode crate"
                exit 1
            }
            ["test", "-p", $crate]
        },
        _ => {
            print $"❌ Invalid mode: \(($mode)\)"
            print "Valid modes: all, unit, integration, crate"
            exit 1
        }
    }

    # Add verbose flags if requested
    let final_args = if $nocapture {
        $test_args | append ["--", "--nocapture"]
    } else if $verbose {
        $test_args | append ["--", "--show-output"]
    } else {
        $test_args
    }

    print $"🔬 Running: cargo (echo $final_args | str join ' ')"

    # Execute tests
    let result = do {
        cargo ...$final_args
    } | complete

    if $result.exit_code == 0 {
        print "\n✅ All tests passed!"

        # Parse and display test summary
        let output = $result.stdout
        if ($output | str contains "test result:") {
            let summary = $output
                | lines
                | find "test result:"
                | first
            print $"\n📊 \(($summary)\)"
        }
    } else {
        print "\n❌ Tests failed!"
        print $result.stderr

        # Show which tests failed
        let failed = $result.stdout
            | lines
            | find "FAILED"

        if not ($failed | is-empty) {
            print "\n❌ Failed tests:"
            $failed | each { |line| print $"  \(($line)\)" }
        }

        exit 1
    }
}
