# Fusabi Visual Identity & Brand Guidelines

**Tagline:** Small. Potent. Functional.

## Brand Voice

Fusabi's brand voice is **punchy, confident, and slightly playful** - like wasabi itself. We balance technical precision with personality:

- **Direct**: Say what we mean. No fluff.
- **Confident**: We know what we're good at.
- **Approachable**: Technical, but not pretentious.
- **Spicy**: A little kick, never bland.

## Color Palette

### Primary Colors

| Color | Hex Code | Usage | Notes |
|-------|----------|-------|-------|
| **Wasabi Green** | `#99CC33` | Primary brand color, success states, highlights | Electric, energetic variant |
| **Natural Wasabi** | `#78A659` | Alternative green, organic contexts | Earthy, natural variant |
| **Rust Orange** | `#B7410E` | Accent color, warnings, Rust references | Deep, rich orange |
| **Sashimi Salmon** | `#DEA584` | Secondary accent, warm highlights | Softer, lighter accent |

### Neutral Colors

| Color | Hex Code | Usage |
|-------|----------|-------|
| **Dark Grey** | `#1E1E1E` | Backgrounds, terminal, code blocks |
| **Off-White** | `#F0F0F0` | Body text, light backgrounds |
| **Charcoal** | `#2D2D2D` | Secondary backgrounds |
| **Steel Grey** | `#7F8C8D` | Muted text, borders |

### Usage Guidelines

- **Primary Actions**: Use Wasabi Green (#99CC33)
- **Links & Navigation**: Wasabi Green or Natural Wasabi
- **Code Blocks**: Dark Grey (#1E1E1E) background with Off-White text
- **Rust Integration**: Rust Orange (#B7410E) for Rust-specific features
- **Success Messages**: Wasabi Green
- **Warnings**: Rust Orange
- **Errors**: Use traditional red (#DC3545), not brand colors

## Typography

### Headers
- **Font Family**: Sans-serif (Inter, Helvetica, or system defaults)
- **Weight**: Bold (700) for H1-H2, Semi-bold (600) for H3-H6
- **Style**: Clean, modern, highly readable

### Body Text
- **Font Family**: Sans-serif (Inter, -apple-system, system-ui)
- **Weight**: Regular (400) for body, Medium (500) for emphasis
- **Line Height**: 1.6 for readability

### Code & Monospace
- **Font Family**: JetBrains Mono, Fira Code, Consolas, or monospace
- **Weight**: Regular (400), Bold (700) for keywords
- **Features**: Ligatures encouraged for better code readability

## Emoji Usage

Fusabi uses specific emojis as visual shorthand across documentation and CLI:

| Emoji | Meaning | Usage |
|-------|---------|-------|
| 🟢 | **Fusabi** | Brand identifier, success states, "ready" status |
| 🦀 | **Rust** | Rust language features, host interop, ABI references |
| 🍣 | **Raw/Host** | Low-level operations, host functions, native bindings |
| ⚡ | **Performance** | Speed, optimization, benchmarks |
| 📦 | **Package/Module** | Crates, bytecode files, modules |
| 🔧 | **Configuration** | Setup, build tools, dev tools |
| 📖 | **Documentation** | Guides, references, learning resources |

### Emoji Guidelines
- Use sparingly and consistently
- Always pair with text (never emoji-only)
- Prefer text in formal documentation
- Use freely in CLI output and casual docs

## Logo & Assets

### Logo Specifications
- **Primary Logo**: SVG format with Wasabi Green (#99CC33)
- **Minimum Size**: 32x32px for digital use
- **Clear Space**: Maintain padding equal to the height of the "F" character
- **Variations**:
  - Full color (default)
  - Monochrome (for print or constraints)
  - Inverted (for dark backgrounds)

### Logo Usage
- ✅ **Do**: Use on white or Dark Grey backgrounds
- ✅ **Do**: Scale proportionally
- ✅ **Do**: Maintain clear space around logo
- ❌ **Don't**: Distort, rotate, or modify colors
- ❌ **Don't**: Place on busy backgrounds
- ❌ **Don't**: Add effects (shadows, gradients, etc.)

## CLI Styling

Terminal output should leverage the color palette:

```
Success: 🟢 Compiled to bytecode (125ms)
Warning: ⚠️  Unused binding 'x' at line 42
Error:   ✗  Type mismatch: expected Int, got String
Info:    →  Using bytecode format v1.0
```

### Terminal Colors
- **Success**: Green (ANSI green or `#99CC33`)
- **Warning**: Yellow or Rust Orange
- **Error**: Red (standard ANSI red)
- **Info**: Cyan or Steel Grey
- **Progress**: Wasabi Green with spinner
- **Code snippets**: Syntax highlighting with muted palette

## Web & Documentation

### CSS Color Variables

```css
:root {
  /* Primary */
  --fusabi-green: #99CC33;
  --fusabi-green-natural: #78A659;
  --rust-orange: #B7410E;
  --salmon: #DEA584;

  /* Neutrals */
  --dark-grey: #1E1E1E;
  --off-white: #F0F0F0;
  --charcoal: #2D2D2D;
  --steel-grey: #7F8C8D;

  /* Semantic */
  --color-primary: var(--fusabi-green);
  --color-accent: var(--rust-orange);
  --color-background: var(--dark-grey);
  --color-text: var(--off-white);
  --color-success: var(--fusabi-green);
  --color-warning: var(--rust-orange);
}
```

### Code Syntax Highlighting

Recommended theme: **One Dark** or **Monokai** with Wasabi Green accents for:
- Function names
- Keywords (`let`, `match`, `type`)
- Success states in output

## Badge Styling

For GitHub badges and shields:
- **Build Status**: Use green (#99CC33) for passing
- **Version**: Use Wasabi Green background
- **License**: Use Steel Grey or standard blue
- **Stars/Social**: Use default GitHub colors

Example:
```markdown
![Build](https://img.shields.io/badge/build-passing-99CC33)
![Version](https://img.shields.io/badge/version-0.5.0-78A659)
```

## Design Principles

1. **Minimalism**: Clean, uncluttered layouts
2. **Functionality First**: Design serves the user, not the other way around
3. **High Contrast**: Ensure readability in all contexts
4. **Consistent Spacing**: Use 8px grid system
5. **Performance**: Fast-loading assets, optimized images

## File Formats

- **Logos**: SVG (primary), PNG (fallback)
- **Screenshots**: PNG with compression
- **Icons**: SVG with fallback fonts
- **Diagrams**: SVG or Mermaid markdown

## Brand Don'ts

- ❌ Don't use gradients (flat colors only)
- ❌ Don't use Comic Sans or decorative fonts
- ❌ Don't overuse emojis
- ❌ Don't use colors outside the palette without justification
- ❌ Don't create "cute" or "quirky" variations of the logo
- ❌ Don't mix hot and cold brand voices in the same document

## References

- **Inspiration**: Wasabi condiment (sharp, green, potent)
- **Industry**: Rust ecosystem (pragmatic, performance-focused)
- **Audience**: Developers who value clarity and performance
- **Competitive Positioning**: Lighter than V8, friendlier than Lua FFI, typesafe unlike JavaScript

---

**Version**: 1.0
**Last Updated**: 2025-11-24
**Maintained By**: Fusabi Core Team
