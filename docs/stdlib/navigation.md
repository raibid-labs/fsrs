# Navigation Module (Nav)

The `Nav` module provides navigation and keymap configuration APIs for Scarab integration. It allows Fusabi scripts to manage focusable UI elements, control hint-based navigation, and configure keymap styles.

## Safety Features

All navigation operations include capability limits and rate limiting:

- **Max Focusables**: Default 1000 elements
- **Rate Limiting**: Default 60 actions/second
- **Bounds Validation**: Prevents abuse

## Keymap Configuration

### Nav.getKeymap() -> string

Returns the current navigation keymap style.

```fsharp
let style = Nav.getKeymap()
printfn "Current keymap: %s" style
```

### Nav.setKeymap(style: string) -> unit

Sets the navigation keymap style. Supported styles:
- `"vimium"` - Vimium-style hints (home row keys)
- `"cosmos"` - Cosmos navigation style
- `"spacemacs"` - Spacemacs-style bindings
- Custom names for user-defined keymaps

```fsharp
Nav.setKeymap("vimium")
```

## Focusable Management

### Nav.registerFocusable(id: string, label: string) -> Result<unit, string>

Registers a focusable element for navigation. Returns an error if the limit is exceeded.

```fsharp
match Nav.registerFocusable("btn-submit", "Submit Button") with
| Ok _ -> printfn "Registered successfully"
| Error msg -> printfn "Failed: %s" msg
```

### Nav.unregisterFocusable(id: string) -> bool

Removes a focusable element. Returns true if it existed.

```fsharp
if Nav.unregisterFocusable("btn-submit") then
    printfn "Removed"
```

### Nav.clearFocusables() -> int

Clears all registered focusables. Returns the count removed.

```fsharp
let count = Nav.clearFocusables()
printfn "Cleared %d focusables" count
```

### Nav.getFocusableCount() -> int

Returns the number of registered focusables.

```fsharp
printfn "Focusables: %d" (Nav.getFocusableCount())
```

### Nav.listFocusables() -> list<{id: string, label: string}>

Returns all registered focusables as a list.

```fsharp
Nav.listFocusables()
|> List.iter (fun f -> printfn "%s: %s" f.id f.label)
```

## Navigation Actions

### Nav.enterHintMode() -> Result<unit, string>

Enters hint mode for keyboard navigation. Rate-limited.

```fsharp
match Nav.enterHintMode() with
| Ok _ -> printfn "Hint mode activated"
| Error msg -> printfn "Rate limited: %s" msg
```

### Nav.exitHintMode() -> unit

Exits hint mode.

```fsharp
Nav.exitHintMode()
```

### Nav.isHintModeActive() -> bool

Checks if hint mode is currently active.

```fsharp
if Nav.isHintModeActive() then
    printfn "Hints visible"
```

### Nav.jumpToAnchor(anchorId: string) -> Result<unit, string>

Jumps to a registered focusable by ID. Rate-limited.

```fsharp
match Nav.jumpToAnchor("btn-submit") with
| Ok _ -> printfn "Jumped to anchor"
| Error msg -> printfn "Error: %s" msg
```

### Nav.getCurrentAnchor() -> Option<string>

Returns the current anchor/focusable ID if any.

```fsharp
match Nav.getCurrentAnchor() with
| Some id -> printfn "Current: %s" id
| None -> printfn "No anchor selected"
```

## Limits Configuration

### Nav.getLimits() -> { maxFocusables: int, maxActionsPerSecond: int, maxHintLength: int }

Returns the current navigation limits.

```fsharp
let limits = Nav.getLimits()
printfn "Max focusables: %d" limits.maxFocusables
printfn "Rate limit: %d/sec" limits.maxActionsPerSecond
```

### Nav.setLimits(maxFocusables: int, maxActionsPerSecond: int) -> unit

Sets custom navigation limits. Useful for host configuration.

```fsharp
// Allow more focusables for complex UIs
Nav.setLimits(5000, 120)
```

## Example: Scarab Plugin Navigation

```fsharp
// Configure navigation for a Scarab plugin
module MyPlugin =
    let setup() =
        // Set preferred keymap
        Nav.setKeymap("vimium")
        
        // Register UI focusables
        Nav.registerFocusable("search", "Search Box") |> ignore
        Nav.registerFocusable("results", "Results Panel") |> ignore
        Nav.registerFocusable("settings", "Settings Button") |> ignore
        
        printfn "Plugin navigation ready (%d focusables)" (Nav.getFocusableCount())
    
    let handleHintKey() =
        if not (Nav.isHintModeActive()) then
            Nav.enterHintMode() |> ignore
        else
            Nav.exitHintMode()
    
    let cleanup() =
        Nav.clearFocusables() |> ignore
```
