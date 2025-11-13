#!/usr/bin/env nu
# Setup script for FSRS development environment

def main [] {
    print "🚀 Setting up FSRS development environment..."

    # Check prerequisites
    check-prerequisites

    # Create directory structure
    create-directories

    # Install Rust dependencies
    install-rust-deps

    # Install F# and Fable
    install-fsharp-deps

    # Setup git hooks
    setup-git-hooks

    print "✅ Setup complete! Run 'just build' to build the project."
}

def check-prerequisites [] {
    print "📋 Checking prerequisites..."

    # Check for Rust
    if (which rustc | is-empty) {
        print "❌ Rust not found. Install from https://rustup.rs"
        exit 1
    }
    print $"✅ Rust (rustc --version | str trim)"

    # Check for Cargo
    if (which cargo | is-empty) {
        print "❌ Cargo not found"
        exit 1
    }

    # Check for .NET SDK
    if (which dotnet | is-empty) {
        print "⚠️  .NET SDK not found. F# scripts require .NET"
        print "   Install from https://dotnet.microsoft.com/download"
    } else {
        print $"✅ .NET (dotnet --version | str trim)"
    }

    # Check for Just
    if (which just | is-empty) {
        print "⚠️  just not found. Installing..."
        cargo install just
    }
    print "✅ Just command runner"

    # Check for Nushell
    print $"✅ Nushell (version | get version)"
}

def create-directories [] {
    print "📁 Creating directory structure..."

    let dirs = [
        "src/host",
        "src/runtime",
        "src/transpiler-extensions",
        "tests/unit",
        "tests/integration",
        "examples",
        "docs",
        "scripts",
        "target",
        ".github/workflows"
    ]

    for dir in $dirs {
        mkdir $dir
        print $"  Created ($dir)"
    }
}

def install-rust-deps [] {
    print "🦀 Installing Rust dependencies..."

    # Install cargo tools
    let tools = [
        "cargo-edit",
        "cargo-audit",
        "cargo-tarpaulin",
        "cargo-flamegraph"
    ]

    for tool in $tools {
        if (which $tool | is-empty) {
            print $"  Installing ($tool)..."
            try {
                cargo install $tool
            } catch {
                print $"  ⚠️  Failed to install ($tool) - continuing..."
            }
        } else {
            print $"  ✅ ($tool) already installed"
        }
    }

    # Install cargo-watch separately with error handling (has known issues on macOS ARM64)
    if (which cargo-watch | is-empty) {
        print "  Installing cargo-watch..."
        try {
            cargo install cargo-watch
            print "  ✅ cargo-watch installed"
        } catch {
            print "  ⚠️  cargo-watch installation failed (known issue on macOS ARM64)"
            print "     Alternative: Use 'watchexec' or run commands manually"
            print "     Install watchexec: brew install watchexec"
        }
    } else {
        print "  ✅ cargo-watch already installed"
    }

    # Install rustfmt and clippy
    rustup component add rustfmt clippy
}

def install-fsharp-deps [] {
    print "📘 Installing F# dependencies..."

    if (which dotnet | is-empty) {
        print "⚠️  Skipping F# setup - .NET not found"
        return
    }

    # Install Fable
    print "  Installing Fable..."
    dotnet tool install -g fable

    # Update Fable to latest
    dotnet tool update -g fable

    print "✅ F# dependencies installed"
}

def setup-git-hooks [] {
    print "🔗 Setting up git hooks..."

    # Pre-commit hook
    let pre_commit = "#!/bin/sh\njust fmt-check && just lint\n"
    $pre_commit | save -f .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

    print "✅ Git hooks configured"
}

# Run main
main
