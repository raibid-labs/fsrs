# Fusabi Phase 3 Completion - Execution Instructions

**Date:** November 20, 2025
**Plan:** [parallel-orchestration-plan.md](./parallel-orchestration-plan.md)
**Status:** Ready to Execute

---

## Quick Start - Immediate Actions

### 1. Close Issue #30 (Already Done ✅)
```bash
# Issue #30 closed via gh CLI
```

### 2. Review the Parallel Orchestration Plan
Read the complete plan: [docs/parallel-orchestration-plan.md](./parallel-orchestration-plan.md)

Key highlights:
- **10 workstreams** organized by priority
- **4-week timeline** with parallel execution
- **Maximum 4 concurrent agents** for optimal throughput
- **Proper git hygiene**: Feature branches → PRs → Squash merge

---

## Execution Option 1: Claude Code Orchestration (Recommended)

### Launch All Critical Path Workstreams in Parallel

```prompt
Execute the Fusabi Phase 3 completion plan in parallel:

1. Read /home/beengud/fusabi-lang/fusabi/docs/parallel-orchestration-plan.md completely
2. Execute these workstreams IN PARALLEL using the Task tool in a SINGLE message:

   - Workstream 1: Fix failing PRs (#72, #73)
   - Workstream 2: Re-entrant Host Functions (Issue #60)
   - Workstream 3: Records & DUs Execution
   - Workstream 5: Mark-and-Sweep GC (Issue #61)

3. Each agent must:
   - Create their feature branch
   - Implement the complete feature
   - Write comprehensive tests
   - Run all quality checks (fmt, clippy, test)
   - Create a PR with proper description
   - Address any CI failures
   - Request squash merge approval

4. Follow the Git Hygiene Protocol exactly as specified in the orchestration plan.

5. Report progress for each workstream with:
   - Branch created: Yes/No
   - Implementation complete: Yes/No
   - Tests passing: X/Y
   - PR created: #Number
   - Ready to merge: Yes/No

Execute all 4 workstreams concurrently. Use the Task tool with 4 parallel invocations in a single message.
```

---

## Execution Option 2: Sequential Agent Orchestration

### Week 1: Critical Path

#### Day 1-2: Fix Existing PRs
```prompt
Execute Workstream 1 from the orchestration plan:

1. Checkout PR #72 branch (feat/ws1-hof-support)
2. Run cargo fmt --all
3. Run cargo clippy --all -- -D warnings
4. Fix all issues
5. Run cargo test --workspace
6. Push fixes
7. Repeat for PR #73 (feat/ws5-mcp-server)
8. Report status of both PRs
```

#### Day 3-5: Parallel Critical Features
```prompt
Execute Workstreams 2, 3, and 5 in parallel:

Launch 3 agents concurrently using the Task tool:
1. Agent 1: Workstream 2 (Re-entrant Host Functions - Issue #60)
2. Agent 2: Workstream 3 (Records & DUs Execution)
3. Agent 3: Workstream 5 (Mark-and-Sweep GC - Issue #61)

Each agent creates their branch, implements feature, writes tests, creates PR.
Report when all 3 PRs are ready for review.
```

### Week 2: High Priority

#### Day 8-10: Bytecode & Prelude
```prompt
After WS3 merges, execute Workstreams 4 and 6 in parallel:

1. Agent 1: Workstream 4 (Bytecode Serialization - Issue #65)
2. Agent 2: Workstream 6 (Implicit Prelude - Issue #63)

Both agents create branches, implement, test, create PRs.
```

#### Day 11-14: Complete WS5 & Start Benchmarks
```prompt
1. Ensure WS5 (GC) is complete and merged
2. Execute Workstream 7 (Benchmarking Suite - Issue #62)

Agent creates branch, implements comprehensive benchmarks, creates PR.
```

### Week 3: Medium Priority

```prompt
Execute Workstream 8 (Example Suite - Issue #66):

Create comprehensive example suite with:
- Bevy scripting example
- Ratatui layout example
- Burn config example
- Axum web server example
- Computation expressions example

Agent creates branch, implements all examples, creates PR.
```

### Week 4: Documentation & Polish

```prompt
Complete final workstreams in parallel:

1. Verify PR #72 merged (contains WS9 and WS10)
2. If anything missing, create follow-up PRs
3. Finalize all documentation
4. Update README.md to reflect Phase 3 complete
5. Create v0.3.0-alpha release
```

---

## Execution Option 3: Manual Execution (For Each Workstream)

### Template for Each Workstream

1. **Read the workstream spec** in orchestration-plan.md
2. **Create feature branch**:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/issue-XX-description
   ```
3. **Implement feature** following the task list
4. **Run quality checks**:
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   cargo test --workspace
   ```
5. **Commit and push**:
   ```bash
   git add .
   git commit -m "feat(scope): Implement feature X"
   git push origin feat/issue-XX-description
   ```
6. **Create PR**:
   ```bash
   gh pr create --title "feat: Implement X (#XX)" --body "[description]"
   ```
7. **Monitor CI and fix failures**
8. **Squash merge after approval**:
   ```bash
   gh pr merge --squash --delete-branch
   ```

---

## Monitoring Progress

### Check Current Status

```bash
# List all open PRs
gh pr list

# Check issue status
gh issue list

# View project status
just test          # Run all tests
cargo build --workspace --release   # Build release

# Check git branches
git branch -a
```

### Weekly Progress Report

```bash
# Generate stats
echo "Issues Closed: $(gh issue list --state closed --search 'closed:>=$(date -d '7 days ago' +%Y-%m-%d)' --json number --jq 'length')"
echo "PRs Merged: $(gh pr list --state merged --search 'merged:>=$(date -d '7 days ago' +%Y-%m-%d)' --json number --jq 'length')"
echo "Tests Passing: $(cargo test --workspace 2>&1 | grep 'test result:' | tail -1)"
```

---

## Success Criteria Checklist

### Week 1 (Critical Path)
- [ ] PR #72 CI failures fixed and merged
- [ ] PR #73 CI failures fixed and merged
- [ ] Issue #60 (Re-entrant Host Functions) completed with PR merged
- [ ] Records & DUs execute correctly in VM with PR merged
- [ ] Mark-and-Sweep GC implemented (PR created, in review)

### Week 2 (High Priority)
- [ ] GC PR merged
- [ ] Issue #65 (Bytecode Serialization) completed
- [ ] `fus grind` command works, creates .fzb files
- [ ] Issue #63 (Implicit Prelude) completed
- [ ] Pipeline operator `|>` works
- [ ] Benchmarking suite implemented

### Week 3 (Medium Priority)
- [ ] Benchmarks show Lua-comparable performance
- [ ] 5 comprehensive ecosystem examples created
- [ ] All examples documented

### Week 4 (Documentation)
- [ ] CONTRIBUTING.md complete
- [ ] docs/abi.md complete
- [ ] docs/security.md complete
- [ ] README.md updated to "Phase 3 Complete"
- [ ] v0.3.0-alpha release created

### Final Verification
- [ ] All 12 issues (#59-71 minus duplicates) closed
- [ ] All tests passing (target: 1,500+ tests)
- [ ] Zero clippy warnings
- [ ] All documentation updated
- [ ] CHANGELOG.md has v0.3.0 entry

---

## Troubleshooting

### If PR CI Fails

1. Checkout the PR branch locally
2. Run checks:
   ```bash
   cargo fmt --all
   cargo clippy --all -- -D warnings
   cargo test --workspace
   cargo doc --workspace --no-deps
   ```
3. Fix issues and push again
4. CI will automatically rerun

### If Tests Fail

1. Read the test output carefully
2. Fix the root cause (not just the symptom)
3. Add regression tests
4. Rerun full suite before pushing

### If Merge Conflicts Occur

1. Rebase on main:
   ```bash
   git checkout feat/my-branch
   git fetch origin
   git rebase origin/main
   ```
2. Resolve conflicts
3. Run full test suite
4. Force push: `git push --force-with-lease`

### If Agent Gets Stuck

1. Check the specific workstream instructions
2. Verify dependencies are met (some workstreams depend on others)
3. Check for any blocking issues
4. Consult the orchestration plan for guidance

---

## Post-Completion Tasks

After all workstreams complete:

1. **Version Bump**:
   ```bash
   # Update version in all Cargo.toml files
   # fusabi/Cargo.toml: version = "0.3.0"
   # fusabi-frontend/Cargo.toml: version = "0.3.0"
   # fusabi-vm/Cargo.toml: version = "0.3.0"
   ```

2. **Update README.md**:
   - Change "Phase 3 - In Progress" → "Phase 3 - Complete"
   - Update test count
   - Add new features to feature list

3. **Update CHANGELOG.md**:
   ```markdown
   ## [0.3.0-alpha] - 2025-11-XX

   ### Added
   - Re-entrant host functions for higher-order operations
   - Mark-and-sweep garbage collection
   - Bytecode serialization (.fzb files)
   - Implicit prelude and pipeline operator
   - Comprehensive benchmarking suite
   - Records and discriminated unions execution
   - Full branding and visual identity

   ### Fixed
   - Memory leaks in recursive structures
   - CI pipeline failures in PRs #72 and #73
   ```

4. **Create Git Tag**:
   ```bash
   git tag -a v0.3.0-alpha -m "Phase 3 Complete: Advanced Features"
   git push origin v0.3.0-alpha
   ```

5. **Create GitHub Release**:
   ```bash
   gh release create v0.3.0-alpha \
     --title "v0.3.0-alpha - Phase 3 Complete" \
     --notes "Phase 3 of Fusabi is complete! This release includes..."
   ```

6. **Update Project Roadmap**:
   - Mark all Phase 3 milestones as ✅ Complete
   - Detail Phase 4 plan
   - Set new timeline for Phase 4

---

## Questions?

- **Documentation**: See [docs/parallel-orchestration-plan.md](./parallel-orchestration-plan.md)
- **Issues**: Check [GitHub Issues](https://github.com/fusabi-lang/fusabi/issues)
- **PRs**: Check [GitHub Pull Requests](https://github.com/fusabi-lang/fusabi/pulls)

---

**Ready to execute!** 🚀

Recommended: Start with Execution Option 1 (parallel orchestration) for maximum efficiency.
