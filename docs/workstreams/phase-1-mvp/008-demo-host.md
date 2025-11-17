# Issue #008: Demo Host Integration

## Overview
Create the demo host application that ties together lexer, parser, compiler, and VM into an end-to-end executable.

## Labels
- `feature`, `phase-1: mvp`, `priority: high`, `component: demo`, `effort: s` (2-3 days)

## Milestone
Phase 1.3: Integration (Week 3)

## Dependencies
- #003 (Parser) - MUST BE COMPLETE
- #006 (VM) - MUST BE COMPLETE
- #007 (Compiler) - MUST BE COMPLETE

## Acceptance Criteria
- [ ] Load .fsrs files from examples/
- [ ] Full pipeline: parse → compile → execute
- [ ] Display results
- [ ] Error reporting
- [ ] 3+ example scripts working

## Technical Specification

```rust
// rust/crates/fsrs-demo/src/main.rs

use fsrs_frontend::{lexer::Lexer, parser::Parser, compiler::Compiler};
use fsrs_vm::vm::Vm;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let script_path = if args.len() > 1 {
        &args[1]
    } else {
        "../examples/arithmetic.fsrs"
    };

    match run_script(script_path) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}

fn run_script(path: &str) -> Result<fsrs_vm::value::Value, Box<dyn std::error::Error>> {
    // 1. Read source
    let source = fs::read_to_string(path)?;

    // 2. Lex
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    // 3. Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    // 4. Compile
    let chunk = Compiler::compile(&ast)?;

    // Optional: Disassemble for debugging
    chunk.disassemble(path);

    // 5. Execute
    let mut vm = Vm::new();
    let result = vm.run(&chunk)?;

    Ok(result)
}
```

## Example Scripts

```fsharp
// examples/arithmetic.fsrs
1 + 2 * 3

// examples/conditional.fsrs
if true then 42 else 0

// examples/let_binding.fsrs
let x = 10 in x + 5
```

## Estimated Effort
**2-3 days**

## Related Issues
- Depends on #003, #006, #007