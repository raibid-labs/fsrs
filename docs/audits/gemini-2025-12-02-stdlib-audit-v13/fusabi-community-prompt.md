# Prompt for `fusabi-community` Initialization

**Context**: You are the maintainer of the new `fusabi-community` repository.
**Goal**: Initialize the repository structure and port the first package.

## Instructions

1.  **Initialize Structure**:
    Create the following directory structure:
    ```text
    packages/
    registry/
    tools/
    ```

2.  **Create Registry Index**:
    Create `registry/index.toml` with an empty package list:
    ```toml
    # Fusabi Community Package Registry
    [packages]
    ```

3.  **Port `commander`**:
    *   Create `packages/commander/`.
    *   Create `packages/commander/fusabi.toml`:
        ```toml
        [package]
        name = "commander"
        version = "0.1.0"
        description = "A TUI file manager"
        authors = ["Fusabi Community"]
        license = "MIT"
        ```
    *   Create `packages/commander/src/main.fsx` and copy the content of `examples/commander.fsx` from the main repo (provided below).

4.  **Create `json` Package**:
    *   Create `packages/json/`.
    *   Create `packages/json/fusabi.toml`.
    *   Create `packages/json/src/lib.fsx` with some basic JSON combinators (using `Json` stdlib).

## Content for `commander.fsx`
(Copy the content of `examples/commander.fsx` here)
