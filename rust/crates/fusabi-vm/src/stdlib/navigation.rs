//! Navigation and Keymap Configuration APIs for Scarab Integration
//!
//! Provides stdlib functions for:
//! - Setting/getting navigation keymaps (Vimium, Cosmos, Spacemacs, custom)
//! - Registering focusable elements with capability limits
//! - Invoking navigation actions (hint mode, jump to anchor)
//! - Safety quotas and rate limiting

use crate::value::Value;
use crate::vm::VmError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Navigation keymap styles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeymapStyle {
    Vimium,
    Cosmos,
    Spacemacs,
    Custom(String),
}

impl KeymapStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "vimium" => KeymapStyle::Vimium,
            "cosmos" => KeymapStyle::Cosmos,
            "spacemacs" => KeymapStyle::Spacemacs,
            _ => KeymapStyle::Custom(s.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            KeymapStyle::Vimium => "vimium".to_string(),
            KeymapStyle::Cosmos => "cosmos".to_string(),
            KeymapStyle::Spacemacs => "spacemacs".to_string(),
            KeymapStyle::Custom(s) => s.clone(),
        }
    }
}

/// Focusable element registered by a script
#[derive(Debug, Clone)]
pub struct Focusable {
    pub id: String,
    pub label: String,
    pub hint: Option<String>,
    pub bounds: Option<(i32, i32, i32, i32)>, // x, y, width, height
}

/// Navigation capability limits for safety
#[derive(Debug, Clone)]
pub struct NavigationLimits {
    pub max_focusables: usize,
    pub max_actions_per_second: usize,
    pub max_hint_length: usize,
}

impl Default for NavigationLimits {
    fn default() -> Self {
        Self {
            max_focusables: 1000,
            max_actions_per_second: 60,
            max_hint_length: 32,
        }
    }
}

/// Rate limiter for navigation actions
struct RateLimiter {
    actions: Vec<Instant>,
    limit: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(limit: usize) -> Self {
        Self {
            actions: Vec::new(),
            limit,
            window: Duration::from_secs(1),
        }
    }

    fn check(&mut self) -> bool {
        let now = Instant::now();
        self.actions
            .retain(|t| now.duration_since(*t) < self.window);
        if self.actions.len() >= self.limit {
            false
        } else {
            self.actions.push(now);
            true
        }
    }
}

lazy_static::lazy_static! {
    /// Global navigation state (thread-safe)
    static ref NAV_STATE: RwLock<NavigationState> = RwLock::new(NavigationState::new());
}

struct NavigationState {
    keymap: KeymapStyle,
    focusables: HashMap<String, Focusable>,
    limits: NavigationLimits,
    rate_limiter: Mutex<RateLimiter>,
    hint_mode_active: bool,
    current_anchor: Option<String>,
}

impl NavigationState {
    fn new() -> Self {
        let limits = NavigationLimits::default();
        Self {
            keymap: KeymapStyle::Vimium,
            focusables: HashMap::new(),
            rate_limiter: Mutex::new(RateLimiter::new(limits.max_actions_per_second)),
            limits,
            hint_mode_active: false,
            current_anchor: None,
        }
    }
}

// ============ Stdlib Functions ============

/// Nav.getKeymap() -> string
/// Returns the current navigation keymap style
pub fn nav_get_keymap(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    Ok(Value::Str(state.keymap.to_string()))
}

/// Nav.setKeymap(style: string) -> unit
/// Sets the navigation keymap style (vimium, cosmos, spacemacs, or custom name)
pub fn nav_set_keymap(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::Runtime(
            "Nav.setKeymap requires a style argument".into(),
        ));
    }

    let style = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    state.keymap = KeymapStyle::from_str(style);

    Ok(Value::Unit)
}

/// Nav.registerFocusable(id: string, label: string) -> Result<unit, string>
/// Registers a focusable element with the navigation system
pub fn nav_register_focusable(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() < 2 {
        return Err(VmError::Runtime(
            "Nav.registerFocusable requires id and label".into(),
        ));
    }

    let id = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let label = args[1].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[1].type_name(),
    })?;

    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    // Check quota
    if state.focusables.len() >= state.limits.max_focusables {
        return Ok(Value::Variant {
            type_name: "Result".to_string(),
            variant_name: "Error".to_string(),
            fields: vec![Value::Str(format!(
                "Focusable limit exceeded (max: {})",
                state.limits.max_focusables
            ))],
        });
    }

    let focusable = Focusable {
        id: id.to_string(),
        label: label.to_string(),
        hint: None,
        bounds: None,
    };

    state.focusables.insert(id.to_string(), focusable);

    Ok(Value::Variant {
        type_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        fields: vec![Value::Unit],
    })
}

/// Nav.unregisterFocusable(id: string) -> bool
/// Removes a focusable element, returns true if it existed
pub fn nav_unregister_focusable(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::Runtime(
            "Nav.unregisterFocusable requires an id".into(),
        ));
    }

    let id = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    let existed = state.focusables.remove(id).is_some();

    Ok(Value::Bool(existed))
}

/// Nav.clearFocusables() -> int
/// Clears all registered focusables, returns count removed
pub fn nav_clear_focusables(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    let count = state.focusables.len() as i64;
    state.focusables.clear();

    Ok(Value::Int(count))
}

/// Nav.getFocusableCount() -> int
/// Returns the number of registered focusables
pub fn nav_get_focusable_count(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    Ok(Value::Int(state.focusables.len() as i64))
}

/// Nav.enterHintMode() -> Result<unit, string>
/// Enters hint mode for keyboard navigation (rate-limited)
pub fn nav_enter_hint_mode(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    // Rate limit check
    {
        let mut limiter = state.rate_limiter.lock().unwrap();
        if !limiter.check() {
            return Ok(Value::Variant {
                type_name: "Result".to_string(),
                variant_name: "Error".to_string(),
                fields: vec![Value::Str("Rate limit exceeded".to_string())],
            });
        }
    }

    state.hint_mode_active = true;

    Ok(Value::Variant {
        type_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        fields: vec![Value::Unit],
    })
}

/// Nav.exitHintMode() -> unit
/// Exits hint mode
pub fn nav_exit_hint_mode(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    state.hint_mode_active = false;
    Ok(Value::Unit)
}

/// Nav.isHintModeActive() -> bool
/// Returns whether hint mode is currently active
pub fn nav_is_hint_mode_active(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    Ok(Value::Bool(state.hint_mode_active))
}

/// Nav.jumpToAnchor(anchorId: string) -> Result<unit, string>
/// Jumps to a named anchor/focusable (rate-limited)
pub fn nav_jump_to_anchor(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.is_empty() {
        return Err(VmError::Runtime(
            "Nav.jumpToAnchor requires an anchor id".into(),
        ));
    }

    let anchor_id = args[0].as_str().ok_or_else(|| VmError::TypeMismatch {
        expected: "string",
        got: args[0].type_name(),
    })?;

    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    // Rate limit check
    {
        let mut limiter = state.rate_limiter.lock().unwrap();
        if !limiter.check() {
            return Ok(Value::Variant {
                type_name: "Result".to_string(),
                variant_name: "Error".to_string(),
                fields: vec![Value::Str("Rate limit exceeded".to_string())],
            });
        }
    }

    // Check if anchor exists
    if !state.focusables.contains_key(anchor_id) {
        return Ok(Value::Variant {
            type_name: "Result".to_string(),
            variant_name: "Error".to_string(),
            fields: vec![Value::Str(format!("Anchor not found: {}", anchor_id))],
        });
    }

    state.current_anchor = Some(anchor_id.to_string());

    Ok(Value::Variant {
        type_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        fields: vec![Value::Unit],
    })
}

/// Nav.getCurrentAnchor() -> Option<string>
/// Returns the current anchor/focusable id if any
pub fn nav_get_current_anchor(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    match &state.current_anchor {
        Some(anchor) => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            fields: vec![Value::Str(anchor.clone())],
        }),
        None => Ok(Value::Variant {
            type_name: "Option".to_string(),
            variant_name: "None".to_string(),
            fields: vec![],
        }),
    }
}

/// Nav.getLimits() -> { maxFocusables: int, maxActionsPerSecond: int }
/// Returns the current navigation capability limits
pub fn nav_get_limits(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    let mut fields = HashMap::new();
    fields.insert(
        "maxFocusables".to_string(),
        Value::Int(state.limits.max_focusables as i64),
    );
    fields.insert(
        "maxActionsPerSecond".to_string(),
        Value::Int(state.limits.max_actions_per_second as i64),
    );
    fields.insert(
        "maxHintLength".to_string(),
        Value::Int(state.limits.max_hint_length as i64),
    );

    Ok(Value::Record(Arc::new(std::sync::Mutex::new(fields))))
}

/// Nav.setLimits(maxFocusables: int, maxActionsPerSecond: int) -> unit
/// Sets custom navigation limits (for host configuration)
pub fn nav_set_limits(_vm: &mut crate::vm::Vm, args: &[Value]) -> Result<Value, VmError> {
    if args.len() < 2 {
        return Err(VmError::Runtime(
            "Nav.setLimits requires maxFocusables and maxActionsPerSecond".into(),
        ));
    }

    let max_focusables = match &args[0] {
        Value::Int(n) => *n as usize,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: args[0].type_name(),
            })
        }
    };

    let max_actions = match &args[1] {
        Value::Int(n) => *n as usize,
        _ => {
            return Err(VmError::TypeMismatch {
                expected: "int",
                got: args[1].type_name(),
            })
        }
    };

    let mut state = NAV_STATE
        .write()
        .map_err(|e| VmError::Runtime(e.to_string()))?;
    state.limits.max_focusables = max_focusables;
    state.limits.max_actions_per_second = max_actions;

    // Update rate limiter
    *state.rate_limiter.lock().unwrap() = RateLimiter::new(max_actions);

    Ok(Value::Unit)
}

/// Nav.listFocusables() -> list<{id: string, label: string}>
/// Returns all registered focusables as a list of records
pub fn nav_list_focusables(_vm: &mut crate::vm::Vm, _args: &[Value]) -> Result<Value, VmError> {
    let state = NAV_STATE
        .read()
        .map_err(|e| VmError::Runtime(e.to_string()))?;

    let focusables: Vec<Value> = state
        .focusables
        .values()
        .map(|f| {
            let mut fields = HashMap::new();
            fields.insert("id".to_string(), Value::Str(f.id.clone()));
            fields.insert("label".to_string(), Value::Str(f.label.clone()));
            Value::Record(Arc::new(std::sync::Mutex::new(fields)))
        })
        .collect();

    Ok(Value::vec_to_cons(focusables))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keymap_style_from_str() {
        assert_eq!(KeymapStyle::from_str("vimium"), KeymapStyle::Vimium);
        assert_eq!(KeymapStyle::from_str("COSMOS"), KeymapStyle::Cosmos);
        assert_eq!(KeymapStyle::from_str("spacemacs"), KeymapStyle::Spacemacs);
        assert_eq!(
            KeymapStyle::from_str("custom"),
            KeymapStyle::Custom("custom".to_string())
        );
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3);
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(limiter.check());
        assert!(!limiter.check()); // 4th should fail
    }

    #[test]
    fn test_navigation_limits_default() {
        let limits = NavigationLimits::default();
        assert_eq!(limits.max_focusables, 1000);
        assert_eq!(limits.max_actions_per_second, 60);
    }
}
