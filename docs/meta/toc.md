# Fusabi Documentation Index

**Last Updated**: November 20, 2025

This document provides a comprehensive index of all Fusabi documentation.

---

## 📚 Getting Started

### For New Users
1. **[README.md](../README.md)** - Project overview and quick start
2. **[setup.md](setup.md)** - Environment setup and prerequisites
3. **[roadmap.md](roadmap.md)** - Development phases and timeline
4. **[01-overview.md](01-overview.md)** - High-level architecture

### For Developers
1. **[claude-config.md](claude-config.md)** - Claude Code configuration and workflows
2. **[claude-code-notes.md](claude-code-notes.md)** - Detailed implementation tasks
3. **[setup.md](setup.md)** - Development environment setup

---

## 🏗️ Architecture & Design

### Core Documentation
- **[01-overview.md](01-overview.md)** - System architecture, components, phased strategy
- **[02-language-spec.md](02-language-spec.md)** - Mini-F# language specification
- **[03-vm-design.md](03-vm-design.md)** - Bytecode VM architecture and instructions
- **[host-interop.md](host-interop.md)** - Host API design and embedding patterns

### Research & Decisions
- **[research-notes.md](research-notes.md)** - Bytecode VM and embedding research
- **[nushell-patterns.md](nushell-patterns.md)** - Scripting patterns and conventions

---

## 🛠️ Implementation Guide

### Task Breakdown
- **[claude-code-notes.md](claude-code-notes.md)** - Phase-by-phase implementation tasks
  - Phase 1: Core AST, tokenizer, parser, VM
  - Phase 2: Extended language features
  - Phase 3: Records, DUs, and embedding

### Development Workflow
- **[roadmap.md](roadmap.md)** - Milestone-based development plan
  - Phase 1: MVP (Weeks 1-3)
  - Phase 2: Language Features (Weeks 4-7)
  - Phase 3: Advanced Features (Weeks 8-11)
  - Phase 4: Production Ready (Weeks 12-16)

---

## 📖 Language Reference

### Language Specification
- **[02-language-spec.md](02-language-spec.md)** - Complete Mini-F# specification
  - Lexical elements
  - Types and type system
  - Expressions and patterns
  - Modules and computation expressions
  - Omitted features

### Examples
- **[../examples/](../examples/)** - Example `.fsx` scripts
  - `fusabi_config.fsx` - Configuration example
  - (More to be added in Phase 1-3)

---

## 🔧 Technical Reference

### VM & Runtime
- **[03-vm-design.md](03-vm-design.md)** - VM internals
  - Value representation
  - Bytecode instructions
  - Call frames and execution
  - GC strategy
  - Performance notes

### Host Integration
- **[host-interop.md](host-interop.md)** - Embedding API
  - Engine configuration
  - Value marshalling
  - Function registration
  - Hot-reload architecture
  - Memory management
  - Use case examples

### Research Background
- **[research-notes.md](research-notes.md)** - Implementation research
  - F# bytecode VM implementations
  - OCaml ZINC machine
  - Erlang BEAM VM
  - Lua embedding patterns
  - Rust embedding examples (Rhai, Gluon, Dyon)
  - Bytecode VM best practices

---

## 🛠️ Build & Tooling

### Build System
- **[../justfile](../justfile)** - Build automation commands
  - Building: `just build`, `just build-release`
  - Testing: `just test`, `just test-coverage`
  - Development: `just dev`, `just watch`
  - Quality: `just check`, `just lint`, `just fmt`

### Automation Scripts
- **[../scripts/build.nu](../scripts/build.nu)** - Build orchestration
- **[../scripts/test.nu](../scripts/test.nu)** - Test runner with filtering
- **[../scripts/bootstrap.nu](../scripts/bootstrap.nu)** - Environment setup

### Scripting Patterns
- **[nushell-patterns.md](nushell-patterns.md)** - Nushell best practices
  - Script structure
  - Error handling
  - Configuration management
  - Integration with Just

---

## 📋 Project Management

### Roadmap & Planning
- **[roadmap.md](roadmap.md)** - Complete project roadmap
  - Executive summary
  - Current state
  - Phase 1-4 detailed breakdown
  - Success metrics
  - Risk assessment
  - Technical decisions

### Configuration
- **[claude-config.md](claude-config.md)** - Claude Code project configuration
  - Concurrent execution patterns
  - File organization rules
  - Development guidelines
  - Agent execution patterns

---

## 🧪 Testing & Quality

### Test Strategy
- **[roadmap.md](roadmap.md)** - Testing approach by phase
- **[claude-code-notes.md](claude-code-notes.md)** - Unit test requirements

### Coverage
- Run `just test-coverage` to generate coverage reports
- Reports saved to `docs/coverage/`

---

## 📝 Contributing

### Development Setup
1. Read **[setup.md](setup.md)** for prerequisites
2. Run `just bootstrap` to initialize environment
3. Review **[claude-config.md](claude-config.md)** for workflows
4. Check **[roadmap.md](roadmap.md)** for current phase

### Implementation Guidelines
- **[claude-code-notes.md](claude-code-notes.md)** - Task-by-task breakdown
- **[claude-config.md](claude-config.md)** - Code style and patterns
- **[02-language-spec.md](02-language-spec.md)** - Language requirements

---

## 🔗 Quick Links

### Most Important Documents
| Document | Purpose | When to Read |
|----------|---------|--------------|
| [README.md](../README.md) | Project overview | First thing |
| [setup.md](setup.md) | Environment setup | Before coding |
| [roadmap.md](roadmap.md) | Development plan | Before starting a phase |
| [claude-code-notes.md](claude-code-notes.md) | Implementation tasks | While implementing |
| [02-language-spec.md](02-language-spec.md) | Language reference | When adding features |
| [03-vm-design.md](03-vm-design.md) | VM architecture | When working on VM |

### By Role

**If you're a new developer**:
1. README.md → setup.md → 01-overview.md → roadmap.md

**If you're implementing Phase 1**:
1. claude-code-notes.md → 02-language-spec.md → 03-vm-design.md

**If you're adding language features**:
1. 02-language-spec.md → roadmap.md (your phase) → claude-code-notes.md

**If you're working on embedding**:
1. host-interop.md → research-notes.md → 03-vm-design.md

**If you're optimizing**:
1. research-notes.md → 03-vm-design.md → roadmap.md (Phase 4)

### By Topic

**Language Design**: 02-language-spec.md, 01-overview.md
**VM Architecture**: 03-vm-design.md, research-notes.md
**Host Embedding**: host-interop.md, research-notes.md
**Build System**: justfile, scripts/*.nu, nushell-patterns.md
**Project Planning**: roadmap.md, claude-code-notes.md
**Development Workflow**: claude-config.md, setup.md

---

## 📊 Documentation Statistics

- **Total Documents**: 11 core documents
- **Lines of Documentation**: ~6,000+ lines
- **Coverage**:
  - ✅ Architecture & Design: Complete
  - ✅ Implementation Guide: Complete
  - ✅ Build System: Complete
  - ✅ Research Background: Complete
  - ⚠️  API Reference: Pending (Phase 3)
  - ⚠️  Tutorial: Pending (Phase 4)
  - ⚠️  Language Reference: Pending (Phase 4)

---

## 🚀 Next Steps

### Phase 1 (Current)
- Implement components per claude-code-notes.md
- Follow roadmap.md milestones
- Reference 02-language-spec.md for language features
- Reference 03-vm-design.md for VM implementation

### Future Documentation Needs
- **API Reference**: Rustdoc-generated (Phase 3)
- **Tutorial Series**: User-facing tutorials (Phase 4)
- **Language Reference**: Complete language manual (Phase 4)
- **Performance Guide**: Optimization techniques (Phase 4)
- **Embedding Cookbook**: Common patterns (Phase 4)

---

**For questions or clarifications**, see:
- GitHub Issues for bugs/features
- GitHub Discussions for general questions
- claude-config.md for workflow guidance
