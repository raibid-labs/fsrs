# Workstream 7: Branding & Visual Identity

## Status
🟡 Ready to Start (fully independent)

## Overview
Establish Fusabi's visual identity and brand voice across all touchpoints: CLI, documentation, and marketing materials. Adopt a "Wasabi + Rust" aesthetic: organic, earthy, spicy, and punchy.

## Objectives
- [ ] Define color palette and typography (Issue 9)
- [ ] Implement styled CLI output (Issue 10)
- [ ] Create "The Omakase" cookbook (Issue 11)
- [ ] Design and create logo assets (Issue 12)
- [ ] Overhaul README with brand voice (Issue 13)

## Agent Assignment
**Suggested Agent Type**: `ui-ux-designer`, `content-marketer`, `frontend-developer`, `docs-architect`
**Skill Requirements**: Design, branding, copywriting, CLI styling, SVG design

## Dependencies
- None (fully independent, can run in parallel with all workstreams)
- **WS6**: Synergy with examples documentation (Omakase ties to examples)

## Tasks

### Task 7.1: Define Visual Identity & Color Palette (Issue 9)
**Description**: Formalize Fusabi's color palette, typography, and visual language.

**Deliverables**:
- **Color Palette**:
  - Primary (Wasabi): `#99CC33` (Electric Green) or `#78A659` (Natural Wasabi)
  - Accent (Rust): `#B7410E` (Rust Orange) or `#DEA584` (Sashimi/Salmon)
  - Background: `#1E1E1E` (Dark Grey)
  - Text: `#F0F0F0` (Off-white)
- **Typography**:
  - Headers: Sans-serif (Inter or Helvetica)
  - Code: Monospace (JetBrains Mono or Fira Code)
- **Emoji usage**: 🟢 (Fusabi), 🦀 (Rust), 🍣 (Raw/Host)
- `docs/BRANDING.md` with full specification
- CSS snippet for future HTML docs

**Files to Create/Modify**:
- `docs/BRANDING.md` (new)
- `README.md` (update badges with brand colors)

**Implementation**:
```markdown
# Fusabi Brand Guidelines

## Color Palette

### Primary Colors
- **Wasabi Green**: `#99CC33` (Electric) or `#78A659` (Natural)
  - Use for: Success messages, primary CTAs, highlights
- **Rust Orange**: `#B7410E` (Rust) or `#DEA584` (Sashimi)
  - Use for: Error messages, warnings, accents

### Background & Text
- **Dark Grey**: `#1E1E1E` - Code blocks, terminal backgrounds
- **Off-white**: `#F0F0F0` - Body text
- **Light Grey**: `#666666` - Hints, secondary text

## Typography

### Headers
- **Font**: Inter, Helvetica, sans-serif
- **Weight**: Bold (700) for H1-H2, Semibold (600) for H3-H6

### Body
- **Font**: System default sans-serif
- **Size**: 16px base

### Code
- **Font**: JetBrains Mono, Fira Code, Monaco, monospace
- **Features**: Ligatures enabled for operators

## Emoji Usage

- 🟢 **Fusabi**: Represents the language/runtime
- 🦀 **Rust**: Host language integration
- 🍣 **Raw/Host**: Raw performance, direct access
- ✅ **Success**: Confirmations, passing tests
- ❌ **Error**: Failures, blocking issues
- ⚠️  **Warning**: Cautions, deprecations

## Voice & Tone

- **Punchy**: Short sentences. Direct language.
- **Confident**: "Don't guess. Know."
- **Playful**: Wasabi metaphors (spicy, potent, fresh)
- **Technical**: Precise, no hand-waving

## CSS Snippet

```css
:root {
  --fusabi-green: #99CC33;
  --fusabi-rust: #B7410E;
  --fusabi-bg: #1E1E1E;
  --fusabi-text: #F0F0F0;
}

.fusabi-badge {
  background: var(--fusabi-green);
  color: var(--fusabi-bg);
  padding: 4px 8px;
  border-radius: 4px;
  font-weight: 600;
}
```
```

**Validation**:
```bash
cat docs/BRANDING.md
# Should contain color palette, typography, emoji guide
```

---

### Task 7.2: Implement "Spicy" CLI Styling (Issue 10)
**Description**: Add brand colors and ASCII art to CLI output.

**Deliverables**:
- Add `colored` or `yansi` dependency
- ASCII art banner for `fus --help` or `fus repl`
- Colorized success/error/warning messages
- Styled REPL prompt

**Files to Create/Modify**:
- `rust/fusabi/Cargo.toml` (add color dependency)
- `rust/fusabi/src/main.rs` (colorize output)
- `rust/fusabi/src/cli/banner.rs` (new, ASCII art)

**Implementation**:
```toml
# rust/fusabi/Cargo.toml

[dependencies]
colored = "2.1"
# OR
yansi = "1.0"
```

```rust
// rust/fusabi/src/cli/banner.rs

use colored::*;

pub fn print_banner() {
    println!(
        r#"
     {}
    {} {}  Fusabi v{}
     {}   Small. Potent. Functional.
        "#,
        "/ \\".bright_green(),
        "(".bright_green(),
        "F".bright_green().bold(),
        ")".bright_green(),
        env!("CARGO_PKG_VERSION"),
        "\\_/".bright_green()
    );
}

pub fn print_welcome() {
    println!("{}", "🟢 Welcome to Fusabi REPL".bright_green());
    println!("{}", "Type :help for help, :quit to exit".truecolor(153, 153, 153));
}
```

```rust
// rust/fusabi/src/main.rs

use colored::*;
mod cli;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--help") | Some("-h") => {
            cli::banner::print_banner();
            print_help();
        }
        Some("repl") => {
            cli::banner::print_banner();
            cli::banner::print_welcome();
            run_repl();
        }
        Some("run") => {
            if let Some(file) = std::env::args().nth(2) {
                match run_file(&file) {
                    Ok(_) => println!("{} {}", "✅".green(), "Execution successful".green()),
                    Err(e) => {
                        eprintln!("{} {}", "❌".red(), "Error:".red().bold());
                        eprintln!("{}", format!("{}", e).truecolor(183, 65, 14)); // Rust orange
                    }
                }
            }
        }
        _ => {
            eprintln!("{} Unknown command", "⚠️ ".yellow());
            eprintln!("Run {} for help", "fus --help".bold());
        }
    }
}

fn run_repl() {
    use std::io::{self, Write};

    loop {
        print!("{} ", "🟢>".bright_green().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == ":quit" {
            println!("{}", "Goodbye! 👋".bright_green());
            break;
        }

        // ... execute input
    }
}
```

**Validation**:
```bash
cargo run -- --help
# Should show ASCII art banner

cargo run -- repl
# Should show green welcome message and 🟢> prompt

cargo run -- run nonexistent.fsx
# Should show red error message with ❌
```

---

### Task 7.3: Create "The Omakase" Cookbook (Issue 11)
**Description**: Restructure examples as a curated "chef's choice" cookbook.

**Deliverables**:
- `docs/OMAKASE.md` index file
- Restructure `examples/README.md` with brand voice
- Categorize examples: Appetizers, Main Courses, Fusion

**Files to Create/Modify**:
- `docs/OMAKASE.md` (new)
- `examples/README.md` (rewrite)

**Implementation**:
```markdown
# The Omakase 🍣

Welcome to the Omakase. These are hand-rolled, chef-selected examples to demonstrate the potency of Fusabi. Pick a dish and start scripting.

## Philosophy

"Omakase" (お任せ) means "I'll leave it up to you" — the chef's choice. These aren't random code snippets. They're carefully curated patterns that show what Fusabi does best: embedding functional scripting into Rust applications.

## Menu

### 🍵 Appetizers (Simple One-Liners)

Quick bites to get a taste of Fusabi:

- **String Manipulation**: `examples/string_ops.fsx`
  ```fsharp
  "Hello, Fusabi!" |> String.toUpper |> String.split "," |> List.map String.trim
  ```

- **Math**: `examples/math_demo.fsx`
  ```fsharp
  [1..100] |> List.filter (fun x -> x % 15 = 0) |> List.sum
  ```

- **Regex**: `examples/regex.fsx`
  ```fsharp
  Regex.matches @"\b[A-Z]\w+" "Hello World From Fusabi"
  ```

### 🍱 Main Courses (Full Configuration Files)

Complete applications showcasing embedded Fusabi:

- **Bevy Game Scripting**: `examples/bevy_scripting/`
  - Lua-style behavior scripts for game entities
  - Hot-reload F# without recompiling Rust

- **Ratatui Terminal Layout**: `examples/ratatui_layout/`
  - Define TUI layouts in functional style
  - Declarative UI composition

- **Web Server Validation**: `examples/web_server/`
  - Axum endpoints with F# validation rules
  - Change business logic without rebuilding

- **Neural Net Configuration**: `examples/burn_config/`
  - Define model architectures in typed F#
  - Type-safe hyperparameter tuning

### 🔥 Fusion (Rust Interop)

Advanced patterns mixing Fusabi and Rust:

- **Host Function Callbacks**: `examples/host_callbacks/`
  - Calling Rust from F#, calling F# from Rust
  - Higher-order functions across the boundary

- **.NET Compatibility**: `examples/interop_net/`
  - Same script runs on Fusabi VM and .NET CLR
  - Proof of syntax compatibility

- **Computation Expressions**: `examples/computations/`
  - Custom DSLs with builder patterns
  - Monadic workflows in embedded scripting

## Serving Suggestions

All examples include:
- 📖 README with explanation
- ✅ Working code (tests pass)
- 🎯 Clear use case

Run any example:
```bash
cargo run -- run examples/<category>/<example>.fsx
```

## Contribute Your Recipe

Found a spicy new pattern? Submit a PR to add your recipe to the Omakase!

---

**Fusabi**: Small. Potent. Functional. 🟢
```

**Validation**:
```bash
cat docs/OMAKASE.md
# Should have playful, branded voice

cat examples/README.md
# Should reference Omakase concept
```

---

### Task 7.4: Design and Create Logo Assets (Issue 12)
**Description**: Create SVG logo and various asset formats.

**Deliverables**:
- `assets/logo.svg` (vector logo)
- `assets/icon.ico` (Windows binary icon)
- `assets/social_preview.png` (1280x640 for GitHub)

**Files to Create/Modify**:
- `assets/logo.svg` (new)
- `assets/icon.ico` (new)
- `assets/social_preview.png` (new)
- `assets/README.md` (explain assets)

**Design Concept**:
- Stylized "F" merging with leaf/wasabi shape
- Minimalist, geometric, flat
- Primary color: `#99CC33` (Wasabi Green)

**SVG Implementation** (simplified example):
```xml
<!-- assets/logo.svg -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <defs>
    <linearGradient id="wasabi" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#99CC33;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#78A659;stop-opacity:1" />
    </linearGradient>
  </defs>

  <!-- Abstract F shape merging with leaf -->
  <path fill="url(#wasabi)" d="M20,20 L20,80 L30,80 L30,55 L60,55 L60,45 L30,45 L30,30 L70,30 L70,20 Z M65,25 Q75,35 75,50 T65,75 L55,70 Q60,60 60,50 T55,30 Z"/>

  <text x="50" y="95" font-family="Inter, sans-serif" font-size="12" fill="#1E1E1E" text-anchor="middle" font-weight="bold">FUSABI</text>
</svg>
```

**Note**: For production logo, consider using a design tool or AI image generator with prompt:
> "Vector logo for 'Fusabi' programming language. Minimalist green shape resembling an abstract letter F merging with a leaf or wasabi dollop. Flat design, geometric, hex color #99CC33. Professional tech logo."

**Social Preview**:
- 1280x640px PNG
- Dark background `#1E1E1E`
- Logo centered
- Text: "Fusabi - Small. Potent. Functional."
- Color palette visual

**Validation**:
```bash
ls -lh assets/
# Should show logo.svg, icon.ico, social_preview.png

# View SVG
open assets/logo.svg
# OR
firefox assets/logo.svg
```

---

### Task 7.5: README Brand Voice Overhaul (Issue 13)
**Description**: Rewrite README with punchy, confident brand voice.

**Deliverables**:
- New headline: "Fusabi - Small. Potent. Functional."
- Value proposition focused on "why"
- Spicy feature descriptions
- Logo embedded at top
- Updated badges with brand colors

**Files to Create/Modify**:
- `README.md` (major rewrite)

**Implementation**:
```markdown
<div align="center">
  <img src="assets/logo.svg" width="120" alt="Fusabi Logo">

  # Fusabi

  **Small. Potent. Functional.**

  [![Build](https://img.shields.io/github/actions/workflow/status/youruser/fusabi/ci.yml?style=flat&color=99CC33)](https://github.com/youruser/fusabi/actions)
  [![Crates.io](https://img.shields.io/crates/v/fusabi?style=flat&color=99CC33)](https://crates.io/crates/fusabi)
  [![License](https://img.shields.io/badge/license-MIT-99CC33?style=flat)](LICENSE)
</div>

---

## Why Fusabi?

**Rust is hard. Configuration shouldn't be.**

You've built a killer Rust app. Now you need:
- Config files that don't suck
- User scripts without a full VM
- Hot-reload logic without recompiling

Enter Fusabi: A typed, functional scripting layer for Rust apps. No bloat. No runtime. Just clean, embeddable F# syntax with Lua-class performance.

## What You Get

🟢 **Typed** - Don't guess. Know. Hindley-Milner inference catches errors before runtime.

🦀 **Embedded** - Fits inside your binary. Zero-copy FFI with Rust. Sub-millisecond startup.

🍣 **Fast** - Lua-class performance. Mark-and-sweep GC. Bytecode caching.

🔥 **F# Compatible** - Write once, run on Fusabi VM *and* .NET CLR. Same syntax.

## Quick Start

```rust
use fusabi::Engine;

let engine = Engine::new();
let result = engine.eval(r#"
    let double x = x * 2
    [1; 2; 3] |> List.map double
"#)?;

println!("{:?}", result); // [2, 4, 6]
```

## Show Me the Spice 🌶️

**Game Scripting** (Bevy):
```fsharp
// behavior.fsx - Hot-reload entity logic
let speed = time * 2.0
let newPos = (radius * cos speed, radius * sin speed)
newPos
```

**Web Validation** (Axum):
```fsharp
// validation.fsx - Business rules in F#
if age < 18 then
    Error "Must be 18 or older"
else if not (email |> String.contains "@") then
    Error "Invalid email"
else
    Ok user
```

**Neural Net Config** (Burn):
```fsharp
// model.fsx - Typed architecture definitions
{ layers = [
    Linear (784, 128)
    ReLU
    Dropout 0.2
    Linear (128, 10)
  ]
  optimizer = Adam { lr = 0.001 }
}
```

## The Omakase 🍣

Explore [The Omakase](docs/OMAKASE.md) for hand-picked recipes showcasing Fusabi's power.

## Installation

```bash
cargo add fusabi
```

Or build from source:
```bash
git clone https://github.com/youruser/fusabi
cd fusabi
cargo build --release
```

## Benchmarks

| Language | fib(30) | sieve(10k) | binary_trees(10) |
|----------|---------|------------|------------------|
| **Fusabi** | 45ms | 32ms | 78ms |
| Rhai | 89ms | 67ms | 145ms |
| Lua | 42ms | 29ms | 71ms |

Fusabi is within 10% of Lua performance while offering full type safety.

## Philosophy

**Small**: Sub-500KB binary. Minimal dependencies.

**Potent**: Hindley-Milner types. Pattern matching. First-class functions.

**Functional**: Immutable by default. Algebraic data types. Computation expressions.

Like wasabi: A little goes a long way.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture details and development setup.

## License

MIT

---

<div align="center">
  Made with 🟢 by the Fusabi community
</div>
```

**Validation**:
```bash
cat README.md | head -30
# Should have logo, punchy headline, brand voice

grep -c "Small. Potent. Functional" README.md
# Should appear multiple times
```

---

## Definition of Done
- [ ] `docs/BRANDING.md` with color palette, typography, emoji guide
- [ ] CLI colorized with brand colors (green success, orange errors)
- [ ] ASCII art banner in `fus --help` and `fus repl`
- [ ] `docs/OMAKASE.md` cookbook created with brand voice
- [ ] `examples/README.md` rewritten with Omakase concept
- [ ] Logo assets created (`logo.svg`, `icon.ico`, `social_preview.png`)
- [ ] README rewritten with punchy brand voice
- [ ] All badges updated with brand colors
- [ ] Documentation reviewed for consistency
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws7-branding-identity"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws7"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "docs/BRANDING.md" --memory-key "swarm/fusabi-gem/ws7/branding-guide"
npx claude-flow@alpha hooks post-edit --file "rust/fusabi/src/main.rs" --memory-key "swarm/fusabi-gem/ws7/cli-styling"
npx claude-flow@alpha hooks post-edit --file "docs/OMAKASE.md" --memory-key "swarm/fusabi-gem/ws7/omakase"
npx claude-flow@alpha hooks post-edit --file "assets/logo.svg" --memory-key "swarm/fusabi-gem/ws7/logo"
npx claude-flow@alpha hooks post-edit --file "README.md" --memory-key "swarm/fusabi-gem/ws7/readme"
npx claude-flow@alpha hooks notify --message "Branding and visual identity complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws7-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 3-4 days
**Complexity**: Medium (design-heavy)

## Task Breakdown by Time:
- Task 7.1 (Visual Identity): 0.5 day
- Task 7.2 (CLI Styling): 1 day
- Task 7.3 (Omakase Cookbook): 0.5 day
- Task 7.4 (Logo Assets): 1 day (most time for design iteration)
- Task 7.5 (README Overhaul): 0.5 day

## References
- [GitHub Social Preview Guidelines](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/customizing-your-repositorys-social-media-preview)
- [Colored Rust Crate](https://docs.rs/colored/)
- [Yansi Rust Crate](https://docs.rs/yansi/)
- [SVG Optimization](https://jakearchibald.github.io/svgomg/)

## Notes
- **Design Skills**: Task 7.4 (logo) may require design tools or AI image generation
- **Brand Consistency**: All future PRs should follow brand guidelines in `docs/BRANDING.md`
- **Marketing**: The "Omakase" concept is strong differentiation for Fusabi
- **User Experience**: Colorful CLI makes Fusabi feel polished and professional

## File Conflicts
- **Minor**: WS6 also touches `examples/README.md`
  - Solution: WS6 creates examples, WS7 adds brand voice to README
  - Coordinate: WS7 can run after WS6 examples are structured
- **No other conflicts**: All other changes are to new files or standalone files

## Parallelization Strategy
- **Tasks 7.1, 7.3, 7.5**: Can be done by content writer/docs person
- **Task 7.2**: Requires Rust developer (CLI coding)
- **Task 7.4**: Requires designer or AI image generation
- **Best approach**: Split into 2 sub-agents:
  1. Content/Docs agent: 7.1, 7.3, 7.5
  2. Implementation agent: 7.2, 7.4
