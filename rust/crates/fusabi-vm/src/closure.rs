// Fusabi VM Closure Support
// Implements closures with upvalue capturing for lexical scoping

use crate::chunk::Chunk;
use crate::value::Value;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// Upvalue - represents a captured variable from an enclosing scope
///
/// Upvalues can be in two states:
/// - Open: Points to a stack slot (variable still on stack)
/// - Closed: Contains the actual value (variable has been moved off stack)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Upvalue {
    /// Open upvalue - points to a stack location
    Open(usize),
    /// Closed upvalue - contains the captured value
    Closed(Value),
}

impl Upvalue {
    /// Create a new open upvalue pointing to a stack location
    pub fn new_open(stack_index: usize) -> Self {
        Upvalue::Open(stack_index)
    }

    /// Create a new closed upvalue containing a value
    pub fn new_closed(value: Value) -> Self {
        Upvalue::Closed(value)
    }

    /// Check if the upvalue is open
    pub fn is_open(&self) -> bool {
        matches!(self, Upvalue::Open(_))
    }

    /// Check if the upvalue is closed
    pub fn is_closed(&self) -> bool {
        matches!(self, Upvalue::Closed(_))
    }

    /// Get the stack index if open, None otherwise
    pub fn stack_index(&self) -> Option<usize> {
        match self {
            Upvalue::Open(idx) => Some(*idx),
            Upvalue::Closed(_) => None,
        }
    }

    /// Close the upvalue by capturing the given value
    pub fn close(&mut self, value: Value) {
        *self = Upvalue::Closed(value);
    }
}

impl PartialEq for Upvalue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Upvalue::Open(a), Upvalue::Open(b)) => a == b,
            (Upvalue::Closed(a), Upvalue::Closed(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Upvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Upvalue::Open(idx) => write!(f, "<upvalue @{}>", idx),
            Upvalue::Closed(val) => write!(f, "<upvalue: {}>", val),
        }
    }
}

/// Closure - a function with captured variables (upvalues)
///
/// A closure combines a function's bytecode chunk with the values
/// it has captured from enclosing scopes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Closure {
    /// The function's bytecode chunk
    pub chunk: Chunk,
    /// Captured upvalues from enclosing scopes
    #[cfg_attr(feature = "serde", serde(skip))] // Runtime state, not serialized for now
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
    /// Number of parameters the function expects
    pub arity: u8,
    /// Function name (for debugging)
    pub name: Option<String>,
}

impl Closure {
    /// Create a new closure from a chunk
    pub fn new(chunk: Chunk) -> Self {
        Closure {
            chunk,
            upvalues: Vec::new(),
            arity: 0,
            name: None,
        }
    }

    /// Create a new closure with a specific arity
    pub fn with_arity(chunk: Chunk, arity: u8) -> Self {
        Closure {
            chunk,
            upvalues: Vec::new(),
            arity,
            name: None,
        }
    }

    /// Create a new closure with a name
    pub fn with_name(chunk: Chunk, name: String) -> Self {
        Closure {
            chunk,
            upvalues: Vec::new(),
            arity: 0,
            name: Some(name),
        }
    }

    /// Create a new closure with arity and name
    pub fn with_arity_and_name(chunk: Chunk, arity: u8, name: String) -> Self {
        Closure {
            chunk,
            upvalues: Vec::new(),
            arity,
            name: Some(name),
        }
    }

    /// Add an upvalue to this closure
    pub fn add_upvalue(&mut self, upvalue: Rc<RefCell<Upvalue>>) {
        self.upvalues.push(upvalue);
    }

    /// Get an upvalue by index
    pub fn get_upvalue(&self, index: usize) -> Option<Rc<RefCell<Upvalue>>> {
        self.upvalues.get(index).cloned()
    }

    /// Get the number of upvalues
    pub fn upvalue_count(&self) -> usize {
        self.upvalues.len()
    }
}

impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        // Two closures are equal if they have the same chunk and arity
        // Note: We don't compare upvalues as they may have different identity
        self.arity == other.arity && self.name == other.name
    }
}

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "<closure {}>", name)
        } else {
            write!(f, "<closure>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkBuilder;

    // ========== Upvalue Tests ==========

    #[test]
    fn test_upvalue_new_open() {
        let upvalue = Upvalue::new_open(5);
        assert!(upvalue.is_open());
        assert!(!upvalue.is_closed());
        assert_eq!(upvalue.stack_index(), Some(5));
    }

    #[test]
    fn test_upvalue_new_closed() {
        let upvalue = Upvalue::new_closed(Value::Int(42));
        assert!(upvalue.is_closed());
        assert!(!upvalue.is_open());
        assert_eq!(upvalue.stack_index(), None);
    }

    #[test]
    fn test_upvalue_close() {
        let mut upvalue = Upvalue::new_open(10);
        assert!(upvalue.is_open());

        upvalue.close(Value::Int(42));
        assert!(upvalue.is_closed());
        assert_eq!(upvalue, Upvalue::Closed(Value::Int(42)));
    }

    #[test]
    fn test_upvalue_equality() {
        let uv1 = Upvalue::new_open(5);
        let uv2 = Upvalue::new_open(5);
        let uv3 = Upvalue::new_open(6);

        assert_eq!(uv1, uv2);
        assert_ne!(uv1, uv3);

        let uv4 = Upvalue::new_closed(Value::Int(42));
        let uv5 = Upvalue::new_closed(Value::Int(42));
        let uv6 = Upvalue::new_closed(Value::Int(43));

        assert_eq!(uv4, uv5);
        assert_ne!(uv4, uv6);
        assert_ne!(uv1, uv4);
    }

    #[test]
    fn test_upvalue_display() {
        let uv_open = Upvalue::new_open(5);
        assert_eq!(format!("{}", uv_open), "<upvalue @5>");

        let uv_closed = Upvalue::new_closed(Value::Int(42));
        assert_eq!(format!("{}", uv_closed), "<upvalue: 42>");
    }

    // ========== Closure Construction Tests ==========

    #[test]
    fn test_closure_new() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::new(chunk);

        assert_eq!(closure.arity, 0);
        assert_eq!(closure.name, None);
        assert_eq!(closure.upvalue_count(), 0);
    }

    #[test]
    fn test_closure_with_arity() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::with_arity(chunk, 2);

        assert_eq!(closure.arity, 2);
        assert_eq!(closure.name, None);
    }

    #[test]
    fn test_closure_with_name() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::with_name(chunk, "add".to_string());

        assert_eq!(closure.arity, 0);
        assert_eq!(closure.name, Some("add".to_string()));
    }

    #[test]
    fn test_closure_with_arity_and_name() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::with_arity_and_name(chunk, 2, "add".to_string());

        assert_eq!(closure.arity, 2);
        assert_eq!(closure.name, Some("add".to_string()));
    }

    // ========== Upvalue Management Tests ==========

    #[test]
    fn test_closure_add_upvalue() {
        let chunk = ChunkBuilder::new().build();
        let mut closure = Closure::new(chunk);

        let upvalue = Rc::new(RefCell::new(Upvalue::new_open(5)));
        closure.add_upvalue(upvalue);

        assert_eq!(closure.upvalue_count(), 1);
    }

    #[test]
    fn test_closure_get_upvalue() {
        let chunk = ChunkBuilder::new().build();
        let mut closure = Closure::new(chunk);

        let upvalue = Rc::new(RefCell::new(Upvalue::new_open(5)));
        closure.add_upvalue(upvalue.clone());

        let retrieved = closure.get_upvalue(0).unwrap();
        assert_eq!(*retrieved.borrow(), Upvalue::new_open(5));
    }

    #[test]
    fn test_closure_get_upvalue_out_of_bounds() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::new(chunk);

        assert!(closure.get_upvalue(0).is_none());
        assert!(closure.get_upvalue(10).is_none());
    }

    #[test]
    fn test_closure_multiple_upvalues() {
        let chunk = ChunkBuilder::new().build();
        let mut closure = Closure::new(chunk);

        let uv1 = Rc::new(RefCell::new(Upvalue::new_open(1)));
        let uv2 = Rc::new(RefCell::new(Upvalue::new_open(2)));
        let uv3 = Rc::new(RefCell::new(Upvalue::new_closed(Value::Int(42))));

        closure.add_upvalue(uv1);
        closure.add_upvalue(uv2);
        closure.add_upvalue(uv3);

        assert_eq!(closure.upvalue_count(), 3);
        assert_eq!(
            *closure.get_upvalue(0).unwrap().borrow(),
            Upvalue::new_open(1)
        );
        assert_eq!(
            *closure.get_upvalue(1).unwrap().borrow(),
            Upvalue::new_open(2)
        );
        assert_eq!(
            *closure.get_upvalue(2).unwrap().borrow(),
            Upvalue::new_closed(Value::Int(42))
        );
    }

    // ========== Closure Equality Tests ==========

    #[test]
    fn test_closure_equality() {
        let chunk1 = ChunkBuilder::new().build();
        let chunk2 = ChunkBuilder::new().build();

        let closure1 = Closure::with_arity_and_name(chunk1, 2, "add".to_string());
        let closure2 = Closure::with_arity_and_name(chunk2, 2, "add".to_string());
        let closure3 =
            Closure::with_arity_and_name(ChunkBuilder::new().build(), 3, "add".to_string());

        assert_eq!(closure1, closure2);
        assert_ne!(closure1, closure3);
    }

    // ========== Display Tests ==========

    #[test]
    fn test_closure_display_unnamed() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::new(chunk);

        assert_eq!(format!("{}", closure), "<closure>");
    }

    #[test]
    fn test_closure_display_named() {
        let chunk = ChunkBuilder::new().build();
        let closure = Closure::with_name(chunk, "fibonacci".to_string());

        assert_eq!(format!("{}", closure), "<closure fibonacci>");
    }
}
