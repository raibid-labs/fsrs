# Fusabi CI/CD Documentation

This document describes the continuous integration and deployment infrastructure for Fusabi.

## Table of Contents

- [Overview](#overview)
- [GitHub Actions Workflows](#github-actions-workflows)
- [Pre-commit Hooks](#pre-commit-hooks)
- [Release Process](#release-process)
- [Running CI Locally](#running-ci-locally)
- [Troubleshooting](#troubleshooting)

---

## Overview

FSRS uses a multi-layered CI/CD strategy:

1. **Pre-commit Hooks**: Fast local checks before commit
2. **Pre-push Hooks**: Comprehensive checks before push
3. **Pull Request Checks**: Automated PR validation
4. **Main CI Pipeline**: Full test suite on all platforms
5. **Release Automation**: Multi-platform binary builds

### CI/CD Goals

- **Fast Feedback**: <2 min for quick checks
- **Comprehensive**: Cover all platforms and configurations
- **Reliable**: No flaky tests or intermittent failures
- **Secure**: Automated security audits
- **Automated**: Minimal manual intervention

---

## GitHub Actions Workflows

### 1. Main CI Workflow (`.github/workflows/ci.yml`)

**Trigger**: Push to `main`/`develop`, PRs to `main`/`develop`

#### Jobs

##### Quick Check (2-3 minutes)
Runs first for fast feedback:
```yaml
- Format check (cargo fmt)
- Clippy lint (cargo clippy)
- Compilation check (cargo check)
```

##### Test Suite (5-10 minutes)
Matrix across platforms and Rust versions:
```yaml
platforms: [ubuntu-latest, macos-latest, windows-latest]
rust: [stable, beta]
```

Tests:
- Unit tests
- Integration tests
- VM tests
- Frontend tests

##### Build Release (3-5 minutes)
Verify release mode builds correctly:
```yaml
- cargo build --release
- Upload artifacts (demo binary)
```

##### Code Coverage (5-7 minutes)
Generate and upload coverage:
```yaml
- Run tarpaulin
- Upload to Codecov
```

##### Security Audit (1-2 minutes)
Check for vulnerable dependencies:
```yaml
- cargo audit
```

##### Documentation (2-3 minutes)
Verify docs build without warnings:
```yaml
- cargo doc --workspace
```

##### Benchmarks (2-3 minutes)
Ensure benchmarks compile:
```yaml
- cargo bench --no-run
```

#### Total Pipeline Time
- **Quick Path** (quick check only): ~2-3 minutes
- **Full Pipeline**: ~15-20 minutes (parallel)

### 2. PR Checks Workflow (`.github/workflows/pr-checks.yml`)

**Trigger**: PR opened, synchronized, reopened

#### Jobs

##### PR Quick Check (2-3 minutes)
Fast feedback on PR code:
```yaml
- Format check
- Clippy
- Build
```

##### PR Title Check (<1 minute)
Enforce conventional commits:
```yaml
types: [feat, fix, docs, style, refactor, perf, test, build, ci, chore]
```

##### TODO Check (<1 minute)
Flag new TODOs in PR:
```yaml
- git diff for TODO comments
- Report as warning
```

##### Size Check (<1 minute)
Warn on large PRs:
```yaml
- >50 files changed
- Report with suggestion to split
```

##### First Contributor Welcome
Automatic welcome message for first-time contributors.

### 3. Release Workflow (`.github/workflows/release.yml`)

**Trigger**: Git tag `v*.*.*` or manual dispatch

#### Jobs

##### Create Release
Create GitHub release draft:
```yaml
- Parse version from tag
- Create release
- Set as draft (manual publish)
```

##### Build Release Binaries (Matrix)
Build for multiple platforms:
```yaml
platforms:
  - ubuntu (x86_64, aarch64)
  - macos (x86_64, aarch64)
  - windows (x86_64)
```

Each platform:
```yaml
- Setup toolchain
- Build release binary
- Strip binary (Unix)
- Upload to release
```

##### Publish to crates.io (Optional)
Automatically publish crates:
```yaml
- Publish fusabi-vm
- Wait 30s
- Publish fusabi-frontend
```

**Note**: Requires `CARGO_TOKEN` secret configured.

---

## Pre-commit Hooks

### Installation Methods

#### Option 1: pre-commit Framework (Recommended)

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Test installation
pre-commit run --all-files
```

Configuration: `.pre-commit-config.yaml`

#### Option 2: Git Hooks Directly

```bash
# Install custom hooks
ln -s ../../.githooks/pre-commit .git/hooks/pre-commit
ln -s ../../.githooks/pre-push .git/hooks/pre-push

# Make executable
chmod +x .githooks/pre-commit .githooks/pre-push
```

### Pre-commit Hook Checks

Runs on **every commit** (~30s):

1. **Format Check**: `cargo fmt --check`
2. **Clippy**: `cargo clippy -- -D warnings`
3. **Compilation**: `cargo check`
4. **Unit Tests**: `cargo test --lib`
5. **Debug Statements**: Grep for `dbg!`, `println!`
6. **Large Files**: Check for files >1MB

### Pre-push Hook Checks

Runs on **every push** (~2-3 min):

1. All pre-commit checks
2. **Release Build**: `cargo build --release`
3. **All Tests**: `cargo test --workspace`

### Bypassing Hooks (Not Recommended)

```bash
# Skip pre-commit
git commit --no-verify

# Skip pre-push
git push --no-verify
```

**Warning**: Only bypass for emergencies. CI will still catch issues.

---

## Release Process

### Manual Release (Recommended)

1. **Update Version Numbers**:
   ```bash
   # Update Cargo.toml in all crates
   vim rust/crates/fusabi-vm/Cargo.toml
   vim rust/crates/fusabi-frontend/Cargo.toml
   vim rust/crates/fusabi-demo/Cargo.toml
   ```

2. **Update Changelog**:
   ```bash
   vim CHANGELOG.md
   ```

3. **Commit and Tag**:
   ```bash
   git add .
   git commit -m "chore: release v0.1.0"
   git tag v0.1.0
   git push origin main --tags
   ```

4. **Monitor CI**:
   - GitHub Actions builds binaries
   - Release draft created automatically

5. **Publish Release**:
   - Review release draft
   - Edit release notes
   - Publish release

6. **Publish to crates.io** (if desired):
   ```bash
   cd rust/crates/fusabi-vm
   cargo publish

   cd ../fusabi-frontend
   cargo publish
   ```

### Automated Release

Use workflow dispatch:

```bash
# Trigger via GitHub UI:
# Actions -> Release -> Run workflow -> Enter version
```

Or use GitHub CLI:

```bash
gh workflow run release.yml -f version=v0.1.0
```

---

## Running CI Locally

### Full CI Simulation

```bash
# Run all CI checks locally
just ci

# Equivalent to:
just check          # Format + lint + test
just build-release  # Release build
```

### Individual CI Steps

```bash
# Quick check (like CI quick check job)
just fmt-check
just lint
just check-compile

# Test suite (like CI test job)
just test
just test-unit
just test-integration

# Coverage (like CI coverage job)
just test-coverage

# Documentation (like CI docs job)
just docs

# Benchmarks (like CI benchmark job)
just bench --no-run
```

### Platform-specific Testing

```bash
# Test on current platform
cargo test --workspace

# Test specific platform features
cargo test --workspace --features linux
cargo test --workspace --features macos
cargo test --workspace --features windows
```

### Matrix Testing Locally

Use Docker for multi-platform testing:

```bash
# Ubuntu
docker run --rm -v $(pwd):/workspace -w /workspace rust:latest \
  cargo test --workspace

# Alpine (musl)
docker run --rm -v $(pwd):/workspace -w /workspace rust:alpine \
  cargo test --workspace
```

---

## Workflow Optimization

### Caching Strategy

All workflows use `Swatinem/rust-cache`:

```yaml
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: rust
    key: ${{ matrix.os }}-${{ matrix.rust }}
```

Benefits:
- ~5-10x faster builds
- Shared across branches
- Automatic cache eviction

### Parallel Execution

Jobs run in parallel where possible:

```
Quick Check (2-3 min)
    ├─> Test Suite (5-10 min)
    ├─> Build Release (3-5 min)
    ├─> Coverage (5-7 min)
    ├─> Security (1-2 min)
    ├─> Docs (2-3 min)
    └─> Benchmarks (2-3 min)
```

Total wall-clock time: ~10-15 min (vs ~30 min sequential)

### Fail-Fast Strategy

- Quick check runs first (fast feedback)
- Fail-fast disabled in matrix (see all failures)
- `continue-on-error` for optional checks

---

## Secrets and Configuration

### Required Secrets

None required for basic CI.

### Optional Secrets

#### `CARGO_TOKEN` (for crates.io publishing)

1. Generate token: https://crates.io/settings/tokens
2. Add to repository secrets:
   ```
   Settings -> Secrets -> Actions -> New repository secret
   Name: CARGO_TOKEN
   Value: <your-token>
   ```

#### `CODECOV_TOKEN` (for coverage reporting)

1. Sign up at https://codecov.io
2. Add repository
3. Copy token
4. Add to repository secrets:
   ```
   Name: CODECOV_TOKEN
   Value: <your-token>
   ```

**Note**: Codecov works without token for public repos.

---

## Monitoring and Alerts

### Build Status

- **Badge**: Add to README.md:
  ```markdown
  ![CI](https://github.com/fusabi-lang/fusabi/workflows/CI/badge.svg)
  ```

- **Dashboard**: https://github.com/fusabi-lang/fusabi/actions

### Failure Notifications

Configure in GitHub repository settings:
```
Settings -> Notifications -> Actions
```

Options:
- Email on workflow failure
- Slack/Discord webhooks
- GitHub mobile app notifications

### Coverage Tracking

View coverage trends at:
```
https://codecov.io/gh/fusabi-lang/fusabi
```

---

## Troubleshooting

### Common Issues

#### 1. Cache Issues

**Problem**: CI uses stale cached dependencies

**Solution**:
```bash
# Clear cache manually
# Go to: Actions -> Caches -> Delete specific cache

# Or via API:
gh cache delete <cache-id>
```

#### 2. Timeout Issues

**Problem**: Job exceeds 6-hour limit

**Solution**:
```yaml
# Add timeout to job
jobs:
  test:
    timeout-minutes: 30  # Default is 360
```

#### 3. Platform-specific Failures

**Problem**: Tests pass locally but fail on CI

**Solution**:
```bash
# Use GitHub Actions with SSH debugging
# Add to workflow:
- name: Setup tmate session
  uses: mxschmitt/action-tmate@v3
```

#### 4. Dependency Resolution

**Problem**: Cargo fails to resolve dependencies

**Solution**:
```bash
# Update Cargo.lock
cd rust
cargo update

# Commit updated lock file
git add Cargo.lock
git commit -m "chore: update dependencies"
```

#### 5. Flaky Tests

**Problem**: Tests intermittently fail

**Solution**:
```rust
// Add retry logic
#[test]
#[ignore]  // Mark as ignored until fixed
fn test_flaky() {
    // Fix or isolate flaky test
}
```

### CI Performance

#### Slow Builds

Optimization strategies:

1. **Reduce test scope**:
   ```bash
   # Unit tests in quick check
   cargo test --lib

   # Full tests in separate job
   cargo test --workspace
   ```

2. **Parallel compilation**:
   ```yaml
   env:
     CARGO_BUILD_JOBS: 4
   ```

3. **Incremental builds**:
   ```yaml
   env:
     CARGO_INCREMENTAL: 1
   ```

4. **sccache** (future):
   ```yaml
   - uses: mozilla-actions/sccache-action@v0.0.3
   ```

---

## Best Practices

### 1. Keep CI Fast

- Quick check <3 minutes
- Full pipeline <20 minutes
- Use matrix for parallelization

### 2. Test Locally First

```bash
# Before pushing
just pre-commit

# Before PR
just ci
```

### 3. Clear Commit Messages

Follow conventional commits:
```
feat: add VM interpreter loop
fix: resolve parser bug with let-bindings
docs: update testing guide
chore: update dependencies
```

### 4. Small, Focused PRs

- <500 lines changed
- Single responsibility
- Include tests
- Update docs

### 5. Monitor CI Trends

- Track build times
- Monitor flaky tests
- Review coverage trends
- Check dependency health

---

## Future Enhancements

### Planned Improvements

1. **Performance Benchmarking**:
   - Automated regression detection
   - Benchmark history tracking
   - Performance budget enforcement

2. **Advanced Testing**:
   - Property-based testing (proptest)
   - Fuzzing (cargo-fuzz)
   - Mutation testing (cargo-mutants)

3. **Security**:
   - SAST scanning (cargo-geiger)
   - Dependency review action
   - Secret scanning

4. **Deploy Automation**:
   - Auto-publish to crates.io
   - Docker image builds
   - Documentation deployment

5. **Monitoring**:
   - Build time tracking
   - Test reliability metrics
   - Coverage trends

---

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://matklad.github.io/2021/09/04/fast-rust-builds.html)
- [Cargo Book - Testing](https://doc.rust-lang.org/cargo/guide/tests.html)
- [pre-commit Documentation](https://pre-commit.com/)

---

## Quick Reference

### CI Commands

```bash
# Local CI simulation
just ci                # Full CI checks
just check             # Quick checks (fmt + lint + test)
just pre-commit        # Pre-commit hook checks

# Individual checks
just fmt-check         # Format check
just lint              # Clippy
just test              # All tests
just build-release     # Release build
```

### Workflow Files

```
.github/workflows/
├── ci.yml           # Main CI pipeline
├── pr-checks.yml    # PR-specific checks
└── release.yml      # Release automation

.githooks/
├── pre-commit       # Local pre-commit hook
└── pre-push         # Local pre-push hook

.pre-commit-config.yaml  # pre-commit framework config
```

### Troubleshooting

```bash
# View workflow runs
gh run list

# View specific run
gh run view <run-id>

# Rerun failed jobs
gh run rerun <run-id>

# View logs
gh run view <run-id> --log
```

---

**Status**: CI/CD infrastructure ready for Phase 1 development

**Next Steps**:
1. Monitor initial CI runs
2. Optimize slow jobs
3. Add coverage tracking
4. Configure Codecov integration

For testing details, see [testing.md](testing.md).
