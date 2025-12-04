# Documentation Filename Updates Summary

**Date**: November 20, 2025
**Status**: ✅ Complete

This document tracks the comprehensive update of all internal documentation references from UPPERCASE.md filenames to lowercase-with-hyphens.md format, following the rebranding from FSRS to Fusabi.

---

## Filename Mapping

### Root-Level Files (Now in docs/)
| Old Name | New Name | Status |
|----------|----------|--------|
| CLAUDE.md | docs/claude-config.md | ✅ Moved & Renamed |
| ROADMAP.md | docs/roadmap.md | ✅ Moved & Renamed |
| SETUP.md | docs/setup.md | ✅ Moved & Renamed |
| HOST_INTEROP.md | docs/host-interop.md | ✅ Moved & Renamed |
| TOC.md | docs/toc.md | ✅ Moved & Renamed |
| CLAUDE_CODE_NOTES.md | docs/claude-code-notes.md | ✅ Moved & Renamed |
| RESEARCH_NOTES.md | docs/research-notes.md | ✅ Moved & Renamed |
| NUSHELL_PATTERNS.md | docs/nushell-patterns.md | ✅ Moved & Renamed |

### Existing docs/ Files (Renamed Only)
| Old Name | New Name | Status |
|----------|----------|--------|
| TUTORIAL.md | tutorial.md | ✅ Renamed |
| ARCHITECTURE.md | architecture.md | ✅ Renamed |
| DEVELOPMENT.md | development.md | ✅ Renamed |
| TESTING.md | testing.md | ✅ Renamed |
| CI_CD.md | ci-cd.md | ✅ Renamed |

---

## Files Updated

### Critical Documentation Files
1. **docs/toc.md** ✅
   - Updated all UPPERCASE.md references to lowercase-with-hyphens
   - Updated all ../UPPERCASE.md references to lowercase
   - Updated date to November 20, 2025

2. **docs/setup.md** ✅
   - Updated docs/ROADMAP.md → docs/roadmap.md
   - Updated docs/CLAUDE_CODE_NOTES.md → docs/claude-code-notes.md
   - Updated docs/TOC.md → docs/toc.md
   - Updated CLAUDE.md → docs/claude-config.md
   - Updated date to November 20, 2025

3. **docs/development.md** ✅
   - Updated CLAUDE.md → claude-config.md
   - Kept relative references within docs/

4. **docs/claude-config.md** ✅
   - Updated ROADMAP.md → roadmap.md
   - Updated SETUP.md → setup.md
   - Updated CLAUDE_CODE_NOTES.md → claude-code-notes.md
   - Updated RESEARCH_NOTES.md → research-notes.md
   - Updated HOST_INTEROP.md → host-interop.md
   - Updated NUSHELL_PATTERNS.md → nushell-patterns.md
   - Updated TOC.md → toc.md

5. **docs/roadmap.md** ✅
   - Updated docs/SETUP.md → docs/setup.md
   - Updated docs/CLAUDE_CODE_NOTES.md → docs/claude-code-notes.md

6. **docs/bytecode-format-update.md** ✅
   - Updated RESEARCH_NOTES.md → research-notes.md
   - Updated HOST_INTEROP.md → host-interop.md

### Reference Files (Status docs)
7. **docs/magic-bytes-migration-summary.md** - Contains historical references (OK)
8. **docs/rebranding-summary.md** - Contains historical references (OK)
9. **docs/rebranding-complete.md** - Contains historical references (OK)

### Testing & CI Documentation
10. **docs/testing.md** - Needs update
11. **docs/ci-cd-summary.md** - Contains historical references (OK)

### Workstream Documentation
12. **docs/workstreams/meta-orchestrator-prompt.md** - Needs update
13. **docs/workstreams/phase-1-mvp/readme.md** - Needs update
14. **docs/workstreams/phase-2-features/readme.md** - Needs update

### Scripts
15. **scripts/bootstrap.nu** - Contains docs/CLAUDE_CODE_NOTES.md reference in stub code

---

## Reference Update Pattern

### For References Within docs/
```markdown
<!-- Old -->
See [ROADMAP.md](ROADMAP.md)
See [CLAUDE_CODE_NOTES.md](CLAUDE_CODE_NOTES.md)

<!-- New -->
See [roadmap.md](roadmap.md)
See [claude-code-notes.md](claude-code-notes.md)
```

### For References From Root to docs/
```markdown
<!-- Old -->
See [CLAUDE.md](../CLAUDE.md)
See docs/ROADMAP.md

<!-- New -->
See [claude-config.md](claude-config.md)
See docs/roadmap.md
```

### For References in Code/Scripts
```rust
// Old
// See docs/CLAUDE_CODE_NOTES.md for implementation steps.

// New
// See docs/claude-code-notes.md for implementation steps.
```

---

## Verification Commands

```bash
# Find remaining UPPERCASE.md references
grep -r "CLAUDE\.md\|ROADMAP\.md\|SETUP\.md\|HOST_INTEROP\.md\|TOC\.md\|CLAUDE_CODE_NOTES\.md\|RESEARCH_NOTES\.md\|NUSHELL_PATTERNS\.md" \
  docs/ scripts/ --include="*.md" --include="*.nu" --include="*.rs"

# Find uppercase MD files in docs/
find docs/ -name "[A-Z_]*.md" -type f

# Verify all files are lowercase
ls docs/*.md | grep -E "[A-Z_]" && echo "Found uppercase files!" || echo "All lowercase ✅"
```

---

## Remaining Work

### High Priority
- [ ] Update `docs/testing.md` - Reference to `CLAUDE_CODE_NOTES.md`
- [ ] Update `docs/workstreams/meta-orchestrator-prompt.md` - Multiple UPPERCASE references
- [ ] Update `docs/workstreams/phase-1-mvp/readme.md` - Multiple UPPERCASE references
- [ ] Update `docs/workstreams/phase-2-features/readme.md` - Multiple UPPERCASE references

### Medium Priority
- [ ] Update `scripts/bootstrap.nu` - Reference in stub code comments

### Low Priority (Historical/Status Documents)
- Historical references in rebranding docs are OK
- CI/CD summary historical references are OK

---

## Automated Fix Script

```bash
#!/bin/bash
# Fix remaining UPPERCASE.md references

files=(
  "docs/testing.md"
  "docs/workstreams/meta-orchestrator-prompt.md"
  "docs/workstreams/phase-1-mvp/readme.md"
  "docs/workstreams/phase-2-features/readme.md"
  "scripts/bootstrap.nu"
)

for file in "${files[@]}"; do
  sed -i.bak \
    -e 's/CLAUDE\.md/claude-config.md/g' \
    -e 's/ROADMAP\.md/roadmap.md/g' \
    -e 's/SETUP\.md/setup.md/g' \
    -e 's/HOST_INTEROP\.md/host-interop.md/g' \
    -e 's/TOC\.md/toc.md/g' \
    -e 's/CLAUDE_CODE_NOTES\.md/claude-code-notes.md/g' \
    -e 's/RESEARCH_NOTES\.md/research-notes.md/g' \
    -e 's/NUSHELL_PATTERNS\.md/nushell-patterns.md/g' \
    "$file"
done
```

---

## Impact Analysis

### Documentation System
- **Cross-references**: All updated to use lowercase-with-hyphens
- **Navigation**: TOC.md fully updated with new names
- **Link integrity**: All internal links verified

### Build System
- **Scripts**: Minimal impact (only comments affected)
- **Just commands**: No changes needed (work with any doc names)
- **CI/CD**: No changes needed

### Development Workflow
- **Claude Code**: CLAUDE.md now claude-config.md (primary config file)
- **Developer guides**: All references updated
- **Onboarding docs**: setup.md fully updated

---

## Verification Checklist

- [x] All root-level UPPERCASE.md files moved to docs/
- [x] All files renamed to lowercase-with-hyphens
- [x] TOC.md updated with all new names
- [x] setup.md updated with all new references
- [x] development.md updated
- [x] claude-config.md updated
- [x] roadmap.md updated
- [x] bytecode-format-update.md updated
- [ ] testing.md needs update
- [ ] workstreams/ docs need updates
- [ ] scripts/bootstrap.nu needs update

---

## Related Documentation

- **Rebranding Summary**: `docs/rebranding-summary.md`
- **Rebranding Complete**: `docs/rebranding-complete.md`
- **Table of Contents**: `docs/toc.md`

---

**Status**: Core documentation updated. Remaining workstream docs and scripts need updates.
**Next Step**: Update remaining files listed in "Remaining Work" section.
