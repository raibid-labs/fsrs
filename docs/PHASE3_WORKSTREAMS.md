# Phase 3 Parallel Workstreams - Implementation Plan

**Status**: Ready for execution
**Timeline**: Weeks 10-13 (Current: Week 10)
**Completion Target**: 60% → 100% (40% currently done)

## Overview

Phase 3 has **3 main workstreams** that can be executed **in parallel** to maximize velocity:

1. **Workstream A**: Module Parser Integration (Frontend)
2. **Workstream B**: Compiler Module Integration (Compiler)
3. **Workstream C**: Standard Library + Host Interop (Runtime)

Each workstream is **independent** and can be worked on concurrently with minimal dependencies.

---

## Workstream A: Module Parser Integration

**Owner**: Frontend team / Parser specialist
**Timeline**: Week 10-11 (2 weeks)
**Dependencies**: None (AST + Registry already complete)
**GitHub Issue**: #58

### Scope

Implement parser support for module syntax to enable multi-file programs.

### Tasks

1. **Parser Methods** (`/rust/crates/fsrs-frontend/src/parser.rs`):
   ```rust
   // Add these methods to Parser impl
   fn parse_program(&mut self) -> Result<Program>
   fn parse_module(&mut self) -> Result<ModuleDef>
   fn parse_import(&mut self) -> Result<Import>
   fn parse_module_items(&mut self) -> Result<Vec<ModuleItem>>
   ```

2. **Integration with Expression Parsing**:
   - Update `parse()` to handle `Program` structure
   - Handle `module` keyword
   - Handle `open` keyword
   - Parse nested modules

3. **Testing**:
   - Unit tests for each parser method (20+ tests)
   - Integration tests with example scripts
   - Error recovery tests

### Success Criteria

- [ ] Parse `module Math = let add x y = x + y`
- [ ] Parse `open Math`
- [ ] Parse nested modules: `module Geo = module Point = ...`
- [ ] All example scripts parse successfully
- [ ] 20+ parser tests passing
- [ ] Zero clippy warnings

### Deliverables

- `parser.rs` updated with module support
- 20+ unit tests
- 3 integration tests using example scripts
- Documentation: parser module support

### Estimated Effort

- Implementation: 4-6 hours
- Testing: 2-3 hours
- **Total**: 6-9 hours

---

## Workstream B: Compiler Module Integration

**Owner**: Compiler team / Backend specialist
**Timeline**: Week 11-12 (2 weeks)
**Dependencies**: Workstream A (parser integration)
**GitHub Issue**: #59

### Scope

Integrate ModuleRegistry with compiler for qualified name resolution and module-aware compilation.

### Tasks

1. **Compiler Updates** (`/rust/crates/fsrs-frontend/src/compiler.rs`):
   ```rust
   // Add ModuleRegistry to Compiler struct
   pub struct Compiler {
       registry: ModuleRegistry,
       // ... existing fields
   }
   ```

2. **Name Resolution**:
   - Implement qualified variable lookup (`Math.add`)
   - Handle `open` imports (bring bindings into scope)
   - Module-aware scope management
   - Resolve type definitions from modules

3. **Compilation Pipeline**:
   - Compile `Program` (not just single `Expr`)
   - Register modules before compilation
   - Compile module items to bytecode
   - Generate module metadata

4. **Testing**:
   - Qualified name resolution tests (15+ tests)
   - Open import tests (10+ tests)
   - Cross-module compilation tests (10+ tests)
   - Error cases (undefined modules, circular imports)

### Success Criteria

- [ ] Compile programs with multiple modules
- [ ] Qualified names resolve correctly: `Math.add 5 10`
- [ ] Open imports work: `open Math; add 5 10`
- [ ] Cross-module type checking
- [ ] 35+ compiler tests passing
- [ ] Zero clippy warnings

### Deliverables

- `compiler.rs` updated with module support
- 35+ unit/integration tests
- Module compilation examples
- Documentation: compiler module integration

### Estimated Effort

- Implementation: 6-8 hours
- Testing: 3-4 hours
- **Total**: 9-12 hours

---

## Workstream C: Standard Library + Host Interop

**Owner**: Runtime team / Interop specialist
**Timeline**: Week 12-13 (2 weeks)
**Dependencies**: Minimal (can prototype in parallel)
**GitHub Issue**: #60

### Scope

Create foundational standard library and host interop API for Rust function registration.

### Tasks

#### Part 1: Standard Library Foundation (Week 12)

1. **List Module** (`/rust/crates/fsrs-vm/src/stdlib/list.rs`):
   ```fsharp
   module List =
       let length : 'a list -> int
       let map : ('a -> 'b) -> 'a list -> 'b list
       let filter : ('a -> bool) -> 'a list -> 'a list
       let fold : ('a -> 'b -> 'a) -> 'a -> 'b list -> 'a
       let reverse : 'a list -> 'a list
       let append : 'a list -> 'a list -> 'a list
   ```

2. **String Module** (`/rust/crates/fsrs-vm/src/stdlib/string.rs`):
   ```fsharp
   module String =
       let length : string -> int
       let concat : string list -> string
       let substring : int -> int -> string -> string
       let trim : string -> string
       let toLower : string -> string
       let toUpper : string -> string
   ```

3. **Option Module** (`/rust/crates/fsrs-vm/src/stdlib/option.rs`):
   ```fsharp
   module Option =
       let map : ('a -> 'b) -> 'a option -> 'b option
       let bind : ('a -> 'b option) -> 'a option -> 'b option
       let defaultValue : 'a -> 'a option -> 'a
       let isSome : 'a option -> bool
       let isNone : 'a option -> bool
   ```

#### Part 2: Host Interop API (Week 13)

1. **Function Registration** (`/rust/crates/fsrs-vm/src/interop.rs`):
   ```rust
   pub trait HostFunction {
       fn call(&self, args: &[Value]) -> Result<Value, RuntimeError>;
   }

   impl VM {
       pub fn register_function<F>(&mut self, name: &str, func: F)
       where
           F: Fn(&[Value]) -> Result<Value, RuntimeError> + 'static
   }
   ```

2. **Type Marshalling**:
   - Automatic conversion: Rust ↔ FSRS types
   - Handle: i64, f64, String, bool, Vec, tuples
   - Error propagation

3. **Host Callback Examples**:
   ```rust
   // Example: Register Rust function
   vm.register_function("get_tab_title", |args| {
       // Convert FSRS Value to Rust type
       let tab_id: i64 = args[0].as_int()?;
       // Call host application
       let title = host_get_tab_title(tab_id)?;
       // Convert back to FSRS Value
       Ok(Value::Str(title))
   });
   ```

4. **Testing**:
   - Standard library tests (40+ tests)
   - Host interop tests (25+ tests)
   - Type marshalling tests (15+ tests)
   - Error handling tests (10+ tests)

### Success Criteria

- [ ] List, String, Option modules complete
- [ ] 15+ stdlib functions implemented
- [ ] Host function registration API working
- [ ] Type marshalling for common types
- [ ] 90+ tests passing
- [ ] Example: Terminal config calling host functions

### Deliverables

- 3 standard library modules
- Host interop API implementation
- 90+ tests
- Terminal emulator example
- Documentation: stdlib reference + interop guide

### Estimated Effort

- Standard library: 8-10 hours
- Host interop: 6-8 hours
- Testing: 4-5 hours
- **Total**: 18-23 hours

---

## Workstream Dependencies

```
Workstream A (Parser)
    ↓
Workstream B (Compiler) ← minimal dependency on A

Workstream C (Stdlib + Interop) ← fully independent
```

**Parallelization Strategy**:
1. Start **A** and **C** immediately (fully parallel)
2. Start **B** prototyping in parallel with A
3. Complete **B** after A finishes
4. All workstreams complete by Week 13

---

## Phase 3 Completion Checklist

### Module System (Workstreams A + B)
- [ ] Module parser integration (20+ tests)
- [ ] Compiler module integration (35+ tests)
- [ ] Multi-file program compilation
- [ ] Qualified name resolution
- [ ] Open imports working
- [ ] Module-aware type checking

### Standard Library (Workstream C - Part 1)
- [ ] List module (8+ functions, 25+ tests)
- [ ] String module (6+ functions, 20+ tests)
- [ ] Option module (5+ functions, 15+ tests)
- [ ] Documentation for all stdlib functions

### Host Interop (Workstream C - Part 2)
- [ ] Function registration API (25+ tests)
- [ ] Type marshalling (15+ tests)
- [ ] Error propagation (10+ tests)
- [ ] Terminal config example working

### Quality Metrics
- [ ] 450+ total tests passing (currently 353+)
- [ ] Zero clippy warnings
- [ ] All examples compile and run
- [ ] Documentation complete

---

## GitHub Issues Created

### Issue #58: Module Parser Integration (Workstream A)

**Title**: `feat: Implement module parser integration for multi-file programs`

**Labels**: `enhancement`, `frontend`, `parser`, `phase-3`

**Assignee**: Frontend team

**Description**:
```markdown
## Objective
Implement parser support for module syntax to enable multi-file FSRS programs.

## Tasks
- [ ] Add `parse_program()`, `parse_module()`, `parse_import()` methods
- [ ] Integrate with existing expression parsing
- [ ] Add 20+ unit tests
- [ ] Add 3 integration tests with example scripts
- [ ] Update documentation

## Success Criteria
- Parse `module Math = let add x y = x + y`
- Parse `open Math`
- Parse nested modules
- All example scripts parse successfully
- Zero clippy warnings

## Estimated Effort
6-9 hours

## Dependencies
None (AST + Registry already complete)
```

---

### Issue #59: Compiler Module Integration (Workstream B)

**Title**: `feat: Integrate ModuleRegistry with compiler for qualified names`

**Labels**: `enhancement`, `compiler`, `phase-3`

**Assignee**: Compiler team

**Description**:
```markdown
## Objective
Integrate ModuleRegistry with compiler for qualified name resolution and module-aware compilation.

## Tasks
- [ ] Add ModuleRegistry to Compiler struct
- [ ] Implement qualified variable lookup (`Math.add`)
- [ ] Handle `open` imports
- [ ] Module-aware scope management
- [ ] Add 35+ unit/integration tests
- [ ] Update documentation

## Success Criteria
- Compile programs with multiple modules
- Qualified names resolve: `Math.add 5 10`
- Open imports work: `open Math; add 5 10`
- Cross-module type checking
- Zero clippy warnings

## Estimated Effort
9-12 hours

## Dependencies
Issue #58 (parser integration)
```

---

### Issue #60: Standard Library + Host Interop API (Workstream C)

**Title**: `feat: Implement standard library foundation and host interop API`

**Labels**: `enhancement`, `stdlib`, `interop`, `vm`, `phase-3`

**Assignee**: Runtime team

**Description**:
```markdown
## Objective
Create foundational standard library (List, String, Option) and host interop API for Rust function registration.

## Tasks

### Part 1: Standard Library (Week 12)
- [ ] Implement List module (8+ functions)
- [ ] Implement String module (6+ functions)
- [ ] Implement Option module (5+ functions)
- [ ] Add 60+ stdlib tests
- [ ] Document all stdlib functions

### Part 2: Host Interop (Week 13)
- [ ] Design function registration API
- [ ] Implement type marshalling
- [ ] Add error propagation
- [ ] Add 30+ interop tests
- [ ] Create terminal config example

## Success Criteria
- List, String, Option modules complete
- 15+ stdlib functions implemented
- Host function registration working
- Type marshalling for common types
- 90+ tests passing
- Terminal config example working

## Estimated Effort
18-23 hours

## Dependencies
None (fully independent workstream)
```

---

## Meta-Orchestrator Coordination

To execute these **3 parallel workstreams** efficiently, we'll use a **meta-orchestrator** pattern:

### Orchestrator Responsibilities

1. **Workstream Coordination**:
   - Launch all 3 workstreams simultaneously
   - Monitor progress and dependencies
   - Sync points for integration
   - Resolve blockers

2. **Communication**:
   - Daily progress updates
   - Dependency resolution
   - Integration testing coordination
   - Documentation sync

3. **Quality Assurance**:
   - Ensure zero clippy warnings across all workstreams
   - Integration testing after each workstream completes
   - Cross-workstream test coverage
   - Final Phase 3 validation

### Orchestration Timeline

```
Week 10:
  - Launch Workstream A (Parser)
  - Launch Workstream C (Stdlib + Interop)
  - Workstream B prototyping

Week 11:
  - Complete Workstream A
  - Launch Workstream B (Compiler)
  - Continue Workstream C (Part 1: Stdlib)

Week 12:
  - Complete Workstream B
  - Continue Workstream C (Part 2: Interop)
  - Integration testing

Week 13:
  - Complete Workstream C
  - Final integration
  - Phase 3 complete! 🎉
```

---

## Launch Command

To start parallel execution, the meta-orchestrator will:

1. **Create GitHub Issues** (#58, #59, #60)
2. **Assign Teams** (Frontend, Compiler, Runtime)
3. **Set Up Tracking** (Project board with 3 columns)
4. **Launch Agents** (3 parallel agent instances)
5. **Monitor Progress** (Daily syncs, blocker resolution)

### Ready to Launch?

Execute: `just phase3-launch` (creates issues, assigns teams, starts parallel work)

---

**Prepared By**: FSRS Roadmap Planning
**Date**: November 19, 2025
**Status**: Ready for execution
**Expected Completion**: Week 13 (Phase 3 → 100%)
