# Fusabi Frontend

The compiler frontend for the **Fusabi** scripting engine. Handles lexing, parsing, and bytecode generation.

## Features

- **F# Dialect**: Supports a subset of F# including let-bindings, pattern matching, records, and DUs.
- **Type Inference**: Hindley-Milner type inference engine.
- **Compiler**: Emits optimized bytecode for `fusabi-vm`.

## Usage

```rust
use fusabi_frontend::{Lexer, Parser, Compiler};

let source = "let x = 42";
let mut lexer = Lexer::new(source);
let tokens = lexer.tokenize()?;
let mut parser = Parser::new(tokens);
let ast = parser.parse()?;
let chunk = Compiler::compile(&ast)?;
```

## License

MIT
