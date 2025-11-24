# Fusabi CLI

The command-line interface for the **Fusabi** scripting engine.

## Features

- **Run Scripts**: Execute `.fsx` source files directly.
- **Compile**: Compile scripts to `.fzb` bytecode for faster startup.
- **REPL**: (Coming soon) Interactive shell.

## Installation

```bash
cargo install fusabi
```

## Usage

```bash
# Run a script
fus run script.fsx

# Compile to bytecode
fus grind script.fsx

# Run bytecode
fus run script.fzb
```

## License

MIT