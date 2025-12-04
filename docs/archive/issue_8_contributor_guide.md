# Issue 8: [Docs] Write Contributor Guide & ABI Spec

**Labels:** `documentation`

## Implementation Plan
1.  Create `CONTRIBUTING.md`:
    * Explain the "3-Layer Architecture" (Source -> AST -> Bytecode).
    * Explain how to add a new Instruction (Update enum -> Update Compiler -> Update VM).
2.  Create `docs/ABI.md`:
    * Document the internal representation of `Value`.
    * Document the `.fzb` file format spec.
3.  Create `docs/SECURITY.md`:
    * Document current lack of sandboxing.
    * Propose future resource limit APIs.
