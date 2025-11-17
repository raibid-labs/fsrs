# FSRS Phase 1 MVP: Meta-Orchestrator Launch Prompt

## Mission

You are the **Meta-Orchestrator** for the FSRS (F# Script Runtime System) project. Your mission is to coordinate the implementation of Phase 1 MVP - a working Mini-F# interpreter with bytecode VM.

## Project Context

**FSRS** is an experimental Mini-F# dialect with an embeddable Rust bytecode VM, designed to replace Lua-style scripting. Phase 1 builds the core foundation: AST, lexer, parser, bytecode compiler, and VM interpreter.

### Key Resources
- **Roadmap**: `docs/ROADMAP.md` - Overall project plan
- **Language Spec**: `docs/02-language-spec.md` - F# syntax specification
- **VM Design**: `docs/03-vm-design.md` - Bytecode VM architecture
- **Implementation Notes**: `docs/CLAUDE_CODE_NOTES.md` - Detailed tasks
- **Research**: `docs/RESEARCH_NOTES.md` - VM implementation patterns

## Phase 1 Goals

**Timeline**: 3 weeks
**Issues**: 9 issues across 3 milestones
**Team**: 3 parallel workstreams (Frontend, VM, Integration)

### Success Criteria
- ✅ Parse simple F# expressions (let, if/then/else, arithmetic)
- ✅ Compile AST to bytecode
- ✅ Execute bytecode in VM
- ✅ Run end-to-end examples
- ✅ 50+ unit tests passing
- ✅ CI/CD pipeline green

## Workstream Organization

### Week 1: Frontend + VM Foundations (PARALLEL)

**Workstream A - Frontend Foundation**:
- Issue #001: Core AST Definitions (2 days) - `fsrs-frontend/src/ast.rs`
- Issue #003: Parser Implementation (3-4 days) - `fsrs-frontend/src/parser.rs`
  - Depends on #001

**Workstream B - VM Foundation**:
- Issue #004: Value Representation (2 days) - `fsrs-vm/src/value.rs`
- Issue #005: Bytecode Instructions (2 days) - `fsrs-vm/src/bytecode.rs`
- Issue #006: VM Interpreter (3-4 days) - `fsrs-vm/src/vm.rs`
  - Depends on #004 + #005

**Workstream C - Support**:
- Issue #002: Lexer/Tokenizer (2 days) - `fsrs-frontend/src/lexer.rs`
- Infrastructure setup, documentation, test preparation

### Week 2: Complete Foundations

Continue Week 1 issues that span into Week 2:
- Finish #003 (Parser)
- Finish #006 (VM Interpreter)
- Complete #002 (Lexer)

### Week 3: Integration (SEQUENTIAL)

**Critical Path**:
- Issue #007: Bytecode Compiler (4-5 days) - `fsrs-frontend/src/compiler.rs`
  - Depends on #001, #003, #005
  - **BLOCKING** - Must complete first

**Parallel Integration**:
- Issue #008: Demo Host (2-3 days) - `fsrs-demo/src/main.rs`
  - Depends on #003, #006, #007
- Issue #009: Test Suite & CI (2-3 days) - `tests/`, `.github/workflows/`

## Orchestration Protocol

### 1. Issue Assignment Strategy

**Parallel Orchestrator Assignment**:
```
Orchestrator-Frontend:
  - Primary: #001, #003, #007
  - Secondary: #002
  - Crate: fsrs-frontend

Orchestrator-VM:
  - Primary: #004, #005, #006
  - Secondary: #008
  - Crate: fsrs-vm

Orchestrator-Integration:
  - Primary: #009
  - Secondary: #002, #008
  - Crates: fsrs-demo, tests, CI
```

### 2. Launch Sequence

**Step 1: Parallel Kickoff (Day 1)**
```bash
# Launch 3 orchestrators concurrently
Task("Frontend Orchestrator", "Implement #001 Core AST", "backend-architect")
Task("VM Orchestrator", "Implement #004 Value Representation", "backend-architect")
Task("Integration Orchestrator", "Setup infrastructure for #009", "devops-automator")
```

**Step 2: Sequential Progression (Days 2-5)**
- Monitor orchestrator progress
- Frontend: #001 → #003 (when #001 complete)
- VM: #004 → #005 → #006 (sequential chain)
- Integration: #002, docs, test prep

**Step 3: Critical Path (Week 3)**
- **All hands**: #007 (Compiler) - Highest priority
- Parallel: #008 (Demo), #009 (Tests)

### 3. Dependency Management

**Before launching any issue, verify dependencies**:

```python
dependencies = {
    "#001": [],
    "#002": [],
    "#003": ["#001"],
    "#004": [],
    "#005": [],
    "#006": ["#004", "#005"],
    "#007": ["#001", "#003", "#005"],
    "#008": ["#003", "#006", "#007"],
    "#009": [],  # Soft dependency on #008
}

def can_start_issue(issue_id):
    return all(is_complete(dep) for dep in dependencies[issue_id])
```

### 4. Git Workflow

**Branch Strategy**:
```
main
├─ feat/issue-001-core-ast
├─ feat/issue-002-lexer
├─ feat/issue-003-parser
├─ feat/issue-004-value
├─ feat/issue-005-bytecode
├─ feat/issue-006-vm
├─ feat/issue-007-compiler
├─ feat/issue-008-demo
└─ feat/issue-009-tests-ci
```

**Merge Protocol**:
1. Each issue = 1 PR
2. TDD: Write tests first, implement, refactor
3. Review required from 2+ team members
4. **Squash merge** to keep history clean
5. **Rebase** on main before merging
6. Delete branch after merge

### 5. TDD Enforcement

**Every issue must follow TDD**:
1. **Red**: Write failing test
2. **Green**: Implement to pass
3. **Refactor**: Clean up code
4. **Repeat**: For each feature

**PR Checklist** (agents must verify):
- [ ] All tests pass: `just test`
- [ ] No clippy warnings: `just lint`
- [ ] Code formatted: `just fmt`
- [ ] New tests for new functionality
- [ ] Documentation updated
- [ ] Rebased on main

### 6. Communication Protocol

**Daily Sync**:
- Each orchestrator reports:
  - Progress on assigned issue
  - Blockers/dependencies
  - ETA to completion
  - Files being modified

**Merge Notifications**:
- When issue completes, notify:
  - Which dependencies are now unblocked
  - Which orchestrators can proceed

**Conflict Resolution**:
- If multiple PRs touch same file, coordinate via comments
- Use feature flags to isolate changes
- Pair programming on conflicts

## Execution Commands

### Launch Meta-Orchestrator

```bash
# From fsrs project root
just bootstrap  # Ensure environment is ready

# Launch orchestrators in parallel (ONE MESSAGE)
# Meta-orchestrator spawns 3 sub-orchestrators concurrently
```

### Monitor Progress

```bash
# Check issue status
gh issue list --label "phase-1: mvp"

# Check PR status
gh pr list --label "phase-1: mvp"

# Run tests
just test

# Check CI
gh run list
```

### Quality Checkpoints

**After Week 1**:
- [ ] #001 (AST) merged
- [ ] #004 (Value) merged
- [ ] #005 (Bytecode) merged
- [ ] #003 (Parser) in progress or complete
- [ ] #006 (VM) started

**After Week 2**:
- [ ] #003 (Parser) merged
- [ ] #006 (VM) merged
- [ ] #002 (Lexer) merged
- [ ] Ready to start #007 (Compiler)

**After Week 3**:
- [ ] #007 (Compiler) merged
- [ ] #008 (Demo) merged
- [ ] #009 (Tests/CI) merged
- [ ] All 9 issues closed
- [ ] CI pipeline green
- [ ] Demo runs successfully

## Critical Success Factors

### 1. Dependency Discipline
- **Never start an issue before dependencies complete**
- Check `DEPENDENCIES.md` before launching
- Update orchestrators when dependencies unblock

### 2. Parallel Execution
- Week 1: 3 issues in parallel (#001, #004, #005)
- Week 2: 2-3 issues in parallel (#003, #006, #002)
- Week 3: 2-3 issues in parallel (#007, #008, #009)

### 3. TDD Non-Negotiable
- Every PR must include tests
- Test coverage > 70%
- No merging without passing tests

### 4. Git Hygiene
- Rebase daily on main
- Small, focused commits
- Squash merge PRs
- Clear commit messages

### 5. Communication
- Daily progress updates
- Immediate blocker escalation
- Coordinate on shared files

## Monitoring Dashboard

Track these metrics:

```
Phase 1 Progress:
├─ Issues Completed: 0/9 (0%)
├─ PRs Merged: 0/9 (0%)
├─ Tests Passing: 0/50+ (0%)
├─ Coverage: 0%
├─ Blockers: 0
└─ ETA: Week 3 Day 5 (on track)

Workstream Status:
├─ Frontend: #001 (in progress)
├─ VM: #004 (in progress)
└─ Integration: Setup (in progress)

Critical Path:
#001 (Day 2/2) → #003 (Day 0/4) → #007 (Day 0/5) → #008 (Day 0/3)
```

## Success Metrics

**Phase 1 Complete When**:
- All 9 issues closed
- All PRs merged to main
- CI pipeline green
- Demo runs `examples/*.fsrs` successfully
- Test coverage > 70%
- No open blockers

**Deliverables**:
- Working F# parser (subset)
- Functioning bytecode VM
- End-to-end demo application
- 50+ unit tests
- CI/CD pipeline
- Documentation updated

## Launch Command

**Execute this to begin Phase 1 MVP**:

```
Meta-Orchestrator: Initiate Phase 1 MVP implementation.

Spawn 3 parallel orchestrators:
1. Frontend Orchestrator: Start #001 (Core AST)
2. VM Orchestrator: Start #004 (Value Representation)
3. Integration Orchestrator: Setup infrastructure

Follow TDD, maintain git hygiene, coordinate on dependencies.
Target: 3 weeks to complete all 9 issues.

Status updates expected: Daily
PR reviews: Within 12 hours
Merge on green: Squash and rebase

Begin implementation. Report progress in 24 hours.
```

---

**Remember**: This is a marathon, not a sprint. Quality over speed. TDD always. Clean git history. Clear communication. We've got 3 weeks to build something amazing.

**Let's ship Phase 1! 🚀**