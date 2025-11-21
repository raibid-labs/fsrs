# Fusabi Gem Issues - Workstream Overview

## Mission

Implement 13 critical enhancements to the Fusabi language across VM, frontend, ecosystem, documentation, and branding. These workstreams build upon the completed Phase 1-3 foundation to add advanced features, tooling, polish, and visual identity.

## Project Context

**Fusabi** has completed Phases 1-3, establishing a working Mini-F# interpreter with:
- ✅ Core AST, lexer, parser, and bytecode compiler
- ✅ Bytecode VM with closures, tuples, lists, arrays
- ✅ Pattern matching and type inference (Hindley-Milner)
- ✅ Module system and standard library foundation

The **Gem Issues** workstreams focus on:
1. **VM Enhancement**: Re-entrant host functions, garbage collection
2. **Frontend Polish**: Stdlib prelude, operators, implicit imports
3. **Performance**: Benchmarking, bytecode serialization
4. **Ecosystem**: MCP server for AI agent integration
5. **Examples**: Comprehensive example suite showcasing dual-runtime
6. **Documentation**: Contributor guide, ABI spec, security docs
7. **Branding**: Visual identity, CLI styling, logo, brand voice

## Workstream Organization

### WS1: VM Core - Re-entrant Host Functions
**Status**: 🟡 Ready to Start
**Priority**: CRITICAL
**Dependencies**: None
**Issue**: #1 (Issue 1: Enable Re-entrant Host Functions)

**Objective**: Refactor `HostFn` to allow native Rust functions to call back into the VM, enabling higher-order functions like `List.map` in the standard library.

**Key Deliverables**:
- Refactored `HostFn` signature with `VmContext` or `&mut Vm`
- Updated VM loop to pass VM instance to host functions
- Helper API: `Vm::call_closure(closure, args)`
- Updated stdlib functions to new signature
- Implemented `List.map` as proof of concept

**Estimated Effort**: 3-4 days
**Agent**: `backend-architect`, `coder`

---

### WS2: VM Core - Garbage Collection
**Status**: 🔴 Blocked by WS1 (or can run carefully in parallel)
**Priority**: HIGH
**Dependencies**: WS1 (recommended) or careful coordination
**Issue**: #2 (Issue 2: Implement Mark-and-Sweep GC)

**Objective**: Implement mark-and-sweep garbage collection to reclaim memory from reference cycles that `Rc<RefCell<T>>` cannot handle.

**Key Deliverables**:
- `Trace` trait for `Value`, `Record`, `Variant`, `Closure`, etc.
- `GcHeap` allocator replacing direct `Rc::new`
- VM integration with gc_roots tracking
- `Vm::collect_garbage()` with mark and sweep phases
- Object header with color bit for marking

**Estimated Effort**: 4-5 days
**Agent**: `backend-architect`, `coder`

**⚠️ Conflict Warning**: Both WS1 and WS2 modify `fusabi-vm/src/vm.rs`. Coordinate carefully or run WS2 after WS1 completes.

---

### WS3: Frontend - Stdlib Prelude & Operators
**Status**: 🔴 Blocked by WS1
**Priority**: HIGH
**Dependencies**: WS1 (needs re-entrant functions for full stdlib)
**Issue**: #4 (Issue 4: Implement Implicit Prelude & Core Operators)

**Objective**: Auto-import core functions and operators, implement pipeline operator `|>`, and provide a polished standard library experience.

**Key Deliverables**:
- Core module with `print`, `printfn`, `id`, `ignore`, `fst`, `snd`
- Implicit open mechanism in compiler
- Pipeline operator `|>` lexer/parser support
- Desugaring `a |> f` to `f a`
- Updated all examples to use new prelude

**Estimated Effort**: 3-4 days
**Agent**: `frontend-developer`, `coder`

---

### WS4: Serialization & Performance
**Status**: 🟡 Ready to Start (parallel with WS1-WS3)
**Priority**: MEDIUM
**Dependencies**: None (can run in parallel)
**Issues**: #6 (Bytecode Serialization), #3 (Benchmarking Suite)

**Objective**: Implement bytecode serialization to `.fzb` files for faster startup, and create comprehensive benchmarking suite comparing Fusabi to Rhai, Rune, and Lua.

**Key Deliverables**:
- **Bytecode Serialization**:
  - Serde serialization for `Chunk`, `Instruction`, `Value`
  - Magic bytes `FZB\x01` for `.fzb` files
  - CLI `fus grind <file.fsx>` to compile to bytecode
  - CLI `fus run` auto-detects and loads `.fzb` files
- **Benchmarking**:
  - Criterion micro-benchmarks (op dispatch, allocation)
  - Macro-benchmarks (fib, sieve, binary_trees)
  - Comparison harness for Rhai, Rune, Lua
  - CI integration with performance regression detection

**Estimated Effort**: 4-5 days
**Agent**: `performance-engineer`, `coder`

---

### WS5: Ecosystem - MCP Server
**Status**: 🟡 Ready to Start (fully independent)
**Priority**: MEDIUM
**Dependencies**: None (new crate, no conflicts)
**Issue**: #5 (Issue 5: Create Fusabi-MCP Server)

**Objective**: Create an MCP (Model Context Protocol) server to allow AI agents like Claude to interact with a running Fusabi instance.

**Key Deliverables**:
- New crate `fusabi-mcp` with stdio-based MCP server
- Tool: `eval_fusabi(script: string) -> string`
- Tool: `get_context() -> json` (dumps globals)
- Script execution with timeout and stdout capture
- Integration with Fusabi engine

**Estimated Effort**: 2-3 days
**Agent**: `ai-engineer`, `backend-developer`

---

### WS6: Examples & Documentation
**Status**: 🟡 Can Start, Full Functionality Needs WS1+WS3
**Priority**: LOW-MEDIUM
**Dependencies**: WS1 + WS3 for full example functionality (can start structure immediately)
**Issues**: #7 (Examples Suite), #8 (Contributor Guide)

**Objective**: Create comprehensive examples showcasing Fusabi's dual-runtime capability and embedded utility, plus contributor documentation.

**Key Deliverables**:
- **Examples**:
  - `examples/bevy_scripting/` - Entity behavior scripting
  - `examples/ratatui_layout/` - UI layout from F# script
  - `examples/burn_config/` - Neural net config in F#
  - `examples/web_server/` - Axum server with F# validation
  - `examples/computations/` - Computation expressions
  - `examples/interop_net/` - Syntax compatibility demo
- **Documentation**:
  - `CONTRIBUTING.md` - 3-layer architecture guide
  - `docs/ABI.md` - Internal Value representation and `.fzb` spec
  - `docs/SECURITY.md` - Sandboxing status and future plans

**Estimated Effort**: 5-6 days (can be split: docs 2 days, examples 3-4 days)
**Agent**: `docs-architect`, `coder`, `tutorial-engineer`

---

### WS7: Branding & Visual Identity
**Status**: 🟡 Ready to Start (fully independent)
**Priority**: LOW-MEDIUM
**Dependencies**: None (can run in parallel)
**Issues**: #9 (Visual Identity), #10 (CLI Styling), #11 (Omakase Cookbook), #12 (Logo Assets), #13 (README Brand Voice)

**Objective**: Establish Fusabi's visual identity and brand voice across CLI, documentation, and marketing materials. Adopt "Wasabi + Rust" aesthetic: organic, earthy, spicy, and punchy.

**Key Deliverables**:
- **Visual Identity**:
  - `docs/BRANDING.md` with color palette, typography, emoji guide
  - Color palette: Wasabi Green (`#99CC33`), Rust Orange (`#B7410E`)
- **CLI Styling**:
  - Colorized output (green success, orange errors, yellow warnings)
  - ASCII art banner for `fus --help` and `fus repl`
  - Styled REPL prompt `🟢>`
- **Omakase Cookbook**:
  - `docs/OMAKASE.md` - Curated examples as "chef's choice"
  - Branded examples introduction
- **Logo Assets**:
  - `assets/logo.svg` (vector logo, minimalist "F" + leaf shape)
  - `assets/icon.ico` (Windows binary icon)
  - `assets/social_preview.png` (1280x640 GitHub preview)
- **README Overhaul**:
  - Punchy brand voice: "Small. Potent. Functional."
  - Value proposition: "Rust is hard. Configuration shouldn't be."
  - Spicy feature descriptions

**Estimated Effort**: 3-4 days (design-heavy)
**Agent**: `ui-ux-designer`, `content-marketer`, `frontend-developer`

---

## Dependency Graph

```
WS1 (HOF Support) ──┬──> WS3 (Stdlib Prelude)
                    │
                    └──> WS2 (GC) [soft dependency]
                    │
                    └──> WS6 (Examples) [partial dependency]

WS4 (Serialization + Benchmarks) [PARALLEL - no dependencies]

WS5 (MCP Server) [PARALLEL - no dependencies]

WS6 (Docs + Examples) [PARALLEL - can start structure, needs WS1+WS3 for full functionality]

WS7 (Branding) [PARALLEL - no dependencies, synergy with WS6]
```

## Execution Strategy

### Phase 1: Critical VM Features (Week 1)
**Parallel Launch**:
- **WS1**: Re-entrant Host Functions (CRITICAL PATH)
- **WS4**: Serialization & Benchmarking (PARALLEL)
- **WS5**: MCP Server (PARALLEL)
- **WS6**: Documentation structure (PARALLEL)
- **WS7**: Branding & Visual Identity (PARALLEL)

### Phase 2: Frontend & GC (Week 2)
**Sequential after WS1**:
- **WS2**: Garbage Collection (after WS1 or careful coordination)
- **WS3**: Stdlib Prelude (after WS1)

### Phase 3: Examples & Polish (Week 2-3)
**After WS1 + WS3**:
- **WS6**: Complete examples with full functionality

### Total Timeline: 2-3 weeks

## Git Workflow

**Branch Strategy**:
```
main
├─ feat/ws1-hof-support
├─ feat/ws2-garbage-collection
├─ feat/ws3-stdlib-prelude
├─ feat/ws4-serialization-benchmarks
├─ feat/ws5-mcp-server
├─ feat/ws6-examples-docs
└─ feat/ws7-branding-identity
```

**Merge Protocol**:
1. Each workstream = 1 PR
2. TDD: Write tests first, implement, refactor
3. **Squash merge** to keep history clean
4. **Rebase** on main before merging
5. Delete branch after merge
6. Sub-agents create PRs, rebase, and squash merge

## Quality Checkpoints

### After Week 1
- [ ] WS1 (HOF Support) merged
- [ ] WS4 (Serialization + Benchmarks) merged or in progress
- [ ] WS5 (MCP Server) merged or in progress
- [ ] WS6 (Docs structure) in progress
- [ ] WS7 (Branding) in progress or complete

### After Week 2
- [ ] WS2 (GC) merged
- [ ] WS3 (Stdlib Prelude) merged
- [ ] WS4 (Serialization + Benchmarks) merged
- [ ] WS5 (MCP Server) merged
- [ ] WS6 (Examples) in progress
- [ ] WS7 (Branding) merged

### After Week 3
- [ ] All 7 workstreams complete
- [ ] All PRs merged
- [ ] CI pipeline green
- [ ] Examples running successfully
- [ ] Brand identity established

## Success Metrics

**Gem Issues Complete When**:
- All 13 issues closed
- All 7 workstream PRs merged to main
- CI pipeline green
- Examples demonstrate dual-runtime capability
- Benchmarks show Lua-class performance
- MCP server allows AI agent interaction
- Documentation complete for contributors
- Brand identity established across all touchpoints

## Agent Coordination Protocol

### Every Sub-Agent MUST:

**BEFORE Work**:
```bash
npx claude-flow@alpha hooks pre-task --description "[workstream]-[task]"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-[ws]"
```

**DURING Work**:
```bash
npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "swarm/fusabi-gem/[ws]/[step]"
npx claude-flow@alpha hooks notify --message "[progress update]"
```

**AFTER Work**:
```bash
npx claude-flow@alpha hooks post-task --task-id "[ws]-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Launch Command

**Execute this to begin Gem Issues implementation**:

```
Meta-Orchestrator: Initiate Gem Issues implementation.

Spawn 5 parallel workstreams immediately:
1. WS1 Orchestrator: Start Issue 1 (HOF Support) - CRITICAL PATH
2. WS4 Orchestrator: Start Issue 6 + 3 (Serialization + Benchmarks)
3. WS5 Orchestrator: Start Issue 5 (MCP Server)
4. WS6 Orchestrator: Start Issue 8 (Documentation structure)
5. WS7 Orchestrator: Start Issues 9-13 (Branding & Visual Identity)

After WS1 completes:
6. WS2 Orchestrator: Start Issue 2 (Garbage Collection)
7. WS3 Orchestrator: Start Issue 4 (Stdlib Prelude)
8. WS6 Orchestrator: Complete Issue 7 (Examples)

Follow TDD, maintain git hygiene, coordinate on dependencies.
Target: 2-3 weeks to complete all 13 issues.

Status updates: Daily
PR reviews: Within 12 hours
Merge: Squash and rebase

Begin implementation.
```

---

**Let's ship the Gem Issues! 💎**
