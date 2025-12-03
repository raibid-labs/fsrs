# Documentation & DX Audit

**Date**: 2025-12-02
**Status**: Analysis Complete

## 1. Documentation State
*   **Reference**: `STDLIB_REFERENCE.md` is excellent and up-to-date (thanks to NuShell script).
*   **Guides**: We have `01-overview.md` and `02-language-spec.md`. These are "Specs", not "Guides".
*   **Tutorials**: We rely entirely on `examples/` folder. This is poor DX for beginners. A user shouldn't have to read source code to learn "Hello World".

**Verdict**: Functional but uninviting.

## 2. Docs Aggregation (The "Raibid Labs" Pull Pattern)
The user clarified the strategy: **Pull-based Aggregation**.
*   **Concept**: The central `fusabi-lang/docs` repo (or similar) will clone this repo and ingest the `docs/` folder.
*   **Requirement for Fusabi Repo**: We do *not* need push workflows. We simply need to ensure our `docs/` folder is pristine and comprehensive.
*   **Current State**: `docs/` is a bit messy (mix of design notes, old roadmaps, and reference).
*   **Action Item**: Organize `docs/` better?
    *   Keep `STDLIB_REFERENCE.md` at root of `docs/` or in `docs/reference/`.
    *   Ensure `STDLIB_REFERENCE.md` is always up to date.

## 3. Ensuring Docs Freshness
Since an external repo pulls our docs, if `STDLIB_REFERENCE.md` is stale (dev changed code but didn't run script), the website will be stale.
*   **Action**: Add a CI check that runs `nu scripts/gen-docs.nu` and fails if `git diff` shows changes. This forces developers to commit up-to-date documentation.

## 4. Website Strategy
This is now likely the responsibility of the central docs repo. Our job is just to provide the content.