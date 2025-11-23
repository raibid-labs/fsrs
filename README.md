# Fusabi 🟢

**Small. Potent. Functional.**

Fusabi is a high-performance embedded scripting engine for Rust. It allows you to write type-safe F# logic that binds directly to your Rust application's ABI.

## Usage

```bash
# 1. Write Logic (Standard F#)
# script.fsx
let handle_request name =
    printfn "Handling request from %s" name

handle_request "Fusabi User"

# 2. Run Directly
fus run script.fsx

# 3. Compile to Bytecode (for faster startup)
fus grind script.fsx
# Output: script.fzb

# 4. Run Bytecode
fus run script.fzb
```

## Project Status

**Version**: 0.1.0 (First Public Release)
**Status**: Phase 3 - Complete (Advanced Features)

### Key Features

- **F#-Style Scripting**: Full F# dialect with records, discriminated unions, pattern matching
- **High-Performance VM**: Stack-based bytecode interpreter with `.fzb` binary format
- **Standard Library**: Built-in `List`, `String`, `Option` modules and pipeline operator `|>`
- **Host Interop**: Safe, re-entrant API for embedding in Rust applications
- **Bytecode Compilation**: `fus grind` for ahead-of-time compilation

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
