# Fusabi 🟢

**Small. Potent. Functional.**

Fusabi is a high-performance embedded scripting engine for Rust. It allows you to write type-safe F# logic that binds directly to your Rust application's ABI.

## Usage

```bash
# 1. Add a dependency (The Root)
fus root add http-client

# 2. Write Logic (Standard F#)
# script.fsx
let handle_request req =
    printfn "Handling request with %s spice" "high"

# 3. Grind (Compile)
fus grind script.fsx
# Output: script.fzb
```

## Project Status

**Version**: 0.2.0-alpha (Fusabi Rebranding)
**Status**: Phase 3 - Advanced Features (In Progress)

### Key Features

- **F#-Style Scripting**: Full F# dialect with records, discriminated unions, pattern matching
- **High-Performance VM**: Stack-based bytecode interpreter with GC
- **Type Safety**: Hindley-Milner type inference
- **Host Interop**: Lua-class embedding API for Rust applications
- **Hot-Reload**: Development-friendly script reloading

## Quick Start

```bash
# Clone the repository
git clone https://github.com/fusabi-lang/fusabi.git
cd fusabi

# Bootstrap the environment
just bootstrap

# Build the project
just build

# Run a script
fus run examples/hello.fsx

# Compile to bytecode
fus grind examples/hello.fsx
```

## Documentation

- [docs/roadmap.md](docs/roadmap.md) - Development roadmap
- [docs/setup.md](docs/setup.md) - Setup guide
- [docs/claude-config.md](docs/claude-config.md) - Development configuration (for AI agents)
- [docs/toc.md](docs/toc.md) - Complete documentation index
- [docs/rebranding-complete.md](docs/rebranding-complete.md) - Rebranding summary
- [docs/bytecode-format.md](docs/bytecode-format.md) - .fzb Bytecode Format specification

## License

MIT License - See LICENSE file for details.

## Repository

https://github.com/fusabi-lang/fusabi
