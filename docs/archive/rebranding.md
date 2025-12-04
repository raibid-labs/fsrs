# REBRANDING.md

**Context:** We are rebranding this project to **Fusabi**.
**Mission:** Fusabi is an embedded F# dialect for Rust, designed for platform engineering and TUI tools. It combines Functional programming with a high-performance ABI.

## Instructions for AI Agent

Please perform the following file operations and refactors to establish the new brand identity.

### 1. Configuration (`Cargo.toml`)
Update the `[package]` metadata:
* **Name:** `fusabi`
* **Description:** "A potent, functional scripting layer for Rust infrastructure."
* **Repository:** `https://github.com/fusabi-lang/fusabi`
* **Default Run:** `fus`
* **Binaries:** Rename the main binary entry to `fus`.

### 2. File Extensions & Types
* **Source:** Maintain support for `.fsx` (F# Script).
* **Output:** The compiler should now target `.fzb` (Fusabi Binary).
* **Magic Bytes:** If there is a bytecode header, update the magic bytes to `b"FZB\x01"`.

### 3. CLI Structure (`src/bin/fus.rs`)
Refactor the CLI entry point (formerly `fsc` or `main`) to be named `fus`. Update the help text to reflect the new commands:
* `fus run <script.fsx>`: JIT execution.
* `fus grind <script.fsx>`: Compiles source to `.fzb` (The build command). *Note: "Grind" is the action used for fresh Wasabi.*
* `fus root <subcommand>`: The package manager (e.g., `add`, `remove`).

### 4. Documentation (`README.md`)
Rewrite the README with the following copy:

> # Fusabi 🟢
>
> **Small. Potent. Functional.**
>
> Fusabi is a high-performance embedded scripting engine for Rust. It allows you to write type-safe F# logic that binds directly to your Rust application's ABI.
>
> ## Usage
>
> ```bash
> # 1. Add a dependency (The Root)
> fus root add http-client
>
> # 2. Write Logic (Standard F#)
> # script.fsx
> let handle_request req =
>     printfn "Handling request with %s spice" "high"
>
> # 3. Grind (Compile)
> fus grind script.fsx
> # Output: script.fzb
> ```

### 5. Global Search & Replace
Perform a case-insensitive replacement across the codebase:
* Replace `fsrs` -> with `fusabi` or `fus` where appropriate.
