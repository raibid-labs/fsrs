#!/usr/bin/env nu
# Development workflow automation

def main [
    command: string = "watch"  # Command: watch, watch-test, repl
] {
    match $command {
        "watch" => watch-build,
        "watch-test" => watch-tests,
        "repl" => start-repl,
        _ => {
            print $"Unknown command: ($command)"
            print "Available commands: watch, watch-test, repl"
            exit 1
        }
    }
}

def watch-build [] {
    print "👀 Watching for changes..."
    print "Press Ctrl+C to stop\n"

    # Check if cargo-watch is installed
    if (which cargo-watch | is-not-empty) {
        cargo watch -x "build" -s "just fmt" -c
    } else if (which watchexec | is-not-empty) {
        print "⚠️  Using watchexec (cargo-watch not available)"
        watchexec -c -r -e rs,toml -- cargo build
    } else {
        print "❌ Neither cargo-watch nor watchexec found."
        print "   Install one of:"
        print "     • cargo install cargo-watch (may fail on macOS ARM64)"
        print "     • brew install watchexec (recommended for macOS)"
        exit 1
    }
}

def watch-tests [] {
    print "👀 Watching for changes and running tests..."
    print "Press Ctrl+C to stop\n"

    # Check if cargo-watch is installed
    if (which cargo-watch | is-not-empty) {
        cargo watch -x "test --quiet" -c
    } else if (which watchexec | is-not-empty) {
        print "⚠️  Using watchexec (cargo-watch not available)"
        watchexec -c -r -e rs,toml -- cargo test --quiet
    } else {
        print "❌ Neither cargo-watch nor watchexec found."
        print "   Install one of:"
        print "     • cargo install cargo-watch (may fail on macOS ARM64)"
        print "     • brew install watchexec (recommended for macOS)"
        exit 1
    }
}

def start-repl [] {
    print "🎮 Starting FSRS REPL..."
    print "Type F# expressions and see them execute in real-time\n"

    # Check if host binary exists
    let binary = if ("target/release/fsrs-host" | path exists) {
        "target/release/fsrs-host"
    } else if ("target/debug/fsrs-host" | path exists) {
        "target/debug/fsrs-host"
    } else {
        print "❌ Host binary not found. Run 'just build' first."
        exit 1
    }

    # Start REPL
    ^$binary --repl
}

# Entry point
main
