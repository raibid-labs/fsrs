# Issue 7: [Docs] Create Comprehensive Example Suite

**Labels:** `documentation`, `examples`

## Context
We need diverse examples to prove the "Dual-Runtime" capability and embedded utility.

## Implementation Plan
**Objective:** Create `examples/` subdirectories with the following fully working prototypes:

1.  **`examples/bevy_scripting/`**: A Rust binary using Bevy. Spawns an entity. Reads `behavior.fsx` to determine movement logic every tick.
2.  **`examples/ratatui_layout/`**: A Rust binary using Ratatui. Reads `layout.fsx`. The script returns a Record describing a grid layout. Rust renders it.
3.  **`examples/burn_config/`**: Define a neural net config (layers, dropout rates) in F#. Rust reads it to initialize a Burn model.
4.  **`examples/web_server/`**: An Axum server where the validation logic for an endpoint `POST /users` is loaded from `validation.fsx`.
5.  **`examples/computations/`**: Define a `result { ... }` computation expression in F# and desugar it to demonstrate custom DSLs.
6.  **`examples/interop_net/`**: A script `common.fsx`. A `run.sh` script that runs it once with `dotnet fsi` and once with `fus run` to prove syntax compatibility.
