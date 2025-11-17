# Issue #009: Test Suite and CI/CD

## Overview
Establish comprehensive test coverage and CI/CD pipeline for automated testing and quality assurance.

## Labels
- `feature`, `phase-1: mvp`, `priority: high`, `component: tests`, `infrastructure`, `effort: m` (2-3 days)

## Milestone
Phase 1.3: Integration (Week 3)

## Dependencies
- #008 (Demo) - Recommended for end-to-end tests

## Acceptance Criteria
- [ ] Unit tests: 50+ tests across all crates
- [ ] Integration tests: 10+ end-to-end scenarios
- [ ] Test coverage > 70%
- [ ] GitHub Actions CI configured
- [ ] Pre-commit hooks working
- [ ] Documentation for running tests

## Technical Specification

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable

      - name: Build
        run: cd rust && cargo build --workspace --verbose

      - name: Run tests
        run: cd rust && cargo test --workspace --verbose

      - name: Check formatting
        run: cd rust && cargo fmt --all -- --check

      - name: Run clippy
        run: cd rust && cargo clippy --all-targets --workspace -- -D warnings

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cd rust && cargo tarpaulin --workspace --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Integration Tests

```rust
// rust/crates/fsrs-demo/tests/integration_test.rs

use fsrs_demo::run_script;

#[test]
fn test_arithmetic_script() {
    let result = run_script("../examples/arithmetic.fsrs").unwrap();
    assert_eq!(result.as_int(), Some(7)); // 1 + 2 * 3
}

#[test]
fn test_conditional_script() {
    let result = run_script("../examples/conditional.fsrs").unwrap();
    assert_eq!(result.as_int(), Some(42));
}
```

## Estimated Effort
**2-3 days**

## Related Issues
- Uses all previous issues for integration testing