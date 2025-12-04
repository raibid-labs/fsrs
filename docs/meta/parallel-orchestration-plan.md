# Fusabi Parallel Orchestration Plan 🚀

**Date:** November 20, 2025
**Status:** Phase 3 Completion Sprint
**Methodology:** Parallel Agent Execution with Git Hygiene

---

## Executive Summary

Based on comprehensive analysis of the Fusabi codebase:
- **Phase 1-2:** ✅ Complete (1,301 passing tests)
- **Phase 3:** ~60% complete (records/DUs parsing done, execution uncertain)
- **Outstanding Work:** 12 active issues + 2 failing PRs

This plan orchestrates **6 parallel workstreams** using feature branches, pull requests, and squash merges to complete Phase 3 and unlock Phase 4.

---

## Current State Analysis

### ✅ What's Complete
- Core language (Phase 1): Lexer, parser, AST, VM, bytecode compiler
- Language features (Phase 2): Closures, recursion, tuples, lists, arrays, pattern matching, type inference
- Host interop: Basic function registration working
- Standard library: List, String, Option modules

### ⚠️ What's Partial
- Records: Parsing complete, bytecode execution uncertain
- Discriminated Unions: Parsing complete, bytecode execution uncertain
- Module system: AST/registry complete, compiler integration unclear
- Benchmarking: Criterion setup done, actual benchmarks missing

### ❌ What's Missing
- Re-entrant host functions (blocks higher-order stdlib functions)
- Mark-and-sweep garbage collection (memory leaks in cycles)
- Bytecode serialization (.fzb files)
- Implicit prelude & pipeline operator
- Comprehensive example suite
- Full branding/design system

### 🔥 Failing PRs
- PR #72: Branding + HOF support (CI failures)
- PR #73: MCP server (CI failures)

---

## Parallel Orchestration Strategy

### Core Principles (From dgx-pixels)

1. **Feature Branch Isolation**: Each workstream gets its own branch
2. **CI-First Development**: All tests must pass before PR creation
3. **Squash Merge Policy**: Clean main branch history
4. **Agent Autonomy**: Each agent owns their full workflow
5. **Concurrent Execution**: Independent workstreams run in parallel
6. **Git Hygiene**: Proper commit messages, PR descriptions, review process

---

## Workstream Definitions

### 🔴 Critical Path (Week 1 - Parallel Execution)

#### **Workstream 1: Fix Failing PRs**
- **Branch:** Multiple (existing PR branches)
- **Agent:** `devops-automator`
- **Issues:** PR #72, PR #73
- **Tasks:**
  1. Checkout `feat/ws1-hof-support`
  2. Run `cargo fmt --all`
  3. Run `cargo clippy --all -- -D warnings`
  4. Fix all lint/format issues
  5. Rerun tests (`cargo test --workspace`)
  6. Push fixes to PR branch
  7. Repeat for `feat/ws5-mcp-server`
  8. Request re-review from maintainer
- **Dependencies:** None
- **Estimate:** 2-4 hours
- **Success Criteria:** Both PRs pass CI checks

#### **Workstream 2: Re-entrant Host Functions (Issue #60)**
- **Branch:** `feat/issue-60-reentrant-host-fns`
- **Agent:** `backend-architect`
- **Priority:** CRITICAL (blocks stdlib HOF)
- **Tasks:**
  1. Create feature branch from main
  2. Refactor `HostFn` type signature in `host.rs`:
     ```rust
     type HostFn = Box<dyn Fn(&mut Vm, &[Value]) -> Result<Value>>;
     ```
  3. Update `HostRegistry::call` to pass `&mut Vm`
  4. Update VM dispatch in `Instruction::Call`
  5. Add `Vm::call_closure()` helper method
  6. Update all stdlib functions (list.rs, string.rs, option.rs)
  7. Implement `List.map` as proof-of-concept
  8. Write 20+ tests for re-entrant calls
  9. Run full test suite
  10. Create PR with description
  11. Squash merge after approval
- **Dependencies:** None
- **Estimate:** 2-3 days
- **Success Criteria:** `List.map` works with script closures

#### **Workstream 3: Records & DU Execution (Issues #65 partial)**
- **Branch:** `feat/issue-65-records-dus-execution`
- **Agent:** `backend-architect` (different instance)
- **Priority:** HIGH (completes Phase 3 core)
- **Tasks:**
  1. Create feature branch from main
  2. Analyze existing record/DU parsing tests
  3. Add bytecode instructions for records:
     - `CreateRecord(field_count)`
     - `GetRecordField(field_index)`
     - `SetRecordField(field_index)`
  4. Add bytecode instructions for DUs:
     - `CreateVariant(tag, field_count)`
     - `GetVariantTag`
     - `GetVariantField(field_index)`
  5. Update compiler to emit new instructions
  6. Update VM to execute new instructions
  7. Migrate parsing tests to execution tests
  8. Write 30+ end-to-end tests
  9. Create PR
  10. Squash merge after approval
- **Dependencies:** None
- **Estimate:** 3-4 days
- **Success Criteria:** Records and DUs execute correctly in VM

---

### 🟡 High Priority (Week 2 - Sequential After Critical)

#### **Workstream 4: Bytecode Serialization (Issue #65)**
- **Branch:** `feat/issue-65-bytecode-serialization`
- **Agent:** `backend-architect`
- **Priority:** HIGH (enables `fus grind` command)
- **Tasks:**
  1. Wait for Workstream 3 to merge (depends on final instruction set)
  2. Create feature branch from main
  3. Add `serde` feature to `fusabi-vm/Cargo.toml`
  4. Derive `Serialize`/`Deserialize` for:
     - `Chunk` (with custom serializer for Rc handling)
     - `Instruction`
     - `Value` (code constants only, not runtime closures)
  5. Define magic bytes: `const FZB_MAGIC: &[u8] = b"FZB\x01";`
  6. Implement CLI command `fus grind`:
     - Compile source to Chunk
     - Serialize with bincode
     - Prepend magic bytes
     - Write to `.fzb` file
  7. Update `fus run` to detect magic bytes and deserialize
  8. Write 15+ serialization tests
  9. Create PR
  10. Squash merge
- **Dependencies:** Workstream 3 (final instruction set)
- **Estimate:** 2-3 days
- **Success Criteria:** `fus grind script.fsx` creates working `.fzb` file

#### **Workstream 5: Mark-and-Sweep GC (Issue #61)**
- **Branch:** `feat/issue-61-mark-sweep-gc`
- **Agent:** `rust-pro` (systems programming specialist)
- **Priority:** HIGH (prevents memory leaks)
- **Tasks:**
  1. Create feature branch from main
  2. Design `Trace` trait in new `gc.rs`:
     ```rust
     trait Trace {
         fn trace(&self, tracer: &mut Tracer);
     }
     ```
  3. Implement `Trace` for Value, Record, Variant, Closure
  4. Create `GcHeap` allocator struct
  5. Add GC metadata to Value (color bits for marking)
  6. Integrate GC roots into VM (stack, globals, upvalues)
  7. Implement `Vm::collect_garbage()`:
     - Mark phase: Traverse roots
     - Sweep phase: Deallocate unmarked objects
  8. Add GC trigger heuristics (allocation count threshold)
  9. Write 25+ GC tests (cycle detection, stress tests)
  10. Run memory leak detection tools
  11. Create PR
  12. Squash merge
- **Dependencies:** None (can run in parallel)
- **Estimate:** 4-5 days
- **Success Criteria:** No memory leaks with recursive structures

---

### 🟢 Medium Priority (Week 3 - Parallel Execution)

#### **Workstream 6: Implicit Prelude & Operators (Issue #63)**
- **Branch:** `feat/issue-63-implicit-prelude`
- **Agent:** `frontend-developer`
- **Priority:** MEDIUM (UX improvement)
- **Tasks:**
  1. Create feature branch from main
  2. Create `stdlib/core.rs` with:
     - `print`, `printfn`
     - `id`, `ignore`, `fst`, `snd`
  3. Register Core functions in `StdlibRegistry`
  4. Modify `Compiler::compile_program` to auto-inject Core bindings
  5. Add `Token::PipeRight` (`|>`) to lexer
  6. Add parser rule for `|>` with correct precedence
  7. Desugar `a |> f` to `f a` in compiler
  8. Write 15+ tests for Core functions
  9. Write 10+ tests for pipeline operator
  10. Create PR
  11. Squash merge
- **Dependencies:** None
- **Estimate:** 1-2 days
- **Success Criteria:** `1 |> add 2 |> mul 3` works

#### **Workstream 7: Comprehensive Benchmarking (Issue #62)**
- **Branch:** `feat/issue-62-benchmarking-suite`
- **Agent:** `performance-benchmarker`
- **Priority:** MEDIUM (performance tracking)
- **Tasks:**
  1. Create feature branch from main
  2. Implement micro-benchmarks in `fusabi-vm/benches/`:
     - `op_dispatch.rs`: Tight loop of Add/Sub/Mul/Div/Call
     - `alloc.rs`: Record vs Tuple creation (10k iterations)
     - `gc.rs`: GC pressure tests
  3. Create macro-benchmark scripts in `examples/benchmarks/`:
     - `fib.fsx`: Recursive fibonacci(30)
     - `sieve.fsx`: Prime sieve with lists
     - `binary_trees.fsx`: GC stress test
  4. Add Rhai/Rune/Lua comparison harness
  5. Create `just benchmark` recipe
  6. Write benchmark documentation
  7. Set up CI benchmark tracking (optional)
  8. Create PR
  9. Squash merge
- **Dependencies:** Workstream 5 helpful (GC benchmarks)
- **Estimate:** 3-4 days
- **Success Criteria:** Benchmarks run and show Lua-comparable performance

---

### 🔵 Low Priority (Week 4 - Documentation & Polish)

#### **Workstream 8: Example Suite Expansion (Issue #66)**
- **Branch:** `feat/issue-66-example-suite`
- **Agent:** `backend-architect`
- **Priority:** LOW (marketing/docs)
- **Tasks:**
  1. Create feature branch from main
  2. Create `examples/bevy_scripting/`:
     - Rust host with Bevy
     - `behavior.fsx` script for entity logic
     - README with setup instructions
  3. Create `examples/ratatui_layout/`:
     - Rust host with Ratatui
     - `layout.fsx` script returning layout Record
     - Demo TUI rendering
  4. Create `examples/burn_config/`:
     - Neural net config in F#
     - Rust code to parse and initialize Burn model
  5. Create `examples/web_server/`:
     - Axum server
     - `validation.fsx` for endpoint logic
  6. Create `examples/computations/`:
     - Result computation expression example
  7. Update `examples/README.md` with catalog
  8. Create PR
  9. Squash merge
- **Dependencies:** Workstream 2 (for HOF in scripts)
- **Estimate:** 5-7 days
- **Success Criteria:** 5 working ecosystem integration examples

#### **Workstream 9: Branding & Documentation (Issues #67-71)**
- **Branch:** Merge PR #72 after fixes (Workstream 1)
- **Agent:** `frontend-developer` + `visual-storyteller`
- **Priority:** LOW (polish)
- **Tasks:**
  1. After PR #72 merges, verify:
     - `docs/branding.md` exists with color palette
     - `docs/omakase.md` exists with cookbook
     - `assets/logo.svg` exists
     - CLI has colorized output
     - README has brand voice
  2. If anything missing, create follow-up branch
  3. Add any remaining brand polish
  4. Create PR (if needed)
  5. Squash merge
- **Dependencies:** Workstream 1 (PR #72 fixes)
- **Estimate:** Already done in PR, just merge
- **Success Criteria:** Full brand identity deployed

#### **Workstream 10: Contributor Guide (Issue #59)**
- **Branch:** Use PR #72 (already includes CONTRIBUTING.md)
- **Agent:** `frontend-developer`
- **Priority:** LOW (onboarding)
- **Tasks:**
  1. Verify PR #72 includes:
     - `CONTRIBUTING.md` with architecture guide
     - `docs/abi.md` with value representation
     - `docs/security.md` with sandboxing notes
  2. If incomplete, create follow-up PR
  3. Expand CONTRIBUTING.md with:
     - How to add new Instruction
     - How to add new stdlib function
     - Testing guidelines
  4. Create PR (if needed)
  5. Squash merge
- **Dependencies:** Workstream 1 (PR #72 merge)
- **Estimate:** 1 day (mostly done)
- **Success Criteria:** Complete contributor onboarding docs

---

## Execution Timeline

### Week 1: Critical Path (Parallel)
```
Day 1-2:
  ├─ [WS1] Fix PR #72 and #73 CI failures ⚡
  ├─ [WS2] Implement re-entrant host functions
  └─ [WS3] Records & DUs execution

Day 3-5:
  ├─ [WS2] Complete HOF support + tests
  ├─ [WS3] Complete records/DUs + tests
  └─ [WS5] Start GC implementation (parallel)

Day 6-7:
  ├─ [WS2] Create PR, review, merge
  ├─ [WS3] Create PR, review, merge
  └─ [WS5] Continue GC work
```

### Week 2: High Priority (Sequential then Parallel)
```
Day 8-10:
  ├─ [WS4] Bytecode serialization (after WS3 merge)
  ├─ [WS5] Complete GC implementation
  └─ [WS6] Implicit prelude (parallel)

Day 11-14:
  ├─ [WS4] Test serialization, create PR, merge
  ├─ [WS5] GC tests, create PR, merge
  ├─ [WS6] Complete prelude, create PR, merge
  └─ [WS7] Start benchmarking suite
```

### Week 3: Medium Priority (Parallel)
```
Day 15-21:
  ├─ [WS7] Complete benchmarks, create PR, merge
  └─ [WS8] Start example suite expansion
```

### Week 4: Low Priority (Documentation)
```
Day 22-28:
  ├─ [WS8] Complete examples, create PR, merge
  ├─ [WS9] Finalize branding (merge PR #72 remnants)
  └─ [WS10] Complete contributor docs
```

---

## Git Hygiene Protocol

### Branch Naming Convention
```
feat/issue-<number>-<short-description>
fix/issue-<number>-<short-description>
docs/issue-<number>-<short-description>
```

Examples:
- `feat/issue-60-reentrant-host-fns`
- `feat/issue-61-mark-sweep-gc`
- `docs/issue-59-contributor-guide`

### Commit Message Format
```
<type>(<scope>): <description>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Example:
```
feat(vm): Implement re-entrant host functions

- Refactor HostFn signature to accept &mut Vm
- Add Vm::call_closure() helper method
- Update stdlib functions to use new signature
- Implement List.map as proof-of-concept

Closes #60
```

### Pull Request Template
```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed? What problem does it solve?

## Changes
- Bullet list of key changes
- Focus on user-visible impact

## Testing
- Describe test strategy
- List new tests added
- Confirm all tests pass

## Checklist
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --all -- -D warnings`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)

Closes #<issue-number>
```

### Review Process
1. Agent creates PR with complete description
2. CI runs all checks (format, lint, test, docs)
3. Agent addresses any CI failures
4. Maintainer reviews (can be automated for agent PRs)
5. Squash merge to main (single commit per feature)
6. Delete feature branch

### Squash Merge Policy
```bash
# When merging PR
gh pr merge <number> --squash --delete-branch

# Squash merge message format:
feat(vm): Implement re-entrant host functions (#60)

* Refactor HostFn signature
* Add Vm::call_closure helper
* Update stdlib functions
* Implement List.map

Co-authored-by: Claude <noreply@anthropic.com>
```

---

## Agent Execution Instructions

### For Each Workstream Agent

**Phase 1: Setup**
```bash
# 1. Create feature branch
git checkout main
git pull origin main
git checkout -b feat/issue-XX-description

# 2. Verify clean workspace
git status
cargo clean
cargo build --workspace
cargo test --workspace
```

**Phase 2: Implementation**
```bash
# 3. Implement feature
# - Edit source files
# - Add tests
# - Update documentation

# 4. Run checks continuously
cargo fmt --all
cargo clippy --all -- -D warnings
cargo test --workspace

# 5. Commit frequently with good messages
git add .
git commit -m "feat(scope): Add feature X"
```

**Phase 3: Finalization**
```bash
# 6. Final quality check
cargo build --release
cargo test --workspace --release
cargo doc --workspace --no-deps

# 7. Update CHANGELOG.md
# Add entry under "Unreleased" section

# 8. Final commit
git add .
git commit -m "chore: Update CHANGELOG for issue #XX"
```

**Phase 4: Pull Request**
```bash
# 9. Push to remote
git push origin feat/issue-XX-description

# 10. Create PR
gh pr create \
  --title "feat: Implement Feature X (#XX)" \
  --body "$(cat <<EOF
## Description
[Brief description]

## Motivation
Closes #XX

## Changes
- Change 1
- Change 2

## Testing
- Added XX tests
- All tests pass

## Checklist
- [x] All tests pass
- [x] Code formatted
- [x] No clippy warnings
- [x] Documentation updated
EOF
)"

# 11. Monitor CI
gh pr checks

# 12. Address CI failures if any
# (fix, commit, push again)
```

**Phase 5: Merge**
```bash
# 13. After approval, squash merge
gh pr merge --squash --delete-branch

# 14. Switch back to main
git checkout main
git pull origin main
```

---

## Parallel Execution Matrix

| Workstream | Week | Agent Type | Can Run Parallel With |
|------------|------|------------|-----------------------|
| WS1: Fix PRs | 1 | devops-automator | All (different branches) |
| WS2: HOF | 1 | backend-architect | WS3, WS5 |
| WS3: Records/DUs | 1 | backend-architect | WS2, WS5 |
| WS4: Serialization | 2 | backend-architect | WS5, WS6 (after WS3) |
| WS5: GC | 1-2 | rust-pro | All (independent) |
| WS6: Prelude | 2 | frontend-developer | WS4, WS5, WS7 |
| WS7: Benchmarks | 2-3 | performance-benchmarker | WS6, WS8 |
| WS8: Examples | 3-4 | backend-architect | WS7, WS9, WS10 |
| WS9: Branding | 4 | frontend-developer | WS8, WS10 |
| WS10: Docs | 4 | frontend-developer | WS8, WS9 |

**Maximum Parallelism**: 4 concurrent workstreams (WS2, WS3, WS5, WS1)

---

## Success Metrics

### Code Quality
- [ ] 100% tests passing (maintain 1,301+ tests)
- [ ] Zero clippy warnings
- [ ] Code coverage > 80%
- [ ] All PRs have >3 tests added

### Performance
- [ ] Benchmarks show <10% regression from baseline
- [ ] Startup time < 5ms for simple scripts
- [ ] 5-10M ops/sec sustained throughput

### Completeness
- [ ] All 12 open issues closed
- [ ] Phase 3 marked 100% complete
- [ ] Phase 4 roadmap updated
- [ ] v0.3.0-alpha release tagged

### Documentation
- [ ] CONTRIBUTING.md complete
- [ ] All new features documented
- [ ] CHANGELOG.md updated
- [ ] README.md reflects current state

---

## Risk Mitigation

### Risk 1: GC Implementation Complexity
- **Mitigation:** Allocate 5 days, use Rust-pro agent with systems programming expertise
- **Fallback:** Keep Rc implementation, document known leak patterns

### Risk 2: Records/DUs Execution Uncertain
- **Mitigation:** Start with thorough analysis of existing tests before implementation
- **Fallback:** If execution works, just add more tests; if broken, budget 2 extra days

### Risk 3: PR Merge Conflicts
- **Mitigation:** Frequent rebases on main, WS1-3 merge before WS4-10 start
- **Fallback:** Agent resolves conflicts with `git rebase main`, reruns tests

### Risk 4: CI Pipeline Failures
- **Mitigation:** Run checks locally before push, agents fix their own CI failures
- **Fallback:** devops-automator agent assists with CI debugging

---

## Post-Completion

### After All Workstreams Complete

1. **Update Version**: Bump to `v0.3.0-alpha`
2. **Update README.md**: Change status to "Phase 3 Complete"
3. **Update Roadmap**: Mark Phase 3 100%, detail Phase 4
4. **Create Release**:
   ```bash
   git tag -a v0.3.0-alpha -m "Phase 3 Complete: Advanced Features"
   git push origin v0.3.0-alpha
   gh release create v0.3.0-alpha --title "v0.3.0-alpha - Phase 3 Complete" --notes-file docs/changelog-0.3.0.md
   ```
5. **Write Blog Post**: Announce Phase 3 completion
6. **Update Social**: Share progress on relevant channels

---

## Monitoring & Reporting

### Daily Standup (Automated)
Each agent reports:
- What I completed yesterday
- What I'm working on today
- Any blockers

### Weekly Summary
- Issues closed: X/12
- PRs merged: X/10
- Test count: Current vs target
- Phase completion: X%

### Completion Report
Final document summarizing:
- All features implemented
- All tests passing
- Performance benchmarks
- Known limitations
- Phase 4 readiness

---

**End of Orchestration Plan**

Next step: Execute Workstream 1 (Fix PRs) immediately, then spawn parallel agents for WS2, WS3, WS5.
