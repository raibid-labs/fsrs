# Issue 9: [Brand] Define Visual Identity & Color Palette

**Labels:** `branding`, `design`, `documentation`

## Context
Fusabi needs a consistent visual language across its CLI, documentation, and potential website. We are adopting a "Wasabi + Rust" aesthetic: organic, earthy, and spicy.

## Implementation Plan
**Objective:** Formalize the color palette and typography.

1.  **Define Color Palette:**
    * **Primary (Wasabi):** `#99CC33` (Electric Green) or `#78A659` (Natural Wasabi).
    * **Accent (Rust):** `#B7410E` (Rust Orange) or `#DEA584` (Sashimi/Salmon).
    * **Background:** `#1E1E1E` (Dark Grey) for terminal/code blocks.
    * **Text:** `#F0F0F0` (Off-white).

2.  **Create `docs/BRANDING.md`:**
    * Document these hex codes.
    * Define typography preference:
        * Headers: Sans-serif (Inter or Helvetica).
        * Code: Monospace (JetBrains Mono or Fira Code).
    * Establish emoji usage: 🟢 (Fusabi), 🦀 (Rust), 🍣 (Raw/Host).

3.  **Action Item:**
    * Update the `README.md` header badge.
    * Create a simple CSS snippet for any future HTML docs.
