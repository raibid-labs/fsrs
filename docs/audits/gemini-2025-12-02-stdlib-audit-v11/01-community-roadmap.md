# Fusabi Community & Ecosystem Roadmap

**Date**: 2025-12-02
**Status**: **Strategic Planning**

## 1. The `fusabi-community` Repository
This repository will serve as the central hub for the Fusabi ecosystem until a decentralized registry is fully mature. It acts as both a **Package Registry Index** and a **Monorepo for Community Packages**.

### Structure Proposal
```text
fusabi-community/
├── README.md           # Index of all packages + Contribution Guide
├── packages/           # Source code for packages hosted here
│   ├── stdlib-ext/     # Extensions to stdlib (experimental)
│   ├── json/           # Pure Fusabi JSON combinators
│   ├── http/           # HTTP client wrappers (using Process/Curl initially?)
│   └── commander/      # The TUI file manager (graduated from examples/)
├── registry/           # Metadata for the package manager
│   ├── index.toml      # Master list of packages (name, version, repo_url)
│   └── ...
└── tools/
    └── porter/         # Future home of the F# -> Fusabi transpiler
```

### Instructions for `fusabi-community` Setup (New Claude Session)
1.  **Initialize Repo**: Create `README.md` and folders `packages/`, `registry/`.
2.  **Migrate Commander**: Copy `fusabi/examples/commander.fsx` to `fusabi-community/packages/commander/src/main.fsx`. Add a `fusabi.toml`.
3.  **Create `fusabi.toml` Template**: Standardize the package manifest format.

## 2. Package Manager Strategy (`fpm`)
We will use a **Git-based Registry** model (like Cargo's early days or CocoaPods).

*   **Registry**: The `fusabi-community` repo *is* the default registry.
*   **Resolution**: `fpm` will clone `fusabi-community`, read `registry/index.toml`, and find the git URL for a requested package.
*   **Installation**: `fpm install <package>` clones the specific tag/commit of the package into `fusabi_packages/` in the user's project.

## 3. Porting Strategy: "The Omakase Bridge"
We cannot simply "import" F# NuGet packages. We must **port** them.

### The "Manual Port" Process (Phase 1)
1.  **Identify**: Find a pure F# library (e.g., a color processing lib, a math lib).
2.  **Analyze**: Check for `System.*` dependencies.
3.  **Transliterate**:
    *   Copy `.fs` content to `.fsx`.
    *   Replace `namespace` with `module`.
    *   Remove classes/interfaces. Convert to Records/DUs.
    *   Replace `System.Console` with `Console`.
4.  **Publish**: Add to `fusabi-community/packages/`.

### The "Automated Port" Tool (Phase 2)
Write a tool (likely in F#) that parses F# AST and emits Fusabi.
*   **Location**: `fusabi-community/tools/fsharp-to-fusabi/`
*   **Logic**:
    *   `ast.Visit(LetBinding)` -> `emit "let ..."`
    *   `ast.Visit(System.String.Format)` -> `emit "String.format"`

## 4. Next Steps for `fusabi` Repo (This Repo)

### Priority 1: `fpm` Implementation
The scaffolding is there (`rust/crates/fusabi-pm`). Now we need logic.
*   **Manifest Parsing**: Ensure `fusabi.toml` can be read/written.
*   **Git Integration**: Add `git2` crate or use `Process` to run git commands for cloning dependencies.

### Priority 2: Async Support
*   **Compiler**: Implement the desugaring of `async { ... }`.
*   **VM**: Add `Value::Task` or `Value::Async` to represent pending operations.

### Priority 3: Documentation
*   **Website**: The "Pull-based" workflow is approved. We need to ensure `docs/` is clean. Move design docs to `docs/design/` and keep `STDLIB_REFERENCE.md` prominent.

## Instructions for Claude (Current Session)
1.  **Refine `fpm`**: Implement `Manifest::load` and `Manifest::save` in `fusabi-pm`.
2.  **Cleanup**: Move design docs in `fusabi/docs/` to `fusabi/docs/design/` to clean up the root for the aggregator.
