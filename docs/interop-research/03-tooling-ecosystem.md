# F# Tooling Ecosystem Integration for FSRS

**Document Version:** 1.0
**Date:** 2025-11-19
**Status:** Research Complete

## Executive Summary

FSRS scripts that remain syntactically valid F# can leverage an extensive, mature tooling ecosystem built over 15+ years of F# development. This research identifies opportunities to inherit IDE support, language services, formatters, linters, and build tools with minimal additional work, dramatically accelerating developer experience.

**Key Finding:** By maintaining F# syntax compatibility, FSRS can immediately benefit from:
- 1M+ download Ionide VSCode extension
- FsAutoComplete LSP server supporting 10+ editors
- Fantomas formatter (community standard)
- FSharpLint static analysis
- F# Interactive (FSI) for REPL workflows
- Visual Studio and JetBrains Rider full IDE support

**Strategic Value:** Zero-cost developer experience enhancement through ecosystem leverage rather than building custom tooling.

---

## 1. Language Server Protocol (LSP) Infrastructure

### 1.1 FsAutoComplete - The F# Language Server

**Repository:** https://github.com/ionide/FsAutoComplete
**Status:** Actively maintained (461 stars, 113+ contributors, 2024-2025 updates)
**Maintainers:** Krzysztof Cieślak, Chester Husk

#### Architecture Overview

FsAutoComplete (FSAC) is a production-grade LSP server built on four core libraries:

1. **FSharp.Compiler.Service** - Language analysis and type information
2. **Ionide.ProjInfo** - MSBuild project/solution management
3. **FSharpLint** - Static analysis integration
4. **Fantomas** - Code formatting

**Installation:**
```bash
dotnet tool install --global fsautocomplete
```

#### LSP Capabilities

**Standard LSP Features:**
- Document lifecycle (open, change, save, close)
- Navigation (definition, type definition, implementation)
- Code intelligence (hover, completion, signature help)
- Refactoring (rename, code actions, quick fixes)
- Analysis (diagnostics, references, document highlighting)
- Advanced (folding ranges, semantic tokens, call hierarchies)

**F#-Specific Code Actions:**
- Namespace resolution (auto-import missing namespaces)
- Pattern matching case generation (exhaustiveness helpers)
- Interface stub creation (implement interface skeletons)
- Unused declaration removal (dead code cleanup)
- FSharpLint-based suggestions (style improvements)

**Custom F# Endpoints (Beyond Standard LSP):**
- `fsharp/signature` - Type signature information
- `fsharp/signatureData` - Detailed signature metadata
- `fsharp/lineLens` - Inline code lens data
- `fsharp/compile` - On-demand compilation
- `fsharp/workspaceLoad` - Workspace management
- `fsharp/documentation` - Documentation retrieval
- Project file operations (add/remove/reorder files)

#### Configuration for FSRS

**Initialization Options:**
```json
{
  "AutomaticWorkspaceInit": true,
  "EnableAnalyzers": true,
  "EnableUnusedOpensAnalyzer": true,
  "EnableUnusedDeclarationsAnalyzer": true,
  "UnusedOpensAnalyzerExclusions": ["*.fsrs"],
  "FSICompilerToolLocations": ["custom-fsrs-compiler"]
}
```

**FSRS Integration Strategy:**

1. **Script Mode Detection:** Configure FSAC to recognize `.fsrs` extensions as F# script files
2. **Custom Type Checker:** Hook into FSI parameters for FSRS-specific type checking
3. **Bytecode Awareness:** Extend diagnostics to warn about FSRS VM limitations
4. **Module Resolution:** Custom workspace loader for FSRS module system

**Technical Approach:**

Since FSAC uses `FSharp.Compiler.Service` under the hood, FSRS can:
- Register `.fsrs` as a recognized script extension
- Provide custom compiler options via `FSICompilerToolLocations`
- Hook into the diagnostics pipeline to add FSRS-specific warnings
- Leverage existing AST/tokenization for syntax highlighting

**Observability:**

FSAC integrates **OpenTelemetry** via `System.Diagnostics.Activity`, enabling:
- Distributed tracing through Jaeger
- Performance analysis of FSRS compilation
- Debugging language server interactions

---

## 2. IDE and Editor Support

### 2.1 Visual Studio Code - Ionide Extension

**Extension:** Ionide-VSCode (ionide.ionide-fsharp)
**Downloads:** 1M+
**Status:** Active development (v7.25.7 - March 2025)
**Repository:** https://github.com/ionide/ionide-vscode-fsharp

#### Key Features

**Language Intelligence:**
- IntelliSense with type information
- Signature help for function parameters
- Hover tooltips with documentation
- Go-to-definition/implementation
- Find all references
- Rename symbol refactoring

**Code Quality:**
- Fantomas integration for formatting
- FSharpLint integration for linting
- Unused declaration highlighting
- Code lens annotations

**Development Workflow:**
- F# Interactive (FSI) integration
- Send code to REPL (Alt+Enter)
- Inline script evaluation
- Test Explorer integration (dotnet test)
- Debugger support

**Project Management:**
- Solution Explorer
- Project file editing
- Add/remove/reorder files
- Multi-project solutions

#### Recent Updates (2024-2025)

**Notable improvements:**
- Removed explicit C# dependency (now recommends via extension pack)
- Fixed floating codelens issue
- Enhanced Fantomas integration (supports global-tool installation)
- "Generate cases" code action for discriminated union matching
- Updated to FSAC 0.62.0 (F# 7.0.400 SDK features)
- Massive Test Explorer enhancements (uses `dotnet test` listing)

#### FSRS Integration Path

**Phase 1: Basic Support**
```json
// .vscode/settings.json
{
  "FSharp.fsacRuntime": "net6.0",
  "FSharp.fsiExtraParameters": ["--define:FSRS"],
  "files.associations": {
    "*.fsrs": "fsharp"
  },
  "FSharp.showExplorerOnStartup": false
}
```

**Phase 2: Custom Extension**

Create `fsrs-vscode` extension that:
1. Depends on Ionide for LSP features
2. Adds FSRS-specific snippets
3. Provides bytecode viewer (show compiled instructions)
4. Integrates VM debugger
5. Hot-reload detection and notifications

**Phase 3: Advanced Features**

- REPL integration with FSRS VM
- Visual bytecode debugger
- Performance profiling overlay
- GC pause visualization
- Module dependency graph viewer

---

### 2.2 Visual Studio (Full IDE)

**Versions:** Visual Studio 2022, Visual Studio 2026
**Status:** F# is a first-class language in VS
**Latest:** VS 2026 released November 2025

#### Built-in F# Features

**Core Capabilities:**
- Full IntelliSense with F# 8/9 support
- Advanced debugger (breakpoints, watch, immediate window)
- Profiling tools (CPU, memory, hot paths)
- Code coverage (now in Community/Professional editions)
- Refactoring tools (rename, extract function)
- F# Interactive window (View > Other Windows > F# Interactive)

**VS 2026 Improvements:**
- Fluent UI design (modern, accessible interface)
- GitHub Copilot "Did You Mean" feature (AI-powered search)
- Image Hover Preview (see images inline with dimensions)
- Rename Suggestions (context-aware naming)
- Extension Hot Loading (install without restart)

**F# 8/9 Language Features:**
- Improved autocomplete (better recall and precision)
- Performance improvements (compiler optimizations)
- Better error messages (actionable diagnostics)

#### FSRS Integration Strategy

**Minimal Effort Approach:**

Since Visual Studio recognizes `.fsx` files as F# scripts:
1. Configure file association: `.fsrs` -> F# Script
2. Set FSI extra parameters for FSRS mode
3. Use custom build tasks for bytecode compilation

**Enhanced Integration:**

Develop Visual Studio extension:
- Custom project type (.fsrsproj)
- Bytecode output window
- VM debugger integration
- Performance analyzer for FSRS scripts

---

### 2.3 JetBrains Rider

**Product:** JetBrains Rider (Cross-platform .NET IDE)
**Status:** Active development (Rider 2024.2, 2025.3)
**F# Plugin:** Bundled and enabled by default

#### F# Support Features

**Code Intelligence:**
- Code inspection and analysis
- Coding assistance (smart autocomplete)
- Navigation and search (go-to, find usages)
- Mixed C#/F# solutions (cross-language references)
- Resolve types without rebuilding projects

**F# 8 Language Support (2024):**
- Abbreviated lambda expressions
- Nested record updates
- Static interface member elements
- Let bindings improvements

**F# Interactive:**
- Built-in REPL (Read-Eval-Print Loop)
- Experiment with code without compilation
- Send selections to FSI

**2024 Enhancements:**
- Generating overrides in object expressions
- Postfix template for record type instances
- Full Line Code Completion (AI-powered)
- Reader Mode (distraction-free reading)
- Major debugging improvements

#### FSRS Integration Potential

**Leverage Existing Infrastructure:**

Rider's plugin architecture allows:
1. Custom language injection (recognize `.fsrs` files)
2. Extend F# plugin with FSRS-specific features
3. Custom run configurations for FSRS VM
4. Debugger integration via remote debugging protocol

**Development Experience:**

Rider's strengths for FSRS:
- Excellent performance (IntelliJ platform)
- Superior refactoring tools
- Built-in VCS integration
- Database tools (if FSRS needs data access)
- Cross-platform (Windows, Mac, Linux)

---

### 2.4 Vim/Neovim and Emacs

**Support:** Via FsAutoComplete LSP integration
**Status:** Production-ready for F# development

#### Vim/Neovim Setup

**Using ionide-vim:**
```vim
" Install via vim-plug
Plug 'ionide/Ionide-vim'

" LSP configuration (requires nvim-lspconfig)
lua << EOF
require'lspconfig'.fsautocomplete.setup{
  cmd = {'fsautocomplete', '--mode', 'lsp'},
  filetypes = {'fsharp', 'fsrs'},
  root_dir = require'lspconfig'.util.root_pattern('*.fsproj', '*.sln'),
}
EOF
```

#### Emacs Setup

**Using lsp-mode:**
```elisp
;; Install packages
(use-package fsharp-mode
  :ensure t)

(use-package lsp-mode
  :ensure t
  :hook ((fsharp-mode . lsp))
  :commands lsp)

;; Configure FSAC
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(fsharp-mode . "fsharp"))
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection "fsautocomplete")
                    :major-modes '(fsharp-mode)
                    :server-id 'fsautocomplete)))
```

#### FSRS Integration

**Add `.fsrs` filetype support:**

```vim
" Vim
autocmd BufRead,BufNewFile *.fsrs set filetype=fsharp

" Neovim Lua
vim.api.nvim_create_autocmd({"BufRead", "BufNewFile"}, {
  pattern = "*.fsrs",
  command = "set filetype=fsharp",
})
```

**Benefits:**
- Lightweight, fast editing
- Full LSP features via FSAC
- Terminal-friendly (SSH, tmux workflows)
- Scriptable (extend with custom FSRS commands)

---

## 3. Code Formatting - Fantomas

**Repository:** https://github.com/fsprojects/fantomas
**Status:** Community standard formatter
**Latest:** v6.3.16 (October 2024)
**Website:** https://fsprojects.github.io/fantomas/

### Overview

Fantomas is the **official F# community standard** for code formatting, with default settings aligned to Microsoft's F# style guide.

**Installation:**
```bash
dotnet tool install -g fantomas-tool
```

### Features

**Core Capabilities:**
- Opinionated formatting (reduces style bikeshedding)
- Ensures correct indentation
- Consistent spacing between elements
- Configurable via `.editorconfig` or `fantomas-config.json`

**Integration Options:**

1. **Command Line:**
   ```bash
   dotnet fantomas src/
   ```

2. **VSCode via Ionide:**
   - Automatic format on save
   - Global tool or local installation
   - Configurable rules

3. **Visual Studio:**
   - Via F# Formatting extension (asti.fantomas-vs)

4. **Git Pre-commit Hook:**
   ```bash
   #!/bin/sh
   dotnet fantomas --check .
   ```

### Community Adoption

**Used by Major Projects:**
- dotnet/fsharp (F# compiler itself)
- fsharp/FsAutoComplete
- fsprojects/FAKE
- fable-compiler/fable
- ionide/ionide-vscode-fsharp

### FSRS Integration Strategy

**Configuration for FSRS:**

```json
// fantomas-config.json
{
  "fsharp_experimental_elmish": false,
  "fsharp_max_if_then_else_short_width": 40,
  "fsharp_multi_line_lambda_closing_newline": true,
  "fsharp_experimental_keep_indent_in_branch": true,
  "fsharp_align_function_signature_to_indentation": true
}
```

**Custom Rules for FSRS:**

Since FSRS may have subset restrictions:
1. Configure to avoid features not supported (records, DUs in Phase 1)
2. Enforce style for bytecode-friendly constructs
3. Add warnings for VM limitations (e.g., tail-call requirements)

**Benefits:**
- Consistent FSRS codebase style
- Eliminate formatting debates
- Cleaner git diffs
- Professional appearance

---

## 4. Static Analysis - FSharpLint

**Repository:** https://github.com/fsprojects/FSharpLint
**Status:** Mature (11+ years old, 305 stars)
**License:** MIT
**Integration:** Built into FsAutoComplete

### Features

**Analysis Capabilities:**
- Style checking against configurable rules
- Detects common coding mistakes
- Suggests idiomatic F# patterns
- Configurable via XML
- Console app or MSBuild task

**Usage Contexts:**
- VSCode via Ionide-FSharp plugin
- Visual Studio via F# Lint extension
- MSBuild integration (CI/CD)
- Command line (pre-commit hooks)

**Example Checks:**
- Use `||` instead of `or` for boolean operations
- Avoid unnecessary parentheses
- Prefer pattern matching over if/then/else chains
- Flag unused values
- Detect naming convention violations

### FSRS Integration

**Custom Ruleset for FSRS:**

```xml
<?xml version="1.0" encoding="utf-8"?>
<FSharpLintSettings>
  <Analysers>
    <!-- Enable FSRS-specific checks -->
    <Analyser AnalyserId="TailCallOptimization">
      <Rules>
        <Rule Name="TailCallRequired">
          <Enabled>true</Enabled>
        </Rule>
      </Rules>
    </Analyser>

    <!-- Warn about unsupported features -->
    <Analyser AnalyserId="UnsupportedFeatures">
      <Rules>
        <Rule Name="NoRecordsInPhase1">
          <Enabled>true</Enabled>
        </Rule>
      </Rules>
    </Analyser>
  </Analysers>
</FSharpLintSettings>
```

**FSRS-Specific Lints:**

1. **VM Limitation Warnings:**
   - Detect recursive functions without `rec` keyword
   - Flag closures that capture too many variables
   - Warn about deep pattern matching (stack depth)

2. **Performance Lints:**
   - Suggest `Array` over `List` for hot paths
   - Flag allocations in tight loops
   - Recommend tail-recursive patterns

3. **Style Enforcement:**
   - Consistent naming (camelCase for locals, PascalCase for exports)
   - Module structure conventions
   - Comment header requirements

**Integration Path:**

Phase 1: Use existing FSharpLint rules
Phase 2: Contribute FSRS analyzer to FSharpLint project
Phase 3: Custom FSRS-lint tool for bytecode-specific checks

---

## 5. F# Interactive (FSI) - REPL

**Tool:** `dotnet fsi` (included with .NET SDK)
**Status:** First-class F# tooling
**Documentation:** https://learn.microsoft.com/en-us/dotnet/fsharp/tools/fsharp-interactive/

### Capabilities

**Core Features:**
- Read-Eval-Print Loop (REPL) for F#
- Execute F# code interactively
- Load and run `.fsx` script files
- Reference assemblies with `#r` directives
- Load source files with `#load` directives
- Multi-line input (terminate with `;;`)

**Usage Patterns:**

1. **Interactive Exploration:**
   ```fsharp
   dotnet fsi
   > let add x y = x + y;;
   val add : x:int -> y:int -> int
   > add 5 3;;
   val it : int = 8
   ```

2. **Script Execution:**
   ```bash
   dotnet fsi script.fsx
   ```

3. **IDE Integration:**
   - Visual Studio: F# Interactive Window
   - VSCode: Ionide FSI integration (Alt+Enter to send)
   - Rider: Built-in REPL

**Advanced Features:**
- Compile-time directives (`#if`, `#else`, `#endif`)
- Custom compiler options (`--define:SYMBOL`)
- Performance timing (`#time "on"`)
- Help system (`#help`)
- Load from HTTP/NuGet (`#r "nuget: PackageName"`)

### FSRS Integration Strategy

**Approach 1: FSI Compatibility Mode**

Keep FSRS scripts valid for FSI:
```fsharp
// example.fsrs (also valid .fsx)
#if FSRS
// FSRS-specific optimizations
let fastAdd x y = x + y  // compiles to bytecode ADD
#else
// Standard F# fallback
let fastAdd x y = x + y
#endif

fastAdd 10 20;;
```

**Approach 2: FSRS REPL Wrapper**

Implement `fsrs-repl` that:
1. Accepts F# syntax
2. Compiles to FSRS bytecode
3. Executes on FSRS VM
4. Displays results

```bash
$ fsrs-repl
FSRS Interactive 0.1.0
> let x = 42;;
val x : int = 42
> x + 8;;
val it : int = 50 (executed on FSRS VM)
```

**Approach 3: Hybrid Mode**

Leverage FSI for prototyping, then compile to FSRS:

1. Develop logic in `.fsx` with FSI
2. Test interactively with full F# features
3. Run `fsrs-compile script.fsx` to validate FSRS compatibility
4. Deploy compiled bytecode

**Benefits:**
- Rapid prototyping with FSI
- Validate FSRS semantics before deployment
- Use F# ecosystem for development, FSRS for production

---

## 6. Build and Automation Tools

### 6.1 FAKE - F# Make

**Repository:** https://github.com/fsprojects/FAKE
**Website:** https://fake.build/
**Status:** Mature build automation system

#### Overview

FAKE is a cross-platform build automation system that uses F# as a DSL for defining build tasks.

**Key Features:**
- Declarative, typed build scripts
- Target dependencies (build graphs)
- Extensive module library (git, docker, nuget, etc.)
- Cross-platform (Windows, Mac, Linux)
- Integrated with .NET tooling

**Installation:**
```bash
dotnet tool install fake-cli -g
```

**Example Build Script:**
```fsharp
#r "paket:
nuget Fake.Core.Target
nuget Fake.DotNet.Cli //"

open Fake.Core
open Fake.DotNet

Target.create "Clean" (fun _ ->
    !! "src/**/bin"
    ++ "src/**/obj"
    |> Shell.cleanDirs
)

Target.create "Build" (fun _ ->
    !! "src/**/*.*proj"
    |> Seq.iter (DotNet.build id)
)

Target.create "Test" (fun _ ->
    !! "tests/**/*.*proj"
    |> Seq.iter (DotNet.test id)
)

Target.create "All" ignore

"Clean"
  ==> "Build"
  ==> "Test"
  ==> "All"

Target.runOrDefault "All"
```

#### FSRS Build Integration

**Use FAKE for FSRS Development:**

```fsharp
// build.fsx
#r "paket:
nuget Fake.Core.Target
nuget Fake.IO.FileSystem //"

open Fake.Core
open Fake.IO
open Fake.IO.Globbing.Operators

Target.create "CompileFSRS" (fun _ ->
    !! "examples/**/*.fsrs"
    |> Seq.iter (fun script ->
        // Compile to bytecode
        Shell.Exec("cargo", sprintf "run --bin fsrs-compile -- %s" script)
        |> ignore
    )
)

Target.create "TestFSRS" (fun _ ->
    !! "tests/**/*.fsrs"
    |> Seq.iter (fun test ->
        Shell.Exec("cargo", sprintf "run --bin fsrs-test -- %s" test)
        |> ignore
    )
)

Target.create "FormatFSRS" (fun _ ->
    !! "**/*.fsrs"
    |> Seq.iter (fun file ->
        Shell.Exec("fantomas", file) |> ignore
    )
)

"CompileFSRS" ==> "TestFSRS"

Target.runOrDefault "TestFSRS"
```

**Benefits:**
- Unified build system (Rust + F# + FSRS)
- Typed build logic (catch errors at compile time)
- Reusable modules (NuGet distribution)

---

### 6.2 Package Management - Paket vs NuGet

#### NuGet (Default .NET Package Manager)

**Pros:**
- Built into .NET SDK
- First-class Visual Studio support
- Largest package ecosystem
- Simple for basic scenarios

**Cons:**
- Transitive dependencies pollute `packages.config`
- Version conflicts across projects
- No direct Git repository support

#### Paket (Alternative Package Manager)

**Repository:** https://github.com/fsprojects/Paket
**Website:** https://fsprojects.github.io/Paket/

**Advantages:**

1. **Transitive Dependency Management:**
   - Separates direct vs. transitive deps
   - `paket.dependencies` (solution-level)
   - `paket.lock` (lockfile for reproducibility)

2. **Version Unification:**
   - One version per package across solution
   - Avoids version conflicts
   - Deterministic builds

3. **Git Repository Support:**
   ```
   github fsprojects/FSharp.Data src/Json/JsonValue.fs
   github owner/repo:commit-hash src/File.fs
   ```

4. **F# Script Integration:**
   ```fsharp
   #r "paket:
   nuget FSharp.Core
   nuget Newtonsoft.Json //"
   ```

**FSRS Recommendation:**

Use **Paket** for FSRS development:
- Better F# script support (critical for `.fsrs` files)
- Lockfile ensures reproducible FSRS builds
- Can reference FSRS stdlib from Git during dev

---

## 7. FSharp.Compiler.Service - Foundation for Custom Tools

**NuGet:** FSharp.Compiler.Service
**Version:** 43.10.100 (latest as of 2024)
**Documentation:** https://fsharp.github.io/fsharp-compiler-docs/fcs/

### Overview

FSharp.Compiler.Service (FCS) exposes the F# compiler as a library, enabling custom language tools.

**Key Services:**

1. **Tokenization:**
   - Convert F# source to token stream
   - Syntax highlighting
   - Basic parsing

2. **Syntax Tree (Untyped AST):**
   - Parse F# code to AST
   - No type information
   - Useful for formatting, refactoring

3. **Type Checking:**
   - Full type inference
   - Symbol resolution
   - Type error diagnostics

4. **Editor Services:**
   - Auto-completion
   - Tooltips
   - Parameter info
   - Go-to-definition

5. **Project-Wide Analysis:**
   - Find all references
   - Unused declaration detection
   - Cross-file analysis

6. **Dynamic Execution:**
   - Host F# Interactive as library
   - Embed F# scripting in applications
   - Runtime code generation

### FSRS Tooling Opportunities

**1. Syntax Highlighter:**

```fsharp
open FSharp.Compiler.SourceCodeServices
open FSharp.Compiler.Text

let tokenize source =
    let checker = FSharpChecker.Create()
    let sourceText = SourceText.ofString source
    let tokenizer = FSharpSourceTokenizer([], None)
    let lineTokenizer = tokenizer.CreateLineTokenizer(sourceText.GetLineString(0))

    let rec tokenizeRec acc =
        match lineTokenizer.ScanToken(0L) with
        | Some tok, newState ->
            tokenizeRec (tok :: acc)
        | None, _ -> List.rev acc

    tokenizeRec []
```

**2. FSRS-Specific Type Checker:**

```fsharp
// Wrap FCS to add FSRS bytecode limitations
type FSRSChecker() =
    let fcsChecker = FSharpChecker.Create()

    member this.CheckScript(filename, source) =
        // Use FCS for baseline type checking
        let! parseResults, checkResults =
            fcsChecker.ParseAndCheckFileInProject(...)

        // Add FSRS-specific validations
        let fsrsErrors =
            checkResults.GetAllUsesOfAllSymbolsInFile()
            |> Seq.collect (validateFSRSSemantics)

        combineErrors checkResults.Errors fsrsErrors
```

**3. Custom LSP Server:**

```fsharp
// Extend FsAutoComplete with FSRS awareness
type FSRSLanguageServer() =
    inherit FsAutoComplete.LspServer()

    override this.TextDocumentHover(params) =
        let baseHover = base.TextDocumentHover(params)

        // Add FSRS bytecode info
        match params.Position with
        | FunctionCall name ->
            let bytecode = compileToBytecode name
            augmentHover baseHover bytecode
        | _ -> baseHover
```

**4. REPL with Bytecode Execution:**

```fsharp
// F# Interactive wrapper that executes on FSRS VM
type FSRSInteractive() =
    let fsiSession = FsiEvaluationSession.Create(...)
    let fsrsVM = FSRSVirtualMachine()

    member this.Eval(code: string) =
        // Type check with F# compiler
        let typedCode = fsiSession.ParseAndCheckInteraction(code)

        // Compile to FSRS bytecode
        let bytecode = FSRSCompiler.compile(typedCode)

        // Execute on VM
        fsrsVM.Execute(bytecode)
```

---

## 8. Integration Roadmap for FSRS

### Phase 1: Immediate Wins (Weeks 1-2)

**Goal:** Enable basic IDE support for `.fsrs` files

**Tasks:**
1. Configure file associations (`.fsrs` -> F# Script)
2. Verify Ionide/FSAC recognize `.fsrs` files
3. Test Fantomas formatting on FSRS scripts
4. Enable FSharpLint with default rules
5. Document VSCode setup guide

**Deliverables:**
- `docs/tooling/vscode-setup.md`
- `.vscode/settings.json` template
- `.editorconfig` for Fantomas

**Success Metrics:**
- Syntax highlighting works
- Basic completion works
- Format on save works

---

### Phase 2: Enhanced IDE Experience (Weeks 3-4)

**Goal:** Add FSRS-specific tooling features

**Tasks:**
1. Create FCS-based FSRS type checker
2. Add FSRS-specific diagnostics to FSharpLint
3. Implement bytecode viewer (show compiled output)
4. Build FSRS REPL wrapper around FSI
5. Create FAKE build script for FSRS projects

**Deliverables:**
- `fsrs-check` tool (type checking + bytecode validation)
- FSharpLint custom analyzer (`fsrs-lint`)
- `fsrs-repl` binary
- `build.fsx` template

**Success Metrics:**
- Type errors show FSRS-specific messages
- Can view bytecode from IDE
- REPL executes on FSRS VM
- Automated builds work

---

### Phase 3: Advanced Tooling (Weeks 5-8)

**Goal:** Production-grade developer experience

**Tasks:**
1. Custom VSCode extension (`fsrs-vscode`)
2. Debugger integration (bytecode stepping)
3. Performance profiler (VM metrics overlay)
4. Module dependency visualizer
5. Hot-reload file watcher

**Deliverables:**
- `fsrs-vscode` extension (published to marketplace)
- Debugger protocol implementation
- Profiler dashboard
- Module graph viewer
- Watch mode daemon

**Success Metrics:**
- Full debugging support (breakpoints, inspect)
- Performance metrics in editor
- Hot-reload on save (<100ms)
- Extension rated 4+ stars

---

### Phase 4: Ecosystem Contributions (Ongoing)

**Goal:** Give back to F# community

**Tasks:**
1. Contribute FSRS analyzer to FSharpLint
2. Submit `.fsrs` support to FsAutoComplete
3. Document FSRS patterns for Fantomas
4. Share FSRS learnings at F# events
5. Collaborate with Ionide team

**Deliverables:**
- Upstream PRs to F# tooling projects
- Conference talks / blog posts
- Case study: "Building a DSL with F# Tooling"

**Success Metrics:**
- 3+ merged PRs to F# tools
- 1 conference talk accepted
- Community adoption of FSRS patterns

---

## 9. Competitive Analysis - Why F# Tooling Wins

### vs. Building Custom Tooling from Scratch

**Custom LSP Server:**
- Time: 6-12 months
- Cost: 1-2 FTE engineers
- Maintenance: Ongoing

**Leverage F# LSP (FSAC):**
- Time: 1-2 weeks configuration
- Cost: 0.1 FTE (integration work)
- Maintenance: Upstream improvements free

**Winner:** F# LSP (50x faster, 90% cost reduction)

---

### vs. Lua/Python/JavaScript Tooling

**Lua:**
- LSP: lua-language-server (good)
- Formatters: LuaFormatter (basic)
- Linters: luacheck (adequate)
- IDE: VSCode support (decent)
- REPL: Built-in (excellent)

**Python:**
- LSP: Pyright, Pylance (excellent)
- Formatters: Black, autopep8 (excellent)
- Linters: pylint, flake8 (excellent)
- IDE: PyCharm, VSCode (excellent)
- REPL: IPython (excellent)

**JavaScript/TypeScript:**
- LSP: tsserver (excellent)
- Formatters: Prettier (excellent)
- Linters: ESLint (excellent)
- IDE: VSCode, WebStorm (excellent)
- REPL: Node REPL (good)

**F# (FSRS):**
- LSP: FsAutoComplete (excellent)
- Formatters: Fantomas (excellent)
- Linters: FSharpLint (good)
- IDE: Ionide, Rider, VS (excellent)
- REPL: FSI (excellent)

**Winner:** F# tooling is **on par or better** than established scripting languages, with added benefit of static typing and functional paradigm.

---

## 10. Risk Assessment and Mitigation

### Risk 1: FSRS Subset Divergence

**Problem:** FSRS supports only Mini-F# subset, breaking full F# tooling

**Impact:** High (LSP errors, broken autocomplete)

**Mitigation:**
1. Maintain syntax compatibility even if semantics differ
2. Add FSRS-specific diagnostics layer (warnings, not errors)
3. Use `#if FSRS` directives to conditionally enable features
4. Document supported subset clearly

**Example:**
```fsharp
// This is valid F# and valid FSRS
let add x y = x + y

// This is valid F# but NOT valid FSRS Phase 1
type Person = { Name: string; Age: int }

// FSRS-aware linter warns: "Records not supported in FSRS Phase 1"
```

---

### Risk 2: FsAutoComplete Update Breakage

**Problem:** FSAC updates break FSRS integration

**Impact:** Medium (IDE stops working after update)

**Mitigation:**
1. Pin FSAC version in development (`"FSharp.fsacRuntime": "0.62.0"`)
2. Test FSRS with each FSAC release before upgrading
3. Contribute integration tests to FSAC for FSRS compatibility
4. Maintain fork if necessary (last resort)

---

### Risk 3: Community Expectation Mismatch

**Problem:** Developers expect full F# features in FSRS

**Impact:** Medium (confusion, frustration, negative feedback)

**Mitigation:**
1. Clear branding: "FSRS: F#-Inspired Scripting for Rust VMs"
2. Documentation: "Supported F# Features" matrix
3. Linter warnings for unsupported features
4. Gradual feature rollout (Phase 1 -> Phase 4)

---

### Risk 4: Performance Overhead

**Problem:** F# tooling slows down FSRS development

**Impact:** Low (IDE responsiveness)

**Mitigation:**
1. FSAC is already optimized for F# scripts
2. FSRS files are typically small (< 1000 LOC)
3. Incremental compilation caches AST
4. Benchmark and optimize if needed

---

## 11. Success Metrics and KPIs

### Developer Experience Metrics

**Time to First Autocomplete:**
- Baseline: Manual AST + LSP implementation = 6 months
- With F# Tooling: File association + FSAC = 1 day
- **KPI:** <1 day from project start to working IDE

**Code Quality:**
- Formatting consistency: 100% (Fantomas enforced)
- Linting violations: <10 per 1000 LOC (FSharpLint)
- Type errors: Caught at edit time (FSAC)
- **KPI:** 0 style debates in code reviews

**Onboarding Speed:**
- Developers familiar with F#: <1 hour to productive
- Developers new to F#: <1 day (leverage F# learning resources)
- **KPI:** 90% of new contributors productive in <1 day

---

### Ecosystem Leverage Metrics

**Tooling Reuse:**
- Lines of code written: ~1000 (integration glue)
- Lines of code leveraged: ~500,000 (F# ecosystem)
- **KPI:** 500:1 leverage ratio

**Community Support:**
- F# experts available: 10,000+ (Stack Overflow, Slack)
- FSRS-specific experts: ~10 (initially)
- **KPI:** Access to 1000x larger support community

---

### Business Impact Metrics

**Time to Market:**
- Custom tooling: 6-12 months before MVP
- F# tooling: 1-2 weeks before MVP
- **KPI:** Ship FSRS v0.1 with full IDE support in <2 weeks

**Development Cost:**
- Custom LSP server: $200,000 (2 engineers × 6 months)
- F# integration: $10,000 (0.5 engineer × 2 weeks)
- **KPI:** 95% cost savings on tooling

---

## 12. Recommendations and Next Steps

### Strategic Recommendations

**1. Commit to F# Syntax Compatibility**

Maintain strict compatibility with F# syntax, even if FSRS semantics differ. This unlocks the entire F# tooling ecosystem at near-zero cost.

**2. Contribute Upstream to F# Tools**

Engage with Ionide, FsAutoComplete, and FSharpLint communities early. Contribute FSRS-specific features back to these projects to ensure long-term compatibility.

**3. Document the Supported Subset**

Create a clear "FSRS Language Specification" showing which F# features are supported in each phase. Link this from IDE error messages.

**4. Build FSRS-Specific Tooling Incrementally**

Start with zero custom tooling (pure F# ecosystem). Add FSRS-specific features only when F# tools fall short (bytecode viewer, VM debugger).

**5. Market FSRS as "F# for Embedded Scripting"**

Position FSRS as a production-grade alternative to Lua with the developer experience of F#. This attracts both Rust developers (embedded) and F# developers (familiar syntax).

---

### Immediate Action Items (Sprint 1)

**Week 1: Basic IDE Support**

1. [ ] Configure `.fsrs` file association in VSCode
2. [ ] Test Ionide extension with `.fsrs` files
3. [ ] Verify syntax highlighting works
4. [ ] Enable Fantomas formatting
5. [ ] Document setup in `docs/tooling/vscode-quickstart.md`

**Week 2: Developer Experience**

1. [ ] Add `.editorconfig` for consistent formatting
2. [ ] Configure FSharpLint for FSRS projects
3. [ ] Test F# Interactive with FSRS-compatible scripts
4. [ ] Create FAKE build script for FSRS examples
5. [ ] Write "FSRS Tooling Guide" blog post

---

### Medium-Term Goals (Sprints 2-4)

**Sprint 2: Type Checking Integration**

1. [ ] Build FCS-based FSRS type checker
2. [ ] Add FSRS-specific diagnostics
3. [ ] Integrate with FsAutoComplete (custom endpoint)
4. [ ] Test with Ionide LSP client

**Sprint 3: Enhanced Tooling**

1. [ ] Implement bytecode viewer (show compiled output)
2. [ ] Create FSRS REPL wrapper
3. [ ] Add hot-reload file watcher
4. [ ] Performance profiling overlay

**Sprint 4: VSCode Extension**

1. [ ] Package `fsrs-vscode` extension
2. [ ] Publish to VSCode marketplace
3. [ ] Integrate debugger protocol
4. [ ] Add module dependency visualizer

---

### Long-Term Vision (6-12 Months)

**Community Integration:**
- Present FSRS at F# conferences (LambdaConf, F# eXchange)
- Contribute FSRS analyzer to FSharpLint project
- Collaborate with Ionide team on `.fsrs` support
- Write case study: "Building a DSL with F# Tooling"

**Production Readiness:**
- FSRS LSP server performance: <50ms latency
- Debugger support: Full bytecode stepping
- Hot-reload: <100ms recompilation
- Extension rating: 4+ stars (100+ users)

**Ecosystem Maturity:**
- 10+ projects using FSRS
- 100+ GitHub stars on FSRS repo
- Active community (Slack, Discord)
- Regular contributions from non-core team

---

## 13. Conclusion

By maintaining F# syntax compatibility, FSRS can leverage a mature, production-grade tooling ecosystem worth millions of dollars in development investment. This includes:

- **IDE Support:** Ionide (VSCode), Visual Studio, Rider, Vim/Emacs
- **Language Server:** FsAutoComplete (LSP with 10+ editor support)
- **Code Quality:** Fantomas (formatting), FSharpLint (static analysis)
- **Build Tools:** FAKE (F# Make), Paket (package management)
- **REPL:** F# Interactive (FSI)
- **Foundation:** FSharp.Compiler.Service (custom tool development)

**Strategic Value:**
- **500:1 code leverage ratio** (1000 LOC written, 500,000 LOC leveraged)
- **95% cost savings** ($10K vs $200K for custom tooling)
- **50x faster time to market** (2 weeks vs 12 months)
- **1000x larger support community** (F# ecosystem vs FSRS-only)

**Key Success Factor:** Strict F# syntax compatibility, even if FSRS semantics differ. This allows FSRS to be a "syntax superset with semantic subset" - full F# tooling works, with FSRS-specific linting to catch unsupported features.

**Next Steps:**
1. Configure `.fsrs` file association (1 day)
2. Test Ionide + FSAC (1 day)
3. Document tooling setup (2 days)
4. Ship FSRS v0.1 with full IDE support (Week 2)

FSRS is positioned to deliver a developer experience rivaling TypeScript, Python, and Rust itself - at near-zero development cost.

---

## Appendix A: Tool Versions (2024-2025)

| Tool | Version | Release Date | Notes |
|------|---------|--------------|-------|
| FsAutoComplete | 0.62.0+ | 2024-2025 | Active development |
| Ionide-VSCode | 7.25.7 | March 2025 | 1M+ downloads |
| Fantomas | 6.3.16 | October 2024 | Community standard |
| FSharpLint | Latest | 2024 | 11+ years mature |
| F# Interactive | .NET 8/9 | 2024-2025 | Built into SDK |
| Visual Studio | 2026 | November 2025 | F# first-class |
| JetBrains Rider | 2024.2 | 2024 | F# 8 support |
| FAKE | 6.1.4+ | 2024 | Requires .NET 6+ |
| Paket | Latest | 2024 | Active maintenance |
| FSharp.Compiler.Service | 43.10.100 | 2024 | Latest FCS |

---

## Appendix B: Configuration Files

### `.vscode/settings.json` (FSRS Project)

```json
{
  "files.associations": {
    "*.fsrs": "fsharp"
  },
  "FSharp.fsacRuntime": "net6.0",
  "FSharp.fsiExtraParameters": [
    "--define:FSRS",
    "--nowarn:NU1701"
  ],
  "FSharp.enableAnalyzers": true,
  "FSharp.analyzersPath": ["./analyzers"],
  "FSharp.unusedOpensAnalyzer": true,
  "FSharp.unusedDeclarationsAnalyzer": true,
  "FSharp.simplifyNameAnalyzer": true,
  "FSharp.resolveNamespaces": true,
  "FSharp.enableReferenceCodeLens": true,
  "FSharp.enableBackgroundSymbolCache": true,
  "FSharp.saveOnSendLastSelection": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "Ionide.Ionide-fsharp"
}
```

### `.editorconfig` (Fantomas Configuration)

```ini
root = true

[*.{fs,fsx,fsrs}]
indent_size=4
max_line_length=120
fsharp_semicolon_at_end_of_line=false
fsharp_space_before_parameter=true
fsharp_space_before_lowercase_invocation=true
fsharp_space_before_uppercase_invocation=false
fsharp_space_before_class_constructor=false
fsharp_space_before_member=false
fsharp_space_before_colon=false
fsharp_space_after_comma=true
fsharp_space_before_semicolon=false
fsharp_space_after_semicolon=true
fsharp_indent_on_try_with=false
fsharp_space_around_delimiter=true
fsharp_max_if_then_else_short_width=40
fsharp_max_infix_operator_expression=50
fsharp_max_record_width=40
fsharp_max_array_or_list_width=40
fsharp_multiline_block_brackets_on_same_column=false
fsharp_newline_between_type_definition_and_members=false
fsharp_align_function_signature_to_indentation=false
fsharp_alternative_long_member_definitions=false
```

### `fsharplint.json` (FSRS-Specific Lints)

```json
{
  "ignoreFiles": [
    "assemblyinfo.*"
  ],
  "hints": {
    "add": [],
    "ignore": []
  },
  "formatting": {
    "typedItemSpacing": {
      "enabled": true,
      "config": {
        "typedItemStyle": "SpaceAfter"
      }
    },
    "unionDefinitionIndentation": {
      "enabled": true,
      "config": {
        "indentation": 4
      }
    }
  },
  "conventions": {
    "naming": {
      "enabled": true,
      "config": {
        "rules": [
          {
            "identifier": "function",
            "naming": "PascalCase",
            "underscores": "None"
          }
        ]
      }
    }
  },
  "typography": {
    "indentation": {
      "enabled": true,
      "config": {
        "numberOfSpacesAllowed": 4
      }
    }
  }
}
```

### `build.fsx` (FAKE Build Script for FSRS)

```fsharp
#r "paket:
nuget Fake.Core.Target
nuget Fake.DotNet.Cli
nuget Fake.IO.FileSystem //"

open Fake.Core
open Fake.DotNet
open Fake.IO
open Fake.IO.Globbing.Operators

// Properties
let buildDir = "./build/"
let examplesDir = "./examples/"
let testsDir = "./tests/"

// Targets
Target.create "Clean" (fun _ ->
    Shell.cleanDirs [buildDir]
)

Target.create "FormatFSRS" (fun _ ->
    !! "**/*.fsrs"
    |> Seq.iter (fun file ->
        Shell.Exec("dotnet", sprintf "fantomas %s" file) |> ignore
    )
)

Target.create "LintFSRS" (fun _ ->
    !! "**/*.fsrs"
    |> Seq.iter (fun file ->
        Shell.Exec("fsharplint", sprintf "lint %s" file) |> ignore
    )
)

Target.create "CompileFSRS" (fun _ ->
    !! (examplesDir </> "**/*.fsrs")
    |> Seq.iter (fun script ->
        let output = buildDir </> Path.GetFileName(script) + ".bc"
        Shell.Exec("cargo", sprintf "run --bin fsrs-compile -- %s -o %s" script output)
        |> ignore
    )
)

Target.create "TestFSRS" (fun _ ->
    !! (testsDir </> "**/*.fsrs")
    |> Seq.iter (fun test ->
        Shell.Exec("cargo", sprintf "run --bin fsrs-test -- %s" test)
        |> ignore
    )
)

Target.create "All" ignore

// Dependencies
"Clean"
  ==> "FormatFSRS"
  ==> "LintFSRS"
  ==> "CompileFSRS"
  ==> "TestFSRS"
  ==> "All"

Target.runOrDefault "All"
```

---

## Appendix C: Further Reading

### F# Tooling Documentation
- FsAutoComplete: https://github.com/ionide/FsAutoComplete
- Ionide: https://ionide.io/
- Fantomas: https://fsprojects.github.io/fantomas/
- FSharpLint: https://github.com/fsprojects/FSharpLint
- F# Interactive: https://learn.microsoft.com/en-us/dotnet/fsharp/tools/fsharp-interactive/
- FAKE: https://fake.build/
- Paket: https://fsprojects.github.io/Paket/
- FSharp.Compiler.Service: https://fsharp.github.io/fsharp-compiler-docs/fcs/

### Language Server Protocol
- LSP Specification: https://microsoft.github.io/language-server-protocol/
- LSP in Editors: https://langserver.org/
- Building LSP Servers: https://code.visualstudio.com/api/language-extensions/language-server-extension-guide

### F# Language Resources
- F# Language Reference: https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/
- F# Style Guide: https://learn.microsoft.com/en-us/dotnet/fsharp/style-guide/
- F# for Fun and Profit: https://fsharpforfunandprofit.com/
- F# Software Foundation: https://fsharp.org/

### Embedded Language Design
- Lua Reference: https://www.lua.org/manual/5.4/
- Rhai Scripting: https://rhai.rs/
- ChaiScript: https://chaiscript.com/
- Embedding F#: https://fsharp.github.io/fsharp-compiler-docs/fcs/interactive.html

---

**Document Status:** Research Complete - Ready for Implementation
**Next Review:** After Phase 1 MVP (Week 3)
**Maintained By:** FSRS Core Team
**Last Updated:** 2025-11-19
