//! Terminal Emulator Configuration Demo
//!
//! This example demonstrates production-ready FSRS embedding in a terminal emulator
//! configuration system, similar to WezTerm's Lua configuration API.
//!
//! # Features
//!
//! - Dynamic configuration loading from FSRS scripts
//! - Host function registration for terminal operations
//! - Type-safe value marshalling between Rust and FSRS
//! - Real-time configuration evaluation
//! - Production-quality error handling
//!
//! # Usage
//!
//! ```bash
//! cargo run --example terminal_config
//! ```

use fsrs_demo::FsrsEngine;
use fsrs_frontend::{Compiler, Lexer, Parser};
use fsrs_vm::{HostRegistry, Value, Vm, VmError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::rc::Rc;

// ============================================================================
// Terminal State Management
// ============================================================================

/// Represents a tab in the terminal emulator
#[derive(Debug, Clone)]
struct Tab {
    id: i64,
    title: String,
    active_pane: Pane,
    is_active: bool,
}

/// Represents a pane within a tab
#[derive(Debug, Clone)]
struct Pane {
    pane_id: i64,
    foreground_process: String,
    current_dir: String,
}

/// Terminal state holds all tabs and configuration
struct TerminalState {
    tabs: Vec<Tab>,
    active_tab_index: usize,
    next_tab_id: i64,
    next_pane_id: i64,
}

impl TerminalState {
    fn new() -> Self {
        TerminalState {
            tabs: Vec::new(),
            active_tab_index: 0,
            next_tab_id: 1,
            next_pane_id: 1,
        }
    }

    fn create_tab(&mut self, title: &str) -> i64 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        let tab = Tab {
            id: tab_id,
            title: title.to_string(),
            active_pane: Pane {
                pane_id,
                foreground_process: "zsh".to_string(),
                current_dir: "/home/user".to_string(),
            },
            is_active: self.tabs.is_empty(),
        };

        self.tabs.push(tab);
        if self.tabs.len() == 1 {
            self.active_tab_index = 0;
        }

        tab_id
    }

    fn close_tab(&mut self, tab_id: i64) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.tabs.remove(index);
            if self.active_tab_index >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab_index = self.tabs.len() - 1;
            }
            true
        } else {
            false
        }
    }

    fn switch_to_tab(&mut self, tab_id: i64) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            // Mark old tab as inactive
            if !self.tabs.is_empty() {
                self.tabs[self.active_tab_index].is_active = false;
            }
            // Mark new tab as active
            self.active_tab_index = index;
            self.tabs[index].is_active = true;
            true
        } else {
            false
        }
    }

    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.tabs[self.active_tab_index].is_active = false;
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
            self.tabs[self.active_tab_index].is_active = true;
        }
    }

    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.tabs[self.active_tab_index].is_active = false;
            if self.active_tab_index == 0 {
                self.active_tab_index = self.tabs.len() - 1;
            } else {
                self.active_tab_index -= 1;
            }
            self.tabs[self.active_tab_index].is_active = true;
        }
    }

    fn get_tab(&self, tab_id: i64) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == tab_id)
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }
}

// ============================================================================
// Terminal Configuration Engine
// ============================================================================

/// Main terminal configuration engine that integrates FSRS scripting
pub struct TerminalConfig {
    vm: Vm,
    host_registry: HostRegistry,
    state: Rc<RefCell<TerminalState>>,
    config_source: String,
}

impl TerminalConfig {
    /// Create a new terminal configuration engine
    pub fn new() -> Self {
        TerminalConfig {
            vm: Vm::new(),
            host_registry: HostRegistry::new(),
            state: Rc::new(RefCell::new(TerminalState::new())),
            config_source: String::new(),
        }
    }

    /// Register all host functions for terminal operations
    pub fn register_host_functions(&mut self) {
        let state = Rc::clone(&self.state);

        // Tab management functions
        self.host_registry.register_fn1("Host.createTab", move |v| {
            let title = v
                .as_str()
                .ok_or_else(|| VmError::Runtime("createTab expects string argument".into()))?;
            let tab_id = state.borrow_mut().create_tab(title);
            println!("  [Host] Created tab '{}' with ID {}", title, tab_id);
            Ok(Value::Int(tab_id))
        });

        let state = Rc::clone(&self.state);
        self.host_registry.register_fn1("Host.closeTab", move |v| {
            let tab_id = v
                .as_int()
                .ok_or_else(|| VmError::Runtime("closeTab expects int argument".into()))?;
            let success = state.borrow_mut().close_tab(tab_id);
            if success {
                println!("  [Host] Closed tab ID {}", tab_id);
            } else {
                println!("  [Host] Tab ID {} not found", tab_id);
            }
            Ok(Value::Bool(success))
        });

        let state = Rc::clone(&self.state);
        self.host_registry.register_fn0("Host.closeCurrentTab", move || {
            let tab_id = state
                .borrow()
                .active_tab()
                .map(|t| t.id)
                .unwrap_or(0);
            if tab_id > 0 {
                state.borrow_mut().close_tab(tab_id);
                println!("  [Host] Closed current tab (ID {})", tab_id);
                Ok(Value::Bool(true))
            } else {
                println!("  [Host] No active tab to close");
                Ok(Value::Bool(false))
            }
        });

        let state = Rc::clone(&self.state);
        self.host_registry.register_fn1("Host.switchToTab", move |v| {
            let tab_id = v
                .as_int()
                .ok_or_else(|| VmError::Runtime("switchToTab expects int argument".into()))?;
            let success = state.borrow_mut().switch_to_tab(tab_id);
            if success {
                println!("  [Host] Switched to tab ID {}", tab_id);
            } else {
                println!("  [Host] Tab ID {} not found", tab_id);
            }
            Ok(Value::Bool(success))
        });

        let state = Rc::clone(&self.state);
        self.host_registry.register_fn0("Host.nextTab", move || {
            state.borrow_mut().next_tab();
            println!("  [Host] Switched to next tab");
            Ok(Value::Unit)
        });

        let state = Rc::clone(&self.state);
        self.host_registry.register_fn0("Host.prevTab", move || {
            state.borrow_mut().prev_tab();
            println!("  [Host] Switched to previous tab");
            Ok(Value::Unit)
        });

        // Utility functions
        self.host_registry.register_fn1("Host.log", |v| {
            println!("  [FSRS Log] {:?}", v);
            Ok(Value::Unit)
        });

        self.host_registry.register("Host.concat", |args| {
            let strings: Vec<String> = args
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .unwrap_or_else(|| format!("{:?}", v))
                })
                .collect();
            Ok(Value::Str(strings.join("")))
        });
    }

    /// Load configuration from an FSRS script file
    pub fn load_config_file(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let source = fs::read_to_string(path.as_ref())?;
        self.config_source = source.clone();
        self.load_config_source(&source)
    }

    /// Load configuration from an FSRS script string
    pub fn load_config_source(&mut self, source: &str) -> Result<(), Box<dyn Error>> {
        println!("=== Loading Terminal Configuration ===\n");

        // Compile the script
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        let chunk = Compiler::compile(&ast)?;

        println!("  Compiled {} instructions", chunk.instructions.len());
        println!("  Constant pool size: {}", chunk.constants.len());
        println!();

        // Execute with host registry
        self.vm.execute_with_host(chunk, &self.host_registry)?;

        println!("\n=== Configuration Loaded Successfully ===\n");
        Ok(())
    }

    /// Create a tab info record for passing to FSRS functions
    fn create_tab_info(&self, tab: &Tab) -> Value {
        let mut fields = HashMap::new();
        fields.insert("id".to_string(), Value::Int(tab.id));
        fields.insert("title".to_string(), Value::Str(tab.title.clone()));
        fields.insert("isActive".to_string(), Value::Bool(tab.is_active));

        // Pane info
        let mut pane_fields = HashMap::new();
        pane_fields.insert("paneId".to_string(), Value::Int(tab.active_pane.pane_id));
        pane_fields.insert(
            "process".to_string(),
            Value::Str(tab.active_pane.foreground_process.clone()),
        );
        pane_fields.insert(
            "cwd".to_string(),
            Value::Str(tab.active_pane.current_dir.clone()),
        );
        fields.insert("pane".to_string(), Value::Record(Rc::new(RefCell::new(pane_fields))));

        Value::Record(Rc::new(RefCell::new(fields)))
    }

    /// Display current terminal state
    pub fn display_state(&self) {
        let state = self.state.borrow();
        println!("=== Terminal State ===");
        println!("Active tab index: {}", state.active_tab_index);
        println!("Total tabs: {}\n", state.tabs.len());

        for (idx, tab) in state.tabs.iter().enumerate() {
            let marker = if tab.is_active { " [ACTIVE]" } else { "" };
            println!(
                "  Tab {}: ID={}, Title='{}', Process='{}'{}",
                idx, tab.id, tab.title, tab.active_pane.foreground_process, marker
            );
        }
        println!();
    }

    /// Get terminal state reference
    pub fn state(&self) -> &Rc<RefCell<TerminalState>> {
        &self.state
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║   FSRS Terminal Emulator Configuration Demo                  ║");
    println!("║   Production-Ready Host Interop Example                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Create terminal configuration engine
    let mut config = TerminalConfig::new();

    // Register all host functions
    println!("=== Registering Host Functions ===\n");
    config.register_host_functions();
    println!("  Registered host functions:");
    println!("    - Host.createTab");
    println!("    - Host.closeTab");
    println!("    - Host.closeCurrentTab");
    println!("    - Host.switchToTab");
    println!("    - Host.nextTab");
    println!("    - Host.prevTab");
    println!("    - Host.log");
    println!("    - Host.concat");
    println!();

    // Load configuration script
    let config_script = r#"
// Terminal Configuration Script (FSRS)
// Demonstrates host interop and configuration management

// Create initial tabs
let tab1 = Host.createTab "Terminal"
let tab2 = Host.createTab "Editor"
let tab3 = Host.createTab "Server Logs"

// Log tab creation
Host.log tab1
Host.log tab2
Host.log tab3

// Switch between tabs
Host.switchToTab tab2

// Close a tab
Host.closeTab tab3

// Navigate tabs
Host.nextTab
Host.prevTab

// Return success
42
"#;

    // Execute the configuration
    config.load_config_source(config_script)?;

    // Display final state
    config.display_state();

    // Demonstrate runtime operations
    println!("=== Runtime Operations Demo ===\n");

    println!("Creating additional tab from Rust...");
    config.state.borrow_mut().create_tab("Build Output");
    config.display_state();

    println!("Navigating to next tab...");
    config.state.borrow_mut().next_tab();
    config.display_state();

    // Summary
    println!("=== Demo Summary ===\n");
    println!("This demo showcases:");
    println!("  ✓ Host function registration");
    println!("  ✓ FSRS script execution with host interop");
    println!("  ✓ Bidirectional state management (Rust ↔ FSRS)");
    println!("  ✓ Type-safe value marshalling");
    println!("  ✓ Production-quality error handling");
    println!("  ✓ Real-world terminal emulator use case");
    println!();
    println!("Next steps:");
    println!("  • Add hot-reload support for live config editing");
    println!("  • Implement tab formatter functions in FSRS");
    println!("  • Add keybinding configuration");
    println!("  • Integrate with actual terminal UI");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_terminal_state_close_tab() {
        let mut state = TerminalState::new();
        let tab_id = state.create_tab("Test Tab");
        assert!(state.close_tab(tab_id));
        assert_eq!(state.tabs.len(), 0);
    }

    #[test]
    fn test_terminal_state_switch_tab() {
        let mut state = TerminalState::new();
        let tab1 = state.create_tab("Tab 1");
        let tab2 = state.create_tab("Tab 2");

        assert!(state.switch_to_tab(tab2));
        assert!(!state.tabs[0].is_active);
        assert!(state.tabs[1].is_active);

        assert!(state.switch_to_tab(tab1));
        assert!(state.tabs[0].is_active);
        assert!(!state.tabs[1].is_active);
    }

    #[test]
    fn test_terminal_state_next_tab() {
        let mut state = TerminalState::new();
        state.create_tab("Tab 1");
        state.create_tab("Tab 2");
        state.create_tab("Tab 3");

        assert_eq!(state.active_tab_index, 0);
        state.next_tab();
        assert_eq!(state.active_tab_index, 1);
        state.next_tab();
        assert_eq!(state.active_tab_index, 2);
        state.next_tab();
        assert_eq!(state.active_tab_index, 0); // Wrap around
    }

    #[test]
    fn test_terminal_state_prev_tab() {
        let mut state = TerminalState::new();
        state.create_tab("Tab 1");
        state.create_tab("Tab 2");
        state.create_tab("Tab 3");

        state.prev_tab();
        assert_eq!(state.active_tab_index, 2); // Wrap around
        state.prev_tab();
        assert_eq!(state.active_tab_index, 1);
        state.prev_tab();
        assert_eq!(state.active_tab_index, 0);
    }

    #[test]
    fn test_config_engine_creation() {
        let mut config = TerminalConfig::new();
        config.register_host_functions();
        assert!(config.host_registry.has_function("Host.createTab"));
        assert!(config.host_registry.has_function("Host.nextTab"));
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

    #[test]
    fn test_config_multiple_tabs() {
        let mut config = TerminalConfig::new();
        config.register_host_functions();

        let script = r#"
            let t1 = Host.createTab "First"
            let t2 = Host.createTab "Second"
            let t3 = Host.createTab "Third"
            t3
        "#;

        let result = config.load_config_source(script);
        assert!(result.is_ok());
        assert_eq!(config.state.borrow().tabs.len(), 3);
    }

    #[test]
    fn test_config_tab_navigation() {
        let mut config = TerminalConfig::new();
        config.register_host_functions();

        let script = r#"
            let t1 = Host.createTab "First"
            let t2 = Host.createTab "Second"
            Host.nextTab
            Host.prevTab
            42
        "#;

        let result = config.load_config_source(script);
        assert!(result.is_ok());
    }
}
