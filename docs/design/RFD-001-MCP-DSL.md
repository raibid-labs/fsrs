# RFD-001: Unified MCP DSL and Standard Library Extensions

## Status
- **Date**: 2025-11-25
- **Author**: Gemini CLI
- **Status**: Proposed

## Context
Analysis of the `raibid-labs` codebase reveals a fragmented approach to implementing Model Context Protocol (MCP) servers:
- `ardour-mcp`: Implemented in **Python** (using `mcp` library).
- `dgx-spark-mcp`: Implemented in **TypeScript**.
- `fusabi-mcp`: Implemented in **Rust**.

This fragmentation leads to:
1.  **Duplicated Boilerplate**: Each implementation re-creates the server setup, error handling, and tool registration logic.
2.  **Inconsistent Developer Experience**: Adding a new tool requires context-switching between languages and frameworks.
3.  **Maintenance Overhead**: Updates to the MCP spec must be propagated across three different tech stacks.

## Proposal
We propose positioning **Fusabi** as the unified language for defining MCP servers within `raibid-labs`. This requires extending the Fusabi Standard Library and creating a dedicated MCP Domain Specific Language (DSL).

### 1. Standard Library Additions
To support this, the Fusabi core libraries must be extended with:

- **`Net` Module**:
    - HTTP Client/Server (for Spark/generic APIs).
    - OSC (Open Sound Control) Client (specifically for Ardour integration).
    - Socket/TCP support.

- **`Json` Module**:
    - Native JSON parsing and serialization (beyond the current debug output).
    - JSON-RPC helper types.

- **`Async` Module**:
    - `Task` or `Promise` equivalent for non-blocking I/O.

### 2. Fusabi MCP DSL
A declarative DSL for defining tools and servers.

#### Comparison: Python (Current) vs. Fusabi (Proposed)

**Python (`ardour-mcp`):**
```python
@self.server.call_tool()
async def transport_play() -> list[Any]:
    """Start playback in Ardour."""
    result = await self.transport_tools.transport_play()
    return [result]
```

**Fusabi (Proposed):**
```fsharp
open Fusabi.Mcp
open Fusabi.Net.Osc

let ardour = Osc.client "localhost" 3819

let server = Mcp.server "ardour-mcp" "1.0.0"

tool "transport_play" "Start playback" {
    description "Start playback in Ardour"
    execute (fun _ ->
        ardour |> Osc.send "/transport_play"
        "Playback started"
    )
}

server |> Mcp.run
```

### 3. Benefits
- **Conciseness**: Drastic reduction in boilerplate code.
- **Type Safety**: Fusabi's type system ensures tool inputs/outputs match schemas.
- **Portability**: The script can run anywhere the Fusabi VM (with `fusabi-mcp`) is deployed.
- **Unification**: One language for all agent interfaces.

## Implementation Plan
1.  **Phase 1**: Implement `Net.Osc` and `Net.Http` in `fusabi-vm`.
2.  **Phase 2**: Create `Fusabi.Mcp` module wrapping the Rust `fusabi-mcp` implementation.
3.  **Phase 3**: Port `ardour-mcp` to Fusabi as a proof-of-concept.
