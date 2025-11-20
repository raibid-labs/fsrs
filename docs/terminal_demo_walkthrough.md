# Terminal Configuration Demo - Complete Walkthrough

**Step-by-step guide to understanding the FSRS terminal emulator configuration example**

## Overview

This walkthrough takes you through the terminal configuration demo, explaining each component and design decision. By the end, you'll understand how to embed FSRS in your own applications.

---

## Table of Contents

1. [Project Structure](#project-structure)
2. [Architecture Overview](#architecture-overview)
3. [Step 1: Terminal State Management](#step-1-terminal-state-management)
4. [Step 2: Host Function Registration](#step-2-host-function-registration)
5. [Step 3: FSRS Configuration Script](#step-3-fsrs-configuration-script)
6. [Step 4: Integration & Execution](#step-4-integration--execution)
7. [Testing](#testing)
8. [Extensions](#extensions)

---

## Project Structure

```
examples/terminal_config/
├── main.rs              # Full demonstration with VM integration
├── simple_demo.rs       # Simplified demo showing host API patterns
├── config.fsrs          # Example FSRS configuration script
└── README.md            # Documentation
```

### File Purposes

- **main.rs**: Production-quality example with complete VM integration
- **simple_demo.rs**: Focused example showing host API usage patterns
- **config.fsrs**: Modular FSRS configuration demonstrating best practices

---

## Architecture Overview

### Component Hierarchy

```
┌─────────────────────────────────────────┐
│  Terminal Application (Your Rust Code) │
│  - Tab management                       │
│  - UI rendering                         │
│  - Event handling                       │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  TerminalConfig (Integration Layer)     │
│  - Host function registry               │
│  - VM instance                          │
│  - Shared state management              │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  FSRS Engine (Scripting Layer)          │
│  - Script compilation                   │
│  - Bytecode execution                   │
│  - Value marshalling                    │
└─────────────────────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  Configuration Script (config.fsrs)     │
│  - User-defined configuration           │
│  - Calls host functions                 │
│  - Defines behavior                     │
└─────────────────────────────────────────┘
```

### Data Flow

```
1. User edits config.fsrs
        ↓
2. Rust loads and compiles script
        ↓
3. VM executes bytecode
        ↓
4. Script calls host functions
        ↓
5. Rust updates terminal state
        ↓
6. Terminal UI reflects changes
```

---

## Step 1: Terminal State Management

### Design: Shared Mutable State

The terminal state needs to be accessible from both Rust and FSRS scripts. We use `Rc<RefCell<T>>` for shared mutable access.

### Code: TerminalState Struct

```rust
#[derive(Debug)]
struct TerminalState {
    tabs: Vec<Tab>,
    active_tab_index: usize,
    next_tab_id: i64,
    next_pane_id: i64,
}
```

**Fields:**
- `tabs`: All open tabs
- `active_tab_index`: Currently focused tab
- `next_tab_id`: Monotonically increasing ID generator
- `next_pane_id`: Monotonically increasing pane ID generator

### Code: Tab Struct

```rust
#[derive(Debug, Clone)]
struct Tab {
    id: i64,
    title: String,
    active_pane: Pane,
    is_active: bool,
}
```

**Fields:**
- `id`: Unique identifier
- `title`: Display name
- `active_pane`: The currently active pane in this tab
- `is_active`: Whether this tab has focus

### Code: Core Operations

```rust
impl TerminalState {
    fn create_tab(&mut self, title: &str) -> i64 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Tab {
            id: tab_id,
            title: title.to_string(),
            active_pane: Pane::new(self.next_pane_id),
            is_active: self.tabs.is_empty(), // First tab is active
        };

        self.tabs.push(tab);
        tab_id
    }

    fn close_tab(&mut self, tab_id: i64) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.tabs.remove(index);
            // Adjust active index if needed
            if self.active_tab_index >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab_index = self.tabs.len() - 1;
            }
            true
        } else {
            false
        }
    }
}
```

**Design Decisions:**
- Tab IDs are immutable once assigned
- First tab created is automatically active
- Closing a tab adjusts the active index
- Boolean return indicates success/failure

---

## Step 2: Host Function Registration

### Pattern: Closure Capture

Each host function is a closure that captures a clone of `Rc<RefCell<TerminalState>>`.

### Code: Registration Function

```rust
fn register_terminal_api(
    engine: &mut FsrsEngine,
    state: Rc<RefCell<TerminalState>>
) {
    // Each function gets its own Rc clone
    {
        let state = Rc::clone(&state);
        engine.register_fn1("createTab", move |v| {
            let title = v.as_str()
                .ok_or_else(|| VmError::Runtime("createTab expects string".into()))?;

            let tab_id = state.borrow_mut().create_tab(title);
            println!("  [Host] Created tab '{}' with ID {}", title, tab_id);

            Ok(Value::Int(tab_id))
        });
    }

    {
        let state = Rc::clone(&state);
        engine.register_fn1("closeTab", move |v| {
            let tab_id = v.as_int()
                .ok_or_else(|| VmError::Runtime("closeTab expects int".into()))?;

            let success = state.borrow_mut().close_tab(tab_id);
            println!("  [Host] Closed tab ID {} (success: {})", tab_id, success);

            Ok(Value::Bool(success))
        });
    }
}
```

### Pattern Breakdown

1. **Scoped Cloning**:
   ```rust
   {
       let state = Rc::clone(&state);
       engine.register_fn1("funcName", move |v| {
           // Use state here
       });
   }
   ```
   Each closure gets its own `Rc` clone in a scope.

2. **Type Validation**:
   ```rust
   let value = arg.as_type()
       .ok_or_else(|| VmError::Runtime("Expected type".into()))?;
   ```
   Extract values with clear error messages.

3. **Mutable Borrowing**:
   ```rust
   state.borrow_mut().method_call()
   ```
   Short-lived borrows prevent runtime panics.

4. **Return Value Marshalling**:
   ```rust
   Ok(Value::Int(tab_id))      // Return int
   Ok(Value::Bool(success))    // Return bool
   Ok(Value::Unit)             // Return unit
   ```

---

## Step 3: FSRS Configuration Script

### Structure: Modular Design

The configuration script uses modules to organize functionality.

### Code: config.fsrs

```fsharp
// ============================================================================
// Module: TabManager
// ============================================================================

module TabManager =
    let formatTitle tab =
        let icon = if tab.isActive then "▶" else " "
        Host.concat icon " " tab.title

    let getTabColor index =
        let colors = ["#FF6B6B"; "#4ECDC4"; "#45B7D1"; "#FFA07A"]
        let colorIndex = index % 4
        "#45B7D1"  // Simplified for demo

    let createFormattedTab baseName index =
        let formatted = Host.concat baseName " " (String.ofInt index)
        Host.createTab formatted

// ============================================================================
// Module: KeyBindings
// ============================================================================

module KeyBindings =
    let onCtrlT () =
        Host.log "Ctrl+T pressed"
        Host.createTab "New Tab"

    let onCtrlW () =
        Host.log "Ctrl+W pressed"
        Host.closeCurrentTab

    let onCtrlTab () =
        Host.log "Ctrl+Tab pressed"
        Host.nextTab

// ============================================================================
// Main Configuration
// ============================================================================

let editorTab = Host.createTab "Editor"
let terminalTab = Host.createTab "Terminal"
let logsTab = Host.createTab "Logs"

let config = {
    formatTabTitle = TabManager.formatTitle;
    colorScheme = ColorScheme.darkTheme;
    keyBindings = [
        ("Ctrl+T", KeyBindings.onCtrlT);
        ("Ctrl+W", KeyBindings.onCtrlW);
        ("Ctrl+Tab", KeyBindings.onCtrlTab)
    ];
    tabs = [editorTab; terminalTab; logsTab]
}

config
```

### Design Patterns

1. **Module Organization**:
   - `TabManager`: Tab-related functionality
   - `KeyBindings`: Keyboard shortcut handlers
   - `ColorScheme`: Visual theming

2. **Separation of Concerns**:
   - Logic in modules
   - Configuration in main section
   - Export at end

3. **Host Function Calls**:
   - Prefixed with `Host.`
   - Clear parameter types
   - Documented behavior

---

## Step 4: Integration & Execution

### Code: TerminalConfig Struct

```rust
pub struct TerminalConfig {
    vm: Vm,
    host_registry: HostRegistry,
    state: Rc<RefCell<TerminalState>>,
    config_source: String,
}
```

### Code: Loading Configuration

```rust
impl TerminalConfig {
    pub fn load_config_file(&mut self, path: impl AsRef<Path>)
        -> Result<(), Box<dyn Error>>
    {
        let source = fs::read_to_string(path.as_ref())?;
        self.config_source = source.clone();
        self.load_config_source(&source)
    }

    pub fn load_config_source(&mut self, source: &str)
        -> Result<(), Box<dyn Error>>
    {
        println!("=== Loading Configuration ===\n");

        // 1. Compile the script
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let chunk = Compiler::compile(&ast)?;

        println!("  Compiled {} instructions", chunk.instructions.len());

        // 2. Execute with host registry
        self.vm.execute_with_host(chunk, &self.host_registry)?;

        println!("\n=== Configuration Loaded ===\n");
        Ok(())
    }
}
```

### Execution Flow

1. **Load**: Read FSRS source code from file
2. **Lex**: Convert source to tokens
3. **Parse**: Build AST from tokens
4. **Compile**: Generate bytecode from AST
5. **Execute**: Run bytecode in VM with host functions
6. **Complete**: Configuration is now active

---

## Step 5: Main Demo

### Code: main() Function

```rust
fn main() -> Result<(), Box<dyn Error>> {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║   FSRS Terminal Configuration Demo   ║");
    println!("╚═══════════════════════════════════════╝\n");

    // 1. Create configuration engine
    let mut config = TerminalConfig::new();

    // 2. Register host functions
    println!("=== Registering Host Functions ===\n");
    config.register_host_functions();

    // 3. Load configuration script
    let script = r#"
        let tab1 = Host.createTab "Terminal"
        let tab2 = Host.createTab "Editor"
        let tab3 = Host.createTab "Logs"

        Host.switchToTab tab2
        Host.closeTab tab3

        tab2
    "#;

    config.load_config_source(script)?;

    // 4. Display results
    config.display_state();

    Ok(())
}
```

### Output

```
╔═══════════════════════════════════════╗
║   FSRS Terminal Configuration Demo   ║
╚═══════════════════════════════════════╝

=== Registering Host Functions ===

  Registered:
    - Host.createTab
    - Host.closeTab
    - Host.switchToTab
    - Host.nextTab
    - Host.prevTab

=== Loading Configuration ===

  Compiled 42 instructions

  [Host] Created tab 'Terminal' with ID 1
  [Host] Created tab 'Editor' with ID 2
  [Host] Created tab 'Logs' with ID 3
  [Host] Switched to tab ID 2
  [Host] Closed tab ID 3

=== Configuration Loaded ===

=== Terminal State ===
Active tab index: 1
Total tabs: 2

  Tab 0: ID=1, Title='Terminal', Process='zsh'
  Tab 1: ID=2, Title='Editor', Process='zsh' [ACTIVE]
```

---

## Testing

### Unit Tests

The demo includes comprehensive tests:

```rust
#[test]
fn test_terminal_state_create_tab() {
    let mut state = TerminalState::new();
    let tab_id = state.create_tab("Test Tab");
    assert_eq!(tab_id, 1);
    assert_eq!(state.tabs.len(), 1);
    assert_eq!(state.tabs[0].title, "Test Tab");
    assert!(state.tabs[0].is_active);
}

#[test]
fn test_config_load_simple_script() {
    let mut config = TerminalConfig::new();
    config.register_host_functions();

    let script = r#"
        let x = Host.createTab "Test"
        x
    "#;

    let result = config.load_config_source(script);
    assert!(result.is_ok());
    assert_eq!(config.state.borrow().tabs.len(), 1);
}
```

### Running Tests

```bash
# Run all tests
cargo test --example terminal_config

# Run with output
cargo test --example terminal_config -- --nocapture

# Run specific test
cargo test --example terminal_config test_terminal_state_create_tab
```

---

## Extensions

### Extension 1: Tab Formatters

Call FSRS functions from Rust to format tab titles:

```rust
impl TerminalConfig {
    pub fn format_tab_title(&mut self, tab: &Tab) -> Result<String, VmError> {
        // Create tab info record
        let tab_info = self.create_tab_info(tab);

        // Call FSRS function
        let result = self.vm.call_function("formatTabTitle", &[tab_info])?;

        // Extract result
        result.as_str()
            .map(String::from)
            .ok_or_else(|| VmError::Runtime("formatTabTitle must return string".into()))
    }
}
```

**FSRS:**
```fsharp
let formatTabTitle tab =
    let icon = if tab.isActive then "▶" else " "
    let process = tab.pane.process
    Host.concat icon " " tab.title " (" process ")"
```

### Extension 2: Hot-Reload

Watch config file for changes:

```rust
use notify::{Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;

impl TerminalConfig {
    pub fn watch_config(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(100))?;

        watcher.watch(path, RecursiveMode::NonRecursive)?;

        println!("Watching {} for changes...", path);

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("Config changed, reloading...");
                    self.load_config_file(path)?;
                }
                Err(e) => eprintln!("Watch error: {}", e),
            }
        }
    }
}
```

### Extension 3: Color Scheme Application

Read and apply color configuration:

```rust
impl TerminalConfig {
    pub fn apply_colors(&mut self) -> Result<(), VmError> {
        let bg = self.vm.get_global("config.colorScheme.background")?;
        let fg = self.vm.get_global("config.colorScheme.foreground")?;

        let bg_color = bg.as_str().ok_or_else(||
            VmError::Runtime("background must be string".into())
        )?;

        let fg_color = fg.as_str().ok_or_else(||
            VmError::Runtime("foreground must be string".into())
        )?;

        println!("Applying colors: bg={}, fg={}", bg_color, fg_color);
        // Apply to actual terminal UI...

        Ok(())
    }
}
```

### Extension 4: Keybinding Integration

Wire up keyboard events:

```rust
use crossterm::event::{Event, KeyCode, KeyModifiers};

impl TerminalConfig {
    pub fn handle_key_event(&mut self, event: &Event) -> Result<(), VmError> {
        if let Event::Key(key) = event {
            match (key.code, key.modifiers) {
                (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                    self.vm.call_function("KeyBindings.onCtrlT", &[])?;
                }
                (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                    self.vm.call_function("KeyBindings.onCtrlW", &[])?;
                }
                (KeyCode::Tab, KeyModifiers::CONTROL) => {
                    self.vm.call_function("KeyBindings.onCtrlTab", &[])?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
```

---

## Key Takeaways

### Design Principles

1. **Shared State**: Use `Rc<RefCell<T>>` for mutable shared access
2. **Closure Capture**: Clone `Rc` for each host function
3. **Type Safety**: Validate values with clear error messages
4. **Modular Scripts**: Organize FSRS code into modules
5. **Clear API**: Namespace host functions logically

### Best Practices

1. **Short Borrows**: Keep `borrow_mut()` scopes minimal
2. **Error Propagation**: Use `?` operator for clean error handling
3. **Documentation**: Comment both Rust and FSRS code
4. **Testing**: Test both Rust and FSRS components
5. **Validation**: Validate configuration after loading

### Common Patterns

1. **State Management**: `Rc<RefCell<AppState>>`
2. **Host Registration**: Closure with captured Rc
3. **Type Extraction**: `value.as_type().ok_or_else(...)?`
4. **Module Organization**: Separate concerns in FSRS
5. **Configuration Export**: Return record from script

---

## Next Steps

1. **Run the Demo**: `cargo run --example terminal_config`
2. **Modify the Script**: Edit `config.fsrs` and experiment
3. **Add Features**: Implement the extensions above
4. **Build Your Own**: Adapt this pattern to your application
5. **Share Feedback**: Contribute to FSRS development

---

## Resources

- **Source Code**: `/examples/terminal_config/`
- **Embedding Guide**: `/docs/EMBEDDING_GUIDE.md`
- **Host Interop**: `/docs/HOST_INTEROP.md`
- **Language Spec**: `/docs/02-language-spec.md`

---

**Happy Coding!** 🎉
