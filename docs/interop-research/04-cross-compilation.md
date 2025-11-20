# Cross-Platform Compilation and Portability for F# Code

**Research Date:** 2025-11-19
**Target Context:** FSRS (F# Script Runtime System) - Understanding F# cross-compilation strategies
**Scope:** Native AOT, transpilation targets, mobile/embedded platforms, performance trade-offs

---

## Executive Summary

F# offers multiple cross-platform compilation strategies across the .NET ecosystem and beyond. This research identifies **seven primary compilation paths**, ranging from traditional JIT compilation on .NET Core to transpilation to JavaScript, Python, Rust, and Dart via Fable. Each approach presents distinct trade-offs in performance, startup time, portability, and ecosystem compatibility.

**Key Findings:**

1. **Native AOT** (.NET 8+) provides fast startup and reduced memory footprint but has **limited F# compatibility** due to reflection-heavy F# runtime features
2. **Fable** transpiler enables F# to target JavaScript, TypeScript, Python, Rust, and Dart, making F# a true multi-platform DSL
3. **WebAssembly** via Bolero/Blazor allows F# to run in browsers with MVU architecture patterns
4. **Mobile development** through .NET MAUI is primarily C#-focused; F# support is limited
5. **Embedded/IoT** via .NET nanoFramework supports C# only; F# is not currently supported
6. **JIT compilation** remains the most robust F# execution model with full language feature support
7. **Performance trade-offs** between JIT and AOT favor different use cases: JIT for long-running apps, AOT for serverless/CLI tools

---

## 1. .NET Core/5+: Traditional Cross-Platform Runtime

### Overview

F# runs natively on .NET Core (now .NET 5+), providing full cross-platform support across Windows, macOS, and Linux with complete language feature compatibility.

### Supported Platforms

- **Windows**: x64, ARM64
- **macOS**: x64 (Intel), ARM64 (Apple Silicon)
- **Linux**: x64, ARM64, ARM32
- **Container platforms**: Docker, Kubernetes (Linux containers)

### Compilation Model

**Two-stage compilation:**
1. F# compiler (`fsc`) compiles `.fs` source to IL (Intermediate Language) bytecode
2. Runtime JIT compiler converts IL to native machine code at execution time

### Performance Characteristics

**F# 8 Improvements (November 2023):**
- Incremental builds via reference assemblies for large project graphs
- CPU parallelization of compiler process
- Zero-allocation optimizations for `ValueOption` functions (16x performance improvement: 0.17ns vs 2.77ns for `None` mapping)
- Lambda allocation reduction in core library functions

**F# 9 Improvements (2024):**
- Nullable reference types for safer C# interop
- Optimized integral ranges for faster `for` loops
- Optimized equality checks avoiding boxing
- Enhanced resilience and type safety

**.NET 9 Runtime Enhancements:**
- 15% faster startup times with reduced CPU usage
- 35% faster JSON serialization
- Dynamic adaptation to application sizes (DATAS) GC enabled by default
- Advanced PGO (Profile-Guided Optimization) for better inlining and loop optimization
- AVX10.1 support with 500+ new SIMD APIs
- Enhanced ARM64 vectorization

### Use Cases

- **Web services**: ASP.NET Core with Giraffe, Saturn, Falco frameworks
- **Backend services**: Long-running applications, microservices
- **Data processing**: High-performance computation with full .NET library access
- **Cross-platform tools**: CLI utilities, build tools, automation

### Strengths

- Full F# language support (type inference, computation expressions, reflection)
- Complete .NET ecosystem access (NuGet packages, libraries)
- Excellent debugging and profiling tools
- Strong IDE support (Visual Studio, VS Code, Rider)
- Runtime optimization via tiered JIT compilation

### Limitations

- Requires .NET runtime installation on target machines
- Larger deployment size compared to native executables
- JIT compilation overhead at startup (5-50ms for typical apps)
- Not suitable for extremely resource-constrained environments

---

## 2. Native AOT Compilation (.NET 8+)

### Overview

Native Ahead-of-Time (AOT) compilation produces self-contained native executables without requiring the .NET runtime. Introduced in .NET 7, significantly enhanced in .NET 8+.

### Supported Platforms

**.NET 8:**
- Windows: x64, ARM64
- Linux: x64, ARM64
- macOS: x64, ARM64
- iOS/tvOS/MacCatalyst: ARM64 (experimental)
- Android: x64, ARM64 (experimental)

**.NET 9+:**
- Adds x86 on Windows and ARM on Linux

### Compilation Process

```bash
# Enable Native AOT in .csproj
<PublishAot>true</PublishAot>

# Publish command
dotnet publish -c Release
```

**Build stages:**
1. F# source → IL bytecode (via `fsc`)
2. IL bytecode → Native machine code (via ILC/CoreRT AOT compiler)
3. Trimming: Unused code elimination
4. Linking: Single-file executable generation

### F# Compatibility Issues

**Critical Limitations for F#:**

1. **Reflection-heavy runtime**: F# string functions use reflection and codegen, making them "slow and AOT and linker unfriendly and liable to crash on runtime" ([Issue #919](https://github.com/fsharp/fslang-suggestions/issues/919))

2. **FSharp.Core incompatibilities**: Many core F# library functions rely on runtime code generation incompatible with AOT constraints

3. **Real-world failures**: Developer reports indicate compilation failures with popular F# frameworks:
   - Giraffe (web framework): Compilation crashes
   - Avalonia.FuncUI (UI framework): Runtime errors
   - Basic functionality like `printf` requires manual `Rd.xml` configuration

4. **No dynamic features**:
   - No `Assembly.LoadFile` or dynamic assembly loading
   - No `System.Reflection.Emit`
   - Limited support for `System.Linq.Expressions` (interpreted only, slower)

5. **Trimming sensitivity**: F#'s structural type system and generic specialization can trigger over-aggressive trimming

### Performance Benefits

**When it works:**
- **Startup time**: Near-instant (0-5ms vs 50-500ms for JIT)
- **Memory footprint**: 30-40% reduction vs framework-dependent deployment
- **Deployment size**: Self-contained single executable
- **Predictable performance**: No JIT warmup period

**.NET 9 Native AOT improvements:**
- Thread-local statics can be inlined (far fewer instructions)
- Better code generation for struct-based generics
- Improved trimming analysis

### Use Cases (When F# Compatible)

- CLI tools with minimal dependencies
- Serverless functions (AWS Lambda, Azure Functions)
- Microservices with fast cold-start requirements
- Embedded scenarios where runtime size matters

### Current Status for F#

**Not recommended for production F# applications** due to extensive compatibility issues. Most F# codebases use language features incompatible with AOT constraints.

**Possible future improvements:**
- F# compiler team exploring AOT-friendly string function implementations
- Community work on AOT-compatible subset of FSharp.Core
- Potential for "F# Lite" dialect targeting AOT scenarios

### Workarounds

For F# developers needing AOT benefits:
1. **Minimize reflection usage**: Avoid dynamic type inspection
2. **Use Source Generators**: Replace runtime codegen with compile-time generation
3. **Manual trimming configuration**: Specify preserved types in `Rd.xml`
4. **Test extensively**: AOT issues often surface only at runtime
5. **Consider C# interop layer**: Write AOT-sensitive code in C#, call from F#

---

## 3. Fable: F# to Multi-Language Transpiler

### Overview

Fable is a revolutionary transpiler that compiles F# source code to **JavaScript, TypeScript, Python, Rust, and Dart**, enabling F# as a universal programming DSL across multiple platforms and ecosystems.

### Supported Compilation Targets

| Target | Status | Primary Use Cases |
|--------|--------|-------------------|
| JavaScript | Stable (v5.0) | Web frontend, Node.js backend |
| TypeScript | Stable | Type-safe web applications |
| Python | Beta | Data science, ML, IoT (BBC micro:bit) |
| Rust | Experimental | Systems programming, performance-critical code |
| Dart | Experimental | Flutter mobile applications |

### Architecture

**Compilation Pipeline:**
```
F# Source Code
    ↓ (FSharp Compiler Services)
F# AST (Abstract Syntax Tree)
    ↓ (Fable Core)
Fable IR (Intermediate Representation)
    ↓ (Language-Specific Backend)
Target Language Code (JS/TS/Python/Rust/Dart)
```

### JavaScript/TypeScript Target (Primary)

**Version:** Fable 5.0 (latest stable)

**Key Features:**
- ES2015+ standard compliance
- Readable, debuggable output
- Full NPM ecosystem access
- Webpack/Vite/Rollup integration
- Source map support

**Performance:**
- Zero-cost abstractions for many F# features
- Efficient handling of discriminated unions (compiled to tagged objects)
- Lazy evaluation support
- Asynchronous computation expressions → JavaScript Promises

**Ecosystem Integration:**
- Import NPM packages directly in F# code
- Type-safe bindings via `Fable.Import` libraries
- React integration via Feliz/Elmish
- Browser API access

**Example Output Quality:**
```fsharp
// F# Input
let add x y = x + y
let result = [1..10] |> List.map (add 5)

// JavaScript Output (readable, idiomatic)
function add(x, y) {
  return x + y;
}
const result = [1,2,3,4,5,6,7,8,9,10].map(x => add(5, x));
```

### Python Target

**Status:** Beta (Fable 4+)

**Use Cases:**
- **Data Science**: NumPy, Pandas integration
- **Web Services**: Flask, FastAPI backends
- **Visualization**: Matplotlib, Tkinter
- **IoT**: BBC micro:bit programming

**Compatibility:**
- Python 3.7+ target
- Type hints generation for better IDE support
- Interop with existing Python libraries

**Limitations:**
- Smaller ecosystem than JavaScript target
- Less mature tooling integration
- Performance overhead for some F# idioms

### Rust Target

**Status:** Experimental (by ncave)

**Goals:**
- Performance-critical algorithms (e.g., ray tracing achieving native Rust performance)
- Systems programming with F# type safety
- WebAssembly via Rust toolchain

**Challenges:**
- Ownership/borrowing model mismatch with F#'s GC semantics
- Lifetime annotation complexity
- Ongoing research project, not production-ready

### Dart Target

**Status:** Experimental

**Primary Goal:** Flutter mobile applications using F#

**Rationale:**
- Dart is typed with null-safety (similar to C#/F#)
- Flutter's popularity for cross-platform mobile
- Better type mapping than JavaScript

**Current State:**
- Proof-of-concept stage
- Limited library bindings
- Community exploration ongoing

### TypeScript Generation

**Critical Feature Request:**
- Good TypeScript type generation identified as "key missing piece" for Fable adoption
- Many teams build in TypeScript; Fable-generated types would enable F# integration
- Active community discussion on implementation approaches

### Fable Versions (2024 Status)

- **Fable 5.0-alpha.9**: Latest alpha with multi-target improvements
- **Fable 4.x**: "Snake Island" release adding Python/Rust/Dart
- **Fable 3.7.x**: Stable JavaScript-only releases

### Performance Considerations

**JavaScript Target:**
- Comparable to hand-written TypeScript
- Virtual DOM frameworks (React/Elmish) add typical SPA overhead
- Async performance excellent (native Promise mapping)

**Python Target:**
- Slower than native Python for compute-heavy tasks
- Library interop overhead minimal
- Best for orchestration/glue code rather than hot loops

**Rust Target:**
- Can achieve native Rust performance for algorithmic code
- Requires careful F# code structure to map cleanly to Rust

### Use Cases

1. **Universal DSL**: Write business logic once, deploy to web (JS), backend (Python), mobile (Dart)
2. **Type-safe web development**: F# for frontend with full type safety
3. **Multi-platform libraries**: Publish npm, PyPI, crates.io from single F# codebase
4. **Algorithm prototyping**: Design in F#, transpile to target platform

### Strengths

- Readable output code (debuggable, reviewable)
- Full F# language support (pattern matching, computation expressions)
- Ecosystem access for each target platform
- Active community and tooling (Fable REPL, IDE plugins)

### Limitations

- Requires understanding target platform idioms for optimal output
- Library bindings need manual creation or community contribution
- Debugging happens in generated code (though source maps help)
- Non-JavaScript targets less mature, limited production usage

---

## 4. WebAssembly: F# in the Browser

### Overview

F# can compile to WebAssembly (WASM) via two primary paths:
1. **Bolero/Blazor**: F# on .NET runtime compiled to WASM
2. **Fable + Rust/C**: F# transpiled to Rust/C, then compiled to WASM

### Bolero: Blazor-Based WASM

**Project:** [fsbolero.io](https://fsbolero.io/)
**Architecture:** F# → IL → Mono runtime (compiled to WASM) → Browser

#### How It Works

**Not a direct F# → WASM compiler**, but rather:
1. F# code compiles to .NET IL bytecode
2. Mono runtime (itself compiled to WASM) executes the IL in the browser
3. Blazor WebAssembly host manages runtime loading and JavaScript interop

**Key Implication:** The entire .NET runtime ships to the browser, resulting in larger initial payloads.

#### MVU Architecture (Model-View-Update)

Bolero is designed around **Elmish MVU pattern**:

```fsharp
type Model = { Count: int }

type Message =
    | Increment
    | Decrement

let update message model =
    match message with
    | Increment -> { model with Count = model.Count + 1 }
    | Decrement -> { model with Count = model.Count - 1 }

let view model dispatch =
    div [] [
        button [on.click (fun _ -> dispatch Decrement)] [text "-"]
        text (string model.Count)
        button [on.click (fun _ -> dispatch Increment)] [text "+"]
    ]
```

#### Key Features

1. **HTML Combinators**: Type-safe HTML generation in F# (no string templates)
2. **Hot Reload**: HTML template files reload without recompilation
3. **Routing**: URL routing defined as F# discriminated unions
4. **Remoting**: Call ASP.NET Core backend as async F# functions (no REST boilerplate)
5. **F# Signature Stripping**: Reduces bundle size by removing F# metadata

#### Performance Characteristics

**Initial Load:**
- **Payload size**: 1.5-3 MB (Mono runtime + .NET assemblies)
- **Compression**: Brotli compression reduces by ~60-70%
- **Load time**: 2-5 seconds on first visit (cached afterward)

**Runtime Performance:**
- **UI updates**: Comparable to React/Vue (virtual DOM reconciliation)
- **Computation**: Near-native speed after JIT warmup
- **Memory**: Higher baseline (~10-20 MB) due to .NET runtime

**Optimizations:**
- Lazy loading of assemblies
- Tree shaking via IL trimming
- AOT compilation (experimental in .NET 8+)

#### Deployment Options

1. **Client-side only**: Fully static hosting (CDN, S3, GitHub Pages)
2. **Hybrid with server**: Blazor Server fallback for slow connections
3. **Progressive Web App (PWA)**: Offline-capable applications

#### Requirements

- .NET SDK 8.0.100+ (6.0.101 minimum)
- Modern browser with WASM support (all evergreen browsers)
- HTTPS for production (Service Worker requirement)

#### Use Cases

- Line-of-business web applications
- Admin dashboards and data-heavy UIs
- Progressive Web Apps (PWAs)
- Internal tools where initial load time is acceptable

#### Strengths

- Full F# language features in browser
- Strong typing across client-server boundary
- No JavaScript required
- Elmish MVU for predictable state management
- Complete .NET ecosystem access

#### Limitations

- Large initial payload (not suitable for content sites)
- Longer time-to-interactive than JavaScript SPAs
- Browser debugging less mature than JavaScript tools
- SEO requires server-side pre-rendering
- Mobile performance can be sluggish on low-end devices

### Alternative: Fable to WASM via Rust

**Approach:** F# → Rust (via Fable) → WASM (via Rust toolchain)

**Benefits:**
- Much smaller bundles (10-100 KB range)
- Near-native performance
- No runtime overhead

**Challenges:**
- Experimental Fable Rust backend
- Complex interop with JavaScript
- Limited ecosystem support

**Status:** Research/proof-of-concept stage

---

## 5. Mobile Development: Xamarin and .NET MAUI

### Xamarin (End of Life)

**Status:** **Officially deprecated as of May 1, 2024**
**Replacement:** .NET MAUI (Multi-platform App UI)

**Historical F# Support:**
- Xamarin.Forms supported F# for cross-platform mobile (iOS, Android)
- Community-driven bindings and samples
- Limited official documentation for F#

### .NET MAUI (Current)

**Overview:** Evolution of Xamarin.Forms for .NET 6+, targeting Android, iOS, macOS, Windows from a single codebase.

#### F# Support Status

**Current Reality:** **Minimal official F# support**

- All official documentation and templates use C# + XAML
- Community reports limited success with F# MAUI projects
- No official F# templates or guidance from Microsoft
- Third-party efforts exist but lack polish/maintenance

**Why Limited Support?**
1. XAML tooling designed for C#
2. Code-behind model assumes C# semantics
3. Microsoft focus on C# developer experience
4. Smaller F# mobile community compared to web/backend

#### Architecture (C#-Focused)

**Single Project Structure:**
```
MyApp/
  Platforms/
    Android/      # Android-specific code
    iOS/          # iOS-specific code
    MacCatalyst/  # macOS-specific code
    Windows/      # Windows-specific code
  Resources/      # Shared images, fonts, etc.
  MainPage.xaml   # Shared UI definition
  App.xaml.cs     # App logic (C#)
```

**Improvements Over Xamarin:**
- Single project vs. multiple platform projects
- More native platform APIs exposed
- Better hot reload
- Enhanced controls and layouts

#### Alternative: Uno Platform

**Better F# Story (but still C#-primary):**
- Uno Platform mentioned as alternative for .NET ecosystem developers
- Supports Android, iOS, Linux, macOS, Windows, and **Web** (via WASM)
- Reactive-style programming model
- Hundreds of UI components

**F# Status:** Community bindings exist; not officially promoted

#### Fabulous Framework (F# MAUI Wrapper)

**Community Solution:** Fabulous provides F# MVU pattern over Xamarin/MAUI

**Status (2024):** Maintenance uncertain; ecosystem shift to web (Fable) over mobile

#### Recommendations for F# Mobile Development

1. **Use Fable + React Native**: Better F# tooling, active community
2. **Use Fable + Capacitor/Cordova**: Web-to-mobile hybrid approach
3. **Use Flutter + Fable Dart** (experimental): If willing to use cutting edge
4. **Accept C# for mobile**: Use F# for backend, C# for mobile UI

**Reality Check:** F# mobile development is not a first-class experience in 2024. The community has largely shifted focus to web (Fable) and backend (.NET Core) where F# tooling excels.

---

## 6. Embedded Systems and IoT: .NET nanoFramework

### Overview

.NET nanoFramework is a free, open-source platform for writing **C# on microcontrollers** (MCUs) like ESP32, STM32, NXP, TI devices.

### Key Features

- Manage code on constrained devices (typically 256KB+ flash, 64KB+ RAM)
- Visual Studio integration for C# development
- Network connectivity: WiFi, Ethernet, AT modems
- Cloud integrations: Azure IoT Hub, AWS IoT
- Peripheral support: GPIO, I2C, SPI, ADC, PWM

### Supported Hardware

**Out-of-box board support:**
- **ST Microelectronics**: Discovery, Nucleo boards (STM32 series)
- **Espressif**: ESP32, ESP32-C3, ESP32-S2/S3
- **Silicon Labs**: Giant Gecko S1
- **Texas Instruments**: CC3220, CC1352 Launchpads
- **NXP**: MIMXRT1060-EVK
- Plus custom board support

### F# Support Status

**CRITICAL:** .NET nanoFramework does **NOT support F#**

**Reasons:**
1. Platform is explicitly "making it easy to write **C# code** for embedded systems"
2. All documentation, samples, and tooling assume C#
3. nanoFramework is a subset of .NET (no generics, no async/await initially)
4. F# relies heavily on advanced .NET features unavailable in nanoFramework subset
5. MCU resource constraints incompatible with F# runtime requirements

**F# Runtime Overhead:**
- FSharp.Core library size (several hundred KB) exceeds typical MCU flash
- Type inference metadata increases binary size
- Reflection-based features (discriminated unions, records) too costly

### .NET Subset Limitations

**Not supported in nanoFramework:**
- Generics (initially; partial support added later)
- `async`/`await` (initially; added in later versions)
- Full reflection
- LINQ (limited support)
- Large portions of BCL (Base Class Library)

**Implications for F#:**
- Most F# language features depend on these .NET capabilities
- Even if F# compiler targeted nanoFramework, runtime would be incompatible

### Use Cases (C# Only)

- IoT sensors and data collection
- Industrial automation and control
- Home automation projects
- Robotics and maker projects
- Wearables and embedded devices

### Alternatives for F# in Embedded Space

**None that are mature**. Options include:

1. **Fable to Python**: Use F# → Python, run MicroPython on MCUs
   - Larger runtime than nanoFramework
   - Slower performance
   - Better F# language support

2. **Fable to Rust**: Compile F# → Rust, then to embedded targets
   - Experimental Fable backend
   - Excellent performance potential
   - Complex toolchain

3. **Host F# on gateway device**: Run F# on Raspberry Pi, communicate with MCUs
   - Full .NET Core on Linux ARM
   - F# coordinates C/MicroPython on MCUs

**Recommendation:** For embedded/IoT, use C# with nanoFramework or MicroPython directly. Reserve F# for gateway devices and cloud backend.

---

## 7. Performance: JIT vs AOT Trade-offs

### Execution Model Comparison

| Characteristic | JIT (Just-In-Time) | AOT (Ahead-Of-Time) |
|----------------|-------------------|---------------------|
| **Compilation** | At runtime (first execution) | At build time |
| **Startup Time** | 50-500ms (initial JIT) | 0-5ms (instant) |
| **Peak Performance** | Can exceed AOT (PGO) | Good but static |
| **Memory Usage** | Higher (JIT compiler + code) | Lower (no JIT) |
| **Binary Size** | Smaller (IL only) | Larger (native code) |
| **Optimization** | Dynamic, profile-guided | Static, limited runtime data |
| **Portability** | IL portable, runtime needed | Platform-specific binary |
| **Adaptability** | Optimizes for actual workload | Optimizes for predicted usage |

### JIT Compilation Deep Dive

#### Tiered Compilation (.NET Core 3.0+)

**Multi-tier approach:**
1. **Tier 0**: Quick compilation (minimal optimization, fast startup)
2. **Tier 1**: Full optimization (triggered after ~30 invocations)

**Process:**
```
Method first called
  → Compile with Tier 0 (fast, low optimization)
  → Install call counting stub
  → Method runs (count increments each call)
  → Count reaches threshold (e.g., 30)
  → Queue for Tier 1 re-compilation
  → Compile with full optimizations
  → Replace Tier 0 code with Tier 1 code
```

**Benefits:**
- Fast startup (no full optimization delay)
- Peak performance for hot paths (Tier 1 for frequently-called methods)
- Efficient for rarely-called code (stays in Tier 0, saves compilation time)

#### Dynamic Profile-Guided Optimization (PGO)

**Enabled by default in .NET 8+**

**How it works:**
1. JIT instruments code during initial runs
2. Collect runtime statistics:
   - Branch frequencies (which `if` branches are taken most often)
   - Virtual call targets (which concrete types are most common)
   - Loop iteration counts
3. Re-optimize code using actual runtime data:
   - Inline hot virtual calls (devirtualization)
   - Eliminate cold branches
   - Unroll loops with predictable counts
   - Optimize for common data patterns

**Performance gains:**
- 10-30% speedup for typical workloads
- Even higher (50%+) for polymorphic code
- Adaptive to changing workloads (re-profiles over time)

**.NET 9 PGO Enhancements:**
- Expanded patterns recognized for profiling
- Better loop optimization using profile data
- Improved inlining decisions

#### JIT Optimization Techniques

**Inlining:**
- Replace function calls with function body
- Eliminates call overhead
- Enables further optimizations (constant propagation, dead code elimination)

**Vectorization (SIMD):**
- Detect loops operating on arrays
- Emit SIMD instructions (SSE, AVX, AVX10.1 on x64; NEON on ARM64)
- Process multiple elements per instruction

**Bounds Check Elimination:**
- Prove array indices are in bounds
- Remove runtime checks for proven-safe accesses
- .NET 9: Improved induction variable analysis for loops

**Escape Analysis:**
- Determine if object escapes method scope
- Allocate on stack instead of heap (avoids GC pressure)
- Scalar replacement (break object into fields)

#### .NET 9 JIT Improvements (2024)

**Quantified Gains:**
- 15% faster startup times
- 35% faster JSON serialization
- Improved SIMD utilization (AVX10.1 support, 500+ new APIs)
- Better ARM64 code generation

**New Optimizations:**
- Induction variable widening (better loop performance)
- Enhanced loop recognition (graph-based analyzer)
- Thread-local static inlining
- Reduced allocations in hot paths

### AOT Compilation Deep Dive

#### Compilation Process

**Build-time steps:**
1. **IL Generation**: F#/C# → IL bytecode (standard compilation)
2. **IL Trimming**: Remove unused types/methods (tree shaking)
3. **Native Code Generation**: IL → platform-specific machine code (ILC compiler)
4. **Linking**: Combine with runtime libraries, produce single executable

**Ahead-of-time optimization:**
- Full method inlining (no size budget constraints)
- Cross-assembly optimization
- Dead code elimination
- Constant propagation across boundaries

#### Trimming

**Aggressive approach:**
- Scan entire application dependency graph
- Identify reachable code from entry points
- Remove unreachable types, methods, fields
- Reduce binary size by 50-80%

**Challenges:**
- Reflection breaks static analysis (can't determine reachable code)
- Dynamic type loading incompatible
- Libraries must be trimming-aware

**F# Trimming Issues:**
- Structural typing (records, DUs) generates metadata
- Reflection-based pretty-printing and comparisons
- FSharp.Core extensive use of runtime type information

#### Performance Characteristics

**Strengths:**
- **Startup**: Near-instant (0-5ms vs 50-500ms JIT)
- **Predictability**: No JIT compilation pauses
- **Memory**: 30-40% lower footprint (no JIT compiler resident)
- **Deployment**: Self-contained, no runtime installation

**Weaknesses:**
- **Peak performance**: Can lag behind JIT+PGO for long-running apps
- **Binary size**: Larger (entire runtime + application statically linked)
- **Flexibility**: No runtime adaptation to workload

#### .NET 9 Native AOT Improvements

- Thread-local static inlining (far fewer instructions)
- Better generic code specialization
- Improved startup (cold-start optimizations)
- Enhanced trimming analysis

### Use Case Decision Matrix

| Scenario | Recommended | Reasoning |
|----------|------------|-----------|
| **CLI tools** | AOT | Instant startup critical, short runtime |
| **Web APIs (long-running)** | JIT | PGO optimizes hot paths, startup amortized |
| **Serverless (AWS Lambda)** | AOT | Cold-start time dominates cost/UX |
| **Desktop applications** | JIT | Startup acceptable, benefits from dynamic optimization |
| **Microservices** | AOT or JIT | Depends on instance lifetime and scale-out frequency |
| **Data processing** | JIT | Long-running jobs benefit from tiered compilation |
| **Mobile apps** | AOT (iOS) / JIT (Android) | iOS requires AOT; Android allows JIT |
| **Real-time systems** | AOT | Predictable performance, no JIT pauses |
| **Containers** | AOT | Faster pod startup, lower memory density |

### F#-Specific Considerations

**JIT is strongly favored for F#:**
- Full language feature support
- FSharp.Core designed for JIT runtime
- Reflection-based features (structural equality, comparison) work correctly
- Community tooling and libraries assume JIT

**AOT limitations for F#:**
- Most F# codebases won't compile to AOT without significant refactoring
- FSharp.Core compatibility issues
- Limited community experience/support

**Recommendation:** Use JIT for F# unless specific AOT requirement (e.g., iOS, serverless), in which case consider Fable or C# interop.

---

## 8. Cross-Compilation Strategy Matrix

### Platform Coverage Summary

| Target Platform | Primary Path | Maturity | F# Support | Performance |
|----------------|-------------|----------|------------|-------------|
| **Windows** | .NET JIT | Production | Excellent | Excellent |
| **macOS** | .NET JIT | Production | Excellent | Excellent |
| **Linux** | .NET JIT | Production | Excellent | Excellent |
| **Web (Browser)** | Fable → JS | Production | Excellent | Good |
| **Web (WASM)** | Bolero/Blazor | Production | Very Good | Good |
| **Mobile (iOS)** | .NET MAUI | Production | Limited | Good |
| **Mobile (Android)** | .NET MAUI | Production | Limited | Good |
| **Mobile (Web-based)** | Fable + React Native | Community | Very Good | Good |
| **Server (Native)** | .NET AOT | Beta | Poor | Excellent |
| **Serverless** | .NET AOT | Beta | Poor | Excellent |
| **Data Science** | Fable → Python | Beta | Good | Fair |
| **Systems Programming** | Fable → Rust | Experimental | Fair | Excellent |
| **Embedded/IoT (MCU)** | None | N/A | None | N/A |
| **Embedded/IoT (Linux)** | .NET JIT (ARM) | Production | Excellent | Good |

### Decision Tree

```
┌─────────────────────────────────────┐
│  Which platform(s) to target?      │
└─────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
    Desktop/Server    Web/Mobile
        │                 │
        │                 │
  ┌─────┴─────┐     ┌─────┴─────┐
  │           │     │           │
Long-run  Short-run  Browser   Mobile
  │           │     │           │
  │           │     │           │
.NET JIT  .NET AOT  Fable JS  React Native
  │           │     │           │
  │           │   Bolero      .NET MAUI
Full F#   Limited  (WASM)     (C# mainly)
          F#       Full F#
```

### Recommendations by Use Case

#### 1. Web Application (SPA)
**Best:** Fable → JavaScript + Elmish + React
**Why:** Mature tooling, readable output, full F# support, NPM ecosystem

#### 2. Server-Side API (Long-Running)
**Best:** .NET Core/8+ JIT + ASP.NET Core + Giraffe
**Why:** Full F# features, excellent performance with PGO, robust ecosystem

#### 3. CLI Tool
**Best:** .NET AOT (if F# compatible) or .NET JIT
**Why:** Fast startup, self-contained deployment; use JIT if AOT issues arise

#### 4. Serverless Function
**Best:** .NET AOT (with C# interop) or Fable → Python
**Why:** Cold-start time critical; AOT ideal but F# compatibility challenging

#### 5. Data Processing / ML
**Best:** .NET JIT or Fable → Python
**Why:** JIT for .NET ML libraries (ML.NET); Fable Python for NumPy/scikit-learn

#### 6. Mobile Application
**Best:** Fable → React Native or accept C# with .NET MAUI
**Why:** Better F# tooling with Fable; MAUI is C#-primary

#### 7. Progressive Web App (PWA)
**Best:** Bolero (WASM) or Fable JS
**Why:** Bolero for offline-first, type-safe remoting; Fable for smaller bundles

#### 8. Embedded Linux (RPi, Beaglebone)
**Best:** .NET JIT on Linux ARM
**Why:** Full .NET Core support on ARM Linux, excellent F# compatibility

#### 9. Microcontroller (MCU)
**Best:** Use C#/nanoFramework or MicroPython
**Why:** F# not supported on MCU platforms

#### 10. Multi-Platform Library
**Best:** Fable multi-target (JS/Python/Rust)
**Why:** Single F# codebase, transpile to multiple ecosystems

---

## 9. FSRS Implications and Recommendations

### FSRS Context

FSRS is a Mini-F# dialect with Rust bytecode VM, targeting:
- Embeddable scripting (Lua replacement)
- Terminal emulator configs
- Plugin systems
- Hot-reload development

### Cross-Compilation Relevance

**Core Question:** Should FSRS support multiple backends like Fable?

#### Option A: Single Rust VM Target (Current)
**Pros:**
- Focused scope, single-backend optimizations
- Simpler implementation, faster to MVP
- Rust VM provides excellent embedding story

**Cons:**
- Limited to Rust host applications
- No browser/web platform support
- Can't leverage existing .NET/JS ecosystems

#### Option B: Multi-Backend (Fable-Like)
**Pros:**
- F# as universal DSL (Run FSRS scripts on web, .NET, Python, Rust)
- Reuse FSRS parser/typechecker, swap codegen backend
- Broader adoption potential

**Cons:**
- Massive scope expansion (3-5x complexity)
- Each backend needs specialized codegen
- Testing/maintenance burden multiplied

### Hybrid Recommendation for FSRS

**Phase 1 (Current):** Rust VM only (MVP focus)
- Get single backend rock-solid
- Prove language design and embedding API
- Establish baseline performance

**Phase 2 (Future):** Consider secondary backends
- **Priority 1**: JavaScript (via Fable-style transpiler)
  - Enables FSRS in browser
  - Web-based config editors, playgrounds
  - Largest ecosystem reach

- **Priority 2**: Python (for ML/data science integration)
  - Complements Rust VM for different use cases
  - Leverage existing Python libraries

- **Priority 3**: WebAssembly (via Rust or direct)
  - Browser execution with better performance than JS
  - WASI for server-side WASM

**Architecture for Multi-Backend:**
```
FSRS Source Code
      ↓
    Lexer
      ↓
    Parser → AST
      ↓
  Type Checker → Typed AST
      ↓
  ┌───┴───┬───────┬────────┐
  │       │       │        │
Rust VM  JS Gen  Py Gen  WASM Gen
Bytecode                  (future)
```

**Shared Components:**
- Lexer, Parser, Type Checker (60-70% of compiler)
- Standard library interface (platform-agnostic)

**Backend-Specific:**
- Code generation (Rust bytecode vs. JS vs. Python)
- Runtime libraries (VM vs. transpiled runtime)
- Interop mechanisms

### Learning from Fable's Success

**Key Takeaways:**
1. **IR Layer**: Fable's intermediate representation enables multiple backends
2. **Readable Output**: Transpiled code should be debuggable (no obfuscation)
3. **Ecosystem Integration**: Each backend must feel native to its platform
4. **Community Bindings**: Enable users to create platform-specific bindings

**For FSRS:**
- Design AST/IR to be backend-agnostic from start
- Even if shipping single backend initially, avoid Rust VM assumptions in frontend
- Document IR format for future backend implementers

### Performance Benchmarking Considerations

**If FSRS adds multiple backends:**
- Rust VM: 5-10M ops/sec (target)
- JavaScript transpiled: 1-5M ops/sec (V8 JIT)
- Python transpiled: 0.1-1M ops/sec (CPython)
- WASM (via Rust): 8-12M ops/sec (near-native)

**Trade-offs:**
- Performance variance across backends acceptable if clearly documented
- Users choose backend based on deployment needs, not just speed

---

## 10. Conclusion and Strategic Guidance

### Key Insights

1. **F# is highly portable** across traditional .NET platforms (Windows, macOS, Linux) with excellent JIT performance and full language support.

2. **Native AOT is not viable** for most F# codebases due to reflection-heavy runtime and FSharp.Core incompatibilities. Use C# interop or wait for F# AOT improvements.

3. **Fable is the cross-compilation breakthrough** for F#, enabling JavaScript, Python, Rust, Dart targets with readable output and strong F# language support.

4. **WebAssembly via Bolero** provides F# in browsers with full language features but larger bundles than transpiled JavaScript.

5. **Mobile (MAUI) is C#-first**; F# mobile development best approached via Fable + React Native or accepting C# for UI.

6. **Embedded/IoT (MCU) lacks F# support**; use C#/nanoFramework or Python alternatives. Reserve F# for gateway devices.

7. **JIT vs. AOT trade-offs** favor JIT for long-running F# applications (PGO, full features) and AOT for short-lived, startup-sensitive scenarios (if F# compatible).

8. **.NET 9 brings significant performance gains** (15% startup, 35% JSON, enhanced SIMD) benefiting all F# applications on JIT runtime.

### Strategic Recommendations

**For General F# Development:**
- Default to .NET JIT on .NET 8+ for maximum compatibility and performance
- Use Fable for web frontends (JavaScript) and multi-platform libraries
- Avoid Native AOT unless specific need and willing to debug compatibility issues
- Embrace .NET 9 for latest performance improvements

**For FSRS Project:**
- **Prioritize Rust VM** for embedded scripting use case (Phase 1)
- **Design for multi-backend future** (IR abstraction, platform-agnostic AST)
- **Consider JavaScript backend** (Phase 2+) for web-based tooling and playgrounds
- **Learn from Fable architecture** (shared frontend, swappable backends)

**For Cross-Platform Targeting:**
- Evaluate deployment platform before choosing compilation strategy
- Use decision tree (Section 8) to match use case with optimal path
- Test thoroughly: cross-compilation bugs often surface only at runtime
- Document performance characteristics across backends

### Future Outlook

**2025-2026 Predictions:**
- F# Native AOT support will improve but remain limited compared to C#
- Fable will mature Python/Rust/Dart backends toward production readiness
- .NET MAUI F# support unlikely to improve (C# focus continues)
- WASM will become more viable as Blazor/Bolero optimize bundle sizes
- Embedded F# (MCU) will remain C#-only via nanoFramework

**Long-term (2027+):**
- Potential "F# Lite" subset targeting AOT scenarios
- Cross-platform REPL and scripting via WASM (WASI)
- Standardized F# IR enabling third-party backend development
- F# as DSL for domain-specific transpilation (config languages, DSLs)

---

## 11. Additional Resources

### Official Documentation
- [.NET Native AOT Overview](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
- [F# 8 Announcement](https://devblogs.microsoft.com/dotnet/announcing-fsharp-8/)
- [F# 9 What's New](https://learn.microsoft.com/en-us/dotnet/fsharp/whats-new/fsharp-9)
- [.NET 9 Performance Improvements](https://devblogs.microsoft.com/dotnet/performance-improvements-in-net-9/)

### Fable Resources
- [Fable Official Site](https://fable.io/)
- [Fable GitHub](https://github.com/fable-compiler/Fable)
- [Fable REPL](https://fable.io/repl/) (Try JS/Python/Rust/Dart output)

### Bolero/WebAssembly
- [Bolero Official Site](https://fsbolero.io/)
- [Bolero GitHub](https://github.com/fsbolero/Bolero)
- [F# Compiler in WASM Demo](https://github.com/fsbolero/TryFSharpOnWasm)

### Mobile/Embedded
- [.NET MAUI Overview](https://learn.microsoft.com/en-us/dotnet/maui/)
- [.NET nanoFramework](https://nanoframework.net/)
- [Uno Platform](https://platform.uno/)

### Performance & Benchmarks
- [.NET Performance Blog](https://devblogs.microsoft.com/dotnet/category/performance/)
- [TechEmpower Benchmarks](https://www.techempower.com/benchmarks/)

### Community Discussions
- [F# Native AOT Issues](https://github.com/fsharp/fslang-suggestions/issues/919)
- [Fable Future Discussion](https://github.com/fable-compiler/Fable/discussions/3351)

---

**Document Version:** 1.0
**Last Updated:** 2025-11-19
**Contributors:** Market Trend Analyst (Claude Code)
**Review Status:** Initial Research Complete

