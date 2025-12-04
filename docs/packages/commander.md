# Commander

A TUI file explorer demonstrating Fusabi's terminal and event capabilities.

## Quick Start

```bash
fus run examples/commander.fsx
```

## Features

- **File Navigation**: Browse directories with vim-style `j`/`k` keys
- **Directory Traversal**: Enter directories with `Enter`, navigate up with `..`
- **Terminal Integration**: Uses `TerminalControl`, `TerminalInfo`, and `Console` modules
- **Event System**: Demonstrates `Events.on`/`Events.off` for keyboard handling
- **Process Execution**: Runs shell commands via `Process.runShell`

## Controls

| Key     | Action                  |
|---------|-------------------------|
| `j`     | Move selection down     |
| `k`     | Move selection up       |
| `Enter` | Open selected directory |
| `q`     | Quit                    |

## Architecture

Commander follows a Model-View-Update (MVU) pattern:

### Model

```fusabi
let initialModel = {
    currentDir = Process.cwd ();
    files = [];
    selectedIndex = 0;
    running = true
}
```

### View

Renders the current state to the terminal using ANSI escape codes:

```fusabi
let clearScreen () =
    TerminalControl.sendText "\x1b[2J\x1b[H"
```

### Update

Handles events and returns a new model:

```fusabi
let update event model =
    match event with
    | "key:j" -> handleKeyDown model
    | "key:k" -> handleKeyUp model
    | "key:enter" -> handleEnter model
    | "key:q" -> handleQuit model
    | _ -> model
```

## Stdlib Modules Used

- `Events` - Event registration and handling
- `TerminalControl` - Terminal escape sequences
- `TerminalInfo` - Terminal size detection
- `Process` - Directory and shell operations
- `Console` - User input/output
- `List` - List manipulation
- `String` - String parsing

## Package Manifest

See [examples/commander/fusabi.toml](../../examples/commander/fusabi.toml) for a sample package manifest.
