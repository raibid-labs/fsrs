# Fusabi Package Management System

## 1. Introduction

### Overview
The Fusabi Package Management system (FPM) is designed to provide developers with a simple, reproducible, and fast way to manage dependencies and share code across the Fusabi ecosystem. This specification outlines the architecture, formats, and workflows for package management in Fusabi.

### Goals
- **Simple**: Minimal configuration, intuitive commands, and clear conventions
- **Reproducible**: Consistent builds across different machines and time periods through lock files
- **Fast**: Efficient dependency resolution, caching, and parallel downloads
- **Secure**: Package verification, checksums, and vulnerability scanning
- **Flexible**: Support for multiple dependency sources (registry, git, path, URL)

### Inspiration
FPM draws inspiration from successful package managers in the ecosystem:
- **Cargo (Rust)**: Manifest format, semantic versioning, and lock file approach
- **npm (Node.js)**: Registry architecture and versioning strategies
- **Deno**: URL-based imports and decentralized package distribution
- **Go Modules**: Version resolution and minimal version selection

---

## 2. Manifest Format: `fusabi.toml`

### Basic Structure
The `fusabi.toml` file is the manifest that describes a Fusabi package and its dependencies. It uses the TOML format for human readability and simplicity.

```toml
[package]
name = "my-app"
version = "0.1.0"
authors = ["Developer <dev@example.com>"]
description = "A sample Fusabi application"
license = "MIT"
repository = "https://github.com/user/my-app"
homepage = "https://my-app.dev"
documentation = "https://docs.my-app.dev"
keywords = ["web", "api", "framework"]
categories = ["web-programming", "api"]
readme = "README.md"
edition = "2025"

[dependencies]
# Simple version constraint
http = "1.0.0"

# Detailed dependency with features
json = { version = "2.0.0", optional = true }

# Local path dependency
local-lib = { path = "../local-lib" }

# Git dependency with specific revision
git-lib = { git = "https://github.com/user/lib", rev = "abc123" }

# Git dependency with branch
async-lib = { git = "https://github.com/user/async", branch = "main" }

# Git dependency with tag
stable-lib = { git = "https://github.com/user/stable", tag = "v1.0.0" }

# Registry dependency with features
database = { version = "3.0.0", features = ["postgres", "mysql"] }

# Alternative registry
private-lib = { version = "1.0.0", registry = "https://private.registry.com" }

[dev-dependencies]
test-framework = "1.0.0"
benchmark = "0.5.0"
mock-server = { version = "2.0.0", features = ["tls"] }

[build-dependencies]
code-generator = "1.0.0"

[features]
default = ["json"]
full = ["json", "xml", "yaml"]
minimal = []
experimental = ["async", "database/experimental"]

# Feature dependencies
async = ["async-lib"]
xml = ["xml-parser"]
yaml = ["yaml-parser"]

[workspace]
members = ["core", "plugins/*", "examples/advanced"]
exclude = ["archive/*", "legacy"]

[profile.dev]
optimization = 0
debug = true

[profile.release]
optimization = 3
debug = false
strip = true

[profile.test]
optimization = 1
debug = true

[scripts]
preinstall = "echo 'Installing dependencies...'"
postinstall = "fpm run setup"
setup = "fusabi scripts/setup.fsx"
clean = "rm -rf .fusabi/cache"

[metadata]
# Custom metadata for tooling
ci = { provider = "github-actions" }
linter = { strict = true }
```

### Package Section Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Package name (lowercase, alphanumeric, hyphens) |
| `version` | Yes | Semantic version (MAJOR.MINOR.PATCH) |
| `authors` | No | List of package authors |
| `description` | No | Short package description |
| `license` | No | SPDX license identifier |
| `repository` | No | Source code repository URL |
| `homepage` | No | Package homepage URL |
| `documentation` | No | Documentation URL |
| `keywords` | No | Search keywords (max 5) |
| `categories` | No | Package categories |
| `readme` | No | Path to README file |
| `edition` | No | Fusabi edition (defaults to latest) |

### Dependency Specification

#### Version Constraints
```toml
# Exact version
http = "1.0.0"
# or
http = { version = "=1.0.0" }

# Caret (compatible with version)
# ^1.2.3 := >=1.2.3, <2.0.0
http = "^1.2.3"

# Tilde (patch-level changes)
# ~1.2.3 := >=1.2.3, <1.3.0
http = "~1.2.3"

# Wildcards
http = "1.*"  # Any 1.x version
http = "1.2.*"  # Any 1.2.x version

# Comparison operators
http = ">=1.0.0, <2.0.0"
http = ">1.0.0"

# Multiple constraints
http = { version = ">=1.0.0, <2.0.0" }
```

#### Dependency Sources

```toml
[dependencies]
# Registry (default)
registry-pkg = "1.0.0"

# Path (local filesystem)
local-pkg = { path = "../my-lib" }
local-pkg-versioned = { path = "../my-lib", version = "1.0.0" }

# Git repository
git-pkg = { git = "https://github.com/user/repo" }
git-pkg-rev = { git = "https://github.com/user/repo", rev = "abc123" }
git-pkg-tag = { git = "https://github.com/user/repo", tag = "v1.0.0" }
git-pkg-branch = { git = "https://github.com/user/repo", branch = "develop" }

# HTTP URL (single file or tarball)
url-pkg = { url = "https://example.com/package.tar.gz", checksum = "sha256:abc123..." }
```

---

## 3. Dependency Resolution

### Semantic Versioning
Fusabi uses Semantic Versioning (SemVer) 2.0.0:
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

### Version Constraint Syntax

| Syntax | Example | Meaning |
|--------|---------|---------|
| Exact | `=1.2.3` | Exactly version 1.2.3 |
| Caret | `^1.2.3` | >=1.2.3, <2.0.0 |
| Tilde | `~1.2.3` | >=1.2.3, <1.3.0 |
| Greater | `>1.2.3` | Any version greater than 1.2.3 |
| Greater-Equal | `>=1.2.3` | Version 1.2.3 or higher |
| Less | `<2.0.0` | Any version less than 2.0.0 |
| Less-Equal | `<=2.0.0` | Version 2.0.0 or lower |
| Wildcard | `1.2.*` | Any version 1.2.x |
| Multiple | `>=1.2.0, <2.0.0` | Combined constraints |

### Lock File Format: `fusabi.lock`

The lock file ensures reproducible builds by recording exact versions and checksums of all dependencies.

```toml
# This file is automatically generated by FPM
# Do not edit manually
version = 1

[[package]]
name = "http"
version = "1.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:abcdef1234567890..."
dependencies = [
    "io 1.0.0 (registry+https://packages.fusabi.dev/)",
    "strings 2.0.0 (registry+https://packages.fusabi.dev/)",
]

[[package]]
name = "json"
version = "2.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:1234567890abcdef..."
dependencies = []
optional = true

[[package]]
name = "local-lib"
version = "0.1.0"
source = "path+file:///home/user/projects/local-lib"
dependencies = []

[[package]]
name = "git-lib"
version = "0.2.0"
source = "git+https://github.com/user/lib#abc123"
checksum = "git-sha256:abc123..."
dependencies = [
    "http 1.0.0 (registry+https://packages.fusabi.dev/)",
]

[metadata]
# Additional metadata for resolution
resolver-version = "1"
fusabi-version = "0.1.0"
```

### Dependency Resolution Algorithm

FPM uses a **Minimal Version Selection** (MVS) algorithm inspired by Go modules:

1. **Build Dependency Graph**: Parse `fusabi.toml` and collect all direct dependencies
2. **Expand Transitive Dependencies**: Recursively fetch dependency manifests
3. **Version Selection**: For each package, select the minimum version that satisfies all constraints
4. **Conflict Detection**: Identify version conflicts and report them
5. **Feature Resolution**: Enable features based on dependency requirements
6. **Lock File Generation**: Write resolved versions to `fusabi.lock`

#### Resolution Steps

```
1. Start with root package
2. For each dependency:
   a. Determine version constraint
   b. Query available versions from source
   c. Select minimum satisfying version
   d. Add to resolution graph
3. Recursively process dependencies
4. Detect cycles and conflicts
5. Generate lock file
```

#### Conflict Resolution Strategy

When multiple packages require different versions of the same dependency:

1. **Compatible Versions**: If constraints overlap, use the highest minimum version
   - Package A requires `http ^1.2.0` (>=1.2.0, <2.0.0)
   - Package B requires `http ^1.3.0` (>=1.3.0, <2.0.0)
   - Resolution: Use `http 1.3.0` (minimum that satisfies both)

2. **Incompatible Versions**: If constraints don't overlap, report error
   - Package A requires `http ^1.0.0` (>=1.0.0, <2.0.0)
   - Package B requires `http ^2.0.0` (>=2.0.0, <3.0.0)
   - Error: Cannot satisfy both constraints

3. **Override Mechanism**: Allow manual overrides in manifest
   ```toml
   [overrides]
   http = "1.5.0"  # Force specific version
   ```

### Caching Strategy

FPM implements a multi-level cache:

1. **Package Cache**: Downloaded packages stored in `.fusabi/cache/packages/`
2. **Git Cache**: Cloned repositories in `.fusabi/cache/git/`
3. **Registry Index Cache**: Package metadata in `.fusabi/cache/index/`
4. **Build Cache**: Compiled artifacts in `.fusabi/cache/build/`

Cache locations:
- **Linux**: `~/.fusabi/cache/`
- **macOS**: `~/Library/Caches/fusabi/`
- **Windows**: `%LOCALAPPDATA%\fusabi\cache\`

---

## 4. Package Registry

### Registry Architecture

The Fusabi Package Registry is a centralized service for hosting and distributing packages.

#### Registry API Design (REST)

**Base URL**: `https://packages.fusabi.dev/api/v1`

##### Authentication
```http
Authorization: Bearer <token>
```

##### Endpoints

**1. Search Packages**
```http
GET /packages/search?q=<query>&limit=10&offset=0
```
Response:
```json
{
  "packages": [
    {
      "name": "http",
      "version": "1.0.0",
      "description": "HTTP client and server",
      "downloads": 15000,
      "created_at": "2025-01-15T10:30:00Z",
      "updated_at": "2025-02-20T14:20:00Z"
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

**2. Get Package Information**
```http
GET /packages/<name>
```
Response:
```json
{
  "name": "http",
  "description": "HTTP client and server",
  "license": "MIT",
  "repository": "https://github.com/fusabi/http",
  "homepage": "https://http.fusabi.dev",
  "versions": ["1.0.0", "0.9.0", "0.8.0"],
  "latest_version": "1.0.0",
  "downloads": 15000,
  "owners": ["user1", "user2"],
  "keywords": ["http", "web", "client"],
  "categories": ["web-programming"]
}
```

**3. Get Specific Version**
```http
GET /packages/<name>/<version>
```
Response:
```json
{
  "name": "http",
  "version": "1.0.0",
  "description": "HTTP client and server",
  "license": "MIT",
  "dependencies": {
    "io": "^1.0.0",
    "strings": "^2.0.0"
  },
  "dev_dependencies": {},
  "features": {
    "default": ["client"],
    "full": ["client", "server"]
  },
  "checksum": "sha256:abcdef...",
  "tarball_url": "https://packages.fusabi.dev/files/http/1.0.0/http-1.0.0.tar.gz",
  "published_at": "2025-02-20T14:20:00Z"
}
```

**4. Download Package**
```http
GET /files/<name>/<version>/<name>-<version>.tar.gz
```
Returns: Binary tarball with checksum header
```http
X-Checksum: sha256:abcdef...
Content-Type: application/gzip
```

**5. Publish Package** (Authenticated)
```http
POST /packages
Content-Type: application/json
Authorization: Bearer <token>

{
  "name": "my-package",
  "version": "1.0.0",
  "manifest": "<base64-encoded fusabi.toml>",
  "tarball": "<base64-encoded package tarball>",
  "checksum": "sha256:..."
}
```

**6. Yank Version** (Authenticated)
```http
DELETE /packages/<name>/<version>/yank
Authorization: Bearer <token>
```

**7. Unyank Version** (Authenticated)
```http
POST /packages/<name>/<version>/unyank
Authorization: Bearer <token>
```

**8. Get User Info**
```http
GET /users/<username>
```

**9. Get Statistics**
```http
GET /stats
```
Response:
```json
{
  "total_packages": 1500,
  "total_downloads": 5000000,
  "total_versions": 8000,
  "recent_packages": [...]
}
```

### Package Publishing Workflow

1. **Authentication**: Developer logs in to get API token
   ```bash
   fpm login
   ```

2. **Validation**: FPM validates manifest and runs checks
   - Valid `fusabi.toml` format
   - Unique package name and version
   - All dependencies resolvable
   - README and documentation present
   - Tests pass

3. **Packaging**: Create tarball of package contents
   ```
   package.tar.gz
   ├── fusabi.toml
   ├── src/
   ├── lib/
   ├── README.md
   └── LICENSE
   ```

4. **Upload**: POST to registry API with credentials

5. **Verification**: Registry validates and stores package
   - Checksum verification
   - Malware scanning
   - License validation
   - Name conflict check

6. **Publication**: Package becomes available for download

7. **Notification**: Announce via webhooks/notifications

### Authentication & Authorization

#### User Registration
```bash
fpm register
```
- Creates account on registry
- Stores credentials locally in `~/.fusabi/credentials.toml`

#### Login
```bash
fpm login
```
- Authenticates with registry
- Receives JWT token
- Stores token securely

#### Token Scopes
- `publish`: Publish new versions
- `yank`: Yank/unyank versions
- `owner`: Manage package owners
- `admin`: Administrative operations

### Namespacing Strategy

**Flat Namespace**: Initially use flat namespace (like crates.io)
- Package names must be unique globally
- First-come, first-served
- Names: lowercase, alphanumeric, hyphens
- Minimum 2 characters, maximum 64 characters

**Future: Scoped Packages** (similar to npm)
```toml
[dependencies]
"@organization/package" = "1.0.0"
```

**Reserved Names**:
- `fusabi-*`: Reserved for official packages
- `std`, `core`, `stdlib`: Reserved
- Profanity and trademark violations: Blocked

---

## 5. FPM CLI Commands

### Project Management

#### `fpm init`
Initialize a new Fusabi project.

```bash
# Interactive mode
fpm init

# Non-interactive with flags
fpm init --name my-app --version 0.1.0

# Create library
fpm init --lib

# Create binary
fpm init --bin

# With template
fpm init --template web-app
```

Creates:
```
my-app/
├── fusabi.toml
├── src/
│   └── main.fsx
└── README.md
```

#### `fpm new <name>`
Create a new project in a new directory.

```bash
fpm new my-app
fpm new my-lib --lib
```

### Dependency Management

#### `fpm add <package>`
Add a dependency to `fusabi.toml`.

```bash
# Add latest version
fpm add http

# Add specific version
fpm add http@1.0.0

# Add with version constraint
fpm add http@^1.0.0

# Add from git
fpm add git-lib --git https://github.com/user/lib

# Add local path
fpm add local-lib --path ../local-lib

# Add as dev dependency
fpm add test-framework --dev

# Add with features
fpm add database --features postgres,mysql

# Add optional dependency
fpm add json --optional
```

#### `fpm remove <package>`
Remove a dependency from `fusabi.toml`.

```bash
fpm remove http
fpm remove test-framework --dev
```

#### `fpm install`
Install all dependencies from `fusabi.toml`.

```bash
# Install dependencies
fpm install

# Install from lock file (no updates)
fpm install --locked

# Offline mode (use cache only)
fpm install --offline

# Clean install (clear cache first)
fpm install --clean
```

#### `fpm update`
Update dependencies to latest compatible versions.

```bash
# Update all dependencies
fpm update

# Update specific package
fpm update http

# Update and show changes
fpm update --verbose

# Dry run (show what would be updated)
fpm update --dry-run
```

#### `fpm tree`
Display dependency tree.

```bash
# Show full tree
fpm tree

# Show only direct dependencies
fpm tree --depth 1

# Show with versions
fpm tree --versions

# Show duplicates
fpm tree --duplicates
```

### Package Publishing

#### `fpm login`
Authenticate with package registry.

```bash
fpm login
fpm login --registry https://private.registry.com
```

#### `fpm logout`
Remove authentication credentials.

```bash
fpm logout
```

#### `fpm publish`
Publish package to registry.

```bash
# Publish to default registry
fpm publish

# Dry run (validate without publishing)
fpm publish --dry-run

# Allow dirty working directory
fpm publish --allow-dirty

# Publish to specific registry
fpm publish --registry https://private.registry.com
```

#### `fpm yank <version>`
Mark a version as yanked (deprecated but not deleted).

```bash
fpm yank 1.0.0
fpm yank 1.0.0 --undo  # Unyank
```

#### `fpm owner`
Manage package owners.

```bash
# List owners
fpm owner list http

# Add owner
fpm owner add http user@example.com

# Remove owner
fpm owner remove http user@example.com
```

### Package Information

#### `fpm search <query>`
Search for packages in registry.

```bash
fpm search http
fpm search "web framework"
fpm search --limit 20
```

#### `fpm show <package>`
Show package information.

```bash
fpm show http
fpm show http@1.0.0
fpm show http --versions  # List all versions
```

#### `fpm outdated`
List outdated dependencies.

```bash
fpm outdated
fpm outdated --format json
```

### Build & Execution

#### `fpm build`
Build the project.

```bash
# Build in debug mode
fpm build

# Build in release mode
fpm build --release

# Build specific target
fpm build --target wasm

# Verbose output
fpm build --verbose
```

#### `fpm run`
Run the project.

```bash
# Run main executable
fpm run

# Run with arguments
fpm run -- --help

# Run in release mode
fpm run --release

# Run specific binary
fpm run --bin my-binary
```

#### `fpm test`
Run tests.

```bash
# Run all tests
fpm test

# Run specific test
fpm test test_http

# Run tests with filter
fpm test --filter "http.*"

# Run in parallel
fpm test --parallel 4

# Show test output
fpm test --verbose
```

#### `fpm check`
Validate project without building.

```bash
fpm check
fpm check --release
```

#### `fpm clean`
Remove build artifacts.

```bash
fpm clean
fpm clean --cache  # Also clear cache
```

### Utility Commands

#### `fpm doc`
Generate and open documentation.

```bash
fpm doc
fpm doc --open
fpm doc --no-deps  # Skip dependencies
```

#### `fpm version`
Show FPM version.

```bash
fpm version
fpm --version
fpm -V
```

#### `fpm help`
Show help information.

```bash
fpm help
fpm help install
fpm --help
```

---

## 6. Directory Structure

### Project Structure

```
my-project/
├── fusabi.toml              # Package manifest
├── fusabi.lock              # Dependency lock file (auto-generated)
├── README.md                # Project documentation
├── LICENSE                  # License file
├── .gitignore              # Git ignore rules
│
├── src/                     # Source code (library)
│   ├── main.fsx            # Main library file
│   ├── http.fsx            # Module: HTTP functionality
│   └── utils.fsx           # Module: Utilities
│
├── bin/                     # Binary executables
│   ├── server.fsx          # Binary: Server
│   └── client.fsx          # Binary: Client
│
├── lib/                     # Additional library code
│   └── internal.fsx        # Internal utilities
│
├── tests/                   # Test files
│   ├── test_http.fsx       # Tests for HTTP module
│   └── test_utils.fsx      # Tests for utilities
│
├── examples/                # Example code
│   ├── basic.fsx           # Basic usage example
│   └── advanced.fsx        # Advanced usage example
│
├── docs/                    # Documentation
│   ├── api.md              # API documentation
│   └── guide.md            # User guide
│
├── scripts/                 # Build and utility scripts
│   ├── setup.fsx           # Setup script
│   └── deploy.fsx          # Deployment script
│
├── benches/                 # Benchmarks
│   └── performance.fsx     # Performance benchmarks
│
└── .fusabi/                 # FPM working directory (auto-generated)
    ├── cache/              # Cached dependencies
    │   ├── packages/       # Downloaded packages
    │   ├── git/           # Git repositories
    │   └── index/         # Registry index cache
    ├── build/              # Build artifacts
    └── tmp/                # Temporary files
```

### Workspace Structure

For multi-package projects:

```
my-workspace/
├── fusabi.toml              # Workspace manifest
├── fusabi.lock              # Shared lock file
│
├── core/                    # Package: Core library
│   ├── fusabi.toml
│   └── src/
│       └── main.fsx
│
├── plugins/                 # Plugin packages
│   ├── auth/
│   │   ├── fusabi.toml
│   │   └── src/
│   ├── logging/
│   │   ├── fusabi.toml
│   │   └── src/
│   └── metrics/
│       ├── fusabi.toml
│       └── src/
│
├── examples/                # Example packages
│   ├── basic/
│   │   ├── fusabi.toml
│   │   └── src/
│   └── advanced/
│       ├── fusabi.toml
│       └── src/
│
└── .fusabi/                 # Shared cache
```

**Workspace Root Manifest** (`fusabi.toml`):
```toml
[workspace]
members = [
    "core",
    "plugins/*",
    "examples/basic",
    "examples/advanced"
]
exclude = ["archive/*"]

# Shared dependencies for workspace
[workspace.dependencies]
http = "1.0.0"
json = "2.0.0"

# Workspace metadata
[workspace.metadata]
repository = "https://github.com/user/my-workspace"
```

**Member Package** can inherit workspace dependencies:
```toml
[package]
name = "auth-plugin"
version = "0.1.0"

[dependencies]
http = { workspace = true }  # Use workspace version
json = { workspace = true, features = ["validation"] }
```

### Cache Directory Structure

**Global Cache** (`~/.fusabi/cache/`):
```
~/.fusabi/
├── cache/
│   ├── packages/            # Downloaded packages
│   │   ├── http-1.0.0/
│   │   └── json-2.0.0/
│   ├── git/                 # Git repositories
│   │   └── github.com/
│   │       └── user/
│   │           └── repo/
│   ├── index/               # Registry metadata
│   │   └── packages.fusabi.dev/
│   │       └── index.json
│   └── build/               # Build cache
│       └── artifacts/
│
├── config.toml              # Global configuration
├── credentials.toml         # Authentication credentials
└── logs/                    # FPM logs
    └── fpm.log
```

### Package Distribution Format

Packages are distributed as gzipped tarballs (`.tar.gz`):

```
http-1.0.0.tar.gz
└── http-1.0.0/
    ├── fusabi.toml
    ├── README.md
    ├── LICENSE
    └── src/
        └── main.fsx
```

---

## 7. Implementation Phases

### Phase 1: Local Manifest Parsing & Validation
**Timeline**: Weeks 1-2

**Goals**:
- Parse `fusabi.toml` files
- Validate manifest structure
- Handle different dependency specifications
- Basic error reporting

**Deliverables**:
- TOML parser for `fusabi.toml`
- Manifest data structures
- Validation logic
- Unit tests for parsing

**Components**:
```
fpm/
├── manifest/
│   ├── parser.fs           # TOML parsing
│   ├── validator.fs        # Manifest validation
│   └── types.fs            # Data structures
└── tests/
    └── manifest_tests.fs
```

**Success Criteria**:
- Successfully parse valid manifests
- Detect and report invalid manifests
- Handle all dependency source types
- 100% test coverage for parser

---

### Phase 2: Dependency Resolution Algorithm
**Timeline**: Weeks 3-5

**Goals**:
- Implement version resolution algorithm
- Generate lock files
- Handle version constraints
- Detect and report conflicts

**Deliverables**:
- Dependency graph builder
- Version resolution algorithm (MVS)
- Lock file generator
- Conflict detection

**Components**:
```
fpm/
├── resolver/
│   ├── graph.fs            # Dependency graph
│   ├── algorithm.fs        # Resolution algorithm
│   ├── version.fs          # Version handling
│   └── lockfile.fs         # Lock file generation
├── cache/
│   └── local.fs            # Local caching
└── tests/
    └── resolver_tests.fs
```

**Success Criteria**:
- Resolve simple dependency chains
- Resolve transitive dependencies
- Generate valid lock files
- Detect version conflicts
- Handle circular dependencies

---

### Phase 3: Registry Integration
**Timeline**: Weeks 6-9

**Goals**:
- Implement registry client
- Package download and verification
- Package publishing
- User authentication

**Deliverables**:
- Registry API client
- Package downloader
- Checksum verification
- Publishing workflow
- Authentication system

**Components**:
```
fpm/
├── registry/
│   ├── client.fs           # HTTP client for registry
│   ├── api.fs              # API endpoints
│   ├── auth.fs             # Authentication
│   └── verify.fs           # Checksum verification
├── publish/
│   ├── prepare.fs          # Package preparation
│   ├── tarball.fs          # Tarball creation
│   └── upload.fs           # Upload to registry
└── tests/
    └── registry_tests.fs
```

**Registry Backend** (separate project):
```
registry/
├── api/
│   ├── packages.rs         # Package endpoints
│   ├── users.rs            # User management
│   └── auth.rs             # Authentication
├── storage/
│   ├── database.rs         # PostgreSQL
│   └── files.rs            # Object storage
└── workers/
    └── indexer.rs          # Index maintenance
```

**Success Criteria**:
- Download packages from registry
- Verify package checksums
- Publish packages to registry
- User authentication working
- Basic registry UI functional

---

### Phase 4: CLI Tool & User Experience
**Timeline**: Weeks 10-12

**Goals**:
- Complete CLI implementation
- User-friendly commands
- Helpful error messages
- Documentation generation

**Deliverables**:
- Full CLI with all commands
- Progress indicators
- Colored output
- Comprehensive help text
- User documentation

**Components**:
```
fpm/
├── cli/
│   ├── main.fs             # CLI entry point
│   ├── commands/
│   │   ├── init.fs
│   │   ├── add.fs
│   │   ├── install.fs
│   │   ├── publish.fs
│   │   └── ...
│   ├── output.fs           # Formatted output
│   └── progress.fs         # Progress indicators
├── docs/
│   └── generator.fs        # Doc generation
└── tests/
    └── cli_tests.fs
```

**Success Criteria**:
- All documented commands implemented
- User-friendly error messages
- Fast command execution
- Comprehensive documentation
- Examples and tutorials

---

### Phase 5: Advanced Features (Future)
**Timeline**: Ongoing

**Features**:
- Workspace support
- Build caching
- Parallel downloads
- Mirror support
- Private registries
- Security auditing
- Vulnerability scanning
- Dependency visualization
- Benchmark integration
- Cross-compilation support

---

## 8. Technical Considerations

### Performance Optimization

1. **Parallel Downloads**: Download multiple packages concurrently
2. **Incremental Resolution**: Cache resolution results
3. **Sparse Index**: Download only required package metadata
4. **Compression**: Use efficient compression for tarballs
5. **HTTP/2**: Use HTTP/2 for multiplexed connections

### Security

1. **Checksum Verification**: Verify SHA-256 checksums for all packages
2. **HTTPS Only**: Require HTTPS for registry communication
3. **Token Security**: Store tokens securely in OS keychain
4. **Malware Scanning**: Scan packages for malware before publishing
5. **Vulnerability Database**: Maintain database of known vulnerabilities
6. **Code Signing**: Optional package signing with GPG

### Reliability

1. **Retry Logic**: Retry failed downloads with exponential backoff
2. **Mirror Support**: Fallback to mirrors if primary registry fails
3. **Offline Mode**: Work offline using cached packages
4. **Integrity Checks**: Validate lock file integrity
5. **Atomic Operations**: Ensure atomic package installation

### Compatibility

1. **Version Migration**: Handle lock file format changes
2. **Backwards Compatibility**: Maintain API compatibility
3. **Registry Versioning**: Support multiple registry API versions
4. **Cross-Platform**: Support Linux, macOS, Windows

---

## 9. Configuration

### Global Configuration (`~/.fusabi/config.toml`)

```toml
[registry]
default = "https://packages.fusabi.dev"
token = "stored-in-keychain"

[registries]
"private-company" = "https://private.company.com/registry"

[http]
timeout = 30
retries = 3
user_agent = "fpm/0.1.0"

[cache]
directory = "~/.fusabi/cache"
max_size = "10GB"
ttl = 604800  # 7 days in seconds

[build]
jobs = 4  # Parallel build jobs
target_dir = ".fusabi/build"

[net]
offline = false
git_fetch_with_cli = false

[term]
color = "auto"  # auto, always, never
progress = "auto"
unicode = true
```

### Project Configuration (`.fusabi/config.toml`)

```toml
[build]
target = "native"
profile = "dev"

[alias]
b = "build"
r = "run"
t = "test"
fmt = "format"
```

---

## 10. Migration & Ecosystem

### Migration from Other Package Managers

Provide tools to migrate from other ecosystems:

```bash
# Import from package.json
fpm import package.json

# Import from Cargo.toml
fpm import Cargo.toml

# Import from requirements.txt
fpm import requirements.txt
```

### Integration with Build Tools

Support integration with common build systems:

- **Make**: Generate Makefile targets
- **CMake**: Provide CMake modules
- **Bazel**: Generate BUILD files
- **Nix**: Provide Nix derivations

### CI/CD Integration

Provide examples and documentation for:

- GitHub Actions
- GitLab CI
- Jenkins
- CircleCI
- Azure Pipelines

Example GitHub Actions:
```yaml
- name: Install FPM
  uses: fusabi/setup-fpm@v1
  with:
    version: latest

- name: Install dependencies
  run: fpm install --locked

- name: Run tests
  run: fpm test
```

---

## 11. Future Enhancements

### Short Term (3-6 months)
- Workspace support for monorepos
- Build caching for faster rebuilds
- Private registry authentication
- Basic security auditing

### Medium Term (6-12 months)
- Dependency visualization tools
- Automated vulnerability scanning
- Package signing and verification
- Mirror/proxy support
- Alternative registries

### Long Term (1+ years)
- Distributed package registry
- IPFS/content-addressed storage
- Machine learning for dependency recommendations
- Automated dependency updates
- Supply chain security features

---

## 12. References & Resources

### Standards
- [Semantic Versioning 2.0.0](https://semver.org/)
- [TOML v1.0.0](https://toml.io/en/v1.0.0)
- [SPDX License List](https://spdx.org/licenses/)

### Inspiration
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [npm Documentation](https://docs.npmjs.com/)
- [Go Modules Reference](https://go.dev/ref/mod)
- [Deno Manual](https://deno.land/manual)

### Tools
- [crates.io](https://crates.io/) - Rust package registry
- [npmjs.com](https://www.npmjs.com/) - JavaScript package registry
- [pkg.go.dev](https://pkg.go.dev/) - Go package discovery

---

## Appendix A: Package Manifest Example

Complete example of a real-world package:

```toml
[package]
name = "web-framework"
version = "1.0.0"
authors = ["Jane Developer <jane@example.com>", "Team <team@example.com>"]
description = "A lightweight web framework for Fusabi"
license = "MIT OR Apache-2.0"
repository = "https://github.com/fusabi/web-framework"
homepage = "https://web.fusabi.dev"
documentation = "https://docs.web.fusabi.dev"
keywords = ["web", "http", "framework", "server"]
categories = ["web-programming"]
readme = "README.md"
edition = "2025"

[dependencies]
http = "1.0.0"
router = { version = "2.0.0", features = ["async"] }
json = { version = "3.0.0", optional = true }
templates = { version = "1.5.0", optional = true }
database = { version = "4.0.0", optional = true }
session = { git = "https://github.com/fusabi/session", tag = "v1.0.0" }

[dev-dependencies]
test-framework = "1.0.0"
mock-server = "2.0.0"

[build-dependencies]
code-generator = "1.0.0"

[features]
default = ["json"]
full = ["json", "templates", "database", "session-store"]
json = ["dep:json"]
templates = ["dep:templates"]
database = ["dep:database", "database/postgres"]
session-store = ["session/redis"]

[profile.dev]
optimization = 0
debug = true

[profile.release]
optimization = 3
debug = false
strip = true

[profile.test]
optimization = 1
debug = true

[scripts]
setup = "fusabi scripts/setup.fsx"
migrate = "fusabi scripts/migrate.fsx"
```

---

## Appendix B: Lock File Example

```toml
# This file is automatically generated by FPM
# Do not edit manually
version = 1

[[package]]
name = "database"
version = "4.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:abc123def456..."
dependencies = [
    "io 1.0.0 (registry+https://packages.fusabi.dev/)",
    "postgres-driver 2.0.0 (registry+https://packages.fusabi.dev/)",
]
features = ["postgres"]

[[package]]
name = "http"
version = "1.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:def456ghi789..."
dependencies = [
    "io 1.0.0 (registry+https://packages.fusabi.dev/)",
]

[[package]]
name = "io"
version = "1.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:ghi789jkl012..."
dependencies = []

[[package]]
name = "json"
version = "3.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:jkl012mno345..."
dependencies = []
optional = true

[[package]]
name = "postgres-driver"
version = "2.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:mno345pqr678..."
dependencies = []

[[package]]
name = "router"
version = "2.0.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:pqr678stu901..."
dependencies = [
    "http 1.0.0 (registry+https://packages.fusabi.dev/)",
]
features = ["async"]

[[package]]
name = "session"
version = "1.0.0"
source = "git+https://github.com/fusabi/session#v1.0.0"
checksum = "git-sha256:stu901vwx234..."
dependencies = [
    "http 1.0.0 (registry+https://packages.fusabi.dev/)",
]

[[package]]
name = "templates"
version = "1.5.0"
source = "registry+https://packages.fusabi.dev/"
checksum = "sha256:vwx234yz5678..."
dependencies = []
optional = true

[metadata]
resolver-version = "1"
fusabi-version = "0.1.0"
locked-at = "2025-12-03T10:30:00Z"
```

---

**Document Version**: 1.0.0
**Last Updated**: 2025-12-03
**Status**: Draft for Review
