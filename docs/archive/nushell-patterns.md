# Nushell Scripting Patterns - Fusabi Project

This document outlines common patterns, conventions, and best practices observed in the Fusabi project.

## Overview

Nushell is used as the primary automation scripting language in Fusabi development, integrated with Justfile for high-level task orchestration. Scripts handle complex workflows while Just acts as the command interface.

## Core Integration Pattern

### Just + Nushell Relationship

```
justfile (user interface)
    ↓
Nushell scripts (implementation)
    ↓
Cargo/system commands (execution)
```

**Key Rules:**
- Justfile contains high-level recipe definitions with `@` for silent execution
- Nushell scripts (`.nu` files) contain all complex logic
- Scripts should be idempotent and composable
- Example from raibid-cli justfile:
  ```makefile
  discover-repos:
      nu scripts/discover-repos.nu
  
  validate:
      nu scripts/validate.nu
  ```

## Script Structure

### Shebang and Header

All scripts must start with:
```nushell
#!/usr/bin/env nu

# Script purpose and description
# Additional context about what it does
```

**Pattern Examples:**
- `discover-repos.nu` - "Discover and catalog repositories in the raibid-labs GitHub organization"
- `collect.nu` - "Sparky data collection script"
- `pipeline.nu` - "Sparky Unified Pipeline (Nushell)"

### Function Definitions

Primary pattern: `def main [args] { ... }`

```nushell
def main [
    --date: string,           # Date parameter with description
    --mode: string = "daily"  # Optional param with default
    --verbose                 # Boolean flags (no colon needed)
] {
    # Implementation
}

# Helper functions (no params shown to user)
def helper_function [param: string] {
    # Private implementation
}

# IMPORTANT: Must call main at script end
main
```

**Key Patterns:**
- Parameter documentation via inline comments
- Type hints for all parameters
- Default values for optional parameters
- Call `main` at the end (Nushell requirement)

### Main Function Signature Pattern

From `collect.nu`:
```nushell
def main [
    --date: string,           # Date to collect (YYYY-MM-DD)
    --mode: string = "daily"  # Collection mode: daily, weekly, monthly
] {
    # Validation and defaults
    let collection_date = if ($date | is-empty) {
        date now | format date "%Y-%m-%d"
    } else {
        $date
    }
    
    # Rest of implementation
}
```

## Error Handling

### Pattern 1: Precondition Checks

Exit early with clear error messages:

```nushell
# Check if external tool exists
if (which gh | is-empty) {
    print "❌ GitHub CLI (gh) not found. Please install it first."
    print "   brew install gh     # macOS"
    print "   apt install gh      # Ubuntu/Debian"
    exit 1
}

# Check if directory/file exists
let ns_exists = (kubectl get namespace $namespace 2>/dev/null | complete)
if $ns_exists.exit_code != 0 {
    print $"❌ Namespace '($namespace)' does not exist"
    print "   Run: just deploy-local"
    return
}
```

**Key Points:**
- Use `exit 1` for script-ending errors
- Use `return` for non-fatal early exits
- Always provide actionable error messages and remediation steps

### Pattern 2: Try-Catch for External Commands

```nushell
try {
    let commits = (
        gh api $"repos/raibid-labs/($repo.name)/commits" 
        | from json
    )
    $all_commits = ($all_commits | append $commits)
} catch {
    # Skip repos with no commits or API errors
}
```

**Use When:**
- Calling external tools (gh, kubectl, etc.)
- Parsing potentially invalid JSON/data
- Operations that might fail per-item in a loop

### Pattern 3: Command Exit Code Check

```nushell
let result = (command 2>/dev/null | complete)

if $result.exit_code == 0 {
    # Success path
} else {
    # Failure path
}
```

**Use When:**
- You need to capture output AND check exit status
- `complete` converts exit code to record with `.exit_code`

## Output and Logging

### Status Indicators with Emoji

Standard emoji conventions used in Fusabi:

```nushell
print "🚀 Starting operation..."        # Launch/begin
print "✅ Operation complete!"          # Success
print "❌ Error occurred"               # Critical error
print "⚠️  Warning - check this"        # Warning/caution
print "📊 Data/statistics"              # Data/metrics
print "📝 Writing/creating"             # File operations
print "🔀 Pull requests/merging"        # Git operations
print "💰 Cost information"             # Financial/resource info
print "🧠 Intelligence/AI systems"      # AI/ML operations
print "🎯 Target/focus point"           # Key operations
print "🔌 Connections/services"         # Network/services
print "📦 Packages/modules"             # Software packages
print "📋 Lists/issues"                 # Collections
```

### Formatted Output Pattern

```nushell
print "=== Validating raibid-cli project structure ==="
print ""

# Building results table
let checks = [
    (check_file "Cargo.toml" "Workspace manifest"),
    (check_file "justfile" "Build automation"),
]

let passed = ($checks | where status == "✅" | length)
let failed = ($checks | where status == "❌" | length)

print "\n=== Validation Summary ==="
$checks | table          # Display as table

print $"\nPassed: ($passed), Failed: ($failed)"

if $failed > 0 {
    exit 1
}
```

## Configuration Management

### Pattern 1: Parameters with Defaults

```nushell
def main [
    --org: string = "raibid-labs"      # GitHub organization name
    --output: string = "repos.json"    # Output JSON file
    --verbose                          # Enable verbose logging
] {
    # Use parameters directly
    print $"Discovering repositories in ($org)..."
}
```

### Pattern 2: Environment-based Selection

```nushell
def main [
    --env: string = "local"  # Environment: local, production
] {
    let namespace = "sparky"
    
    match $env {
        "local" => { /* local logic */ }
        "production" => { /* prod logic */ }
        _ => { exit 1 }
    }
}
```

### Pattern 3: File-based Configuration

```nushell
let config_dir = "config"
let config_file = $"($config_dir)/($env).toml"

if not ($config_file | path exists) {
    print $"❌ Config file not found: ($config_file)"
    exit 1
}
```

## Data Handling

### Pattern 1: JSON Pipeline

```nushell
# Fetch, transform, save
gh api $"/orgs/($org)/repos" --paginate
| from json
| each { |repo|
    {
        name: $repo.name,
        full_name: $repo.full_name,
        description: $repo.description,
        # ... more fields
    }
}
| to json 
| save --force $output
```

### Pattern 2: Mutation Pattern

```nushell
mut all_commits = []

for repo in $repos {
    let commits = (fetch_commits $repo)
    $all_commits = ($all_commits | append $commits)
}

$all_commits
```

### Pattern 3: Conditional Data Filtering

```nushell
let cutoff_date = match $mode {
    "daily" => (date now | format date "%Y-%m-%dT00:00:00Z"),
    "weekly" => (/* week calculation */),
    "monthly" => (/* month calculation */),
    _ => (date now | format date "%Y-%m-%dT%H:%M:%SZ")
}

let active_repos = ($repos | where pushedAt >= $cutoff_date)
```

## Helper Function Patterns

### Validation Functions

```nushell
def check_file [path: string, description: string] {
    if ($path | path exists) {
        {status: "✅", check: $description, path: $path}
    } else {
        {status: "❌", check: $description, path: $path}
    }
}

def check_directory [path: string, description: string] {
    if ($path | path exists) and (ls $path | length) > 0 {
        {status: "✅", check: $description, path: $path}
    } else {
        {status: "❌", check: $description, path: $path}
    }
}
```

### Data Collection Helpers

```nushell
def collect_commits [repos: list, mode: string] {
    let since = match $mode {
        "daily" => "24 hours ago",
        "weekly" => "7 days ago",
        "monthly" => "30 days ago",
        _ => "24 hours ago"
    }

    mut all_commits = []

    for repo in $repos {
        try {
            let commits = (gh api $"repos/raibid-labs/($repo.name)/commits" 
                          | from json)
            $all_commits = ($all_commits | append $commits)
        } catch {
            # Skip on error
        }
    }

    $all_commits
}
```

## Control Flow Patterns

### Match Pattern (Multiway Conditional)

```nushell
match $mode {
    "daily" => {
        print "🚀 Running Daily Pipeline"
        nu scripts/collect-daily.nu
    }
    "weekly" => {
        print "🚀 Running Weekly Pipeline"
        nu scripts/collect-weekly.nu
    }
    _ => {
        print $"❌ Invalid mode: ($mode)"
        exit 1
    }
}
```

### For Loop Pattern

```nushell
for repo in $repos {
    try {
        let commits = (gh api $"repos/raibid-labs/($repo.name)/commits")
        # Process commits
    } catch {
        # Handle error and continue
    }
}
```

## Command Execution Patterns

### Pattern 1: Cargo Commands

```nushell
mut command = ["cargo", "run", "--bin", "raibid", "--", "sync", "--all"]

if $dry_run {
    $command = ($command | append "--dry-run")
}

print $"Running: ($command | str join ' ')"
^$command.0 ...$command.1..
```

**Breaking Down:**
- `mut` creates a mutable variable
- Array operations with `append`
- `^` invokes external command (caret prefix)
- `...$command.1..` unpacks array elements as args

### Pattern 2: Shell Scripts within Nushell

From raibid-ci justfile:
```nushell
test-e2e:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running E2E tests..."
    TEST_EXTERNAL=1 cargo test --test ci_pipeline_test
```

Use `#!/usr/bin/env bash` shebang within recipe for complex bash logic.

## File and Directory Operations

### Path Handling

```nushell
# Create directory if not exists
mkdir $output_dir

# Check if path exists
if ($path | path exists) {
    # ...
}

# List directory
ls $raw_dir | sort-by modified -r | first 5

# Humanize dates
print $"  ($file.name) - ($file.modified | date humanize)"
```

### Saving Data

```nushell
# Save with force overwrite
$repos | to json | save --force $output

# Formatted save
$summary | to json | save -f $"($output_dir)/($collection_date)-summary.json"
```

## Testing and Validation Pattern

```nushell
def check_rust_files [] {
    let rust_files = (ls **/*.rs | length)
    if $rust_files > 0 {
        {status: "✅", check: $"Rust source files \(($rust_files)\)", path: "**/*.rs"}
    } else {
        {status: "❌", check: "Rust source files", path: "**/*.rs"}
    }
}

def check_docs_structure [] {
    let required_docs = [
        "docs/architecture.md",
        "docs/roadmap.md",
    ]

    let missing = ($required_docs | where { |doc| not ($doc | path exists) })

    if ($missing | length) == 0 {
        {status: "✅", check: "Required documentation files", path: "docs/"}
    } else {
        {status: "❌", check: $"Missing docs: ($missing | str join ', ')", path: "docs/"}
    }
}
```

## Common Pitfalls to Avoid

1. **Forgetting `main` call**: Scripts must end with `main` or `main <args>`
2. **Not handling empty results**: Use optional operators `|.0?` or `is-empty`
3. **Bare `exit` vs `exit 1`**: Use proper exit codes (0 for success, 1 for error)
4. **String interpolation confusion**: Use `$"text ($var) more"` format
5. **Missing type hints**: Always specify parameter types
6. **Not checking external tool availability**: Use `which tool | is-empty` check

## Templates for Fusabi Scripts

### build.nu Template

```nushell
#!/usr/bin/env nu

# Build script for Fusabi project
# Orchestrates compilation of F# scripts and Rust host

def main [
    --release                          # Build in release mode
    --verbose                          # Show detailed output
] {
    print "🚀 FSRS Build"
    print "==============="
    print ""

    # Check prerequisites
    if (which cargo | is-empty) {
        print "❌ Cargo not found. Please install Rust."
        exit 1
    }

    # Build configuration
    mut cargo_args = ["build"]
    if $release {
        $cargo_args = ($cargo_args | append "--release")
    }

    print "Compiling Rust host..."
    cargo ...$cargo_args

    print ""
    print "✅ Build complete!"
}

main
```

### test.nu Template

```nushell
#!/usr/bin/env nu

# Test script for Fusabi project

def main [
    --unit                             # Run unit tests only
    --integration                      # Run integration tests only
    --coverage                         # Generate coverage report
] {
    print "🧪 FSRS Tests"
    print "==============="
    print ""

    if (which cargo | is-empty) {
        print "❌ Cargo not found"
        exit 1
    }

    if $unit or (not $integration) {
        print "Running unit tests..."
        cargo test --lib
    }

    if $integration or (not $unit) {
        print "Running integration tests..."
        cargo test --test '*'
    }

    if $coverage {
        print "Generating coverage..."
        cargo tarpaulin --out Html --output-dir coverage
    }

    print ""
    print "✅ Tests complete!"
}

main
```

### transpile.nu Template

```nushell
#!/usr/bin/env nu

# Transpile F# scripts to Rust

def main [
    input_file: string                 # F# script to transpile
    --output: string                   # Output file path
    --watch                            # Watch mode
] {
    print $"📝 Transpiling ($input_file)"

    if not ($input_file | path exists) {
        print $"❌ File not found: ($input_file)"
        exit 1
    }

    if (which fable | is-empty) {
        print "❌ Fable not found. Install Fable Rust backend."
        exit 1
    }

    let output_path = if ($output | is-empty) {
        ($input_file | str replace ".fsx" ".rs")
    } else {
        $output
    }

    fable $input_file --lang rust -o $output_path

    print $"✅ Transpiled to ($output_path)"
}

main
```

## Summary Checklist

- [ ] Start with `#!/usr/bin/env nu` and description
- [ ] Use `def main [args] { ... }` with type hints
- [ ] Call `main` at script end
- [ ] Check prerequisites with `which` command
- [ ] Use `exit 1` for fatal errors, `return` for non-fatal
- [ ] Wrap external commands in `try-catch` when appropriate
- [ ] Use emoji indicators for status (✅, ❌, 🚀, etc.)
- [ ] Store results in `let` (immutable) unless mutation needed
- [ ] Use `match` for multiway conditionals
- [ ] Document parameters with inline comments
- [ ] Provide actionable error messages with remediation steps

