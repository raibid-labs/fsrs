# Workstream 6: Examples & Documentation

## Status
🟡 Can Start Structure, Full Functionality Needs WS1+WS3

## Overview
Create comprehensive examples showcasing Fusabi's dual-runtime capability (runs on both Fusabi VM and .NET CLR) and embedded utility. Also create contributor documentation including architecture guide, ABI spec, and security documentation.

## Objectives
**Examples**:
- [ ] `examples/bevy_scripting/` - Entity behavior scripting
- [ ] `examples/ratatui_layout/` - UI layout from F# script
- [ ] `examples/burn_config/` - Neural net config in F#
- [ ] `examples/web_server/` - Axum server with F# validation
- [ ] `examples/computations/` - Computation expressions
- [ ] `examples/interop_net/` - Syntax compatibility demo

**Documentation**:
- [ ] `CONTRIBUTING.md` - 3-layer architecture guide
- [ ] `docs/ABI.md` - Internal Value representation and `.fzb` spec
- [ ] `docs/SECURITY.md` - Sandboxing status and future plans

## Agent Assignment
**Suggested Agent Type**: `docs-architect`, `tutorial-engineer`, `coder`, `rust-pro`
**Skill Requirements**: Technical writing, Rust, Bevy, Ratatui, Axum, F# syntax

## Dependencies
- **WS1 (HOF Support)**: Required for full example functionality (List.map in examples)
- **WS3 (Stdlib Prelude)**: Required for examples to use pipeline operator
- **Can start structure immediately**: Docs and example scaffolding don't need WS1/WS3

## Tasks

### Part A: Example Applications

#### Task 6A.1: Bevy Scripting Example
**Description**: Demonstrate using Fusabi to script game entity behavior in a Bevy game.

**Deliverables**:
- Rust binary using Bevy game engine
- Spawn entity with transform component
- Read `behavior.fsx` to determine movement logic
- Execute movement script every frame
- README explaining the example

**Files to Create/Modify**:
- `examples/bevy_scripting/Cargo.toml`
- `examples/bevy_scripting/src/main.rs`
- `examples/bevy_scripting/behavior.fsx`
- `examples/bevy_scripting/README.md`

**Implementation**:
```rust
// examples/bevy_scripting/src/main.rs

use bevy::prelude::*;
use fusabi::*;

#[derive(Component)]
struct ScriptedMovement {
    script: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_scripted_entities)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn scripted entity
    let script = std::fs::read_to_string("behavior.fsx")
        .expect("Failed to read behavior.fsx");

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        ScriptedMovement { script },
    ));
}

fn update_scripted_entities(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ScriptedMovement)>,
) {
    for (mut transform, scripted) in &mut query {
        // Execute Fusabi script to determine movement
        let mut vm = Vm::new();

        // Inject time and current position
        vm.set_global("time", Value::Float(time.elapsed_seconds_f64()));
        vm.set_global("x", Value::Float(transform.translation.x as f64));
        vm.set_global("y", Value::Float(transform.translation.y as f64));

        // Run script
        let result = vm.eval(&scripted.script).expect("Script error");

        // Extract new position
        if let Value::Tuple(pos) = result {
            transform.translation.x = pos[0].as_float().unwrap() as f32;
            transform.translation.y = pos[1].as_float().unwrap() as f32;
        }
    }
}
```

```fsharp
// examples/bevy_scripting/behavior.fsx

// Circular movement
let radius = 100.0
let speed = 2.0

let newX = radius * cos (time * speed)
let newY = radius * sin (time * speed)

(newX, newY)
```

**Validation**:
```bash
cd examples/bevy_scripting
cargo run
# Should show a sprite moving in a circle
```

---

#### Task 6A.2: Ratatui Layout Example
**Description**: Use Fusabi script to define terminal UI layout in Ratatui.

**Deliverables**:
- Rust binary using Ratatui TUI framework
- Read `layout.fsx` for UI structure
- Script returns Record describing grid layout
- Rust renders the layout

**Files to Create/Modify**:
- `examples/ratatui_layout/Cargo.toml`
- `examples/ratatui_layout/src/main.rs`
- `examples/ratatui_layout/layout.fsx`
- `examples/ratatui_layout/README.md`

**Implementation**:
```rust
// examples/ratatui_layout/src/main.rs

use ratatui::prelude::*;
use fusabi::*;

fn main() -> std::io::Result<()> {
    // Read layout script
    let script = std::fs::read_to_string("layout.fsx")?;

    // Execute script to get layout definition
    let mut vm = Vm::new();
    let layout_value = vm.eval(&script).expect("Script error");

    // Convert to Ratatui layout
    let layout = value_to_layout(layout_value);

    // Render TUI
    // ... (ratatui setup code)

    Ok(())
}

fn value_to_layout(value: Value) -> Layout {
    // Extract layout configuration from Fusabi value
    // ...
}
```

```fsharp
// examples/ratatui_layout/layout.fsx

// Define a 3-column layout
{
    direction = "horizontal"
    constraints = [
        { type = "percentage"; value = 20 }
        { type = "percentage"; value = 60 }
        { type = "percentage"; value = 20 }
    ]
}
```

**Validation**:
```bash
cd examples/ratatui_layout
cargo run
# Should display 3-column terminal layout
```

---

#### Task 6A.3: Burn Neural Net Config Example
**Description**: Define neural network architecture in F# and initialize a Burn model.

**Deliverables**:
- Rust binary using Burn deep learning framework
- Read `model_config.fsx` for layer definitions
- Script returns Record describing layers, dropout, etc.
- Rust initializes Burn model from config

**Files to Create/Modify**:
- `examples/burn_config/Cargo.toml`
- `examples/burn_config/src/main.rs`
- `examples/burn_config/model_config.fsx`
- `examples/burn_config/README.md`

**Implementation**:
```rust
// examples/burn_config/src/main.rs

use burn::prelude::*;
use fusabi::*;

fn main() {
    // Read model config
    let script = std::fs::read_to_string("model_config.fsx")
        .expect("Failed to read config");

    // Execute script
    let mut vm = Vm::new();
    let config_value = vm.eval(&script).expect("Script error");

    // Build Burn model from config
    let model = build_model_from_config(config_value);

    println!("Model initialized: {:?}", model);
}

fn build_model_from_config(value: Value) -> impl burn::module::Module {
    // Extract layer definitions and build Burn model
    // ...
}
```

```fsharp
// examples/burn_config/model_config.fsx

// Neural network configuration
{
    layers = [
        { type = "linear"; inputSize = 784; outputSize = 128 }
        { type = "relu" }
        { type = "dropout"; rate = 0.2 }
        { type = "linear"; inputSize = 128; outputSize = 10 }
    ]
    optimizer = "adam"
    learningRate = 0.001
}
```

**Validation**:
```bash
cd examples/burn_config
cargo run
# Should print model architecture
```

---

#### Task 6A.4: Web Server Validation Example
**Description**: Axum web server where validation logic is loaded from Fusabi script.

**Deliverables**:
- Rust binary using Axum web framework
- Endpoint `POST /users` with validation
- Read `validation.fsx` for validation rules
- Script validates user data and returns errors

**Files to Create/Modify**:
- `examples/web_server/Cargo.toml`
- `examples/web_server/src/main.rs`
- `examples/web_server/validation.fsx`
- `examples/web_server/README.md`

**Implementation**:
```rust
// examples/web_server/src/main.rs

use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use fusabi::*;

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
    age: i32,
}

#[derive(Serialize)]
struct ValidationError {
    field: String,
    message: String,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<String>, Json<Vec<ValidationError>>> {
    // Load validation script
    let script = std::fs::read_to_string("validation.fsx")
        .expect("Failed to read validation script");

    // Execute validation
    let mut vm = Vm::new();
    vm.set_global("username", Value::String(payload.username.clone()));
    vm.set_global("email", Value::String(payload.email.clone()));
    vm.set_global("age", Value::Int(payload.age as i64));

    let result = vm.eval(&script).expect("Script error");

    // Check validation result
    match result {
        Value::List(errors) if !errors.borrow().is_empty() => {
            // Has validation errors
            let errors: Vec<ValidationError> = /* convert from Fusabi values */;
            Err(Json(errors))
        }
        _ => {
            // Validation passed
            Ok(Json("User created".into()))
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/users", post(create_user));

    println!("Server running on http://localhost:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

```fsharp
// examples/web_server/validation.fsx

let errors = []

let errors =
    if String.length username < 3 then
        { field = "username"; message = "Username must be at least 3 characters" } :: errors
    else errors

let errors =
    if not (String.contains "@" email) then
        { field = "email"; message = "Invalid email format" } :: errors
    else errors

let errors =
    if age < 18 then
        { field = "age"; message = "Must be 18 or older" } :: errors
    else errors

errors
```

**Validation**:
```bash
cd examples/web_server
cargo run

# In another terminal:
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"username":"ab","email":"invalid","age":15}'

# Should return validation errors
```

---

#### Task 6A.5: Computation Expressions Example
**Description**: Demonstrate custom DSL with computation expressions (desugared).

**Deliverables**:
- F# script showing `result { ... }` computation expression
- Explanation of desugaring
- Comparison with hand-written code

**Files to Create/Modify**:
- `examples/computations/result.fsx`
- `examples/computations/README.md`

**Implementation**:
```fsharp
// examples/computations/result.fsx

// Define Result type
type Result<'T, 'E> =
    | Ok of 'T
    | Error of 'E

// Define computation expression builder
let result = {
    bind = fun (r, f) ->
        match r with
        | Ok v -> f v
        | Error e -> Error e

    return = fun v -> Ok v
}

// Usage with computation expression (desugared manually)
let divide x y =
    if y = 0 then Error "Division by zero"
    else Ok (x / y)

let compute a b c =
    result.bind (divide a b, fun x ->
        result.bind (divide x c, fun y ->
            result.return y
        )
    )

// Test
let test1 = compute 100 10 2  // Ok 5
let test2 = compute 100 0 2   // Error "Division by zero"

printfn "Test 1: %A" test1
printfn "Test 2: %A" test2
```

**Validation**:
```bash
cargo run -- run examples/computations/result.fsx
# Should print:
# Test 1: Ok 5
# Test 2: Error "Division by zero"
```

---

#### Task 6A.6: .NET Interop Example
**Description**: Prove syntax compatibility by running same script on both Fusabi and .NET.

**Deliverables**:
- F# script: `common.fsx`
- Bash script: `run.sh` that runs on both
- Comparison of outputs

**Files to Create/Modify**:
- `examples/interop_net/common.fsx`
- `examples/interop_net/run.sh`
- `examples/interop_net/README.md`

**Implementation**:
```fsharp
// examples/interop_net/common.fsx

// Subset of F# that works on both Fusabi and .NET
let rec fib n =
    if n <= 1 then n
    else fib (n - 1) + fib (n - 2)

let nums = [1; 2; 3; 4; 5]
let doubled = List.map (fun x -> x * 2) nums

printfn "fib(10) = %d" (fib 10)
printfn "Doubled: %A" doubled
```

```bash
#!/bin/bash
# examples/interop_net/run.sh

echo "=== Running with Fusabi ==="
cargo run --quiet -- run common.fsx

echo ""
echo "=== Running with .NET FSI ==="
dotnet fsi common.fsx
```

**Validation**:
```bash
cd examples/interop_net
chmod +x run.sh
./run.sh

# Should show identical output from both runtimes
```

---

### Part B: Documentation

#### Task 6B.1: Create CONTRIBUTING.md
**Description**: Write comprehensive contributor guide.

**Deliverables**:
- Explain 3-layer architecture (AST, Bytecode, VM)
- How to add a new Instruction
- How to add a stdlib function
- Testing guidelines
- PR process

**Files to Create/Modify**:
- `CONTRIBUTING.md`

**Content Outline**:
```markdown
# Contributing to Fusabi

## Architecture Overview

Fusabi has a 3-layer architecture:

1. **Frontend Layer** (`fusabi-frontend`)
   - Lexer: Source → Tokens
   - Parser: Tokens → AST
   - Compiler: AST → Bytecode

2. **VM Layer** (`fusabi-vm`)
   - Value representation
   - Bytecode instructions
   - Stack-based interpreter
   - Garbage collection

3. **Integration Layer** (`fusabi`)
   - CLI interface
   - Error reporting
   - Standard library registration

## Adding a New Instruction

1. Add variant to `Instruction` enum (`fusabi-vm/src/bytecode.rs`)
2. Update compiler to emit instruction (`fusabi-frontend/src/compiler.rs`)
3. Handle instruction in VM loop (`fusabi-vm/src/vm.rs`)
4. Add tests

## Adding a Stdlib Function

1. Implement function in appropriate module (`fusabi-vm/src/stdlib/*.rs`)
2. Register in `HostRegistry`
3. Add tests
4. Document in language spec

## Testing

- Unit tests: `cargo test --lib`
- Integration tests: `cargo test`
- Benchmarks: `cargo bench`

## PR Process

1. Create feature branch: `feat/your-feature`
2. Write tests (TDD)
3. Implement feature
4. Run `just lint` and `just test`
5. Create PR with description
6. Squash merge after approval
```

**Validation**:
```bash
# Check that CONTRIBUTING.md exists and is comprehensive
cat CONTRIBUTING.md | wc -l
# Should be 200+ lines
```

---

#### Task 6B.2: Create docs/ABI.md
**Description**: Document internal Value representation and `.fzb` file format.

**Deliverables**:
- `Value` enum layout in memory
- How `Rc<RefCell<T>>` is used
- `.fzb` bytecode file format spec
- Magic bytes and versioning

**Files to Create/Modify**:
- `docs/ABI.md`

**Content Outline**:
```markdown
# Fusabi ABI Specification

## Value Representation

Fusabi uses a tagged union for runtime values:

```rust
pub enum Value {
    Int(i64),          // 8 bytes + tag
    Float(f64),        // 8 bytes + tag
    Bool(bool),        // 1 byte + tag
    String(Rc<String>),  // Pointer + tag
    Unit,              // 0 bytes + tag
    // ... compound types
}
```

## Memory Management

- Primitives: Copied on stack
- Heap types: `Rc<RefCell<T>>` for shared ownership
- GC: Mark-and-sweep for cycle collection

## .fzb Bytecode Format

```
[Magic Bytes: 4 bytes] "FZB\x01"
[Version: 1 byte]
[Chunk Data: bincode-serialized]
```

### Instruction Encoding

Each instruction is encoded as:
- 1 byte opcode
- Variable args (depending on instruction)

### Constant Pool

Constants are stored as:
- Count: u32
- Values: [Value; Count]
```

**Validation**:
```bash
cat docs/ABI.md | grep -c "##"
# Should have multiple sections
```

---

#### Task 6B.3: Create docs/SECURITY.md
**Description**: Document current security status and future plans.

**Deliverables**:
- Current lack of sandboxing
- Resource limit plans
- Security best practices
- Vulnerability reporting

**Files to Create/Modify**:
- `docs/SECURITY.md`

**Content Outline**:
```markdown
# Security Policy

## Current Status

⚠️ **Fusabi does not currently implement sandboxing or resource limits.**

Running untrusted scripts can:
- Consume infinite memory
- Infinite loop (no timeout)
- Access host functions without restrictions

## Recommendations

For production use:
1. Run Fusabi in isolated container
2. Set OS-level resource limits (ulimit)
3. Only run trusted scripts
4. Implement timeout wrapper around VM execution

## Future Plans

- [ ] Memory limits per VM instance
- [ ] Execution timeout built into VM
- [ ] Restricted host function mode
- [ ] Capability-based security model
- [ ] Sandboxed file I/O

## Vulnerability Reporting

Please report security issues to: security@fusabi.dev
```

**Validation**:
```bash
cat docs/SECURITY.md
# Should clearly warn about lack of sandboxing
```

---

## Definition of Done

**Examples**:
- [ ] Bevy scripting example working
- [ ] Ratatui layout example working
- [ ] Burn config example working
- [ ] Web server validation example working
- [ ] Computation expressions example documented
- [ ] .NET interop example proves compatibility
- [ ] All examples have comprehensive READMEs
- [ ] All examples tested and verified

**Documentation**:
- [ ] CONTRIBUTING.md complete (200+ lines)
- [ ] docs/ABI.md documents Value representation and .fzb format
- [ ] docs/SECURITY.md warns about current limitations
- [ ] All documentation reviewed for accuracy
- [ ] PR ready for review

## Agent Coordination Hooks
```bash
# BEFORE Work:
npx claude-flow@alpha hooks pre-task --description "ws6-examples-docs"
npx claude-flow@alpha hooks session-restore --session-id "swarm-fusabi-gem-ws6"

# DURING Work:
npx claude-flow@alpha hooks post-edit --file "examples/bevy_scripting/src/main.rs" --memory-key "swarm/fusabi-gem/ws6/bevy-example"
npx claude-flow@alpha hooks post-edit --file "CONTRIBUTING.md" --memory-key "swarm/fusabi-gem/ws6/contributing"
npx claude-flow@alpha hooks post-edit --file "docs/ABI.md" --memory-key "swarm/fusabi-gem/ws6/abi-spec"
npx claude-flow@alpha hooks notify --message "Examples and documentation complete"

# AFTER Work:
npx claude-flow@alpha hooks post-task --task-id "ws6-complete"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## Estimated Effort
**Duration**: 5-6 days (2 days docs, 3-4 days examples)
**Complexity**: Medium

## References
- [Bevy Game Engine](https://bevyengine.org/)
- [Ratatui TUI Framework](https://ratatui.rs/)
- [Burn Deep Learning](https://burn-rs.github.io/)
- [Axum Web Framework](https://docs.rs/axum/)
- [F# Computation Expressions](https://docs.microsoft.com/en-us/dotnet/fsharp/language-reference/computation-expressions)

## Notes
- **Examples**: Focus on showcasing Fusabi's embedded utility and dual-runtime capability
- **Documentation**: Be honest about current limitations (no sandboxing) while showing future plans
- **Interop**: The .NET compatibility example is powerful marketing for F# developers
- **Dependencies**: Can start documentation and example structure immediately, but full functionality needs WS1+WS3

## File Conflicts
- **No conflicts** with other workstreams
- Can run in parallel, but examples won't work fully until WS1+WS3 complete
- Docs can be written immediately
