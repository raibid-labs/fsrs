# CI/CD Setup Guide for Fusabi

This document provides step-by-step instructions for setting up the CI/CD infrastructure for Fusabi.

## Table of Contents

- [Overview](#overview)
- [Local Setup](#local-setup)
- [GitHub Actions Setup](#github-actions-setup)
- [Pre-commit Hooks Setup](#pre-commit-hooks-setup)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

---

## Overview

The Fusabi CI/CD infrastructure includes:

1. **GitHub Actions Workflows**: Automated testing, building, and releasing
2. **Pre-commit Hooks**: Local checks before commits and pushes
3. **Test Scaffolding**: Ready-to-use test structure for all crates
4. **Coverage Reporting**: Integrated code coverage tracking
5. **Security Auditing**: Automated dependency vulnerability scanning

---

## Local Setup

### Prerequisites

Ensure you have the following installed:

```bash
# Check Rust
rustc --version  # Should be 1.70+

# Check Cargo
cargo --version

# Check Nushell
nu --version  # Should be 0.90+

# Check Just (optional but recommended)
just --version
```

### Install Development Tools

```bash
# Install Just (if not already installed)
cargo install just

# Install useful cargo extensions
cargo install cargo-watch cargo-edit

# Optional: Install testing tools
cargo install cargo-tarpaulin  # For coverage
cargo install cargo-audit      # For security audits
cargo install cargo-outdated   # For dependency updates
```

---

## GitHub Actions Setup

### Files Created

The following workflow files have been created:

```
.github/workflows/
├── ci.yml          # Main CI pipeline (all branches, PRs)
├── pr-checks.yml   # PR-specific checks
└── release.yml     # Release automation
```

### What Each Workflow Does

#### ci.yml (Main CI)

Runs on:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual trigger

Jobs:
1. **Quick Check** (2-3 min)
   - Format check
   - Clippy linting
   - Compilation check

2. **Test Suite** (5-10 min)
   - Runs on: Ubuntu, macOS, Windows
   - Rust versions: stable, beta
   - Unit tests, integration tests
   - Crate-specific tests

3. **Build Release** (3-5 min)
   - Build optimized binaries
   - Upload artifacts

4. **Coverage** (5-7 min)
   - Generate coverage reports
   - Upload to Codecov

5. **Security Audit** (1-2 min)
   - Check for vulnerable dependencies

6. **Documentation** (2-3 min)
   - Verify docs build without warnings

7. **Benchmarks** (2-3 min)
   - Ensure benchmarks compile

#### pr-checks.yml (PR Checks)

Additional checks for pull requests:
- PR title format validation
- New TODO detection
- PR size warnings
- First-contributor welcome messages

#### release.yml (Release Automation)

Triggered by:
- Git tags matching `v*.*.*`
- Manual workflow dispatch

Creates:
- Multi-platform binaries (Linux, macOS, Windows)
- GitHub release with artifacts
- Optional crates.io publishing

### Enabling CI

**The workflows are already enabled!**

No additional setup is required. Once you push to GitHub:

1. Push the CI configuration:
   ```bash
   git add .github/
   git commit -m "ci: add CI/CD workflows"
   git push
   ```

2. Workflows will automatically run on:
   - Every push to `main`/`develop`
   - Every pull request
   - Every tag push

### Optional: Configure Secrets

For enhanced functionality, add these secrets:

#### Codecov Token (for coverage reports)

1. Sign up at https://codecov.io
2. Add your repository
3. Copy the token
4. Add to GitHub:
   ```
   Settings → Secrets → Actions → New repository secret
   Name: CODECOV_TOKEN
   Value: <your-token>
   ```

#### Cargo Token (for crates.io publishing)

1. Generate token at https://crates.io/settings/tokens
2. Add to GitHub:
   ```
   Settings → Secrets → Actions → New repository secret
   Name: CARGO_TOKEN
   Value: <your-token>
   ```

---

## Pre-commit Hooks Setup

### Option 1: Using pre-commit Framework (Recommended)

```bash
# Install pre-commit
pip install pre-commit

# Or using pipx (isolated install)
pipx install pre-commit

# Install the hooks
pre-commit install

# Test the installation
pre-commit run --all-files
```

Configuration file: `.pre-commit-config.yaml`

### Option 2: Direct Git Hooks

```bash
# Use the provided setup script
nu scripts/setup-hooks.nu

# Or manually
ln -s ../../.githooks/pre-commit .git/hooks/pre-commit
ln -s ../../.githooks/pre-push .git/hooks/pre-push
chmod +x .githooks/*
```

### What Gets Checked

#### Pre-commit Hook (runs before each commit)

Fast checks (~30 seconds):
- ✓ Code formatting (`cargo fmt`)
- ✓ Clippy lints (`cargo clippy`)
- ✓ Compilation
- ✓ Unit tests
- ✓ No debug statements (`dbg!`, `println!`)
- ✓ No large files (>1MB)

#### Pre-push Hook (runs before each push)

Comprehensive checks (~2-3 minutes):
- ✓ All pre-commit checks
- ✓ Release build
- ✓ All tests (unit + integration)

### Testing Hooks Manually

```bash
# Test pre-commit hook
.githooks/pre-commit

# Test pre-push hook
.githooks/pre-push

# Or with pre-commit framework
pre-commit run --all-files
```

---

## Verification

### Step 1: Verify Justfile Commands

```bash
# Quick check
just fmt-check
just lint
just check-compile

# Build
just build
just build-release

# Test
just test
just test-unit
just test-vm
just test-frontend

# All checks (like CI)
just check
```

All commands should complete successfully.

### Step 2: Verify Git Hooks

```bash
# Check hooks are installed
ls -la .git/hooks/pre-commit
ls -la .git/hooks/pre-push

# Test pre-commit hook
.githooks/pre-commit

# Should output:
# ✓ Code formatting OK
# ✓ Clippy checks passed
# ✓ Compilation check passed
# ✓ Unit tests passed
# ✓ No debug statements found
# ✓ No large files found
# ✓ All pre-commit checks passed!
```

### Step 3: Verify Test Infrastructure

```bash
# Run all tests
just test

# Should show:
# ✅ All tests passed!
# 📊 test result: ok. X passed; 0 failed; 0 ignored...
```

### Step 4: Verify CI Workflows

```bash
# Check workflow files exist
ls -la .github/workflows/

# Should show:
# ci.yml
# pr-checks.yml
# release.yml
```

### Step 5: Test a Commit

```bash
# Make a small change
echo "// test" >> rust/crates/fusabi-vm/src/lib.rs

# Try to commit (hooks will run)
git add .
git commit -m "test: verify hooks"

# Hooks should run and pass
# Then reset the test change
git reset HEAD~1
git checkout -- .
```

---

## Troubleshooting

### Issue: Git hooks not running

**Solution**:
```bash
# Re-install hooks
nu scripts/setup-hooks.nu

# Or manually check
ls -la .git/hooks/pre-commit
# Should be a symlink to ../../.githooks/pre-commit
```

### Issue: Pre-commit framework not working

**Solution**:
```bash
# Reinstall pre-commit
pip install --upgrade pre-commit

# Reinstall hooks
pre-commit uninstall
pre-commit install

# Test
pre-commit run --all-files
```

### Issue: Tests fail during pre-commit

**Solution**:
```bash
# Run tests manually to see details
just test -- --nocapture

# Or bypass hook temporarily (not recommended)
git commit --no-verify
```

### Issue: Clippy fails with warnings

**Solution**:
```bash
# See what clippy found
just lint

# Auto-fix if possible
just lint-fix

# Manually fix remaining issues
```

### Issue: Formatting check fails

**Solution**:
```bash
# Apply formatting
just fmt

# Verify
just fmt-check
```

### Issue: GitHub Actions not running

**Solution**:
1. Ensure workflows are committed and pushed:
   ```bash
   git add .github/workflows/
   git commit -m "ci: add workflows"
   git push
   ```

2. Check GitHub Actions tab in repository

3. Verify workflow syntax:
   ```bash
   # Install actionlint (optional)
   go install github.com/rhysd/actionlint/cmd/actionlint@latest

   # Check workflows
   actionlint .github/workflows/*.yml
   ```

### Issue: Benchmark compilation fails

**Solution**:
This is expected - benchmarks are scaffolded but not fully implemented yet.
The CI is configured with `continue-on-error: true` for benchmark jobs.

---

## Next Steps

### For Development

1. **Start Development**:
   ```bash
   just watch-test
   ```

2. **Before Each Commit**:
   - Hooks run automatically
   - Or manually: `just check`

3. **Create Pull Requests**:
   ```bash
   git checkout -b feat/my-feature
   # ... make changes ...
   git push -u origin feat/my-feature
   gh pr create
   ```

### For CI/CD Enhancement

1. **Add Codecov Integration**:
   - Sign up at codecov.io
   - Add repository
   - Configure `CODECOV_TOKEN` secret

2. **Enable crates.io Publishing**:
   - Generate token at crates.io
   - Add `CARGO_TOKEN` secret
   - Update `release.yml` if needed

3. **Monitor CI Performance**:
   - Check GitHub Actions dashboard
   - Optimize slow jobs
   - Review cache effectiveness

---

## Configuration Files Reference

### Workflow Configuration

```
.github/workflows/ci.yml        - Main CI pipeline
.github/workflows/pr-checks.yml - PR validation
.github/workflows/release.yml   - Release automation
```

### Hook Configuration

```
.githooks/pre-commit            - Pre-commit hook script
.githooks/pre-push              - Pre-push hook script
.pre-commit-config.yaml         - pre-commit framework config
```

### Test Infrastructure

```
rust/crates/fusabi-frontend/tests/test_placeholder.rs - Frontend tests
rust/crates/fusabi-vm/tests/test_placeholder.rs       - VM tests
rust/crates/fusabi-vm/benches/vm_benchmarks.rs        - VM benchmarks
rust/crates/fusabi-demo/tests/test_integration.rs     - E2E tests
```

### Documentation

```
docs/testing.md      - Testing guide
docs/ci-cd.md        - CI/CD documentation
docs/development.md  - Development guide
docs/setup-ci.md     - This file
```

---

## Quick Reference Commands

```bash
# Setup
nu scripts/setup-hooks.nu  # Install git hooks
just bootstrap             # Bootstrap environment

# Development
just watch-test            # Watch and test
just check                 # All quality checks

# Testing
just test                  # All tests
just test-unit             # Unit tests only
just test-coverage         # Generate coverage

# CI Simulation
just ci                    # Run CI checks locally
just pre-commit            # Pre-commit checks

# Git Workflow
git add .
git commit -m "feat: ..."  # Hooks run automatically
git push                   # Hooks run automatically
```

---

## Support

If you encounter issues not covered here:

1. Check [ci-cd.md](ci-cd.md) for detailed CI/CD documentation
2. Check [testing.md](testing.md) for testing details
3. Check [development.md](development.md) for development workflows
4. Open an issue on GitHub

---

**Status**: CI/CD infrastructure fully operational

**Last Updated**: 2025-11-17

**Next Phase**: Implement Phase 1 features with automated testing
