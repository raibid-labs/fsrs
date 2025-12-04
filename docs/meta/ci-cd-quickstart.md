# CI/CD Quick Start Guide

**TL;DR**: CI/CD infrastructure is ready. Install hooks, push to GitHub, start developing.

---

## 30-Second Setup

```bash
# 1. Install git hooks
nu scripts/setup-hooks.nu

# 2. Verify everything works
just check

# 3. Push to GitHub (activates CI)
git add .
git commit -m "ci: add CI/CD infrastructure"
git push
```

**Done!** CI/CD is now active.

---

## What You Get

### Automatic Testing

- ✅ **On every commit**: Format, lint, unit tests (local)
- ✅ **On every push**: Full test suite (local)
- ✅ **On every PR**: Multi-platform tests (GitHub Actions)
- ✅ **On every merge**: Release builds, coverage, security audits

### Multi-Platform CI

Tests run automatically on:
- Ubuntu Linux
- macOS
- Windows
- Rust stable + beta

### Code Quality

- Format checking (rustfmt)
- Linting (clippy)
- Security audits (cargo-audit)
- Code coverage (tarpaulin + Codecov)

---

## Daily Workflow

### Development

```bash
# Start watch mode (auto-test on save)
just watch-test

# Make changes...
# Tests run automatically

# Before commit
just check  # Optional - hooks run anyway
```

### Committing

```bash
git add .
git commit -m "feat: my feature"
# Hooks run automatically (~30s)
# ✓ Format ✓ Lint ✓ Tests
```

### Pushing

```bash
git push
# Hooks run automatically (~2-3 min)
# ✓ Release build ✓ All tests
# GitHub Actions starts running
```

---

## Common Commands

```bash
# Testing
just test              # All tests
just test-unit         # Unit tests only
just test-coverage     # Generate coverage

# Quality
just check             # Format + lint + test
just fmt               # Format code
just lint              # Run clippy

# CI Simulation
just ci                # Run full CI locally

# Development
just watch-test        # Watch and test
just dev               # Watch, test, and run
```

---

## Troubleshooting

### Hooks not running?

```bash
nu scripts/setup-hooks.nu
```

### Tests failing?

```bash
just test -- --nocapture  # See test output
```

### Need to bypass hooks? (Emergency only)

```bash
git commit --no-verify
git push --no-verify
```

---

## Documentation

Full documentation in `docs/`:

- **testing.md** - Complete testing guide
- **ci-cd.md** - CI/CD details
- **development.md** - Development workflows
- **setup-ci.md** - Detailed setup
- **ci-cd-summary.md** - What was created

---

## Quick Reference

```bash
just                   # List all commands
just check             # All quality checks
just test              # Run tests
just watch-test        # Watch mode
just ci                # Simulate CI
nu scripts/setup-hooks.nu  # Install hooks
```

---

## Status

✅ **All systems operational**

- CI/CD workflows: Ready
- Git hooks: Installed
- Test infrastructure: Complete
- Documentation: Comprehensive

**Ready for Phase 1 development!**

---

## Support

Questions? Check:
1. `docs/` directory for detailed guides
2. `just --list` for available commands
3. GitHub Actions tab for CI status
4. Open an issue on GitHub

---

**Next Step**: Start developing! `just watch-test`
