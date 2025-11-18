#!/usr/bin/env nu

# Setup Git Hooks for FSRS
# This script installs pre-commit and pre-push hooks

print "🔧 Setting up Git hooks for FSRS..."

# Check if we're in the right directory
if not (".git" | path exists) {
    print "❌ Not in a Git repository root"
    print "Run this script from the repository root"
    exit 1
}

# Ensure .githooks directory exists
if not (".githooks" | path exists) {
    print "❌ .githooks directory not found"
    exit 1
}

# Create symlinks for hooks
let hooks = ["pre-commit", "pre-push"]

for hook in $hooks {
    let source = $".githooks/($hook)"
    let target = $".git/hooks/($hook)"

    # Remove existing hook if it's a file (not a symlink)
    if ($target | path exists) and (not ($target | path type | str contains "symlink")) {
        print $"⚠️  Removing existing ($hook) hook"
        rm $target
    }

    # Create symlink
    if not ($target | path exists) {
        print $"✓ Installing ($hook) hook"
        ln -s $"../../($source)" $target
        chmod +x $source
    } else {
        print $"ℹ️  ($hook) hook already installed"
    }
}

print ""
print "✅ Git hooks installed successfully!"
print ""
print "Installed hooks:"
print "  • pre-commit: Fast checks before each commit"
print "  • pre-push: Comprehensive checks before push"
print ""
print "To bypass hooks (not recommended):"
print "  git commit --no-verify"
print "  git push --no-verify"
print ""
print "To test hooks manually:"
print "  .githooks/pre-commit"
print "  .githooks/pre-push"
