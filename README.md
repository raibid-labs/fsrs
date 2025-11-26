<div align="center">
  <img src="assets/logo.png" alt="Fusabi Logo" width="200">
  <h1>Fusabi 🟢</h1>
  <h3>Small. Potent. Functional.</h3>
</div>

[![CI](https://github.com/fusabi-lang/fusabi/actions/workflows/ci.yml/badge.svg)](https://github.com/fusabi-lang/fusabi/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/fusabi.svg)](https://crates.io/crates/fusabi)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/fusabi-lang/fusabi.svg)](https://github.com/fusabi-lang/fusabi/stargazers)

---

**Rust is hard. Configuration shouldn't be.**

Fusabi is a high-performance embedded scripting engine that brings typed, functional programming to your Rust applications. Write F# logic that binds directly to your application's ABI—no bloat, no compromise, no guesswork.

Think of it as the wasabi to your Rust sushi: **small kick, big impact**. 🍣

## Why Fusabi?

**Don't guess. Know.** Static types catch bugs before runtime. Pattern matching makes logic crystal clear. F#'s functional style eliminates whole classes of concurrency bugs.

**Fits inside your binary.** Embed Fusabi in your Rust app—no separate runtime, no heavy dependencies. Just add the crate and start scripting.

**Fast enough to forget it's there.** Stack-based bytecode VM with ahead-of-time compilation. Startup in microseconds, not milliseconds.

**Safe host interop.** Re-entrant API designed for production Rust. Call Rust from F#, call F# from Rust. No unsafe blocks required. 🦀

## Quick Taste

```bash
# 1. Write Logic (Standard F#)
# config.fsx
type Server = { host: string; port: int }

let validate_server server =
    match server.port with
    | p when p > 1024 && p < 65535 -> Some server
    | _ -> None

let my_server = { host = "localhost"; port = 8080 }
validate_server my_server

# 2. Run It
fus run config.fsx

# 3. Grind to Bytecode (AOT compilation)
fus grind config.fsx
# → config.fzb (binary bytecode)

# 4. Deploy Bytecode
fus run config.fzb  # Faster startup, same logic
```

## What You Get

### 🟢 Typed & Functional
F# dialect with the good stuff: records, discriminated unions, pattern matching, higher-order functions. No `any`, no `undefined`, no runtime surprises.

### ⚡ High-Performance VM
Stack-based bytecode interpreter with AOT compilation. `.fzb` bytecode loads in microseconds. Lua-class performance, F# ergonomics.

### 📦 Batteries Included
Built-in `List`, `String`, `Option` modules. Pipeline operator `|>`. No hunting for std libs or fighting with imports.

### 🦀 Rust-Native Interop
Safe, re-entrant API designed for embedding. Call Rust functions from F#. Expose F# logic to Rust. Zero-copy when it matters. Type-safe always.

### 🔧 Developer-First Tooling
`fus run` for instant iteration. `fus grind` for production builds. Clear error messages. No ceremony, no config files unless you want them.

## Current Status

**Version**: 0.12.0
**Phase**: Production-Ready Embedding

All core features shipped. Host interop tested. Bytecode format stable. Ready for embedding in production applications.

## Get Started

```bash
# Clone & Build
git clone https://github.com/fusabi-lang/fusabi.git
cd fusabi
just bootstrap  # Sets up dev environment
just build      # Compiles Fusabi

# Take It for a Spin
fus run examples/hello.fsx           # Instant gratification
fus run examples/stdlib_demo.fsx     # See the std lib in action
fus grind examples/fibonacci.fsx     # AOT compile to bytecode
fus run examples/fibonacci.fzb       # Run the bytecode

# Explore More
ls examples/  # 30+ examples covering all features
```

### Installation (Coming Soon)
```bash
cargo install fusabi  # Not yet published, but soon™
```

## Embedding in Rust

### Basic Usage

```rust
use fusabi::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    // Register a host function
    engine.register_fn1("double", |x| {
        let n = x.as_int().unwrap_or(0);
        Ok(fusabi::Value::Int(n * 2))
    });

    // Execute script
    let result = engine.eval("double 21")?;
    assert_eq!(result.as_int(), Some(42));

    Ok(())
}
```

### Bytecode Compilation API

**New in v0.12.0:** Compile Fusabi scripts to bytecode for production deployments.

```rust
use fusabi::{compile_to_bytecode, execute_bytecode};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile source to bytecode
    let source = r#"
        let factorial n =
            if n <= 1 then 1
            else n * factorial (n - 1)
        in
        factorial 10
    "#;

    let bytecode = compile_to_bytecode(source)?;

    // Save to file
    fs::write("factorial.fzb", &bytecode)?;

    // Load and execute
    let bytecode = fs::read("factorial.fzb")?;
    let result = execute_bytecode(&bytecode)?;

    println!("Result: {:?}", result); // Int(3628800)
    Ok(())
}
```

### Production Pattern: Bytecode Caching

```rust
use fusabi::{compile_file_to_bytecode, execute_bytecode};
use std::path::Path;
use std::fs;

fn load_or_compile(source_path: &str, cache_path: &str) -> Vec<u8> {
    // Check if cached bytecode exists and is fresh
    if Path::new(cache_path).exists() {
        let source_time = fs::metadata(source_path).unwrap().modified().unwrap();
        let cache_time = fs::metadata(cache_path).unwrap().modified().unwrap();

        if cache_time > source_time {
            return fs::read(cache_path).unwrap();
        }
    }

    // Compile and cache
    let bytecode = compile_file_to_bytecode(source_path).unwrap();
    fs::write(cache_path, &bytecode).unwrap();
    bytecode
}

fn main() {
    let bytecode = load_or_compile("config.fsx", "/tmp/config.fzb");
    let result = execute_bytecode(&bytecode).unwrap();
}
```

**Why bytecode?**
- 3-5x faster startup time
- 60-80% smaller file size
- Production-ready caching patterns
- No compilation overhead in hot paths

See the [Embedding Guide](docs/embedding-guide.md) for production patterns and performance optimization.

## Learn More

### Core Documentation
- **[Language Spec](docs/02-language-spec.md)** - What F# features are supported? (Spoiler: most of them)
- **[Bytecode Format](docs/bytecode-format.md)** - The `.fzb` binary format specification
- **[Embedding Guide](docs/embedding-guide.md)** - Production deployment with bytecode compilation
- **[Host Interop](docs/host-interop.md)** - Embedding Fusabi in your Rust app
- **[Standard Library](docs/stdlib-implementation.md)** - What's in the box

### Project Info
- **[Roadmap](docs/roadmap.md)** - What's next for Fusabi
- **[Setup Guide](docs/setup.md)** - Get your dev environment running
- **[Branding Guidelines](docs/BRANDING.md)** - Colors, logos, and brand voice
- **[Full Docs Index](docs/toc.md)** - Everything else

## Use Cases

**Configuration DSLs**: Type-safe config files that are actually pleasant to write.
**Plugin Systems**: Let users extend your app without exposing your entire API.
**Observability Agents**: Fast, type-safe data pipelines (see [Hibana](https://github.com/raibid-labs/hibana)).
**Game Scripting**: Fast enough for game logic, safe enough you won't cry debugging.
**Build Tools**: Express complex build logic in a real language, not bash.
**Data Pipelines**: Functional pipelines with compile-time guarantees.

If you're embedding Lua but miss types, or using JavaScript but hate the ecosystem, Fusabi might be your speed.

## Real-World Example: Hibana

[Hibana](https://github.com/raibid-labs/hibana) is an observability agent that uses Fusabi for pipeline configuration:

```rust
// Hibana: Compile pipeline configs to bytecode at startup
let pipeline_bytecode = compile_file_to_bytecode("config/pipeline.fsx")?;

// Execute on each event (microsecond latency)
let result = execute_bytecode(&pipeline_bytecode)?;
```

**Performance:**
- Development: Hot-reload `.fsx` files for fast iteration
- Production: Load `.fzb` bytecode for microsecond startup
- Result: Best of both worlds

## Contributing

Found a bug? Want a feature? Have a wild idea? **Open an issue.**
Ready to code? Check the [roadmap](docs/roadmap.md) for what's next.

We're friendly. Promise.

## License

MIT License - Use it, abuse it, ship it. See [LICENSE](LICENSE) for legalese.

## Community

- **Issues**: [GitHub Issues](https://github.com/fusabi-lang/fusabi/issues)
- **Discussions**: [GitHub Discussions](https://github.com/fusabi-lang/fusabi/discussions)
- **Stars**: If you dig it, star it. We're vain like that.

[![Star History Chart](https://api.star-history.com/svg?repos=fusabi-lang/fusabi&type=Date)](https://star-history.com/#fusabi-lang/fusabi&Date)

---

<div align="center">
  <sub>Made with 🟢 and a healthy dose of Rust. <br/>
  Wasabi not included.</sub>
</div>
