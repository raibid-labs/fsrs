# Issue #001: Core AST Definitions

## Overview
Define the core Abstract Syntax Tree (AST) data structures for the Mini-F# dialect. This is the foundational type system that represents parsed F# code in memory before compilation to bytecode.

## Labels
- `feature`
- `phase-1: mvp`
- `priority: critical`
- `foundational`
- `parallel-safe`
- `component: frontend`
- `effort: s` (1-2 days)

## Milestone
Phase 1.1: Frontend Foundation (Week 1)

## Dependencies
None - This is a foundational issue

## Acceptance Criteria
- [ ] `Literal` enum defined with int, float, bool, string, unit variants
- [ ] `BinOp` enum defined with arithmetic and comparison operators
- [ ] `Expr` enum defined with all Phase 1 expression types
- [ ] All types implement `Debug`, `Clone`, and include doc comments
- [ ] Unit tests for AST construction
- [ ] Documentation examples showing AST for sample F# code

## Technical Specification

### File Location
`rust/crates/fsrs-frontend/src/ast.rs`

### Core Types

```rust
// rust/crates/fsrs-frontend/src/ast.rs

/// Literal values in the AST
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div,
    // Comparison
    Eq, Neq, Lt, Lte, Gt, Gte,
    // Logical
    And, Or,
}

/// Core expression types (Phase 1 subset)
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference: `x`
    Var(String),

    /// Literal value: `42`, `true`, `"hello"`
    Lit(Literal),

    /// Binary operation: `x + y`, `a > b`
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Let binding (non-recursive): `let x = expr in body`
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Function definition: `fun x -> body`
    Lambda {
        param: String,
        body: Box<Expr>,
    },

    /// Function application: `f x`
    App {
        func: Box<Expr>,
        arg: Box<Expr>,
    },

    /// Conditional: `if cond then t_branch else f_branch`
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
}

/// Position information for error reporting (optional for Phase 1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

## Implementation Details

### Step 1: Create ast.rs Module

```rust
// rust/crates/fsrs-frontend/src/lib.rs
pub mod ast;

// Optionally re-export for convenience
pub use ast::{Expr, Literal, BinOp};
```

### Step 2: Add Utility Methods

```rust
impl Expr {
    /// Check if expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Lit(_))
    }

    /// Check if expression is a variable
    pub fn is_var(&self) -> bool {
        matches!(self, Expr::Var(_))
    }
}

impl BinOp {
    /// Check if operator is arithmetic
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div)
    }

    /// Check if operator is comparison
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte
        )
    }
}
```

### Step 3: Add Display Implementations (Optional)

```rust
impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "=",
            BinOp::Neq => "<>",
            BinOp::Lt => "<",
            BinOp::Lte => "<=",
            BinOp::Gt => ">",
            BinOp::Gte => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        };
        write!(f, "{}", s)
    }
}
```

## Testing Requirements

### Unit Tests

```rust
// rust/crates/fsrs-frontend/src/ast.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_int() {
        let lit = Literal::Int(42);
        assert_eq!(lit, Literal::Int(42));
    }

    #[test]
    fn test_simple_binop() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        };
        assert!(matches!(expr, Expr::BinOp { .. }));
    }

    #[test]
    fn test_let_binding() {
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(10))),
            body: Box::new(Expr::Var("x".to_string())),
        };
        assert!(matches!(expr, Expr::Let { .. }));
    }

    #[test]
    fn test_if_expression() {
        let expr = Expr::If {
            cond: Box::new(Expr::Var("flag".to_string())),
            then_branch: Box::new(Expr::Lit(Literal::Int(1))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };
        assert!(matches!(expr, Expr::If { .. }));
    }

    #[test]
    fn test_lambda_and_app() {
        // (fun x -> x + 1) 5
        let lambda = Expr::Lambda {
            param: "x".to_string(),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(1))),
            }),
        };

        let app = Expr::App {
            func: Box::new(lambda),
            arg: Box::new(Expr::Lit(Literal::Int(5))),
        };

        assert!(matches!(app, Expr::App { .. }));
    }
}
```

### Manual Testing
- [ ] Create sample AST structures manually in tests
- [ ] Verify Clone and Debug implementations work
- [ ] Ensure all variants can be constructed

## Documentation

### Doc Comments Example

```rust
/// Represents an expression in the Mini-F# AST.
///
/// # Examples
///
/// Creating a simple addition expression:
/// ```
/// use fsrs_frontend::ast::{Expr, BinOp, Literal};
///
/// let expr = Expr::BinOp {
///     op: BinOp::Add,
///     left: Box::new(Expr::Lit(Literal::Int(1))),
///     right: Box::new(Expr::Lit(Literal::Int(2))),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ...
}
```

### Files to Create/Update
- [x] `rust/crates/fsrs-frontend/src/ast.rs` - Main AST definitions
- [x] `rust/crates/fsrs-frontend/src/lib.rs` - Export ast module
- [ ] `docs/architecture/ast-design.md` - AST design doc (optional)

## Estimated Effort
**1-2 days** (Small)

### Breakdown:
- Define core types: 2-3 hours
- Add utility methods: 1 hour
- Write unit tests: 2-3 hours
- Documentation and doc comments: 1-2 hours

## Related Issues
- Blocks #003 (Parser) - Parser needs AST types to construct
- Used by #007 (Compiler) - Compiler transforms AST to bytecode

## Notes

### Design Decisions
- **Box<Expr>** for recursive types to avoid infinite size
- **Simple enums** without position info for Phase 1 (add Span later)
- **Non-recursive Let** for Phase 1 (let-rec comes in Phase 2)
- **Single-parameter Lambda** (currying for multi-arg functions)

### Future Extensions (Phase 2+)
- Add `Span` or `SourceLocation` to each variant
- Add `LetRec` for recursive bindings
- Add `Match` for pattern matching
- Add `Tuple`, `List`, `Record`, `Variant` constructors

### Parallel Work Opportunity
✅ **PARALLEL-SAFE**: Can be developed independently. Only exports types, no file conflicts.

### Critical Path
⚠️ **FOUNDATIONAL**: Parser (#003) and Compiler (#007) depend on these types.

### Success Metrics
- All unit tests pass
- No clippy warnings
- Clear documentation
- Can manually construct any Phase 1 expression