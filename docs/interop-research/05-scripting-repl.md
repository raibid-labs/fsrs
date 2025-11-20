# F# Scripting & REPL Integration Research

**Research Date**: 2025-11-19
**Target**: FSRS (F# Script Runtime System) Design
**Focus Areas**: F# Interactive (FSI), Scripting Capabilities, Notebook Integration, Hot Reload

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [F# Interactive (FSI) Overview](#f-interactive-fsi-overview)
3. [Scripting Mode vs Compiled Projects](#scripting-mode-vs-compiled-projects)
4. [REPL Features & Directives](#repl-features--directives)
5. [NuGet Package Integration](#nuget-package-integration)
6. [Polyglot Notebooks & .NET Interactive](#polyglot-notebooks--net-interactive)
7. [Embedding F# Compiler Service](#embedding-f-compiler-service)
8. [Hot Reload Capabilities](#hot-reload-capabilities)
9. [Development Workflows](#development-workflows)
10. [Performance Characteristics](#performance-characteristics)
11. [FSRS Design Implications](#fsrs-design-implications)
12. [Implementation Recommendations](#implementation-recommendations)

---

## Executive Summary

### Key Findings

F# provides a mature scripting and REPL infrastructure through **F# Interactive (dotnet fsi)** that offers:

1. **Native Scripting Support**: `.fsx` files execute via FSI without compilation overhead
2. **On-the-Fly Compilation**: FSI compiles to IL rather than interpreting, providing near-compiled performance
3. **Rich Package Ecosystem**: Direct NuGet package references via `#r "nuget: Package"` syntax (F# 5+)
4. **Embeddable Compiler**: FSharp.Compiler.Service enables hosting F# as a scripting language
5. **Notebook Integration**: First-class support in Polyglot Notebooks with variable sharing
6. **Limited Hot Reload**: Official .NET Hot Reload doesn't support F#, but workarounds exist

### Strategic Implications for FSRS

FSRS should **selectively adopt** F# scripting patterns while addressing F#'s limitations:

- **Adopt**: Directive-based package loading, embedding API patterns, notebook-style execution
- **Improve**: Hot reload (F# lacks this), startup performance, cross-language interop
- **Differentiate**: Rust-native embedding, lightweight runtime, production-ready hot reload

---

## F# Interactive (FSI) Overview

### What is FSI?

F# Interactive is a **REPL (Read-Eval-Print Loop)** environment that:
- Compiles F# code to IL on-the-fly
- Executes compiled assemblies immediately
- Maintains session state across evaluations
- Provides rich debugging and profiling tools

### Modern Access (2024)

```bash
# .NET SDK integrated
dotnet fsi                    # Launch REPL
dotnet fsi script.fsx         # Execute script
dotnet fsi --help             # View options

# Shebang support (Unix-like systems)
#!/usr/bin/env -S dotnet fsi
```

### Recent Updates (F# 9, 2024)

- **Optional quotation marks**: Simplified directive syntax
- **Enhanced #help**: Function-specific documentation via `#help List.map`
- **Improved error messages**: Better diagnostics with location information

---

## Scripting Mode vs Compiled Projects

### File Extension Semantics

| Extension | Purpose | Execution | Compilation |
|-----------|---------|-----------|-------------|
| `.fs` | Source files for compiled projects | Compiled to assembly | Part of project build |
| `.fsx` | Script files for interactive execution | FSI on-the-fly compilation | Not included in project output |
| `.fsi` | Interface/signature files | Type declarations only | N/A |

### Key Differences

#### .fsx Script Files

**Advantages**:
- Rapid iteration without build cycles
- Dynamic dependency loading via `#r` and `#load`
- Self-contained deployment (copy script + .NET SDK)
- NuGet packages inline: `#r "nuget: Newtonsoft.Json"`
- Rich IDE support (IntelliSense, syntax highlighting)

**Limitations**:
- Cannot load framework references (e.g., ASP.NET)
- Startup overhead from compilation
- Single-file constraint (unless using `#load`)
- Not suitable for multi-file projects

#### .fs Compiled Files

**Advantages**:
- Project structure with references
- Optimal performance (ahead-of-time compilation)
- Multi-file organization
- Full framework access

**Limitations**:
- Requires full build cycle for changes
- More complex setup and tooling
- Deployment includes compiled assemblies

### Hybrid Approach

**Common Pattern**:
1. **Prototype** in `.fsx` for rapid exploration
2. **Migrate** to `.fs` when stabilizing
3. **Maintain** automation scripts as `.fsx`

**Technical Details**:
- `.fsx` files can `#load` `.fs` files
- `.fsproj` files can compile `.fsx` files
- `dotnet fsi` only executes `.fsx` files directly

---

## REPL Features & Directives

### Core Directives

#### `#r` - Reference Loading

```fsharp
// Load compiled assembly
#r "path/to/assembly.dll"

// Load NuGet package (F# 5+)
#r "nuget: Newtonsoft.Json"
#r "nuget: Farmer, 1.3.2"                    // Specific version
#r "nuget: DiffSharp-lite, 1.0.0-preview"   // Preview versions

// With package source
#i "nuget: https://custom-source.com/index.json"
#r "nuget: CustomPackage"
```

**Key Behaviors**:
- Highest non-preview version selected by default
- Multiple consecutive references resolve together (transitive dependencies)
- Only available in F# Interactive (not compiled code)

#### `#load` - File Loading

```fsharp
// Load and compile script file
#load "utilities.fsx"

// Load source file
#load "helpers.fs"

// Multiple files
#load "file1.fsx" "file2.fsx" "file3.fsx"
```

**Execution Model**: "Reads a source file, compiles it, and runs it"

#### `#I` - Assembly Search Path

```fsharp
// Add search path for DLLs
#I @"C:\Libraries\CustomLib"
#I "lib/external"

// Add package source
#i "nuget: https://my-remote-package-source/index.json"
#i """nuget: C:\path\to\local\source"""  // Absolute paths required
```

#### `#help` - Documentation

```fsharp
#help                    // General help
#help List.map           // Function-specific docs (F# 9+)
```

#### `#time` - Performance Profiling

```fsharp
#time "on"               // Enable timing metrics
// --> Displays: real time, CPU time, GC info
#time "off"              // Disable timing
```

#### `#quit` - Exit Session

```fsharp
#quit;;                  // Terminate FSI
```

### Session Management

#### Evaluation Termination

```fsharp
// All expressions require double semicolon
let x = 42;;            // Evaluates and binds x
printfn "Hello";;       // Executes side effect
```

#### Result Storage

```fsharp
5 + 3;;                 // Stores in `it`
it * 2;;                // Uses previous result -> 16
```

#### Conditional Compilation

```fsharp
#if INTERACTIVE
// FSI-specific code
printfn "Running in interactive mode"
#else
// Compiled code
printfn "Running as compiled application"
#endif
```

**Preprocessor Symbols**:
- `INTERACTIVE`: Defined in FSI sessions
- `COMPILED`: Defined for standard compilation

---

## NuGet Package Integration

### Native Package Support (F# 5+)

F# Interactive gained **native NuGet integration** in F# 5.0 (.NET 5), eliminating dependency on external tools like Paket.

### Syntax & Usage

```fsharp
// Basic package reference
#r "nuget: FSharp.Data"

// Specific version
#r "nuget: Newtonsoft.Json, 13.0.1"

// Preview versions
#r "nuget: MyPackage, 1.0.0-preview-328097867"

// Multiple packages (resolved together)
#r "nuget: FSharp.Data"
#r "nuget: Deedle"
#r "nuget: XPlot.GoogleCharts"
```

### Custom Package Sources

```fsharp
// Remote source
#i "nuget: https://my-company.com/nuget/index.json"

// Local source (absolute path required)
#i """nuget: C:\local-packages"""

// Then reference packages
#r "nuget: CustomLibrary"
```

### Dependency Resolution

**Key Features**:
- Transitive dependencies resolved automatically
- Multiple package references resolve consistently
- Version conflicts detected and reported
- Package restoration happens on first reference

**Limitations**:
- Relative paths not supported for `#i` directive
- Requires internet connection for remote sources
- No `packages.config` or `paket.dependencies` equivalent

### Pre-F# 5 Approach: Paket

**Historical Context**: Before native NuGet support, Paket was recommended for F# scripting:

```fsharp
// Paket generates load scripts
#load "packages/FSharp.Data/FSharp.Data.fsx"
```

**Paket Advantages**:
- Stores packages without version numbers
- Scripts don't break on version updates
- Generate-load-scripts command

**Current Recommendation**: Use native `#r "nuget:"` for new projects (F# 5+)

---

## Polyglot Notebooks & .NET Interactive

### Overview

**Polyglot Notebooks** (formerly .NET Interactive Notebooks) is a VS Code extension enabling multi-language notebook development powered by **.NET Interactive**.

### Architecture

**.NET Interactive** is "an engine and API for running and editing code interactively" with three core capabilities:

1. **Code Execution**: Run code and retrieve results
2. **Language Services**: Completions, diagnostics, hover information
3. **Variable Sharing**: Cross-language data transfer

### F# Support

F# is a **first-class language** in Polyglot Notebooks with:

| Feature | F# Support |
|---------|------------|
| Language Server | ✅ Full IntelliSense |
| Variable Sharing | ✅ Cross-language |
| Syntax Highlighting | ✅ |
| Code Execution | ✅ |
| Magic Commands | ✅ |

### Supported Languages (8 total)

**With Variable Sharing**:
- C#
- F#
- PowerShell
- JavaScript
- SQL
- KQL (Kusto Query Language)

**Without Variable Sharing**:
- HTML
- Mermaid (diagrams)

### Cross-Language Workflows

**Example**: Query SQL → Process in F# → Visualize in JavaScript

```fsharp
// SQL cell
SELECT * FROM users WHERE active = 1

// F# cell
#!fsharp
#!share --from sql users
let activeCount = users |> Seq.length

// JavaScript cell
#!javascript
#!share --from fsharp activeCount
console.log(`Active users: ${activeCount}`)
```

### Integration Platforms

- **Jupyter**: Full support for Jupyter Notebook, JupyterLab, nteract
- **VS Code**: Polyglot Notebooks extension
- **Small Devices**: Raspberry Pi, pi-top [4] compatible
- **Experimental**: .NET REPL

### Installation

```bash
# Install VS Code extension
code --install-extension ms-dotnettools.dotnet-interactive-vscode

# Or search: "Polyglot Notebooks" in VS Code Extensions
```

### .NET 7+ Support

**Current Version** supports:
- .NET 7/8 SDK
- C# 11
- F# 7
- Enhanced performance and stability

---

## Embedding F# Compiler Service

### FSharp.Compiler.Service

**FSharp.Compiler.Service** is a NuGet package exposing F# compiler APIs for:
- Implementing F# language bindings
- Building refactoring tools
- **Embedding F# as a scripting language**

### Core API: FsiEvaluationSession

The `FsiEvaluationSession` type enables hosting F# Interactive in .NET applications.

#### Initialization

```csharp
// Create session
var config = FsiEvaluationSession.GetDefaultConfiguration();
var args = new[] { "fsi.exe" };
var session = FsiEvaluationSession.Create(
    config,
    args,
    Console.In,      // Input stream
    Console.Out,     // Output stream
    Console.Error    // Error stream
);
```

#### Evaluation Methods

**1. EvalExpression** - Evaluate and return values

```fsharp
// Returns FsiValue option with reflection values and types
session.EvalExpression("2 + 2")
// --> Some(FsiValue { ReflectionValue = 4; ReflectionType = int })

// Strongly-typed extraction
let result = session.EvalExpression("42")
             |> Option.map (fun v -> v.ReflectionValue :?> int)
// --> Some(42)
```

**2. EvalInteraction** - Side effects and declarations

```fsharp
// No return value, used for side effects
session.EvalInteraction("printfn \"Hello\"")
session.EvalInteraction("let x = 42")
session.EvalInteraction("#time \"on\"")
// Note: Does NOT require `;;` terminator
```

**3. EvalScript** - Execute complete `.fsx` files

```fsharp
session.EvalScript("path/to/script.fsx")
```

#### Error Handling

**Throwing Versions** (default):
- Exceptions terminate on errors
- Unsuitable for untrusted input

**Non-Throwing Versions**:

```fsharp
let result, diagnostics =
    session.EvalExpressionNonThrowing("invalid code")
// result: Choice<FsiValue option, exn>
// diagnostics: FSharpDiagnostic[]

match result with
| Choice1Of2 value ->
    printfn "Success: %A" value
| Choice2Of2 exn ->
    printfn "Error: %s" exn.Message

// Diagnostics include line/column information
diagnostics |> Array.iter (fun d ->
    printfn "%s at %d:%d" d.Message d.StartLine d.StartColumn
)
```

#### Advanced Features

**Type Checking**:

```fsharp
// Parse and check code for intellisense-like features
session.ParseAndCheckInteraction("let x = ")
// --> Provides completion suggestions, type information
```

**Async Execution**:

```fsharp
// Submit async computations
let task =
    session.EvalInteraction("async { return 42 }")
    |> Async.StartAsTask
```

**Collectible Assemblies**:

```fsharp
// Enable GC of generated code
let session = FsiEvaluationSession.Create(
    config, args, stdin, stdout, stderr,
    collectible = true
)
// Generated assemblies GC'd when session disposed and no references remain
```

**Custom FSI Object**:

```fsharp
// Override default FSI configuration
let customConfig =
    FsiEvaluationSession.GetDefaultConfiguration(customFsiObj)
```

### Security Considerations

**Critical Warning**: "There is no way to handle `StackOverflowException` and so a poorly written script can terminate the host process."

**Recommendations**:
- **Separate Process**: Run FSI in isolated process for untrusted input
- **Resource Limits**: Implement timeouts and memory limits
- **Sandboxing**: Use AppDomains or containers for isolation

**Option 1: Process Isolation**

```fsharp
// Communicate via stdin/stdout with fsi.exe process
let proc = Process.Start("dotnet", "fsi --noninteractive")
proc.StandardInput.WriteLine("let x = 42")
let result = proc.StandardOutput.ReadLine()
```

**Option 2: API with Safeguards**

```fsharp
// Use non-throwing API with timeout
let cts = new CancellationTokenSource(TimeSpan.FromSeconds(5))
async {
    let! result =
        session.EvalExpressionNonThrowing("potentiallyDangerous()")
        |> Async.StartAsTask
        |> Async.AwaitTask
    return result
} |> Async.RunSynchronously // With cancellation token
```

### Reflection and Type Extraction

**Function Extraction Pattern**:

```fsharp
// 1. Locate type by fully-qualified name
let asm = session.DynamicAssembly
let moduleType = asm.GetType("MyModule")

// 2. Get PropertyInfo (functions compile to properties)
let propInfo =
    moduleType.GetProperty(
        "myFunction",
        BindingFlags.Static ||| BindingFlags.Public
    )

// 3. Extract value
let funcObj = propInfo.GetValue(null)

// 4. Downcast to function type
let typedFunc = funcObj :?> (int -> int)

// 5. Invoke
let result = typedFunc(42)
```

**Constraints**:
- **Type Visibility**: All types in signatures must exist in host application
- **Script Types**: Types declared in scripts cannot be exposed
- **Module Naming**: Conflicts with type names add `Module` suffix

### Integration Patterns

**Pattern 1: Configuration Scripts**

```fsharp
// Host application loads user config
session.EvalScript("user-config.fsx")

// Extract configuration values
let configValue =
    session.EvalExpression("Config.serverPort")
    |> Option.map (fun v -> v.ReflectionValue :?> int)
```

**Pattern 2: Plugin System**

```fsharp
// Define plugin interface in host
type IPlugin =
    abstract Execute : Context -> Result

// Load plugin script
session.EvalScript("plugins/custom-plugin.fsx")

// Extract and invoke
let plugin = extractFunction<Context -> Result>("Plugin.execute")
let result = plugin(context)
```

**Pattern 3: REPL Integration**

```fsharp
// Interactive console loop
while true do
    printf "> "
    let input = Console.ReadLine()
    if input = "exit" then exit 0

    let result, diagnostics =
        session.EvalInteractionNonThrowing(input)

    match result with
    | Choice1Of2 _ ->
        printfn "Success"
    | Choice2Of2 exn ->
        printfn "Error: %s" exn.Message
```

---

## Hot Reload Capabilities

### Current State (2024)

**Official Status**: F# **does not support** .NET Hot Reload in Visual Studio 2022 or VS Code.

**Quote from Microsoft**: "Hot Reload is not supported in F# and VB applications. Planning to support in a future release based on customer feedback."

### Technical Limitations

**Why F# Lacks Hot Reload**:
1. **Language complexity**: F# type system more complex than C#
2. **Edit & Continue**: .NET Hot Reload builds on Edit & Continue, which F# never supported
3. **Compilation model**: F# order-dependent compilation complicates incremental updates
4. **Priority**: C# receives more tooling investment

**Status**: No official support in .NET 6, 7, 8, or 9 as of 2024.

### Community Workarounds

#### 1. Fable Hot Reload (Web Development)

**Fable** (F# to JavaScript transpiler) has robust hot reload support through **Elmish.HMR**.

**Technical Approach**:
- **Conditional Compilation**: `#if DEBUG` includes HMR code, removed in production
- **Function Inlining**: `inline` keyword injects reload handlers into source
- **Caller Information**: `CallerFilePath` attributes identify components

**Advantage**: "Library authors won't need bundler-specific plugins" - works with Webpack, Vite, etc.

**Example**:

```fsharp
#if DEBUG
open Elmish.HMR
#endif

let init() =
    { State = initialState }

let update msg state =
    match msg with
    | Increment -> { state with Count = state.Count + 1 }

let view state dispatch =
    div [] [
        button [ onClick (fun _ -> dispatch Increment) ] [ text "+" ]
        span [] [ text (string state.Count) ]
    ]

Program.mkProgram init update view
#if DEBUG
|> Program.withHMR  // Hot reload enabled
#endif
|> Program.run
```

#### 2. Giraffe.HotReload (ASP.NET)

**Giraffe.HotReload** is a community extension for Giraffe (F# web framework) using **FSharp.Compiler.Portacode**.

**GitHub**: https://github.com/baronfel/Giraffe.HotReload

**Status**: Experimental, not production-ready

#### 3. dotnet watch (Limited)

**dotnet watch** supports F# theoretically, but with significant issues:

```bash
dotnet watch run  # Should work for F# projects
```

**Problems**:
- `.fsproj` files not recognized correctly (Nov 2024 issue)
- "File extension not associated with a language" errors
- Unreliable compared to C# experience

**Recommendation**: Avoid for F# until officially supported.

#### 4. Custom File Watcher

**DIY Approach**: Use `FileSystemWatcher` to detect changes and recompile.

```fsharp
open System.IO

let watcher = new FileSystemWatcher()
watcher.Path <- __SOURCE_DIRECTORY__
watcher.Filter <- "*.fs"
watcher.NotifyFilter <- NotifyFilters.LastWrite

watcher.Changed.Add(fun e ->
    printfn "File changed: %s" e.Name
    // Recompile logic
    let proc = Process.Start("dotnet", "build")
    proc.WaitForExit()
    if proc.ExitCode = 0 then
        printfn "Rebuild successful"
)

watcher.EnableRaisingEvents <- true
```

### Alternative Development Workflows

**Fast Feedback Without Hot Reload**:

1. **F# Interactive**: Develop in `.fsx` files, test via FSI
2. **Fast Compilation**: F# compiler is fast, rebuild cycles tolerable
3. **Watch + Auto-Restart**: Custom script watches files, restarts on change
4. **Notebook Development**: Use Polyglot Notebooks for exploratory coding

---

## Development Workflows

### Workflow 1: Interactive Development with FSI

**Best For**: Prototyping, data analysis, automation scripts

**Steps**:

1. **Create `.fsx` file** with complete script
2. **Use IDE** (VS Code + Ionide) for IntelliSense
3. **Execute selections** via `Alt+Enter` to FSI
4. **Iterate rapidly** without full compilation
5. **Profile with `#time`** for performance checks
6. **Migrate to `.fs`** when stabilizing

**Example**:

```fsharp
// script.fsx
#r "nuget: FSharp.Data"
open FSharp.Data

// Load data (happens once, cached in session)
type Stocks = CsvProvider<"data/stocks.csv">
let data = Stocks.Load("data/stocks.csv");;

// Experiment with queries (modify and re-run)
let averageClose =
    data.Rows
    |> Seq.averageBy (fun row -> float row.Close);;

// Test different filters
let highVolume =
    data.Rows
    |> Seq.filter (fun row -> row.Volume > 1000000)
    |> Seq.length;;
```

**Productivity Tips**:
- Use `it` to reference last result
- Leverage `__SOURCE_DIRECTORY__` for relative paths
- Enable `#time` to profile iterations
- Organize complex scripts with `#load` directives

### Workflow 2: Notebook-Driven Development

**Best For**: Data exploration, documentation, teaching

**Platform**: VS Code + Polyglot Notebooks extension

**Advantages**:
- Markdown + code in single document
- Cell-based execution (like Jupyter)
- Variable sharing across languages
- Rich visualizations

**Example Notebook**:

```markdown
# Stock Analysis

## Load Data

```fsharp
#r "nuget: FSharp.Data"
#r "nuget: Plotly.NET"
open FSharp.Data
open Plotly.NET

type Stocks = CsvProvider<"stocks.csv">
let data = Stocks.Load("stocks.csv")
```

## Calculate Statistics

```fsharp
let avgClose = data.Rows |> Seq.averageBy (fun r -> float r.Close)
let maxVolume = data.Rows |> Seq.maxBy (fun r -> r.Volume)
```

## Visualize

```fsharp
let chart =
    data.Rows
    |> Seq.map (fun r -> r.Date, r.Close)
    |> Chart.Line
    |> Chart.Show
```
```

### Workflow 3: Test-Driven with Script Files

**Best For**: Developing reusable libraries

**Pattern**:

1. **Create module** in `.fs` file
2. **Create test script** in `.fsx` file
3. **Load module** via `#load "Module.fs"`
4. **Test interactively** in FSI
5. **Commit** when tests pass

**Example**:

```fsharp
// StringUtils.fs (module)
module StringUtils

let reverse (s: string) =
    s.ToCharArray() |> Array.rev |> System.String

let truncate maxLen (s: string) =
    if s.Length <= maxLen then s
    else s.Substring(0, maxLen) + "..."

// test.fsx (test script)
#load "StringUtils.fs"
open StringUtils

// Test cases
reverse "hello" = "olleh" |> printfn "Reverse: %b"
truncate 5 "hello world" = "hello..." |> printfn "Truncate: %b"
```

### Workflow 4: Automation Scripts

**Best For**: Build automation, dev tools, CI/CD

**Use Case**: Replace bash/Python scripts with type-safe F#

**Example - Clean Build Artifacts**:

```fsharp
#!/usr/bin/env -S dotnet fsi

#time "on"

open System.IO

let rec enumerateDirectories path =
    seq {
        yield path
        for dir in Directory.EnumerateDirectories(path) do
            yield! enumerateDirectories dir
    }

let isObjOrBinFolder path =
    let name = Path.GetFileName(path)
    name = "bin" || name = "obj"

let getFoldersToDelete rootPath =
    rootPath
    |> enumerateDirectories
    |> Seq.filter isObjOrBinFolder

let clean() =
    let root = __SOURCE_DIRECTORY__
    printfn "Cleaning build artifacts in %s" root

    getFoldersToDelete root
    |> Seq.iter (fun dir ->
        printfn "Deleting: %s" dir
        Directory.Delete(dir, true)
    )

    printfn "Clean complete!"

// Execute
clean()
```

**Make Executable**:

```bash
chmod +x clean.fsx
./clean.fsx  # Runs via shebang
```

### Workflow 5: Script → Compiled Migration

**Best For**: Transitioning from prototype to production

**Steps**:

1. **Start**: Prototype in `script.fsx`
2. **Extract**: Move core logic to `Module.fs`
3. **Test**: Keep `test.fsx` loading module
4. **Create Project**: Add `.fsproj` file
5. **Compile**: Build with `dotnet build`
6. **Maintain**: Keep scripts for testing/automation

**Migration Pattern**:

```fsharp
// 1. Original script.fsx
let processData input =
    input |> List.filter (fun x -> x > 0)
          |> List.sum

let result = processData [-1; 2; 3; 4]
printfn "Result: %d" result

// 2. Extract to DataProcessor.fs
module DataProcessor

let processData input =
    input |> List.filter (fun x -> x > 0)
          |> List.sum

// 3. Script loads module
#load "DataProcessor.fs"
open DataProcessor

let result = processData [-1; 2; 3; 4]
printfn "Result: %d" result

// 4. Compiled application
module Program

open DataProcessor

[<EntryPoint>]
let main argv =
    let result = processData [-1; 2; 3; 4]
    printfn "Result: %d" result
    0
```

### IDE Support

#### VS Code + Ionide

**Features**:
- IntelliSense for `.fsx` and `.fs` files
- Send code to FSI via `Alt+Enter`
- Debugging with breakpoints
- Syntax highlighting
- Code lens (type annotations)

**Installation**:

```bash
code --install-extension Ionide-fsharp
```

**Configuration** (settings.json):

```json
{
  "FSharp.fsiSdkFilePath": "dotnet fsi",
  "FSharp.fsiExtraParameters": ["--optimize+"]
}
```

#### Visual Studio

**Features**:
- F# Interactive window (View → F# Interactive)
- Send to Interactive (`Alt+Enter`)
- Debugging scripts
- Package management

**Limitations**:
- No Hot Reload for F#
- Slower than VS Code for scripts

#### JetBrains Rider

**Features**:
- F# Interactive tool window
- Script debugging
- NuGet integration
- Performance profiling

**Advantage**: Better refactoring tools than VS Code

---

## Performance Characteristics

### Compilation Model

**Key Insight**: F# Interactive is **not an interpreter** - it compiles to IL just like `fsc`.

**Process**:
1. Parse F# source
2. Type check
3. Compile to IL
4. JIT compile IL to native code
5. Execute native code

**Performance**: Near-identical to compiled F# code for compute-intensive tasks.

### Benchmark Findings

**From Community Reports**:

1. **Compute Performance**: "F# Interactive is always going to compile the code anyway, so the performance will be exactly the same as standard compiled F# files"

2. **Inconsistencies Observed**:
   - Some operations (e.g., `Array.sum`) slower in FSI
   - Debugging overhead can affect benchmarks
   - `fsi.exe` vs `fsianycpu.exe` performance differences

3. **Startup Overhead**: FSI has compilation cost upfront
   - Mitigated by session persistence (compile once, run many)
   - Can precompile scripts with `fsc` for faster startup

### Optimization Strategies

#### Strategy 1: Precompile Scripts

```bash
# Compile script to executable
fsc script.fsx -o script.exe

# Run compiled version
./script.exe  # Much faster startup
```

**Tradeoff**: Loses dynamic loading benefits

#### Strategy 2: Session Reuse

```fsharp
// Load libraries once
#r "nuget: FSharp.Data"
open FSharp.Data;;

// Modify and re-run analysis code
let analyze data =
    data |> Seq.filter (fun x -> x > 100);;

// Data stays in memory, only analysis recompiles
let result1 = analyze data1;;
let result2 = analyze data2;;
```

#### Strategy 3: Profile with #time

```fsharp
#time "on";;

// Expensive operation
let result = [1..1000000] |> List.sum;;
// --> Real: 00:00:00.123, CPU: 00:00:00.120, GC gen0: 1

#time "off";;
```

### Performance Comparison Table

| Scenario | FSI (.fsx) | Compiled (.exe) | Notes |
|----------|------------|-----------------|-------|
| First Run | Slower (compilation) | Faster (AOT compiled) | FSI pays upfront cost |
| Subsequent Runs | Fast (cached session) | Fast | FSI amortizes compilation |
| Startup Time | ~100-500ms | ~10-50ms | FSI loads compiler |
| Compute Performance | Near-identical | Baseline | Both compile to IL |
| Memory Overhead | Higher (compiler in-process) | Lower | FSI ~50-100MB baseline |
| Large Datasets | Competitive | Competitive | GC behavior similar |

### When to Choose Each

**Use FSI (.fsx) for**:
- Rapid prototyping
- Data exploration
- Automation scripts
- One-off analysis
- Interactive development

**Use Compiled (.exe) for**:
- Production applications
- Performance-critical code
- Long-running services
- Minimal startup time requirements
- Distribution without .NET SDK

**Hybrid Approach**:
- Develop in `.fsx` for feedback loop
- Compile to `.exe` for deployment

---

## FSRS Design Implications

### What to Adopt from F# Scripting

#### 1. Directive-Based Package Loading

**F# Pattern**:
```fsharp
#r "nuget: FSharp.Data"
#load "utilities.fsx"
```

**FSRS Equivalent**:
```fsharp
// Mini-F# script
#r "crate: serde"              // Load Rust crate at runtime
#load "helpers.fsrs"           // Load other FSRS scripts
#native "libcustom.so"         // Load native library
```

**Implementation**: Build runtime package/crate loader into VM

#### 2. REPL-First Design

**F# Pattern**: FSI as first-class development tool

**FSRS Equivalent**:
```bash
fsrs-repl                      # Interactive REPL
fsrs script.fsrs               # Execute script
fsrs --watch script.fsrs       # Watch mode
```

**Implementation**: Separate `fsrs-repl` crate with line editor, history

#### 3. Embedding API (FSharp.Compiler.Service Pattern)

**F# Pattern**:
```csharp
var session = FsiEvaluationSession.Create(...);
session.EvalExpression("2 + 2");
```

**FSRS Equivalent**:
```rust
// Host application
let mut engine = FsrsEngine::new();
engine.eval_expr("let x = 42")?;
let result: i64 = engine.eval_expr("x * 2")?;
assert_eq!(result, 84);
```

**Implementation**: Expose `FsrsEngine` with Lua-like API (Phase 3)

#### 4. Session State Management

**F# Pattern**: Persist bindings across evaluations

**FSRS Equivalent**:
```rust
// Session maintains environment
engine.eval("let data = loadData()")?;  // Loads once
engine.eval("analyze(data)")?;          // Reuses data
engine.eval("visualize(data)")?;        // Still available
```

**Implementation**: VM maintains global environment between evals

#### 5. Conditional Compilation

**F# Pattern**:
```fsharp
#if INTERACTIVE
// REPL-specific code
#else
// Compiled code
#endif
```

**FSRS Equivalent**:
```fsharp
#if REPL
printfn "Running in REPL"
#else
printfn "Running as script"
#endif
```

**Implementation**: Preprocessor symbols during compilation

### What to Improve Beyond F#

#### 1. Hot Reload (F# Lacks This)

**Problem**: F# has no hot reload support in 2024

**FSRS Opportunity**: First-class hot reload for scripts

**Implementation**:
```rust
// Watch mode with hot reload
fsrs --watch --hot-reload app.fsrs

// API for host applications
engine.enable_hot_reload()?;
engine.watch_file("script.fsrs")?;
// Auto-reloads on file change, preserves state
```

**Technical Approach** (inspired by Fable):
- File watcher detects changes
- Recompile changed module only
- Patch bytecode in running VM
- Preserve state where possible (configurable)

**Phases**:
- Phase 1: Basic file watching + restart
- Phase 2: Incremental compilation
- Phase 3: Live bytecode patching
- Phase 4: State preservation

#### 2. Startup Performance

**Problem**: FSI has 100-500ms startup overhead

**FSRS Goal**: <5ms startup for simple scripts

**Implementation Strategies**:
- Ahead-of-time compilation option
- Bytecode caching (`.fsrsc` compiled format)
- Minimal runtime footprint (<1MB)
- Lazy loading of standard library

```bash
# AOT compile for fast startup
fsrs compile script.fsrs -o script.fsrsc

# Run compiled bytecode
fsrs script.fsrsc  # <5ms startup
```

#### 3. Better Error Messages

**Problem**: F# error messages can be cryptic

**FSRS Opportunity**: Rust-quality error messages for scripting

**Implementation**:
```
Error: Type mismatch in function call
  ┌─ script.fsrs:42:15
  │
42│     processData "hello"
  │                 ^^^^^^^ expected List<int>, found string
  │
  = help: Did you mean to pass a list? Try: processData [1, 2, 3]
  = note: processData is defined at line 10
```

**Features**:
- Codespan-style formatting
- Type inference traces
- Suggestions for fixes
- Links to documentation

#### 4. Cross-Platform Package Management

**Problem**: F# NuGet is .NET-specific

**FSRS Opportunity**: Universal package system

**Design**:
```fsharp
#r "crate: serde"              // Rust crate (compile-time)
#r "npm: lodash"               // JavaScript library
#r "pypi: numpy"               // Python package (via FFI)
#r "url: https://cdn.../lib.js" // Remote script
```

**Implementation**: Plugin system for package sources (Phase 4)

#### 5. Lightweight Embedding

**Problem**: FSharp.Compiler.Service is heavy (~50MB+)

**FSRS Goal**: Minimal embedding footprint

**Target**:
- Bytecode VM: <500KB
- Standard library: <1MB
- Total embedded: <2MB

**Implementation**: Rust's zero-cost abstractions, no JIT

### Differentiation Strategy

| Feature | F# Interactive | FSRS Target |
|---------|----------------|-------------|
| Startup Time | 100-500ms | <5ms |
| Memory Overhead | ~50-100MB | <1MB |
| Hot Reload | ❌ None | ✅ Built-in |
| Error Messages | Good | Excellent (Rust-quality) |
| Embedding Complexity | Medium | Very Low (Lua-like) |
| Package Ecosystem | .NET only | Cross-platform |
| Native Performance | JIT | Bytecode VM (faster startup) |
| Production Ready | Yes (with caveats) | Yes (by design) |

---

## Implementation Recommendations

### Phase 1: MVP REPL

**Scope**: Basic interactive execution

**Components**:
- `fsrs-repl` crate
- Line editor (rustyline)
- Expression evaluation
- Error reporting

**Example**:
```bash
$ fsrs-repl
FSRS REPL v0.1.0
> let x = 42
val x : int = 42
> x + 8
val it : int = 50
> quit
```

**Dependencies**:
- `rustyline` for line editing
- `codespan-reporting` for errors

### Phase 2: Script Execution

**Scope**: Run `.fsrs` files with directives

**Features**:
- `#load` directive (load other scripts)
- `#time` directive (profiling)
- Basic error recovery

**Example**:
```fsharp
// script.fsrs
#time on

let fibonacci n =
    let rec fib a b n =
        if n = 0 then a
        else fib b (a + b) (n - 1)
    fib 0 1 n

let result = fibonacci 40
printfn "Result: %d" result
```

```bash
$ fsrs script.fsrs
Time: 0.123s
Result: 102334155
```

### Phase 3: Package Loading

**Scope**: `#r` directive for dependencies

**Design**:
```fsharp
// Dynamic crate loading (complex, Phase 4?)
#r "crate: serde_json"
open Serde.Json

// Or: FFI boundary for host-provided functions
#r "host: JsonApi"
let data = Json.parse("{\"key\": \"value\"}")
```

**Implementation**:
- Phase 3a: Host-provided functions only
- Phase 3b: Static crate linking
- Phase 4: Dynamic loading (if feasible)

### Phase 4: Hot Reload

**Scope**: Watch mode with live reloading

**Architecture**:
```rust
// File watcher
let (tx, rx) = channel();
let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
watcher.watch("script.fsrs", RecursiveMode::NonRecursive)?;

// VM with hot reload
let mut vm = VM::new();
vm.load_script("script.fsrs")?;

loop {
    match rx.recv() {
        Ok(event) => {
            println!("Reloading...");
            vm.reload_script("script.fsrs")?;
        }
        Err(e) => break,
    }
}
```

**Challenges**:
- State preservation vs. fresh start
- Incremental compilation
- Bytecode patching
- Type compatibility across reloads

**Phases**:
1. Simple restart (Phase 3)
2. Incremental compilation (Phase 4)
3. Live patching (Phase 5+)

### Phase 5: Embedding API

**Scope**: Lua-like embedding for Rust hosts

**API Design**:
```rust
use fsrs_vm::Engine;

fn main() -> Result<()> {
    let mut engine = Engine::new();

    // Register host function
    engine.register_fn("greet", |name: String| {
        format!("Hello, {}!", name)
    })?;

    // Eval expression
    let result: i32 = engine.eval("21 + 21")?;
    assert_eq!(result, 42);

    // Eval script
    engine.eval_file("config.fsrs")?;

    // Call script function
    let msg: String = engine.call("greet", ("Alice",))?;
    println!("{}", msg);  // "Hello, Alice!"

    Ok(())
}
```

**Features**:
- Type-safe marshalling
- Error propagation
- Sandboxing options
- Resource limits

### Phase 6: Notebook Integration

**Scope**: Polyglot Notebooks kernel

**Implementation**:
- Implement Jupyter kernel protocol
- Variable sharing via JSON
- Rich output (HTML, images)

**Example Notebook**:
```markdown
# Data Analysis with FSRS

```fsrs
let data = [1; 2; 3; 4; 5]
let sum = List.fold (+) 0 data
```

Output: `val sum : int = 15`

```fsrs
#r "plot: Chart"
Chart.line(data)
```

(Chart renders inline)
```

**Protocol**: Jupyter messaging spec
**Transport**: ZeroMQ
**Reference**: Look at evcxr (Rust Jupyter kernel)

---

## Conclusion

### Summary of Learnings

F# provides a **mature scripting and REPL infrastructure** with:

1. **Robust Interactive Environment**: FSI with rich tooling support
2. **Seamless Package Integration**: Native NuGet support since F# 5
3. **Embeddable Compiler**: FSharp.Compiler.Service for hosting scenarios
4. **Notebook Support**: First-class Polyglot Notebooks integration
5. **Production Performance**: On-the-fly compilation to IL, not interpretation

**Limitations** FSRS can address:
- No hot reload support
- Startup overhead
- .NET-only ecosystem
- Heavy embedding footprint

### Key Takeaways for FSRS

**Adopt**:
- Directive-based package loading (`#r`, `#load`)
- REPL-first development workflow
- Session state management
- Embeddable scripting API
- Conditional compilation

**Improve**:
- Hot reload (F# lacks this entirely)
- Startup performance (<5ms vs 100-500ms)
- Error messages (Rust-quality diagnostics)
- Cross-platform packages (not just .NET)
- Embedding simplicity (Lua-like API)

**Differentiate**:
- Rust-native embedding (no .NET dependency)
- Bytecode VM (faster startup than JIT)
- Production-ready hot reload
- Minimal footprint (<2MB embedded)
- First-class terminal emulator support

### Recommended Implementation Path

**Phase 1-2**: Basic REPL + script execution (current MVP)
**Phase 3**: Embedding API + host interop
**Phase 4**: Hot reload + watch mode
**Phase 5**: Package loading system
**Phase 6**: Notebook integration (optional)

### References

1. **F# Interactive**: https://learn.microsoft.com/en-us/dotnet/fsharp/tools/fsharp-interactive/
2. **FSharp.Compiler.Service**: https://fsharp.github.io/fsharp-compiler-docs/fcs/
3. **Polyglot Notebooks**: https://github.com/dotnet/interactive
4. **F# Scripting Tips**: https://brandewinder.com/2016/02/06/10-fsharp-scripting-tips/
5. **Fable Hot Reload**: https://fable.io/blog/2022/2022-10-26-hot-reload.html
6. **Embedding F# Compiler**: https://queil.net/2021/05/embedding-fsharp-compiler/

---

**Document Version**: 1.0
**Last Updated**: 2025-11-19
**Next Review**: Phase 3 completion
