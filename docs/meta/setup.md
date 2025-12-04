# Fusabi Development Environment Setup

**Version**: 0.1.0-alpha
**Last Updated**: November 20, 2025

This guide will help you set up your development environment for Fusabi.

---

## Prerequisites

### Required Tools

#### 1. Rust (Latest Stable)

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
cargo --version
rustc --version

# Expected: cargo 1.70+ and rustc 1.70+
```

#### 2. Nushell (v0.90+)

Nushell is used for build automation scripts.

```bash
# macOS (via Homebrew)
brew install nushell

# Linux (via cargo)
cargo install nu

# Verify installation
nu --version
```

**Alternative**: You can use the project without Nushell, but you'll miss out on some automation scripts.

#### 3. Just (Optional but Recommended)

Just is a command runner (like make, but better).

```bash
# Via cargo
cargo install just

# Verify installation
just --version
```

**Note**: Just is optional. You can run commands directly, but Just provides a better developer experience.

### Optional Tools

#### Development Tools

```bash
# Auto-rebuild on file changes
cargo install cargo-watch

# Code coverage
cargo install cargo-tarpaulin

# Manage dependencies
cargo install cargo-edit

# Check for outdated dependencies
cargo install cargo-outdated

# Count lines of code
cargo install tokei
```

---

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/fusabi-lang/fusabi.git
cd fusabi
```

### 2. Bootstrap the Environment

Run the bootstrap script to set up and validate the development environment:

```bash
# Option 1: Using Just (recommended)
just bootstrap

# Option 2: Using Nushell directly
nu scripts/bootstrap.nu

# Option 3: Manual steps (see below)
```

The bootstrap script will:
- Check for required tools
- Validate the workspace structure
- Verify that the project compiles
- Display recommended tools

### 3. Build the Project

```bash
# Using Just
just build

# Or directly with cargo
cd rust && cargo build --workspace
```

### 4. Run Tests

```bash
# Using Just
just test

# Or directly with cargo
cd rust && cargo test --workspace
```

### 5. Run the Demo

```bash
# Using Just
just demo

# Or directly with cargo
cd rust && cargo run -p fusabi-demo
```

**Note**: In the early stages, the demo will just print a placeholder message.

---

## Manual Setup (Without Just/Nushell)

If you prefer not to use Just or Nushell, here's how to work with the project manually:

### Build Commands

```bash
# Navigate to Rust workspace
cd rust

# Build all crates in development mode
cargo build --workspace

# Build in release mode (optimized)
cargo build --workspace --release

# Build specific crate
cargo build -p fusabi-frontend
cargo build -p fusabi-vm
cargo build -p fusabi-demo
```

### Test Commands

```bash
cd rust

# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p fusabi-frontend
cargo test -p fusabi-vm

# Run tests with output visible
cargo test --workspace -- --nocapture

# Generate coverage report (requires tarpaulin)
cargo tarpaulin --workspace --out Html --output-dir ../docs/coverage/
```

### Code Quality

```bash
cd rust

# Format code
cargo fmt --all

# Check formatting (CI mode)
cargo fmt --all -- --check

# Run clippy linter
cargo clippy --all-targets --workspace -- -D warnings

# Auto-fix clippy issues
cargo clippy --all-targets --workspace --fix --allow-dirty

# Check compilation without building
cargo check --all-targets --workspace
```

### Development

```bash
cd rust

# Watch and rebuild on changes (requires cargo-watch)
cargo watch -x 'build --workspace'

# Watch and run tests on changes
cargo watch -x 'test --workspace'

# Watch and run demo on changes
cargo watch -x 'run -p fusabi-demo'
```

---

## Project Structure

After setup, your directory structure should look like this:

```
fusabi/
├── rust/                   # Rust workspace
│   ├── Cargo.toml          # Workspace configuration
│   ├── crates/
│   │   ├── fusabi-frontend/  # Parser, typechecker, compiler
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── fusabi-vm/        # Bytecode VM runtime
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   └── fusabi-demo/      # Demo host application
│   │       ├── Cargo.toml
│   │       └── src/
│   │           └── main.rs
│   └── target/             # Build artifacts (gitignored)
├── docs/                   # Documentation
│   ├── roadmap.md
│   ├── setup.md (this file)
│   ├── 01-overview.md
│   ├── 02-language-spec.md
│   ├── 03-vm-design.md
│   ├── claude-code-notes.md
│   └── ...
├── examples/               # Example .fsx scripts
│   └── fusabi_config.fsx
├── scripts/                # Nushell automation scripts
│   ├── build.nu
│   ├── test.nu
│   └── bootstrap.nu
├── tests/                  # Integration tests (future)
├── .claude/                # Claude Code configuration
├── justfile                # Just command definitions
├── claude-config.md               # Project configuration for Claude
├── README.md               # Project overview
└── .gitignore
```

---

## Workspace Validation

### Verify Rust Workspace

```bash
cd rust

# Check workspace structure
cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]'

# Expected output:
# fusabi-frontend 0.1.0 (path+file:///.../rust/crates/fusabi-frontend)
# fusabi-vm 0.1.0 (path+file:///.../rust/crates/fusabi-vm)
# fusabi-demo 0.1.0 (path+file:///.../rust/crates/fusabi-demo)
```

### Verify Dependencies

```bash
cd rust

# Show dependency tree
cargo tree

# Check for outdated dependencies (requires cargo-outdated)
cargo outdated

# Update dependencies to latest compatible versions
cargo update
```

---

## Common Issues & Solutions

### Issue: Rust Not Found

**Error**: `cargo: command not found`

**Solution**:
```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your shell or run
source $HOME/.cargo/env
```

### Issue: Nushell Not Found

**Error**: `nu: command not found`

**Solution**:
```bash
# Install via cargo
cargo install nu

# Or use Homebrew on macOS
brew install nushell
```

### Issue: Compilation Errors

**Error**: Various compilation errors after cloning

**Solution**:
```bash
# Clean build artifacts
cd rust && cargo clean

# Update Rust toolchain
rustup update

# Rebuild
cargo build --workspace
```

### Issue: Missing Just

**Error**: `just: command not found`

**Solution**:
```bash
# Install Just
cargo install just

# Or manually run commands (see "Manual Setup" section)
```

### Issue: Permission Denied for Scripts

**Error**: Permission denied when running `./scripts/*.nu`

**Solution**:
```bash
# Make scripts executable
chmod +x scripts/*.nu

# Or run via nu directly
nu scripts/bootstrap.nu
```

---

## Development Workflow

### Typical Development Cycle

```bash
# 1. Start with clean workspace
just check

# 2. Start watch mode (auto-rebuild on changes)
just watch

# 3. In another terminal, run tests on changes
just watch-test

# 4. Make changes to code...

# 5. Run full checks before committing
just pre-commit

# 6. Commit changes
git add .
git commit -m "feat: implement lexer for Phase 1"
```

### Using Just for Common Tasks

```bash
# Show all available commands
just

# Build
just build              # Development mode
just build-release      # Release mode

# Test
just test               # All tests
just test-unit          # Unit tests only
just test-integration   # Integration tests
just test-coverage      # Coverage report

# Development
just dev                # Watch mode with tests
just watch              # Watch and rebuild
just demo               # Run demo host

# Quality
just check              # fmt + lint + test
just fmt                # Format code
just lint               # Run clippy

# Documentation
just docs               # Generate and open docs
```

---

## IDE Setup

### VS Code

**Recommended Extensions**:
- `rust-lang.rust-analyzer` - Rust language support
- `tamasfe.even-better-toml` - Better TOML support
- `serayuzgur.crates` - Manage Cargo dependencies
- `swellaby.vscode-rust-test-adapter` - Test explorer

**Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### CLion / IntelliJ IDEA

**Plugins**:
- Rust Plugin (IntelliJ Rust)

**Configuration**:
- Open the `rust/` directory as the project root
- CLion will auto-detect the Cargo workspace

---

## Next Steps

After setup is complete:

1. **Read the documentation**:
   - Start with `docs/roadmap.md` for the development plan
   - Read `docs/claude-code-notes.md` for implementation tasks
   - Review `docs/01-overview.md` for architecture overview

2. **Start implementing**:
   - Begin with Phase 1, Milestone 1.1 (AST, lexer, parser)
   - Follow the task breakdown in `claude-code-notes.md`
   - Use Claude Code with prompts from the documentation

3. **Run the checks**:
   ```bash
   just check    # Ensure everything builds and tests pass
   ```

4. **Explore the codebase**:
   ```bash
   # View the workspace structure
   cd rust && cargo tree

   # Read the stub implementations
   cat rust/crates/fusabi-frontend/src/lib.rs
   cat rust/crates/fusabi-vm/src/lib.rs
   cat rust/crates/fusabi-demo/src/main.rs
   ```

---

## Getting Help

- **Documentation**: See `docs/toc.md` for the complete documentation index
- **Issues**: Report bugs or request features on GitHub Issues
- **Discussions**: Ask questions on GitHub Discussions
- **Workflow**: Refer to `docs/claude-config.md` for development patterns

---

## Summary Checklist

Before starting development, ensure:

- ✅ Rust (latest stable) is installed
- ✅ Nushell is installed (or you're comfortable with manual commands)
- ✅ Just is installed (recommended)
- ✅ Project builds successfully: `just build`
- ✅ Tests pass: `just test`
- ✅ You've read `docs/roadmap.md`
- ✅ You've skimmed `docs/claude-code-notes.md`
- ✅ You understand the project structure
- ✅ Your IDE is configured (optional but helpful)

**You're ready to start developing! 🚀**

Proceed to `docs/roadmap.md` to understand the development phases, then dive into `docs/claude-code-notes.md` for specific implementation tasks.
