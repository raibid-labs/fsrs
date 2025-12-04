# Fusabi Development Guide

Quick reference for Fusabi development workflows, tools, and best practices.

## Table of Contents

- [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
- [Git Hooks](#git-hooks)
- [Code Quality](#code-quality)
- [Testing](#testing)
- [Debugging](#debugging)
- [Performance](#performance)
- [Contributing](#contributing)

---

## Quick Start

### Initial Setup

```bash
# Clone repository
git clone https://github.com/fusabi-lang/fusabi.git
cd fsrs

# Bootstrap environment
just bootstrap

# Install Git hooks
nu scripts/setup-hooks.nu

# Build project
just build

# Run tests
just test
```

### Development Environment

**Required**:
- Rust 1.70+ ([rustup.rs](https://rustup.rs))
- Nushell 0.90+ ([nushell.sh](https://nushell.sh))

**Recommended**:
- Just command runner: `cargo install just`
- cargo-watch: `cargo install cargo-watch`
- cargo-edit: `cargo install cargo-edit`

**Optional**:
- cargo-tarpaulin: `cargo install cargo-tarpaulin` (coverage)
- pre-commit: `pip install pre-commit` (enhanced hooks)

---

## Development Workflow

### Daily Development

```bash
# 1. Start watch mode
just watch-test

# 2. Make changes
vim rust/crates/fusabi-vm/src/vm.rs

# 3. Tests run automatically
# ... watch for failures ...

# 4. Before commit
just check

# 5. Commit (hooks run automatically)
git add .
git commit -m "feat: add feature"

# 6. Push (hooks run automatically)
git push
```

### TDD Workflow

```bash
# 1. Write failing test
vim rust/crates/fusabi-vm/tests/test_vm.rs

# 2. Run test (should fail)
just test-vm

# 3. Implement feature
vim rust/crates/fusabi-vm/src/vm.rs

# 4. Run test (should pass)
just test-vm

# 5. Refactor and verify
just test-vm
```

### Feature Development

```bash
# 1. Create feature branch
git checkout -b feat/my-feature

# 2. Develop with watch mode
just dev

# 3. Run checks before commit
just check

# 4. Commit with conventional commits
git commit -m "feat: add my feature"

# 5. Push and create PR
git push -u origin feat/my-feature
gh pr create
```

---

## Git Hooks

### Installation

```bash
# Install hooks
nu scripts/setup-hooks.nu

# Or manually
ln -s ../../.githooks/pre-commit .git/hooks/pre-commit
ln -s ../../.githooks/pre-push .git/hooks/pre-push
chmod +x .githooks/*
```

### Pre-commit Hook

Runs on every commit (~30s):

```
✓ Check formatting
✓ Run clippy
✓ Check compilation
✓ Run unit tests
✓ Check for debug statements
✓ Check for large files
```

### Pre-push Hook

Runs on every push (~2-3 min):

```
✓ All pre-commit checks
✓ Build release
✓ Run all tests
```

### Bypassing Hooks

```bash
# Emergency only - CI will still catch issues
git commit --no-verify
git push --no-verify
```

---

## Code Quality

### Formatting

```bash
# Check formatting
just fmt-check

# Apply formatting
just fmt

# Auto-format on save (VS Code)
# Add to .vscode/settings.json:
{
  "rust-analyzer.rustfmt.extraArgs": ["+nightly"],
  "[rust]": {
    "editor.formatOnSave": true
  }
}
```

### Linting

```bash
# Run clippy
just lint

# Auto-fix clippy issues
just lint-fix

# Clippy with extra pedantic lints
cd rust && cargo clippy --all-targets --workspace -- -W clippy::pedantic
```

### Code Style Guidelines

**Rust**:
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Max line length: 100 characters
- Use descriptive names (readability > brevity)
- Document public APIs with `///`
- Add `#[must_use]` for important return values

**Modules**:
- Keep files under 500 lines
- One public type per module (exceptions allowed)
- Use `mod.rs` for module organization

**Error Handling**:
- Use `Result<T, E>` for fallible operations
- Create custom error types per crate
- Provide context with error messages

---

## Testing

### Running Tests

```bash
# All tests
just test

# Unit tests only
just test-unit

# Integration tests
just test-integration

# Specific crate
just test-vm
just test-frontend

# Specific test
cd rust && cargo test test_name

# With output
cd rust && cargo test -- --nocapture

# Watch mode
just watch-test
```

### Writing Tests

```rust
// Unit tests (in src/)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        let result = my_function(42);
        assert_eq!(result, expected);
    }
}

// Integration tests (in tests/)
use fusabi_vm::VM;

#[test]
fn test_end_to_end() {
    let vm = VM::new();
    let result = vm.execute("let x = 42");
    assert!(result.is_ok());
}
```

### Coverage

```bash
# Generate coverage report
just test-coverage

# View report
open docs/coverage/index.html
```

See [testing.md](testing.md) for comprehensive testing guide.

---

## Debugging

### Debug Logging

```rust
// Use debug logging (remove before commit)
dbg!(value);
println!("Debug: {:?}", value);

// Use proper logging (keep)
log::debug!("Processing {}", value);
log::error!("Failed: {}", err);
```

### GDB/LLDB

```bash
# Build with debug symbols
just build

# Debug with rust-gdb
rust-gdb rust/target/debug/fusabi-demo

# Debug with rust-lldb (macOS)
rust-lldb rust/target/debug/fusabi-demo
```

### Print Debugging

```rust
// Temporary debug output
#[cfg(debug_assertions)]
eprintln!("DEBUG: value = {:?}", value);
```

### VS Code Debugging

Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug fusabi-demo",
      "cargo": {
        "args": ["build", "-p", "fusabi-demo"]
      },
      "args": [],
      "cwd": "${workspaceFolder}/rust"
    }
  ]
}
```

---

## Performance

### Profiling

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flame graph
just flamegraph

# View flame graph
open flamegraph.svg
```

### Benchmarking

```bash
# Run benchmarks
just bench

# Run VM benchmarks only
just bench-vm

# Compare benchmarks
cargo bench --bench vm_benchmarks -- --save-baseline main
# ... make changes ...
cargo bench --bench vm_benchmarks -- --baseline main
```

### Optimization Tips

1. **Profile First**: Don't optimize without data
2. **Release Mode**: Always benchmark in release mode
3. **Inline Hot Paths**: Use `#[inline]` for small, hot functions
4. **Avoid Allocations**: Reuse buffers in loops
5. **Use References**: Pass large types by reference

---

## Contributing

### Conventional Commits

Follow conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `perf`: Performance improvement
- `test`: Add/update tests
- `build`: Build system changes
- `ci`: CI configuration
- `chore`: Maintenance

Examples:
```
feat(vm): add bytecode interpreter loop
fix(parser): resolve let-binding precedence bug
docs(testing): update testing guide
test(vm): add stack overflow tests
```

### Pull Request Process

1. **Create Feature Branch**:
   ```bash
   git checkout -b feat/my-feature
   ```

2. **Develop and Test**:
   ```bash
   just watch-test
   ```

3. **Run Checks**:
   ```bash
   just check
   ```

4. **Commit**:
   ```bash
   git add .
   git commit -m "feat: add feature"
   ```

5. **Push**:
   ```bash
   git push -u origin feat/my-feature
   ```

6. **Create PR**:
   ```bash
   gh pr create --fill
   ```

7. **Address Review Comments**:
   ```bash
   # Make changes
   git add .
   git commit -m "fix: address review comments"
   git push
   ```

### Code Review Checklist

Before requesting review:

- [ ] Tests pass locally (`just test`)
- [ ] Format and lint pass (`just check`)
- [ ] Added tests for new features
- [ ] Updated documentation
- [ ] No debug statements (`dbg!`, `println!`)
- [ ] Conventional commit messages
- [ ] PR description explains changes

---

## Useful Commands

### Building

```bash
just build              # Dev build
just build-release      # Release build
just build-crate NAME   # Build specific crate
just clean              # Clean build artifacts
```

### Testing

```bash
just test               # All tests
just test-unit          # Unit tests
just test-integration   # Integration tests
just test-coverage      # Generate coverage
just watch-test         # Watch and test
```

### Quality

```bash
just check              # All checks (fmt + lint + test)
just fmt                # Format code
just fmt-check          # Check formatting
just lint               # Run clippy
just lint-fix           # Auto-fix clippy
```

### Development

```bash
just dev                # Watch mode (test + run)
just watch              # Watch and build
just watch-demo         # Watch and run demo
just demo               # Run demo
```

### Documentation

```bash
just docs               # Generate and open docs
just docs-serve         # Serve docs on localhost:8000
```

### Utilities

```bash
just info               # Project information
just deps               # Dependency tree
just outdated           # Check for outdated deps
just update             # Update dependencies
just loc                # Lines of code
```

### CI/CD

```bash
just ci                 # Run CI checks locally
just pre-commit         # Pre-commit checks
```

---

## IDE Configuration

### VS Code

Recommended extensions:
- rust-analyzer
- CodeLLDB (debugging)
- Even Better TOML
- GitLens

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Import as Cargo project
3. Enable "Run clippy on save"
4. Set formatter to rustfmt

---

## Troubleshooting

### Common Issues

#### Cargo Lock Conflicts

```bash
# Update lock file
cd rust && cargo update

# Or reset
rm rust/Cargo.lock
cargo build
```

#### Cache Issues

```bash
# Clean build
just clean

# Or remove target directory
rm -rf rust/target
```

#### Hook Failures

```bash
# Re-install hooks
nu scripts/setup-hooks.nu

# Or bypass (emergency only)
git commit --no-verify
```

#### Test Failures

```bash
# Run with output
cd rust && cargo test -- --nocapture

# Run specific test
cd rust && cargo test test_name -- --nocapture
```

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [Fusabi Documentation](docs/)

---

## Quick Reference Card

```bash
# Setup
just bootstrap          # Initial setup
nu scripts/setup-hooks.nu  # Install hooks

# Development
just dev                # Watch mode
just watch-test         # Watch tests
just demo               # Run demo

# Quality
just check              # All checks
just fmt                # Format
just lint               # Lint
just test               # Test

# Before Commit
just pre-commit         # Fast checks

# Before Push
just ci                 # Full CI checks
```

---

**Happy Coding!**

For more details, see:
- [testing.md](testing.md) - Testing guide
- [ci-cd.md](ci-cd.md) - CI/CD documentation
- [claude-config.md](claude-config.md) - Claude Code configuration
