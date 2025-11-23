# Fusabi Testing Guide

This document describes the testing infrastructure, strategy, and best practices for the Fusabi project.

## Table of Contents

- [Overview](#overview)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [CI/CD Integration](#cicd-integration)
- [Coverage](#coverage)
- [Benchmarking](#benchmarking)
- [Pre-commit Hooks](#pre-commit-hooks)

---

## Overview

Fusabi uses a comprehensive testing strategy to ensure code quality:

- **Unit Tests**: Test individual modules and functions
- **Integration Tests**: Test interactions between components
- **End-to-End Tests**: Test complete script execution
- **Benchmarks**: Measure performance of critical paths
- **Property-Based Tests**: Test invariants (planned for future)

### Testing Goals

- **>80% code coverage** across all crates
- **Zero clippy warnings** (enforced in CI)
- **Fast feedback**: Unit tests complete in <10s
- **Comprehensive**: Cover edge cases and error paths
- **Maintainable**: Clear, readable test code

---

## Test Organization

### Directory Structure

```
rust/
├── crates/
│   ├── fusabi-frontend/
│   │   ├── src/
│   │   │   └── lib.rs          # Unit tests via #[cfg(test)]
│   │   ├── tests/
│   │   │   └── test_*.rs       # Integration tests
│   │   └── benches/            # Benchmarks (future)
│   ├── fusabi-vm/
│   │   ├── src/
│   │   │   └── lib.rs          # Unit tests
│   │   ├── tests/
│   │   │   └── test_*.rs       # Integration tests
│   │   └── benches/
│   │       └── vm_benchmarks.rs # VM performance benchmarks
│   └── fusabi-demo/
│       ├── src/
│       │   └── main.rs         # Minimal unit tests
│       └── tests/
│           └── test_integration.rs # E2E tests
└── Cargo.toml
```

### Test Categories

#### Unit Tests (in `src/`)

```rust
// src/lib.rs or src/module.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

#### Integration Tests (in `tests/`)

```rust
// tests/test_parser.rs
use fusabi_frontend::parser::Parser;

#[test]
fn test_parse_let_binding() {
    let input = "let x = 42";
    let result = Parser::parse(input);
    assert!(result.is_ok());
}
```

#### Benchmarks (in `benches/`)

```rust
// benches/vm_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn vm_benchmark(c: &mut Criterion) {
    c.bench_function("vm_execute", |b| {
        b.iter(|| {
            // Benchmark code
        })
    });
}

criterion_group!(benches, vm_benchmark);
criterion_main!(benches);
```

---

## Running Tests

### Quick Commands (via Just)

```bash
# Run all tests
just test

# Run unit tests only
just test-unit

# Run integration tests
just test-integration

# Run tests for specific crate
just test-crate fusabi-vm
just test-vm        # Shorthand for fusabi-vm
just test-frontend  # Shorthand for fusabi-frontend

# Run tests with output
nu scripts/test.nu --verbose

# Run tests without capturing stdout
nu scripts/test.nu --nocapture
```

### Manual Commands

```bash
# All tests
cd rust && cargo test --workspace

# Unit tests only
cd rust && cargo test --workspace --lib

# Integration tests only
cd rust && cargo test --workspace --test '*'

# Specific crate
cd rust && cargo test -p fusabi-vm

# Specific test
cd rust && cargo test test_name

# Show test output
cd rust && cargo test -- --nocapture

# Run ignored tests
cd rust && cargo test -- --ignored
```

### Watch Mode

```bash
# Watch and run tests on file changes
just watch-test

# Or manually
cd rust && cargo watch -x 'test --workspace'
```

---

## Writing Tests

### Unit Test Best Practices

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_handling() {
        // Test that function panics correctly
        function_that_panics();
    }

    #[test]
    fn test_result_error() -> Result<(), Box<dyn std::error::Error>> {
        let result = fallible_function()?;
        assert_eq!(result, expected);
        Ok(())
    }
}
```

### Integration Test Best Practices

```rust
// tests/test_feature.rs
use fusabi_frontend::*;

#[test]
fn test_end_to_end_workflow() {
    // Setup
    let input = "let x = 42";

    // Execute through multiple layers
    let tokens = Lexer::tokenize(input).unwrap();
    let ast = Parser::parse(&tokens).unwrap();
    let bytecode = Compiler::compile(&ast).unwrap();

    // Verify
    assert!(!bytecode.is_empty());
}
```

### Test Organization Patterns

```rust
// Group related tests
#[cfg(test)]
mod parser_tests {
    use super::*;

    mod let_bindings {
        use super::*;

        #[test]
        fn test_simple_let() { /* ... */ }

        #[test]
        fn test_let_with_type() { /* ... */ }
    }

    mod functions {
        use super::*;

        #[test]
        fn test_function_definition() { /* ... */ }
    }
}
```

### Assertion Helpers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Custom assertions
    fn assert_parse_ok(input: &str) {
        assert!(Parser::parse(input).is_ok(), "Failed to parse: {}", input);
    }

    fn assert_parse_error(input: &str) {
        assert!(Parser::parse(input).is_err(), "Expected parse error: {}", input);
    }

    #[test]
    fn test_using_helpers() {
        assert_parse_ok("let x = 42");
        assert_parse_error("let = 42");
    }
}
```

---

## CI/CD Integration

### GitHub Actions Workflows

The project has three main CI workflows:

#### 1. Main CI (`ci.yml`)

Runs on all pushes and pull requests:

- **Quick Check**: Format, lint, compilation (fast feedback)
- **Test Suite**: Run on Ubuntu, macOS, Windows (stable + beta)
- **Release Build**: Verify release mode works
- **Coverage**: Generate code coverage reports
- **Security Audit**: Check for vulnerable dependencies
- **Documentation**: Verify docs build without warnings

```yaml
# Trigger
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
```

#### 2. PR Checks (`pr-checks.yml`)

Additional checks for pull requests:

- **Quick Check**: Fast feedback on PR code
- **Title Check**: Enforce conventional commit format
- **TODO Check**: Flag new TODOs in PR
- **Size Check**: Warn on large PRs
- **First Contributor**: Welcome message

#### 3. Release (`release.yml`)

Automated release process:

- **Multi-platform Builds**: Linux, macOS, Windows (x86_64 + ARM64)
- **GitHub Release**: Create draft release with binaries
- **Crates.io Publish**: Publish to crates.io (optional)

### Running CI Locally

```bash
# Run same checks as CI
just ci

# Individual CI steps
just check        # Format + lint + test
just build-release
just test-coverage
```

---

## Coverage

### Generate Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
just test-coverage

# Or manually
cd rust
cargo tarpaulin --workspace --out Html --output-dir ../docs/coverage/
```

### View Coverage

```bash
# Open HTML report
open docs/coverage/index.html

# Or on Linux
xdg-open docs/coverage/index.html
```

### Coverage Goals

- **Overall**: >80% line coverage
- **Critical Paths**: 100% coverage
  - Parser core logic
  - VM interpreter loop
  - Bytecode generation
- **Error Handling**: Cover all error paths

### CI Coverage Integration

Coverage reports are automatically uploaded to Codecov on CI:

```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: ./rust/cobertura.xml
```

---

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
just bench

# Run VM benchmarks only
just bench-vm

# Or manually
cd rust && cargo bench --workspace
cd rust && cargo bench -p fusabi-vm
```

### Writing Benchmarks

Benchmarks are written using [Criterion.rs](https://github.com/bheisler/criterion.rs):

```rust
// benches/vm_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fusabi_vm::{VM, Chunk, OpCode};

fn vm_arithmetic_benchmark(c: &mut Criterion) {
    let mut chunk = Chunk::new();
    // Setup bytecode

    c.bench_function("vm_arithmetic", |b| {
        b.iter(|| {
            let mut vm = VM::new();
            vm.run(black_box(&chunk)).unwrap();
        })
    });
}

criterion_group!(benches, vm_arithmetic_benchmark);
criterion_main!(benches);
```

### Benchmark Best Practices

1. **Use `black_box`**: Prevent compiler optimizations
2. **Realistic Workloads**: Benchmark real-world scenarios
3. **Isolate**: Benchmark one thing at a time
4. **Compare**: Use baseline comparisons for regressions
5. **Document**: Explain what's being measured

---

## Pre-commit Hooks

### Installation

#### Using pre-commit framework

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run manually
pre-commit run --all-files
```

#### Using git hooks directly

```bash
# Install custom hooks
ln -s ../../.githooks/pre-commit .git/hooks/pre-commit
ln -s ../../.githooks/pre-push .git/hooks/pre-push

# Make executable
chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push
```

### What Gets Checked

#### Pre-commit Hook

Fast checks before each commit:

- ✓ Code formatting (`cargo fmt`)
- ✓ Clippy lints (`cargo clippy`)
- ✓ Compilation check
- ✓ Unit tests
- ✓ No debug statements (`dbg!`, `println!`)
- ✓ No large files (>1MB)

#### Pre-push Hook

Comprehensive checks before pushing:

- ✓ All pre-commit checks
- ✓ Release build
- ✓ All tests (unit + integration)

### Bypass Hooks (Not Recommended)

```bash
# Skip pre-commit
git commit --no-verify

# Skip pre-push
git push --no-verify
```

### Manual Check

```bash
# Run pre-commit checks manually
just pre-commit

# Or
.githooks/pre-commit
```

---

## Testing Workflow

### Development Workflow

```bash
# 1. Start watch mode
just watch-test

# 2. Write code and tests
# ... edit files ...

# 3. Before commit
just check        # Format, lint, test

# 4. Commit (pre-commit hook runs automatically)
git add .
git commit -m "feat: add feature"

# 5. Before push (pre-push hook runs automatically)
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

# 5. Refactor
# ... improve code ...

# 6. Verify tests still pass
just test-vm
```

---

## Test Scaffolding Reference

### Current Test Files

All test files are scaffolded with TODO comments for Phase 1 implementation:

#### fusabi-frontend tests

- `rust/crates/fusabi-frontend/tests/test_placeholder.rs`
  - Parser tests (TODO)
  - AST tests (TODO)
  - Lexer tests (TODO)

#### fusabi-vm tests

- `rust/crates/fusabi-vm/tests/test_placeholder.rs`
  - VM execution tests (TODO)
  - Bytecode chunk tests (TODO)
  - Value tests (TODO)
  - GC tests (TODO)
- `rust/crates/fusabi-vm/benches/vm_benchmarks.rs`
  - VM performance benchmarks (TODO)

#### fusabi-demo tests

- `rust/crates/fusabi-demo/tests/test_integration.rs`
  - Script execution tests (TODO)
  - Host interop tests (TODO)
  - Hot-reload tests (TODO)

### Enabling Tests

As you implement features, uncomment the TODO sections and implement the tests:

```rust
// Before
// TODO: Add parser tests
// #[test]
// fn test_parse_let_binding() { ... }

// After
#[test]
fn test_parse_let_binding() {
    let input = "let x = 42";
    let result = Parser::parse(input).unwrap();
    assert!(matches!(result, Expr::Let { .. }));
}
```

---

## Troubleshooting

### Tests Fail in CI but Pass Locally

- **Platform differences**: Run tests on multiple platforms
- **Environment variables**: Check CI env vars
- **Timing issues**: Add timeouts or retries
- **Parallelism**: Use `cargo test -- --test-threads=1`

### Slow Tests

- **Parallelize**: Use `cargo test` default parallel execution
- **Isolate**: Move slow tests to separate files
- **Mock**: Use mocks for expensive operations
- **Benchmark**: Profile to find bottlenecks

### Flaky Tests

- **Non-determinism**: Use fixed seeds for RNGs
- **Race conditions**: Add proper synchronization
- **External dependencies**: Mock external services
- **Timing**: Avoid time-dependent assertions

---

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [cargo-tarpaulin Coverage](https://github.com/xd009642/tarpaulin)
- [Pre-commit Hooks](https://pre-commit.com/)

---

## Quick Reference

```bash
# Essential commands
just test              # Run all tests
just test-unit         # Unit tests only
just test-coverage     # Generate coverage
just check             # All quality checks
just pre-commit        # Pre-commit checks

# CI simulation
just ci                # Run full CI locally

# Watch mode
just watch-test        # Auto-run tests on changes

# Benchmarks
just bench             # Run benchmarks
```

---

**Next Steps**:

1. Implement features alongside tests (TDD)
2. Aim for >80% coverage
3. Add benchmarks for critical paths
4. Enable coverage tracking in CI
5. Monitor CI build times and optimize

For implementation details, see [claude-code-notes.md](claude-code-notes.md).
