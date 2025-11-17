# Issue #004: VM Value Representation

## Overview
Define the runtime value representation for the FSRS bytecode VM. This includes the `Value` enum and associated types that represent all runtime values.

## Labels
- `feature`, `phase-1: mvp`, `priority: high`, `foundational`, `parallel-safe`, `component: vm`, `effort: s` (1-2 days)

## Milestone
Phase 1.2: VM Foundation (Week 2)

## Dependencies
None - Can work in parallel with frontend

## Acceptance Criteria
- [ ] `Value` enum with Phase 1 types (Int, Bool, Str, Unit)
- [ ] Memory-efficient representation
- [ ] Debug and Display implementations
- [ ] Type checking helper methods
- [ ] 20+ unit tests for value operations

## Technical Specification

```rust
// rust/crates/fsrs-vm/src/value.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
            Value::Str(_) => "string",
            Value::Unit => "unit",
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) => false,
            Value::Unit => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
            Value::Unit => write!(f, "()"),
        }
    }
}
```

## Estimated Effort
**1-2 days**

## Related Issues
- Used by #006 (VM Interpreter)
- Parallel with #001-#003 (Frontend)