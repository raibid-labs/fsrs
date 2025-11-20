//! Simplified Terminal Configuration Demo
//!
//! This example demonstrates FSRS host interop using the current VM API.
//! It shows how to integrate FSRS scripting into a terminal emulator configuration system.
//!
//! # Architecture
//!
//! 1. Host application (Rust) registers callable functions
//! 2. FSRS scripts call these functions to configure the terminal
//! 3. Results are marshalled back to Rust for application
//!
//! # Usage
//!
//! ```bash
//! cargo run --package fsrs-demo --example simple_terminal_demo
//! ```

use fsrs_demo::FsrsEngine;
use fsrs_vm::{Value, VmError};
use std::sync::{Arc, Mutex};

// ============================================================================
// Terminal State Management
// ============================================================================

/// Represents a tab in the terminal
#[derive(Debug, Clone)]
struct Tab {
    id: i64,
    title: String,
    is_active: bool,
}

/// Shared terminal state accessible from both Rust and FSRS
#[derive(Debug)]
struct TerminalState {
    tabs: Vec<Tab>,
    next_tab_id: i64,
}

impl TerminalState {
    fn new() -> Self {
        TerminalState {
            tabs: Vec::new(),
            next_tab_id: 1,
        }
    }

    fn create_tab(&mut self, title: &str) -> i64 {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let tab = Tab {
            id: tab_id,
            title: title.to_string(),
            is_active: self.tabs.is_empty(),
        };

        println!("  [Host] Created tab '{}' with ID {}", title, tab_id);
        self.tabs.push(tab);
        tab_id
    }

    fn close_tab(&mut self, tab_id: i64) -> bool {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            let tab = self.tabs.remove(index);
            println!("  [Host] Closed tab '{}' (ID {})", tab.title, tab_id);
            true
        } else {
            println!("  [Host] Tab ID {} not found", tab_id);
            false
        }
    }

    fn list_tabs(&self) {
        println!("\n=== Current Tabs ===");
        for (idx, tab) in self.tabs.iter().enumerate() {
            let marker = if tab.is_active { " [ACTIVE]" } else { "" };
            println!("  {}: {} (ID: {}){}",  idx, tab.title, tab.id, marker);
        }
        println!();
    }
}

// ============================================================================
// Host API Registration
// ============================================================================

/// Register host functions that FSRS scripts can call
fn register_terminal_api(engine: &mut FsrsEngine, state: Arc<Mutex<TerminalState>>) {
    // Create tab
    {
        let state = Arc::clone(&state);
        engine.register_fn1("createTab", move |v| {
            let title = v
                .as_str()
                .ok_or_else(|| VmError::Runtime("createTab expects string".into()))?;
            let tab_id = state.lock().unwrap().create_tab(title);
            Ok(Value::Int(tab_id))
        });
    }

    // Close tab
    {
        let state = Arc::clone(&state);
        engine.register_fn1("closeTab", move |v| {
            let tab_id = v
                .as_int()
                .ok_or_else(|| VmError::Runtime("closeTab expects int".into()))?;
            let success = state.lock().unwrap().close_tab(tab_id);
            Ok(Value::Bool(success))
        });
    }

    // Log function for debugging
    engine.register_fn1("log", |v| {
        println!("  [FSRS Log] {:?}", v);
        Ok(Value::Unit)
    });

    // String concatenation helper
    engine.register("concat", |args| {
        let result: String = args
            .iter()
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| format!("{:?}", v))
            })
            .collect::<Vec<_>>()
            .join("");
        Ok(Value::Str(result))
    });
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n");
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║   FSRS Terminal Configuration Demo                           ║");
    println!("║   Host Interop Example                                       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // Create shared terminal state
    let state = Arc::new(Mutex::new(TerminalState::new()));

    // ========================================================================
    // Demo 1: Basic Configuration
    // ========================================================================

    println!("═══ Demo 1: Basic Configuration ═══\n");

    let mut engine1 = FsrsEngine::new();
    register_terminal_api(&mut engine1, Arc::clone(&state));

    println!("Executing configuration via host functions...\n");

    let result1 = engine1.call_host("createTab", &[Value::Str("Terminal".to_string())])?;
    println!("Result: {:?}", result1);

    let result2 = engine1.call_host("createTab", &[Value::Str("Editor".to_string())])?;
    println!("Result: {:?}", result2);

    let result3 = engine1.call_host("createTab", &[Value::Str("Logs".to_string())])?;
    println!("Result: {:?}", result3);

    state.lock().unwrap().list_tabs();

    // ========================================================================
    // Demo 2: Tab Management Operations
    // ========================================================================

    println!("═══ Demo 2: Tab Management ═══\n");

    println!("Creating additional tabs...\n");
    let _build_tab = engine1.call_host("createTab", &[Value::Str("Build Output".to_string())])?;
    let debug_tab = engine1.call_host("createTab", &[Value::Str("Debug Console".to_string())])?;

    state.lock().unwrap().list_tabs();

    println!("Closing debug tab...\n");
    let close_result = engine1.call_host("closeTab", &[debug_tab])?;
    println!("Close result: {:?}", close_result);

    state.lock().unwrap().list_tabs();

    // ========================================================================
    // Demo 3: Host Function Registry Demonstration
    // ========================================================================

    println!("═══ Demo 3: Host Function Registry ═══\n");

    let functions = engine1.host_function_names();
    println!("Registered host functions:");
    for func in &functions {
        println!("  • {}", func);
    }
    println!();

    // Test each function
    println!("Testing host functions:\n");

    // Test concat
    let concat_result = engine1.call_host(
        "concat",
        &[
            Value::Str("Hello".to_string()),
            Value::Str(" ".to_string()),
            Value::Str("World".to_string()),
        ],
    )?;
    println!("  concat('Hello', ' ', 'World') = {:?}", concat_result);

    // Test log
    engine1.call_host("log", &[Value::Str("Test message".to_string())])?;

    println!();

    // ========================================================================
    // Demo 4: Production Patterns
    // ========================================================================

    println!("═══ Demo 4: Production Patterns ═══\n");

    // Pattern 1: Configuration validation
    println!("Pattern 1: Configuration Validation\n");
    let tab_count = state.lock().unwrap().tabs.len();
    println!("  Current tab count: {}", tab_count);
    if tab_count > 10 {
        println!("  Warning: Many tabs open, consider closing some");
    } else {
        println!("  OK: Tab count within normal range");
    }
    println!();

    // Pattern 2: Conditional tab creation
    println!("Pattern 2: Conditional Tab Creation\n");
    let should_create_debug = cfg!(debug_assertions);
    if should_create_debug {
        println!("  Debug mode: Creating debug tabs...");
        engine1.call_host("createTab", &[Value::Str("Debug Info".to_string())])?;
    } else {
        println!("  Release mode: Skipping debug tabs");
    }
    println!();

    // Pattern 3: Bulk operations
    println!("Pattern 3: Bulk Tab Operations\n");
    let workspace_tabs = vec!["src/main.rs", "src/lib.rs", "tests/test.rs"];
    println!("  Creating workspace tabs:");
    for tab_name in workspace_tabs {
        engine1.call_host("createTab", &[Value::Str(tab_name.to_string())])?;
    }
    println!();

    state.lock().unwrap().list_tabs();

    // ========================================================================
    // Summary
    // ========================================================================

    println!("═══ Demo Summary ═══\n");
    println!("This demo showcased:");
    println!("  • Host function registration with FsrsEngine");
    println!("  • Shared mutable state (Arc<Mutex<T>>)");
    println!("  • Type-safe value marshalling (Value enum)");
    println!("  • Error handling across FFI boundary");
    println!("  • Production patterns for configuration");
    println!();
    println!("Final state:");
    let final_state = state.lock().unwrap();
    println!("  Total tabs created: {}", final_state.tabs.len());
    println!("  Next tab ID: {}", final_state.next_tab_id);
    println!();
    println!("Next steps:");
    println!("  • Integrate host registry with VM execution");
    println!("  • Add hot-reload support for live config editing");
    println!("  • Implement tab formatter functions");
    println!("  • Add keybinding configuration");
    println!("  • Connect to actual terminal UI");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_state_basics() {
        let mut state = TerminalState::new();
        assert_eq!(state.tabs.len(), 0);

        let id = state.create_tab("Test");
        assert_eq!(id, 1);
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.tabs[0].title, "Test");
    }

    #[test]
    fn test_terminal_state_close() {
        let mut state = TerminalState::new();
        let id = state.create_tab("Test");
        assert!(state.close_tab(id));
        assert_eq!(state.tabs.len(), 0);
    }

    #[test]
    fn test_host_api_registration() {
        let state = Arc::new(Mutex::new(TerminalState::new()));
        let mut engine = FsrsEngine::new();
        register_terminal_api(&mut engine, state);

        assert!(engine.has_host_function("createTab"));
        assert!(engine.has_host_function("closeTab"));
        assert!(engine.has_host_function("log"));
        assert!(engine.has_host_function("concat"));
    }

    #[test]
    fn test_create_tab_via_host() {
        let state = Arc::new(Mutex::new(TerminalState::new()));
        let mut engine = FsrsEngine::new();
        register_terminal_api(&mut engine, Arc::clone(&state));

        let result = engine
            .call_host("createTab", &[Value::Str("Test".to_string())])
            .unwrap();

        assert_eq!(result, Value::Int(1));
        assert_eq!(state.lock().unwrap().tabs.len(), 1);
    }

    #[test]
    fn test_concat_function() {
        let state = Arc::new(Mutex::new(TerminalState::new()));
        let mut engine = FsrsEngine::new();
        register_terminal_api(&mut engine, state);

        let result = engine
            .call_host(
                "concat",
                &[
                    Value::Str("Hello".to_string()),
                    Value::Str(" ".to_string()),
                    Value::Str("World".to_string()),
                ],
            )
            .unwrap();

        assert_eq!(result, Value::Str("Hello World".to_string()));
    }

    #[test]
    fn test_bulk_tab_creation() {
        let state = Arc::new(Mutex::new(TerminalState::new()));
        let mut engine = FsrsEngine::new();
        register_terminal_api(&mut engine, Arc::clone(&state));

        let tabs = vec!["Tab1", "Tab2", "Tab3"];
        for tab in tabs {
            engine
                .call_host("createTab", &[Value::Str(tab.to_string())])
                .unwrap();
        }

        assert_eq!(state.lock().unwrap().tabs.len(), 3);
    }
}
