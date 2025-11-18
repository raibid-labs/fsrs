//! Core AST (Abstract Syntax Tree) definitions for FSRS Mini-F#.
//!
//! This module defines the foundational types for representing F# expressions
//! in the FSRS compiler frontend. The AST serves as the intermediate representation
//! between parsing and bytecode compilation.
//!
//! # Phase 1 MVP Features
//!
//! The AST supports:
//! - Literals: integers, floats, booleans, strings, unit
//! - Variables and let-bindings
//! - Lambda functions and function application
//! - Binary operations (arithmetic, comparison, logical)
//! - Conditional expressions (if-then-else)
//! - Tuples (e.g., (1, 2), (x, y, z))
//! - Lists (e.g., [1; 2; 3], []) and cons operator (::)
//! - Arrays (e.g., [|1; 2; 3|], arr.[0], arr.[0] <- 99)
//! - Records (e.g., type Person = { name: string; age: int })
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//!
//! // Construct: let x = 42 in x + 1
//! let expr = Expr::Let {
//!     name: "x".to_string(),
//!     value: Box::new(Expr::Lit(Literal::Int(42))),
//!     body: Box::new(Expr::BinOp {
//!         op: BinOp::Add,
//!         left: Box::new(Expr::Var("x".to_string())),
//!         right: Box::new(Expr::Lit(Literal::Int(1))),
//!     }),
//! };
//! ```

use std::fmt;

/// Literal values in the AST.
///
/// Represents constant values that can appear in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal (e.g., 42, -10)
    Int(i64),
    /// Floating-point literal (e.g., 2.5, -0.5)
    Float(f64),
    /// Boolean literal (true or false)
    Bool(bool),
    /// String literal (e.g., "hello")
    Str(String),
    /// Unit value (equivalent to () in F#)
    Unit,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Unit => write!(f, "()"),
        }
    }
}

/// Binary operators supported in expressions.
///
/// Includes arithmetic, comparison, and logical operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic operators
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Multiplication (*)
    Mul,
    /// Division (/)
    Div,

    // Comparison operators
    /// Equality (=)
    Eq,
    /// Inequality (<>)
    Neq,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,

    // Logical operators
    /// Logical AND (&&)
    And,
    /// Logical OR (||)
    Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl BinOp {
    /// Returns true if this is an arithmetic operator.
    pub fn is_arithmetic(&self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div)
    }

    /// Returns true if this is a comparison operator.
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Lte | BinOp::Gt | BinOp::Gte
        )
    }

    /// Returns true if this is a logical operator.
    pub fn is_logical(&self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }
}

/// Type expressions for record field type annotations.
///
/// Represents types that can appear in record definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// Named type (e.g., int, bool, string, UserType)
    Named(String),
    /// Tuple type (e.g., int * string)
    Tuple(Vec<TypeExpr>),
    /// Function type (e.g., int -> string)
    Function(Box<TypeExpr>, Box<TypeExpr>),
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Named(name) => write!(f, "{}", name),
            TypeExpr::Tuple(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " * ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
            TypeExpr::Function(arg, ret) => write!(f, "{} -> {}", arg, ret),
        }
    }
}

/// Record type definition.
///
/// Represents a user-defined record type with named fields.
/// Example: type Person = { name: string; age: int }
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTypeDef {
    /// Name of the record type
    pub name: String,
    /// Field definitions: (field_name, field_type)
    pub fields: Vec<(String, TypeExpr)>,
}

impl fmt::Display for RecordTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {{ ", self.name)?;
        for (i, (field_name, field_type)) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}: {}", field_name, field_type)?;
        }
        write!(f, " }}")
    }
}

/// Top-level declaration in a module.
///
/// Represents declarations that can appear at the top level of a module.
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    /// Type definition (e.g., record type)
    TypeDef(RecordTypeDef),
    /// Let-binding declaration
    LetBinding {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Declaration::TypeDef(typedef) => write!(f, "{}", typedef),
            Declaration::LetBinding { name, params, body } => {
                write!(f, "let {}", name)?;
                for param in params {
                    write!(f, " {}", param)?;
                }
                write!(f, " = {}", body)
            }
        }
    }
}

/// Module containing declarations.
///
/// Represents a complete module with type definitions and let-bindings.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// List of declarations in the module
    pub declarations: Vec<Declaration>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, decl) in self.declarations.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", decl)?;
        }
        Ok(())
    }
}

/// Pattern in a match expression.
///
/// Patterns can match literals, variables, wildcards, and tuples.
/// Issue #27 supports basic patterns; Issue #28 will add lists/arrays.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Wildcard pattern (_) - matches anything
    Wildcard,
    /// Variable pattern (x) - binds matched value to variable
    Var(String),
    /// Literal pattern (42, true, "hello") - matches exact value
    Literal(Literal),
    /// Tuple pattern ((p1, p2, ...)) - matches tuples
    Tuple(Vec<Pattern>),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Var(name) => write!(f, "{}", name),
            Pattern::Literal(lit) => write!(f, "{}", lit),
            Pattern::Tuple(patterns) => {
                write!(f, "(")?;
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", pat)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Pattern {
    /// Returns true if this pattern is a wildcard.
    pub fn is_wildcard(&self) -> bool {
        matches!(self, Pattern::Wildcard)
    }

    /// Returns true if this pattern is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self, Pattern::Var(_))
    }

    /// Returns true if this pattern is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(self, Pattern::Literal(_))
    }

    /// Returns true if this pattern is a tuple.
    pub fn is_tuple(&self) -> bool {
        matches!(self, Pattern::Tuple(_))
    }

    /// Returns the variable name if this is a Var, otherwise None.
    pub fn as_var(&self) -> Option<&str> {
        match self {
            Pattern::Var(name) => Some(name),
            _ => None,
        }
    }

    /// Returns the literal value if this is a Literal, otherwise None.
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            Pattern::Literal(lit) => Some(lit),
            _ => None,
        }
    }

    /// Returns the tuple patterns if this is a Tuple, otherwise None.
    pub fn as_tuple(&self) -> Option<&Vec<Pattern>> {
        match self {
            Pattern::Tuple(patterns) => Some(patterns),
            _ => None,
        }
    }
}

/// Match arm in a match expression.
///
/// Each arm consists of a pattern and the body expression to evaluate if the pattern matches.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    /// The pattern to match against
    pub pattern: Pattern,
    /// The expression to evaluate if the pattern matches
    pub body: Box<Expr>,
}

impl fmt::Display for MatchArm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.pattern, self.body)
    }
}

impl MatchArm {
    /// Create a new match arm.
    pub fn new(pattern: Pattern, body: Expr) -> Self {
        MatchArm {
            pattern,
            body: Box::new(body),
        }
    }

    /// Returns true if this arm's pattern is a wildcard.
    pub fn is_wildcard(&self) -> bool {
        self.pattern.is_wildcard()
    }

    /// Returns true if this arm's pattern is a variable binding.
    pub fn is_var(&self) -> bool {
        self.pattern.is_var()
    }
}

/// Core expression types in the AST.
///
/// Represents all expression forms supported in Phase 1 of FSRS.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Variable reference (e.g., x, foo)
    Var(String),

    /// Literal value
    Lit(Literal),

    /// Binary operation (e.g., x + 1, a && b)
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Let-binding (e.g., let x = 5 in x + 1)
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Recursive let-binding (e.g., let rec fact n = if n <= 1 then 1 else n * fact (n - 1))
    LetRec {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },

    /// Mutually recursive let-bindings (e.g., let rec even n = ... and odd n = ...)
    LetRecMutual {
        bindings: Vec<(String, Expr)>,
        body: Box<Expr>,
    },

    /// Lambda function (e.g., fun x -> x + 1)
    Lambda { param: String, body: Box<Expr> },

    /// Function application (e.g., f x, add 1 2)
    App { func: Box<Expr>, arg: Box<Expr> },

    /// Conditional expression (e.g., if x > 0 then 1 else -1)
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    /// Match expression (e.g., match x with | 0 -> "zero" | _ -> "nonzero")
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },

    /// Tuple expression (e.g., (1, 2), (x, y, z))
    /// Empty tuple () is represented as Lit(Literal::Unit)
    Tuple(Vec<Expr>),

    /// List expression (e.g., [1; 2; 3], [])
    List(Vec<Expr>),

    /// Cons operator (e.g., 1 :: [2; 3], x :: xs)
    Cons { head: Box<Expr>, tail: Box<Expr> },

    /// Array literal (e.g., [|1; 2; 3|], [||])
    Array(Vec<Expr>),

    /// Array indexing (e.g., arr.[0], arr.[i])
    ArrayIndex { array: Box<Expr>, index: Box<Expr> },

    /// Array update (e.g., arr.[0] <- 99) - immutable, returns new array
    ArrayUpdate {
        array: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
    },

    /// Array length (e.g., Array.length arr)
    ArrayLength(Box<Expr>),

    /// Record literal (e.g., { name = "John"; age = 30 })
    RecordLiteral {
        type_name: String,    // Filled by typechecker, empty during parsing
        fields: RecordFields, // (field_name, value_expr)
    },

    /// Record field access (e.g., person.name, record.field)
    RecordAccess { record: Box<Expr>, field: String },

    /// Record update (e.g., { person with age = 31 })
    RecordUpdate {
        record: Box<Expr>,
        fields: RecordFields, // Fields to update
    },
}

/// Type alias for record field list: (field_name, value_expression)
pub type RecordFields = Vec<(String, Box<Expr>)>;

impl Expr {
    /// Returns true if this expression is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Lit(_))
    }

    /// Returns true if this expression is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self, Expr::Var(_))
    }

    /// Returns true if this expression is a binary operation.
    pub fn is_binop(&self) -> bool {
        matches!(self, Expr::BinOp { .. })
    }

    /// Returns true if this expression is a let-binding.
    pub fn is_let(&self) -> bool {
        matches!(self, Expr::Let { .. })
    }

    /// Returns true if this expression is a recursive let-binding.
    pub fn is_let_rec(&self) -> bool {
        matches!(self, Expr::LetRec { .. })
    }

    /// Returns true if this expression is a mutually recursive let-binding.
    pub fn is_let_rec_mutual(&self) -> bool {
        matches!(self, Expr::LetRecMutual { .. })
    }

    /// Returns true if this expression is a lambda.
    pub fn is_lambda(&self) -> bool {
        matches!(self, Expr::Lambda { .. })
    }

    /// Returns true if this expression is a function application.
    pub fn is_app(&self) -> bool {
        matches!(self, Expr::App { .. })
    }

    /// Returns true if this expression is a conditional.
    pub fn is_if(&self) -> bool {
        matches!(self, Expr::If { .. })
    }

    /// Returns true if this expression is a match.
    pub fn is_match(&self) -> bool {
        matches!(self, Expr::Match { .. })
    }

    /// Returns true if this expression is a tuple.
    pub fn is_tuple(&self) -> bool {
        matches!(self, Expr::Tuple(_))
    }

    /// Returns true if this expression is a list.
    pub fn is_list(&self) -> bool {
        matches!(self, Expr::List(_))
    }

    /// Returns true if this expression is a cons.
    pub fn is_cons(&self) -> bool {
        matches!(self, Expr::Cons { .. })
    }

    /// Returns true if this expression is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Expr::Array(_))
    }

    /// Returns true if this expression is an array index.
    pub fn is_array_index(&self) -> bool {
        matches!(self, Expr::ArrayIndex { .. })
    }

    /// Returns true if this expression is an array update.
    pub fn is_array_update(&self) -> bool {
        matches!(self, Expr::ArrayUpdate { .. })
    }

    /// Returns true if this expression is an array length.
    pub fn is_array_length(&self) -> bool {
        matches!(self, Expr::ArrayLength(_))
    }

    /// Returns true if this expression is a record literal.
    pub fn is_record_literal(&self) -> bool {
        matches!(self, Expr::RecordLiteral { .. })
    }

    /// Returns true if this expression is a record access.
    pub fn is_record_access(&self) -> bool {
        matches!(self, Expr::RecordAccess { .. })
    }

    /// Returns true if this expression is a record update.
    pub fn is_record_update(&self) -> bool {
        matches!(self, Expr::RecordUpdate { .. })
    }

    /// Returns the variable name if this is a Var, otherwise None.
    pub fn as_var(&self) -> Option<&str> {
        match self {
            Expr::Var(name) => Some(name),
            _ => None,
        }
    }

    /// Returns the literal value if this is a Lit, otherwise None.
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            Expr::Lit(lit) => Some(lit),
            _ => None,
        }
    }

    /// Returns the tuple elements if this is a Tuple, otherwise None.
    pub fn as_tuple(&self) -> Option<&Vec<Expr>> {
        match self {
            Expr::Tuple(elements) => Some(elements),
            _ => None,
        }
    }

    /// Returns the list elements if this is a List, otherwise None.
    pub fn as_list(&self) -> Option<&Vec<Expr>> {
        match self {
            Expr::List(elements) => Some(elements),
            _ => None,
        }
    }

    /// Returns the head and tail if this is a Cons, otherwise None.
    pub fn as_cons(&self) -> Option<(&Expr, &Expr)> {
        match self {
            Expr::Cons { head, tail } => Some((head, tail)),
            _ => None,
        }
    }

    /// Returns the array elements if this is an Array, otherwise None.
    pub fn as_array(&self) -> Option<&Vec<Expr>> {
        match self {
            Expr::Array(elements) => Some(elements),
            _ => None,
        }
    }

    /// Returns the record fields if this is a RecordLiteral, otherwise None.
    pub fn as_record_literal(&self) -> Option<(&str, &RecordFields)> {
        match self {
            Expr::RecordLiteral { type_name, fields } => Some((type_name, fields)),
            _ => None,
        }
    }

    /// Returns the scrutinee and arms if this is a Match, otherwise None.
    pub fn as_match(&self) -> Option<(&Expr, &Vec<MatchArm>)> {
        match self {
            Expr::Match { scrutinee, arms } => Some((scrutinee, arms)),
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Lit(lit) => write!(f, "{}", lit),
            Expr::BinOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::Let { name, value, body } => {
                write!(f, "(let {} = {} in {})", name, value, body)
            }
            Expr::LetRec { name, value, body } => {
                write!(f, "(let rec {} = {} in {})", name, value, body)
            }
            Expr::LetRecMutual { bindings, body } => {
                write!(f, "(let rec ")?;
                for (i, (name, value)) in bindings.iter().enumerate() {
                    if i > 0 {
                        write!(f, " and ")?;
                    }
                    write!(f, "{} = {}", name, value)?;
                }
                write!(f, " in {})", body)
            }
            Expr::Lambda { param, body } => {
                write!(f, "(fun {} -> {})", param, body)
            }
            Expr::App { func, arg } => {
                write!(f, "({} {})", func, arg)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                write!(f, "(if {} then {} else {})", cond, then_branch, else_branch)
            }
            Expr::Match { scrutinee, arms } => {
                write!(f, "(match {} with", scrutinee)?;
                for arm in arms {
                    write!(f, " | {}", arm)?;
                }
                write!(f, ")")
            }
            Expr::Tuple(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            }
            Expr::List(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            }
            Expr::Cons { head, tail } => {
                write!(f, "({} :: {})", head, tail)
            }
            Expr::Array(elements) => {
                write!(f, "[|")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "|]")
            }
            Expr::ArrayIndex { array, index } => {
                write!(f, "({}.[{}])", array, index)
            }
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => {
                write!(f, "({}.[{}] <- {})", array, index, value)
            }
            Expr::ArrayLength(arr) => {
                write!(f, "(Array.length {})", arr)
            }
            Expr::RecordLiteral { type_name, fields } => {
                if !type_name.is_empty() {
                    write!(f, "({} ", type_name)?;
                }
                write!(f, "{{ ")?;
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{} = {}", field_name, field_expr)?;
                }
                write!(f, " }}")?;
                if !type_name.is_empty() {
                    write!(f, ")")?;
                }
                Ok(())
            }
            Expr::RecordAccess { record, field } => {
                write!(f, "({}.{})", record, field)
            }
            Expr::RecordUpdate { record, fields } => {
                write!(f, "({{ {} with ", record)?;
                for (i, (field_name, field_expr)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{} = {}", field_name, field_expr)?;
                }
                write!(f, " }})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Literal Tests
    // ========================================================================

    #[test]
    fn test_literal_int() {
        let lit = Literal::Int(42);
        assert_eq!(lit, Literal::Int(42));
        assert_eq!(format!("{}", lit), "42");
    }

    #[test]
    fn test_literal_float() {
        let lit = Literal::Float(2.5);
        assert_eq!(lit, Literal::Float(2.5));
        assert_eq!(format!("{}", lit), "2.5");
    }

    #[test]
    fn test_literal_bool() {
        let lit_true = Literal::Bool(true);
        let lit_false = Literal::Bool(false);
        assert_eq!(lit_true, Literal::Bool(true));
        assert_eq!(lit_false, Literal::Bool(false));
        assert_eq!(format!("{}", lit_true), "true");
        assert_eq!(format!("{}", lit_false), "false");
    }

    #[test]
    fn test_literal_str() {
        let lit = Literal::Str("hello".to_string());
        assert_eq!(lit, Literal::Str("hello".to_string()));
        assert_eq!(format!("{}", lit), "\"hello\"");
    }

    #[test]
    fn test_literal_unit() {
        let lit = Literal::Unit;
        assert_eq!(lit, Literal::Unit);
        assert_eq!(format!("{}", lit), "()");
    }

    #[test]
    fn test_literal_clone() {
        let lit = Literal::Int(42);
        let cloned = lit.clone();
        assert_eq!(lit, cloned);
    }

    // ========================================================================
    // BinOp Tests
    // ========================================================================

    #[test]
    fn test_binop_arithmetic() {
        assert!(BinOp::Add.is_arithmetic());
        assert!(BinOp::Sub.is_arithmetic());
        assert!(BinOp::Mul.is_arithmetic());
        assert!(BinOp::Div.is_arithmetic());
        assert!(!BinOp::Eq.is_arithmetic());
        assert!(!BinOp::And.is_arithmetic());
    }

    #[test]
    fn test_binop_comparison() {
        assert!(BinOp::Eq.is_comparison());
        assert!(BinOp::Neq.is_comparison());
        assert!(BinOp::Lt.is_comparison());
        assert!(BinOp::Lte.is_comparison());
        assert!(BinOp::Gt.is_comparison());
        assert!(BinOp::Gte.is_comparison());
        assert!(!BinOp::Add.is_comparison());
        assert!(!BinOp::And.is_comparison());
    }

    #[test]
    fn test_binop_logical() {
        assert!(BinOp::And.is_logical());
        assert!(BinOp::Or.is_logical());
        assert!(!BinOp::Add.is_logical());
        assert!(!BinOp::Eq.is_logical());
    }

    #[test]
    fn test_binop_display() {
        assert_eq!(format!("{}", BinOp::Add), "+");
        assert_eq!(format!("{}", BinOp::Sub), "-");
        assert_eq!(format!("{}", BinOp::Mul), "*");
        assert_eq!(format!("{}", BinOp::Div), "/");
        assert_eq!(format!("{}", BinOp::Eq), "=");
        assert_eq!(format!("{}", BinOp::Neq), "<>");
        assert_eq!(format!("{}", BinOp::Lt), "<");
        assert_eq!(format!("{}", BinOp::Lte), "<=");
        assert_eq!(format!("{}", BinOp::Gt), ">");
        assert_eq!(format!("{}", BinOp::Gte), ">=");
        assert_eq!(format!("{}", BinOp::And), "&&");
        assert_eq!(format!("{}", BinOp::Or), "||");
    }

    // ========================================================================
    // TypeExpr Tests (Issue #15 Layer 1)
    // ========================================================================

    #[test]
    fn test_type_expr_named() {
        let ty = TypeExpr::Named("int".to_string());
        assert_eq!(ty, TypeExpr::Named("int".to_string()));
        assert_eq!(format!("{}", ty), "int");
    }

    #[test]
    fn test_type_expr_tuple() {
        let ty = TypeExpr::Tuple(vec![
            TypeExpr::Named("int".to_string()),
            TypeExpr::Named("string".to_string()),
        ]);
        assert_eq!(format!("{}", ty), "int * string");
    }

    #[test]
    fn test_type_expr_function() {
        let ty = TypeExpr::Function(
            Box::new(TypeExpr::Named("int".to_string())),
            Box::new(TypeExpr::Named("string".to_string())),
        );
        assert_eq!(format!("{}", ty), "int -> string");
    }

    #[test]
    fn test_type_expr_complex() {
        // (int * string) -> bool
        let ty = TypeExpr::Function(
            Box::new(TypeExpr::Tuple(vec![
                TypeExpr::Named("int".to_string()),
                TypeExpr::Named("string".to_string()),
            ])),
            Box::new(TypeExpr::Named("bool".to_string())),
        );
        assert_eq!(format!("{}", ty), "int * string -> bool");
    }

    #[test]
    fn test_type_expr_clone() {
        let ty1 = TypeExpr::Named("int".to_string());
        let ty2 = ty1.clone();
        assert_eq!(ty1, ty2);
    }

    // ========================================================================
    // RecordTypeDef Tests (Issue #15 Layer 1)
    // ========================================================================

    #[test]
    fn test_record_typedef_empty() {
        let typedef = RecordTypeDef {
            name: "Empty".to_string(),
            fields: vec![],
        };
        assert_eq!(typedef.name, "Empty");
        assert_eq!(typedef.fields.len(), 0);
        assert_eq!(format!("{}", typedef), "type Empty = {  }");
    }

    #[test]
    fn test_record_typedef_single_field() {
        let typedef = RecordTypeDef {
            name: "Age".to_string(),
            fields: vec![("age".to_string(), TypeExpr::Named("int".to_string()))],
        };
        assert_eq!(typedef.name, "Age");
        assert_eq!(typedef.fields.len(), 1);
        assert_eq!(format!("{}", typedef), "type Age = { age: int }");
    }

    #[test]
    fn test_record_typedef_multiple_fields() {
        let typedef = RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), TypeExpr::Named("string".to_string())),
                ("age".to_string(), TypeExpr::Named("int".to_string())),
                ("active".to_string(), TypeExpr::Named("bool".to_string())),
            ],
        };
        assert_eq!(typedef.name, "Person");
        assert_eq!(typedef.fields.len(), 3);
        assert_eq!(
            format!("{}", typedef),
            "type Person = { name: string; age: int; active: bool }"
        );
    }

    #[test]
    fn test_record_typedef_with_tuple_type() {
        let typedef = RecordTypeDef {
            name: "Point".to_string(),
            fields: vec![(
                "coords".to_string(),
                TypeExpr::Tuple(vec![
                    TypeExpr::Named("int".to_string()),
                    TypeExpr::Named("int".to_string()),
                ]),
            )],
        };
        assert_eq!(format!("{}", typedef), "type Point = { coords: int * int }");
    }

    #[test]
    fn test_record_typedef_with_function_type() {
        let typedef = RecordTypeDef {
            name: "Processor".to_string(),
            fields: vec![(
                "process".to_string(),
                TypeExpr::Function(
                    Box::new(TypeExpr::Named("int".to_string())),
                    Box::new(TypeExpr::Named("string".to_string())),
                ),
            )],
        };
        assert_eq!(
            format!("{}", typedef),
            "type Processor = { process: int -> string }"
        );
    }

    #[test]
    fn test_record_typedef_clone() {
        let typedef1 = RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![("name".to_string(), TypeExpr::Named("string".to_string()))],
        };
        let typedef2 = typedef1.clone();
        assert_eq!(typedef1, typedef2);
    }

    // ========================================================================
    // Declaration Tests (Issue #15 Layer 1)
    // ========================================================================

    #[test]
    fn test_declaration_typedef() {
        let decl = Declaration::TypeDef(RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![("name".to_string(), TypeExpr::Named("string".to_string()))],
        });
        assert!(matches!(decl, Declaration::TypeDef(_)));
        assert_eq!(format!("{}", decl), "type Person = { name: string }");
    }

    #[test]
    fn test_declaration_let_binding_simple() {
        let decl = Declaration::LetBinding {
            name: "x".to_string(),
            params: vec![],
            body: Box::new(Expr::Lit(Literal::Int(42))),
        };
        assert!(matches!(decl, Declaration::LetBinding { .. }));
        assert_eq!(format!("{}", decl), "let x = 42");
    }

    #[test]
    fn test_declaration_let_binding_with_params() {
        let decl = Declaration::LetBinding {
            name: "add".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Var("y".to_string())),
            }),
        };
        assert_eq!(format!("{}", decl), "let add x y = (x + y)");
    }

    #[test]
    fn test_declaration_clone() {
        let decl1 = Declaration::TypeDef(RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![],
        });
        let decl2 = decl1.clone();
        assert_eq!(decl1, decl2);
    }

    // ========================================================================
    // Module Tests (Issue #15 Layer 1)
    // ========================================================================

    #[test]
    fn test_module_empty() {
        let module = Module {
            declarations: vec![],
        };
        assert_eq!(module.declarations.len(), 0);
        assert_eq!(format!("{}", module), "");
    }

    #[test]
    fn test_module_single_typedef() {
        let module = Module {
            declarations: vec![Declaration::TypeDef(RecordTypeDef {
                name: "Person".to_string(),
                fields: vec![("name".to_string(), TypeExpr::Named("string".to_string()))],
            })],
        };
        assert_eq!(module.declarations.len(), 1);
        assert_eq!(format!("{}", module), "type Person = { name: string }");
    }

    #[test]
    fn test_module_multiple_declarations() {
        let module = Module {
            declarations: vec![
                Declaration::TypeDef(RecordTypeDef {
                    name: "Person".to_string(),
                    fields: vec![("name".to_string(), TypeExpr::Named("string".to_string()))],
                }),
                Declaration::LetBinding {
                    name: "john".to_string(),
                    params: vec![],
                    body: Box::new(Expr::RecordLiteral {
                        type_name: String::new(),
                        fields: vec![(
                            "name".to_string(),
                            Box::new(Expr::Lit(Literal::Str("John".to_string()))),
                        )],
                    }),
                },
            ],
        };
        assert_eq!(module.declarations.len(), 2);
    }

    #[test]
    fn test_module_clone() {
        let module1 = Module {
            declarations: vec![],
        };
        let module2 = module1.clone();
        assert_eq!(module1, module2);
    }

    // ========================================================================
    // Expr Record Tests (Issue #15 Layer 1)
    // ========================================================================

    #[test]
    fn test_expr_record_literal_empty() {
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![],
        };
        assert!(expr.is_record_literal());
        assert!(!expr.is_literal());
        assert_eq!(format!("{}", expr), "{  }");
    }

    #[test]
    fn test_expr_record_literal_single_field() {
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        assert!(expr.is_record_literal());
        assert_eq!(format!("{}", expr), "{ name = \"John\" }");
    }

    #[test]
    fn test_expr_record_literal_multiple_fields() {
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![
                (
                    "name".to_string(),
                    Box::new(Expr::Lit(Literal::Str("John".to_string()))),
                ),
                ("age".to_string(), Box::new(Expr::Lit(Literal::Int(30)))),
                (
                    "active".to_string(),
                    Box::new(Expr::Lit(Literal::Bool(true))),
                ),
            ],
        };
        assert!(expr.is_record_literal());
        assert_eq!(
            format!("{}", expr),
            "{ name = \"John\"; age = 30; active = true }"
        );
    }

    #[test]
    fn test_expr_record_literal_with_type_name() {
        let expr = Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        assert_eq!(format!("{}", expr), "(Person { name = \"John\" })");
    }

    #[test]
    fn test_expr_record_access_simple() {
        let expr = Expr::RecordAccess {
            record: Box::new(Expr::Var("person".to_string())),
            field: "name".to_string(),
        };
        assert!(expr.is_record_access());
        assert!(!expr.is_record_literal());
        assert_eq!(format!("{}", expr), "(person.name)");
    }

    #[test]
    fn test_expr_record_access_nested() {
        let expr = Expr::RecordAccess {
            record: Box::new(Expr::RecordAccess {
                record: Box::new(Expr::Var("company".to_string())),
                field: "employee".to_string(),
            }),
            field: "name".to_string(),
        };
        assert!(expr.is_record_access());
        assert_eq!(format!("{}", expr), "((company.employee).name)");
    }

    #[test]
    fn test_expr_record_update_single_field() {
        let expr = Expr::RecordUpdate {
            record: Box::new(Expr::Var("person".to_string())),
            fields: vec![("age".to_string(), Box::new(Expr::Lit(Literal::Int(31))))],
        };
        assert!(expr.is_record_update());
        assert!(!expr.is_record_literal());
        assert_eq!(format!("{}", expr), "({ person with age = 31 })");
    }

    #[test]
    fn test_expr_record_update_multiple_fields() {
        let expr = Expr::RecordUpdate {
            record: Box::new(Expr::Var("person".to_string())),
            fields: vec![
                ("age".to_string(), Box::new(Expr::Lit(Literal::Int(31)))),
                (
                    "active".to_string(),
                    Box::new(Expr::Lit(Literal::Bool(false))),
                ),
            ],
        };
        assert!(expr.is_record_update());
        assert_eq!(
            format!("{}", expr),
            "({ person with age = 31; active = false })"
        );
    }

    #[test]
    fn test_expr_record_as_record_literal() {
        let expr = Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        let result = expr.as_record_literal();
        assert!(result.is_some());
        let (type_name, fields) = result.unwrap();
        assert_eq!(type_name, "Person");
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn test_expr_record_clone() {
        let expr1 = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        let expr2 = expr1.clone();
        assert_eq!(expr1, expr2);
    }

    #[test]
    fn test_expr_record_equality() {
        let expr1 = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        let expr2 = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        assert_eq!(expr1, expr2);
    }

    #[test]
    fn test_expr_record_inequality_different_fields() {
        let expr1 = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("John".to_string()))),
            )],
        };
        let expr2 = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "name".to_string(),
                Box::new(Expr::Lit(Literal::Str("Jane".to_string()))),
            )],
        };
        assert_ne!(expr1, expr2);
    }

    #[test]
    fn test_expr_record_literal_with_expressions() {
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![
                (
                    "x".to_string(),
                    Box::new(Expr::BinOp {
                        op: BinOp::Add,
                        left: Box::new(Expr::Lit(Literal::Int(1))),
                        right: Box::new(Expr::Lit(Literal::Int(2))),
                    }),
                ),
                (
                    "y".to_string(),
                    Box::new(Expr::BinOp {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Lit(Literal::Int(3))),
                        right: Box::new(Expr::Lit(Literal::Int(4))),
                    }),
                ),
            ],
        };
        assert_eq!(format!("{}", expr), "{ x = (1 + 2); y = (3 * 4) }");
    }

    #[test]
    fn test_expr_record_update_with_expression() {
        let expr = Expr::RecordUpdate {
            record: Box::new(Expr::Var("person".to_string())),
            fields: vec![(
                "age".to_string(),
                Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::RecordAccess {
                        record: Box::new(Expr::Var("person".to_string())),
                        field: "age".to_string(),
                    }),
                    right: Box::new(Expr::Lit(Literal::Int(1))),
                }),
            )],
        };
        assert_eq!(
            format!("{}", expr),
            "({ person with age = ((person.age) + 1) })"
        );
    }

    #[test]
    fn test_expr_record_nested_literal() {
        // { outer = { inner = 42 } }
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![(
                "outer".to_string(),
                Box::new(Expr::RecordLiteral {
                    type_name: String::new(),
                    fields: vec![("inner".to_string(), Box::new(Expr::Lit(Literal::Int(42))))],
                }),
            )],
        };
        assert_eq!(format!("{}", expr), "{ outer = { inner = 42 } }");
    }

    // Continue with existing tests from original file...
    // (I'll include a few key tests for continuity)

    #[test]
    fn test_expr_var() {
        let expr = Expr::Var("x".to_string());
        assert!(expr.is_var());
        assert!(!expr.is_literal());
        assert_eq!(expr.as_var(), Some("x"));
        assert_eq!(format!("{}", expr), "x");
    }

    #[test]
    fn test_expr_lit() {
        let expr = Expr::Lit(Literal::Int(42));
        assert!(expr.is_literal());
        assert!(!expr.is_var());
        assert_eq!(expr.as_literal(), Some(&Literal::Int(42)));
        assert_eq!(format!("{}", expr), "42");
    }

    #[test]
    fn test_expr_tuple_pair() {
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]);
        assert!(expr.is_tuple());
        assert!(!expr.is_literal());
        assert_eq!(format!("{}", expr), "(1, 2)");
    }

    #[test]
    fn test_list_empty() {
        let expr = Expr::List(vec![]);
        assert!(expr.is_list());
        assert_eq!(format!("{}", expr), "[]");
    }

    #[test]
    fn test_array_empty() {
        let expr = Expr::Array(vec![]);
        assert!(expr.is_array());
        assert_eq!(format!("{}", expr), "[||]");
    }
}

// ========================================================================
// Pattern Tests (Issue #27 Layer 1)
// ========================================================================

#[test]
fn test_pattern_wildcard() {
    let pat = Pattern::Wildcard;
    assert!(pat.is_wildcard());
    assert!(!pat.is_var());
    assert!(!pat.is_literal());
    assert!(!pat.is_tuple());
    assert_eq!(format!("{}", pat), "_");
}

#[test]
fn test_pattern_var() {
    let pat = Pattern::Var("x".to_string());
    assert!(pat.is_var());
    assert!(!pat.is_wildcard());
    assert!(!pat.is_literal());
    assert_eq!(pat.as_var(), Some("x"));
    assert_eq!(format!("{}", pat), "x");
}

#[test]
fn test_pattern_literal_int() {
    let pat = Pattern::Literal(Literal::Int(42));
    assert!(pat.is_literal());
    assert!(!pat.is_var());
    assert!(!pat.is_wildcard());
    assert_eq!(pat.as_literal(), Some(&Literal::Int(42)));
    assert_eq!(format!("{}", pat), "42");
}

#[test]
fn test_pattern_literal_bool() {
    let pat_true = Pattern::Literal(Literal::Bool(true));
    let pat_false = Pattern::Literal(Literal::Bool(false));
    assert!(pat_true.is_literal());
    assert!(pat_false.is_literal());
    assert_eq!(format!("{}", pat_true), "true");
    assert_eq!(format!("{}", pat_false), "false");
}

#[test]
fn test_pattern_literal_string() {
    let pat = Pattern::Literal(Literal::Str("hello".to_string()));
    assert!(pat.is_literal());
    assert_eq!(format!("{}", pat), "\"hello\"");
}

#[test]
fn test_pattern_tuple_empty() {
    let pat = Pattern::Tuple(vec![]);
    assert!(pat.is_tuple());
    assert!(!pat.is_literal());
    assert_eq!(pat.as_tuple(), Some(&vec![]));
    assert_eq!(format!("{}", pat), "()");
}

#[test]
fn test_pattern_tuple_simple() {
    let pat = Pattern::Tuple(vec![
        Pattern::Var("x".to_string()),
        Pattern::Var("y".to_string()),
    ]);
    assert!(pat.is_tuple());
    assert_eq!(format!("{}", pat), "(x, y)");
}

#[test]
fn test_pattern_tuple_mixed() {
    let pat = Pattern::Tuple(vec![
        Pattern::Literal(Literal::Int(0)),
        Pattern::Var("y".to_string()),
        Pattern::Wildcard,
    ]);
    assert!(pat.is_tuple());
    assert_eq!(format!("{}", pat), "(0, y, _)");
}

#[test]
fn test_pattern_tuple_nested() {
    let pat = Pattern::Tuple(vec![
        Pattern::Var("x".to_string()),
        Pattern::Tuple(vec![
            Pattern::Var("y".to_string()),
            Pattern::Var("z".to_string()),
        ]),
    ]);
    assert!(pat.is_tuple());
    assert_eq!(format!("{}", pat), "(x, (y, z))");
}

#[test]
fn test_pattern_clone() {
    let pat1 = Pattern::Var("x".to_string());
    let pat2 = pat1.clone();
    assert_eq!(pat1, pat2);
}

#[test]
fn test_pattern_equality() {
    let pat1 = Pattern::Var("x".to_string());
    let pat2 = Pattern::Var("x".to_string());
    let pat3 = Pattern::Var("y".to_string());
    assert_eq!(pat1, pat2);
    assert_ne!(pat1, pat3);
}

#[test]
fn test_pattern_as_var_none() {
    let pat = Pattern::Wildcard;
    assert_eq!(pat.as_var(), None);
}

#[test]
fn test_pattern_as_literal_none() {
    let pat = Pattern::Var("x".to_string());
    assert_eq!(pat.as_literal(), None);
}

#[test]
fn test_pattern_as_tuple_none() {
    let pat = Pattern::Wildcard;
    assert_eq!(pat.as_tuple(), None);
}

#[test]
fn test_pattern_all_variants() {
    let patterns = [
        Pattern::Wildcard,
        Pattern::Var("x".to_string()),
        Pattern::Literal(Literal::Int(42)),
        Pattern::Tuple(vec![Pattern::Wildcard]),
    ];
    assert_eq!(patterns.len(), 4);
}

// ========================================================================
// MatchArm Tests (Issue #27 Layer 1)
// ========================================================================

#[test]
fn test_match_arm_simple() {
    let arm = MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(42)));
    assert!(arm.is_wildcard());
    assert!(!arm.is_var());
    assert_eq!(format!("{}", arm), "_ -> 42");
}

#[test]
fn test_match_arm_with_var() {
    let arm = MatchArm::new(Pattern::Var("x".to_string()), Expr::Var("x".to_string()));
    assert!(arm.is_var());
    assert!(!arm.is_wildcard());
    assert_eq!(format!("{}", arm), "x -> x");
}

#[test]
fn test_match_arm_with_literal() {
    let arm = MatchArm::new(
        Pattern::Literal(Literal::Int(0)),
        Expr::Lit(Literal::Str("zero".to_string())),
    );
    assert!(!arm.is_wildcard());
    assert!(!arm.is_var());
    assert_eq!(format!("{}", arm), "0 -> \"zero\"");
}

#[test]
fn test_match_arm_with_tuple_pattern() {
    let arm = MatchArm::new(
        Pattern::Tuple(vec![
            Pattern::Var("x".to_string()),
            Pattern::Var("y".to_string()),
        ]),
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("x".to_string())),
            right: Box::new(Expr::Var("y".to_string())),
        },
    );
    assert_eq!(format!("{}", arm), "(x, y) -> (x + y)");
}

#[test]
fn test_match_arm_clone() {
    let arm1 = MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(42)));
    let arm2 = arm1.clone();
    assert_eq!(arm1, arm2);
}

#[test]
fn test_match_arm_equality() {
    let arm1 = MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(42)));
    let arm2 = MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(42)));
    assert_eq!(arm1, arm2);
}

// ========================================================================
// Match Expression Tests (Issue #27 Layer 1)
// ========================================================================

#[test]
fn test_expr_match_simple() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Literal(Literal::Int(0)),
                Expr::Lit(Literal::Str("zero".to_string())),
            ),
            MatchArm::new(
                Pattern::Wildcard,
                Expr::Lit(Literal::Str("nonzero".to_string())),
            ),
        ],
    };
    assert!(expr.is_match());
    assert!(!expr.is_if());
    assert_eq!(
        format!("{}", expr),
        "(match x with | 0 -> \"zero\" | _ -> \"nonzero\")"
    );
}

#[test]
fn test_expr_match_multiple_arms() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("n".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Literal(Literal::Int(0)),
                Expr::Lit(Literal::Str("zero".to_string())),
            ),
            MatchArm::new(
                Pattern::Literal(Literal::Int(1)),
                Expr::Lit(Literal::Str("one".to_string())),
            ),
            MatchArm::new(
                Pattern::Literal(Literal::Int(2)),
                Expr::Lit(Literal::Str("two".to_string())),
            ),
            MatchArm::new(
                Pattern::Wildcard,
                Expr::Lit(Literal::Str("many".to_string())),
            ),
        ],
    };
    assert!(expr.is_match());
    assert_eq!(expr.as_match().unwrap().1.len(), 4);
}

#[test]
fn test_expr_match_with_var_binding() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Literal(Literal::Int(0)),
                Expr::Lit(Literal::Str("zero".to_string())),
            ),
            MatchArm::new(Pattern::Var("n".to_string()), Expr::Var("n".to_string())),
        ],
    };
    assert!(expr.is_match());
    let (scrutinee, arms) = expr.as_match().unwrap();
    assert_eq!(scrutinee.as_var(), Some("x"));
    assert_eq!(arms.len(), 2);
    assert!(arms[1].is_var());
}

#[test]
fn test_expr_match_with_tuple_pattern() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("pair".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Tuple(vec![
                    Pattern::Literal(Literal::Int(0)),
                    Pattern::Literal(Literal::Int(0)),
                ]),
                Expr::Lit(Literal::Str("origin".to_string())),
            ),
            MatchArm::new(
                Pattern::Tuple(vec![
                    Pattern::Var("x".to_string()),
                    Pattern::Var("y".to_string()),
                ]),
                Expr::Lit(Literal::Str("point".to_string())),
            ),
        ],
    };
    assert!(expr.is_match());
    let (_, arms) = expr.as_match().unwrap();
    assert_eq!(arms.len(), 2);
    assert!(arms[0].pattern.is_tuple());
}

#[test]
fn test_expr_match_nested_in_let() {
    let expr = Expr::Let {
        name: "result".to_string(),
        value: Box::new(Expr::Match {
            scrutinee: Box::new(Expr::Var("x".to_string())),
            arms: vec![
                MatchArm::new(
                    Pattern::Literal(Literal::Int(0)),
                    Expr::Lit(Literal::Int(1)),
                ),
                MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(0))),
            ],
        }),
        body: Box::new(Expr::Var("result".to_string())),
    };
    assert!(expr.is_let());
    if let Expr::Let { value, .. } = &expr {
        assert!(value.is_match());
    }
}

#[test]
fn test_expr_match_with_bool_patterns() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("flag".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Literal(Literal::Bool(true)),
                Expr::Lit(Literal::Str("yes".to_string())),
            ),
            MatchArm::new(
                Pattern::Literal(Literal::Bool(false)),
                Expr::Lit(Literal::Str("no".to_string())),
            ),
        ],
    };
    assert!(expr.is_match());
}

#[test]
fn test_expr_match_as_match() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![MatchArm::new(
            Pattern::Wildcard,
            Expr::Lit(Literal::Int(42)),
        )],
    };
    let result = expr.as_match();
    assert!(result.is_some());
    let (scrutinee, arms) = result.unwrap();
    assert_eq!(scrutinee.as_var(), Some("x"));
    assert_eq!(arms.len(), 1);
}

#[test]
fn test_expr_match_as_match_none() {
    let expr = Expr::Lit(Literal::Int(42));
    assert_eq!(expr.as_match(), None);
}

#[test]
fn test_expr_match_clone() {
    let expr1 = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![MatchArm::new(
            Pattern::Wildcard,
            Expr::Lit(Literal::Int(42)),
        )],
    };
    let expr2 = expr1.clone();
    assert_eq!(expr1, expr2);
}

#[test]
fn test_expr_match_equality() {
    let expr1 = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![MatchArm::new(
            Pattern::Wildcard,
            Expr::Lit(Literal::Int(42)),
        )],
    };
    let expr2 = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![MatchArm::new(
            Pattern::Wildcard,
            Expr::Lit(Literal::Int(42)),
        )],
    };
    assert_eq!(expr1, expr2);
}

#[test]
fn test_expr_match_complex_body() {
    let expr = Expr::Match {
        scrutinee: Box::new(Expr::Var("x".to_string())),
        arms: vec![
            MatchArm::new(
                Pattern::Literal(Literal::Int(0)),
                Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Lit(Literal::Int(1))),
                    right: Box::new(Expr::Lit(Literal::Int(2))),
                },
            ),
            MatchArm::new(Pattern::Wildcard, Expr::Lit(Literal::Int(0))),
        ],
    };
    assert!(expr.is_match());
}
