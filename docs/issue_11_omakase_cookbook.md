# Issue 11: [Docs] "The Omakase" - Cookbook & Patterns

**Labels:** `documentation`, `branding`

## Context
"Omakase" means "I'll leave it up to you" (chef's choice). To differentiate Fusabi from generic scripting tools, we will frame our "Examples" section as "The Omakase" — a curated set of best-practice patterns.

## Implementation Plan
**Objective:** Restructure and rename the examples directory.

1.  **Rename Structure:**
    * Keep the physical `examples/` folder for tool compatibility.
    * Create a new index file: `docs/OMAKASE.md`.

2.  **Structure the Cookbook (`OMAKASE.md`):**
    * **Appetizers:** Simple one-liners (Regex, Math, String manipulation).
    * **Main Courses:** Full configuration files (Terminal layout, Game logic, Web Server).
    * **Fusion:** Rust Interop examples (calling Host functions, passing Records).

3.  **Copywriting:**
    * Rewrite `examples/README.md` introduction:
        > "Welcome to the Omakase. These are hand-rolled, chef-selected examples to demonstrate the potency of Fusabi. Pick a dish and start scripting."
