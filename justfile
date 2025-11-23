# Fusabi - F# Script Engine
# Build automation for Mini-F# bytecode VM and runtime

# Default recipe - show available commands
default:
    @just --list

# ============================================================================
# Building
# ============================================================================

# Build all workspace crates in development mode
build:
    cd rust && cargo build --workspace

# Build all crates optimized for production
build-release:
    cd rust && cargo build --workspace --release

# Build specific crate
build-crate CRATE:
    cd rust && cargo build -p {{CRATE}}

# Build frontend compiler
build-frontend:
    cd rust && cargo build -p fusabi-frontend

# Build VM runtime
build-vm:
    cd rust && cargo build -p fusabi-vm

# Build demo host
build-demo:
    cd rust && cargo build -p fusabi

# Clean build artifacts
clean:
    cd rust && cargo clean
    rm -rf target/

# ============================================================================
# Testing
# ============================================================================

# Run all tests
test:
    nu scripts/test.nu --mode all

# Run tests for specific crate
test-crate CRATE:
    cd rust && cargo test -p {{CRATE}}

# Run tests with coverage
test-coverage:
    cd rust && cargo tarpaulin --workspace --out Html --output-dir ../docs/coverage/

# Run integration tests
test-integration:
    nu scripts/test.nu --mode integration

# Run unit tests only
test-unit:
    nu scripts/test.nu --mode unit

# Run VM tests
test-vm:
    cd rust && cargo test -p fusabi-vm

# Run frontend tests
test-frontend:
    cd rust && cargo test -p fusabi-frontend

# ============================================================================
# Development
# ============================================================================

# Run demo host with example script
demo:
    cd rust && cargo run -p fusabi

# Run demo with specific example
demo-example EXAMPLE:
    cd rust && cargo run -p fusabi -- ../examples/{{EXAMPLE}}.fsx

# Run in development mode with watch
watch:
    cd rust && cargo watch -x 'build --workspace'

# Watch and run tests
watch-test:
    cd rust && cargo watch -x 'test --workspace'

# Watch and run demo
watch-demo:
    cd rust && cargo watch -x 'run -p fusabi'

# Start development environment (watch + test)
dev:
    cd rust && cargo watch -x 'test --workspace' -x 'run -p fusabi'

# ============================================================================
# Code Quality
# ============================================================================

# Run all quality checks (fmt, lint, test)
check: fmt-check lint test

# Validate code compilation
check-compile:
    cd rust && cargo check --all-targets --workspace

# Run clippy linter with strict warnings
lint:
    cd rust && cargo clippy --all-targets --workspace -- -D warnings

# Automatically fix linting issues
lint-fix:
    cd rust && cargo clippy --all-targets --workspace --fix --allow-dirty

# Verify code formatting
fmt-check:
    cd rust && cargo fmt --all -- --check

# Apply code formatting
fmt:
    cd rust && cargo fmt --all

# ============================================================================
# Transpilation (Future)
# ============================================================================

# Transpile F# script to bytecode (future feature)
transpile SCRIPT:
    @echo "Transpilation not yet implemented"
    @echo "Future: nu scripts/transpile.nu --input {{SCRIPT}}"

# Compile F# script to .fsbin bytecode file
compile SCRIPT OUTPUT:
    @echo "Compilation not yet implemented"
    @echo "Future: compile {{SCRIPT}} to {{OUTPUT}}.fsbin"

# ============================================================================
# Examples
# ============================================================================

# List available example scripts
examples:
    @echo "=== Available Examples ==="
    @ls -1 examples/*.fsx 2>/dev/null || echo "No examples yet"

# Run specific example with verbose output
run-example EXAMPLE:
    cd rust && cargo run -p fusabi -- ../examples/{{EXAMPLE}}.fsx --verbose

# ============================================================================
# Documentation
# ============================================================================

# Generate and open Rust API documentation
docs:
    cd rust && cargo doc --workspace --no-deps --open

# Generate documentation for specific crate
docs-crate CRATE:
    cd rust && cargo doc -p {{CRATE}} --no-deps --open

# Serve documentation locally
docs-serve:
    @echo "Opening documentation..."
    @python3 -m http.server 8000 -d docs/ &
    @echo "Docs available at http://localhost:8000"

# ============================================================================
# Setup and Installation
# ============================================================================

# Initial project setup and dependency installation
setup:
    nu scripts/bootstrap.nu

# Install development tools (cargo-watch, tarpaulin, etc.)
install-tools:
    cargo install cargo-watch cargo-tarpaulin cargo-edit cargo-outdated

# Install just command runner (if not already installed)
install-just:
    cargo install just

# Bootstrap the entire development environment
bootstrap: install-tools setup
    @echo "✅ Development environment ready!"
    @echo "Run 'just build' to build the project"
    @echo "Run 'just demo' to run the demo host"

# ============================================================================
# Utilities
# ============================================================================

# Display dependency tree
deps:
    cd rust && cargo tree

# Check for outdated dependencies
outdated:
    cd rust && cargo outdated

# Update all dependencies
update:
    cd rust && cargo update

# Display workspace metadata
info:
    @echo "=== Fusabi Project Info ==="
    @echo "Project: F# Script Engine (Mini-F#)"
    @echo "Workspace: rust/"
    @echo "Crates:"
    @echo "  - fusabi-frontend: Parser, typechecker, bytecode compiler"
    @echo "  - fusabi-vm: Bytecode VM runtime"
    @echo "  - fusabi: Demo host application"
    @cd rust && cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]'

# Count lines of code in the project
loc:
    @echo "=== Lines of Code ==="
    @tokei rust/ || (echo "Install tokei: cargo install tokei" && cloc rust/)

# Show current git status
status:
    @echo "=== Git Status ==="
    @git branch --show-current
    @git status --short

# ============================================================================
# Benchmarking (Future)
# ============================================================================

# Run performance benchmarks
bench:
    cd rust && cargo bench --workspace

# Run VM performance benchmarks
bench-vm:
    cd rust && cargo bench -p fusabi-vm

# Profile demo execution
profile:
    @echo "Profiling not yet configured"
    @echo "Future: flamegraph, perf, or valgrind integration"

# ============================================================================
# CI/CD
# ============================================================================

# Run CI checks locally (same as GitHub Actions)
ci: check test build-release
    @echo "✅ All CI checks passed!"

# Pre-commit checks (format, lint, test)
pre-commit: fmt lint test
    @echo "✅ Pre-commit checks passed!"

# ============================================================================
# Release Management
# ============================================================================

# Build release binaries for distribution
release:
    nu scripts/build.nu --mode release

# Publish all crates to crates.io (dry run)
publish-dry-run:
    cd rust && cargo publish -p fusabi-vm --dry-run
    cd rust && cargo publish -p fusabi-frontend --dry-run
    cd rust && cargo publish -p fusabi --dry-run

# Publish all crates to crates.io
publish:
    @echo "Publishing fusabi-vm..."
    cd rust && cargo publish -p fusabi-vm
    @echo "Waiting for crates.io propagation..."
    @sleep 20
    @echo "Publishing fusabi-frontend..."
    cd rust && cargo publish -p fusabi-frontend
    @echo "Waiting for crates.io propagation..."
    @sleep 20
    @echo "Publishing fusabi..."
    cd rust && cargo publish -p fusabi

# Package release artifacts
package:
    @echo "Packaging not yet implemented"
    @echo "Future: create distribution archives"

# ============================================================================
# Advanced
# ============================================================================

# Run under Miri (Rust undefined behavior detector)
miri:
    cd rust && cargo +nightly miri test

# Run address sanitizer
asan:
    cd rust && RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Generate flame graph for performance analysis
flamegraph:
    cd rust && cargo flamegraph --bin fus

# ============================================================================
# Help
# ============================================================================

# Show help for a specific command
help COMMAND:
    just --show {{COMMAND}}

# Show version information
version:
    @echo "Fusabi Version: 0.1.0-alpha"
    @echo ""
    @cargo --version
    @rustc --version
    @nu --version
