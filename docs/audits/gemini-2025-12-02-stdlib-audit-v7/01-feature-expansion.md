# Feature Expansion & Porting Strategy

**Date**: 2025-12-02
**Status**: Conceptual Design

## The "F# -> Fusabi" Bridge

We cannot load F# assemblies (.dll) because Fusabi runs on a Rust VM, not the CLR. Therefore, to leverage the F# ecosystem, we must **transpile** source code.

### 1. The Transpiler Architecture
We should build a CLI tool (likely written in F# itself, to leverage the official F# Compiler Services) that takes an F# project and emits Fusabi source.

*   **Input**: F# Source (`.fs`, `.fsx`) or Project (`.fsproj`).
*   **Process**:
    1.  Use `FSharp.Compiler.Service` to parse the AST.
    2.  Filter out unsupported features (Classes, Interfaces, Reflection).
    3.  Map standard F# modules (`FSharp.Core`) to Fusabi equivalents.
    4.  Emit Fusabi source code.
*   **Output**: `.fsx` files compatible with the Fusabi VM.

### 2. Target Libraries (Priorities)
1.  **FParsec**: A parser combinator library. If we can port a subset of this, we unlock writing parsers in Fusabi.
2.  **FSharp.Data (JsonProvider subset)**: We can't do the "Provider" part (compile-time generation), but we can port the runtime JSON handling logic.
3.  **Fantomas**: Code formatting logic (ambitious, but useful).

### 3. Community Repository
**Action**: Initialize `fusabi-lang/fusabi-community`.
**Purpose**:
-   Store the "Ported" versions of libraries.
-   Store Fusabi-native community packages.
-   Serve as the package registry index (for a future `fpm` package manager).

## Missing DX Features
1.  **Debugger**: We have no interactive debugger. The VM needs a debug adapter (DAP) implementation to allow VS Code to attach.
2.  **LSP (Language Server)**: We have `tree-sitter` grammar (likely), but a real LSP for autocomplete/hover is the biggest DX gap.
3.  **Repl**: `Console` module exists now, so a pure-Fusabi REPL is possible!

## Recommendation
Start the **Community Repo** immediately. Use it to host manual ports of small utilities to verify the ecosystem flow before building the automated transpiler.
