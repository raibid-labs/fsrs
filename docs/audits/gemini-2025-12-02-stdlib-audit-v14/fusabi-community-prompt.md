# Prompt for `fusabi-community` Expansion

**Context**: You are working in the `fusabi-community` repository.
**Goal**: establish CI and fleshing out the first library package.

## Instructions

1.  **Create CI Workflow**:
    *   Create `.github/workflows/test-packages.yml`.
    *   It should checkout the repo, install `fusabi` (you might need to build it from source or download a release artifact - assume `cargo install --git https://github.com/fusabi-lang/fusabi` works for now, or just run a mock test script).
    *   Iterate through `packages/*` and run `fus run src/main.fsx` (or equivalent entry point) to verify they parse/compile.

2.  **Implement `json` Package**:
    *   Edit `packages/json/src/lib.fsx`.
    *   Implement a functional JSON API on top of the `Json` stdlib (which just does Parse/Stringify).
    *   Add: `Json.get : string -> Value -> Option<Value>` (path traversal).
    *   Add: `Json.asString`, `Json.asInt`, etc.

3.  **Update Registry**:
    *   Add `commander` and `json` to `registry/index.toml`.
