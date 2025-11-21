# Issue 10: [CLI] Implement "Spicy" Terminal Output

**Labels:** `enhancement`, `cli`, `good-first-issue`

## Context
The `fus` binary should reflect the brand in its output. Errors, warnings, and success messages should use the brand colors to stand out in a user's terminal.

## Implementation Plan
**Objective:** Add color and style to `fusabi/src/main.rs`.

1.  **Dependencies:**
    * Add `colored` or `yansi` to `fusabi/Cargo.toml`.

2.  **Design Banner:**
    * Create a simple ASCII art banner for `fus --help` or `fus repl`.
    * *Concept:*
        ```text
         / \
        ( F )  Fusabi v0.2.0
         \_/   Small. Potent. Functional.
        ```

3.  **Colorize Output:**
    * **Success:** Green (Wasabi). Example: "✅ Loaded config.fsx"
    * **Error:** Red/Orange (Rust). Example: "❌ Parse Error: Unexpected token"
    * **Warning:** Yellow.
    * **Hints:** Grey/Italic.

4.  **REPL Prompt:**
    * Change standard input prompt to `fusabi> ` or `🟢> `.
