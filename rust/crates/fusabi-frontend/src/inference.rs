//! Hindley-Milner Type Inference (Algorithm W)
//!
//! This module implements Layer 2 of the Fusabi type system: constraint-based type inference
//! using the Hindley-Milner algorithm. It builds on Layer 1 (types.rs) to provide complete
//! type checking for F# expressions.
//!
//! # Architecture
//!
//! The type inference algorithm follows the classic Hindley-Milner approach:
//! 1. **Constraint Generation**: Traverse the AST and generate type constraints
//! 2. **Unification**: Solve constraints using Robinson's unification algorithm
//! 3. **Generalization**: Generalize types in let-bindings for polymorphism
//! 4. **Instantiation**: Instantiate polymorphic type schemes with fresh variables
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::inference::TypeInference;
//! use fusabi_frontend::types::TypeEnv;
//! use fusabi_frontend::ast::{Expr, Literal};
//!
//! let mut inference = TypeInference::new();
//! let env = TypeEnv::new();
//! let expr = Expr::Lit(Literal::Int(42));
//!
//! let ty = inference.infer_and_solve(&expr, &env).unwrap();
//! // ty is Type::Int
//! ```
//!
//! # Key Features
//!
//! - **Let-polymorphism**: Automatic generalization in let-bindings
//! - **Occurs check**: Prevents infinite types
//! - **Pattern matching**: Full support for match expressions
//! - **Records and variants**: Type checking for structural types
//! - **Helpful errors**: Detailed error messages with suggestions
//! - **Auto-recursive detection**: Automatically detects recursive lambdas (issue #126)

use crate::ast::{BinOp, Expr, Literal, MatchArm, Pattern};
use crate::error::{TypeError, TypeErrorKind};
use crate::modules::ModuleRegistry;
use crate::types::{Substitution, Type, TypeEnv, TypeScheme, TypeVar};
use std::collections::HashMap;

/// Constraint representing equality between two types.
///
/// During inference, we generate constraints like `t1 = t2` which are later
/// solved through unification.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Two types must be equal
    Equal(Type, Type),
}

/// Type inference engine implementing Algorithm W.
///
/// Maintains state for fresh type variable generation and constraint accumulation.
pub struct TypeInference {
    /// Counter for generating fresh type variables
    next_var_id: usize,
    /// Accumulated type constraints
    constraints: Vec<Constraint>,
    /// Optional module registry for type definition lookups
    module_registry: Option<ModuleRegistry>,
}

#[allow(clippy::result_large_err)]
impl TypeInference {
    /// Create a new type inference instance.
    pub fn new() -> Self {
        TypeInference {
            next_var_id: 0,
            constraints: Vec::new(),
            module_registry: None,
        }
    }

    /// Create a new type inference instance with a module registry for type validation.
    pub fn with_module_registry(module_registry: ModuleRegistry) -> Self {
        TypeInference {
            next_var_id: 0,
            constraints: Vec::new(),
            module_registry: Some(module_registry),
        }
    }

    /// Set the module registry for type definition lookups.
    pub fn set_module_registry(&mut self, registry: ModuleRegistry) {
        self.module_registry = Some(registry);
    }

    /// Generate a fresh type variable.
    ///
    /// Each call produces a unique type variable that hasn't been used before.
    pub fn fresh_var(&mut self) -> TypeVar {
        let id = self.next_var_id;
        self.next_var_id += 1;
        TypeVar::fresh(id)
    }

    /// Add a constraint to the constraint set.
    fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Check if an expression references a variable (for auto-recursion detection).
    ///
    /// This performs a simple free variable analysis to detect if `name` appears
    /// anywhere in the expression. Used to automatically treat `let x = fun ... x ...`
    /// as recursive without requiring explicit `let rec`.
    fn expr_references_var(expr: &Expr, name: &str) -> bool {
        match expr {
            Expr::Var(var_name) => var_name == name,
            Expr::Lambda { param, body } => {
                // If the lambda parameter shadows the name, don't look inside
                if param == name {
                    false
                } else {
                    Self::expr_references_var(body, name)
                }
            }
            Expr::App { func, arg } => {
                Self::expr_references_var(func, name) || Self::expr_references_var(arg, name)
            }
            Expr::Let {
                name: let_name,
                value,
                body,
            } => {
                // Check value, but if let shadows the name, don't check body
                Self::expr_references_var(value, name)
                    || (let_name != name && Self::expr_references_var(body, name))
            }
            Expr::LetRec {
                name: rec_name,
                value,
                body,
            } => {
                // Similar to Let
                Self::expr_references_var(value, name)
                    || (rec_name != name && Self::expr_references_var(body, name))
            }
            Expr::LetRecMutual { bindings, body } => {
                // Check all binding values
                bindings
                    .iter()
                    .any(|(_, expr)| Self::expr_references_var(expr, name))
                    // Check body unless one of the bindings shadows the name
                    || (!bindings.iter().any(|(n, _)| n == name)
                        && Self::expr_references_var(body, name))
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                Self::expr_references_var(cond, name)
                    || Self::expr_references_var(then_branch, name)
                    || Self::expr_references_var(else_branch, name)
            }
            Expr::BinOp { left, right, .. } => {
                Self::expr_references_var(left, name) || Self::expr_references_var(right, name)
            }
            Expr::Tuple(elements) | Expr::List(elements) | Expr::Array(elements) => {
                elements.iter().any(|e| Self::expr_references_var(e, name))
            }
            Expr::Cons { head, tail } => {
                Self::expr_references_var(head, name) || Self::expr_references_var(tail, name)
            }
            Expr::ArrayIndex { array, index } => {
                Self::expr_references_var(array, name) || Self::expr_references_var(index, name)
            }
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => {
                Self::expr_references_var(array, name)
                    || Self::expr_references_var(index, name)
                    || Self::expr_references_var(value, name)
            }
            Expr::ArrayLength(array) => Self::expr_references_var(array, name),
            Expr::RecordLiteral { fields, .. } => fields
                .iter()
                .any(|(_, expr)| Self::expr_references_var(expr, name)),
            Expr::RecordAccess { record, .. } => Self::expr_references_var(record, name),
            Expr::RecordUpdate { record, fields } => {
                Self::expr_references_var(record, name)
                    || fields
                        .iter()
                        .any(|(_, expr)| Self::expr_references_var(expr, name))
            }
            Expr::VariantConstruct { fields, .. } => fields
                .iter()
                .any(|expr| Self::expr_references_var(expr, name)),
            Expr::Match { scrutinee, arms } => {
                Self::expr_references_var(scrutinee, name)
                    || arms.iter().any(|arm| {
                        // Check if pattern binds the name (shadows it)
                        let pattern_binds = Self::pattern_binds(&arm.pattern, name);
                        // Only check body if pattern doesn't shadow the name
                        !pattern_binds && Self::expr_references_var(&arm.body, name)
                    })
            }
            Expr::MethodCall { receiver, args, .. } => {
                Self::expr_references_var(receiver, name)
                    || args.iter().any(|e| Self::expr_references_var(e, name))
            }
            Expr::While { cond, body } => {
                Self::expr_references_var(cond, name) || Self::expr_references_var(body, name)
            }
            Expr::ComputationExpr { body, .. } => {
                // Check if any statement in the CE body references the variable
                body.iter().any(|stmt| {
                    use crate::ast::CEStatement;
                    match stmt {
                        CEStatement::Let { value, .. }
                        | CEStatement::LetBang { value, .. }
                        | CEStatement::DoBang { value }
                        | CEStatement::Return { value }
                        | CEStatement::ReturnBang { value }
                        | CEStatement::Yield { value }
                        | CEStatement::YieldBang { value }
                        | CEStatement::Expr { value } => Self::expr_references_var(value, name),
                    }
                })
            }
            // Literals and control flow don't reference variables
            Expr::Lit(_) | Expr::Break | Expr::Continue => false,
        }
    }

    /// Check if a pattern binds a variable name.
    fn pattern_binds(pattern: &Pattern, name: &str) -> bool {
        match pattern {
            Pattern::Var(var_name) => var_name == name,
            Pattern::Tuple(patterns) | Pattern::Variant { patterns, .. } => {
                patterns.iter().any(|p| Self::pattern_binds(p, name))
            }
            Pattern::Wildcard | Pattern::Literal(_) => false,
        }
    }

    /// Infer the type of an expression in the given environment.
    ///
    /// This is the main entry point for type inference. It generates constraints
    /// and returns a type (possibly containing type variables).
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to type check
    /// * `env` - The type environment containing variable bindings
    ///
    /// # Returns
    ///
    /// The inferred type, or a type error if inference fails.
    pub fn infer(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        match expr {
            // Literals have concrete types
            Expr::Lit(lit) => Ok(self.infer_literal(lit)),

            // Variables: lookup in environment and instantiate
            Expr::Var(name) => self.infer_var(name, env),

            // Lambda: fun x -> body
            Expr::Lambda { param, body } => self.infer_lambda(param, body, env),

            // Function application: f arg
            Expr::App { func, arg } => self.infer_app(func, arg, env),

            // Let-binding: let x = value in body
            Expr::Let { name, value, body } => self.infer_let(name, value, body, env, false),

            // Recursive let-binding: let rec f = value in body
            Expr::LetRec { name, value, body } => self.infer_let(name, value, body, env, true),

            // Mutually recursive bindings: let rec f = ... and g = ... in body
            Expr::LetRecMutual { bindings, body } => self.infer_let_rec_mutual(bindings, body, env),

            // Conditional: if cond then t else e
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => self.infer_if(cond, then_branch, else_branch, env),

            // Binary operations: e1 op e2
            Expr::BinOp { op, left, right } => self.infer_binop(*op, left, right, env),

            // Tuple: (e1, e2, ...)
            Expr::Tuple(elements) => self.infer_tuple(elements, env),

            // List: [e1; e2; ...]
            Expr::List(elements) => self.infer_list(elements, env),

            // Cons: e1 :: e2
            Expr::Cons { head, tail } => self.infer_cons(head, tail, env),

            // Array: [|e1; e2; ...|]
            Expr::Array(elements) => self.infer_array(elements, env),

            // Array indexing: arr.[idx]
            Expr::ArrayIndex { array, index } => self.infer_array_index(array, index, env),

            // Array update: arr.[idx] <- value
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => self.infer_array_update(array, index, value, env),

            // Array length: Array.length arr
            Expr::ArrayLength(array) => self.infer_array_length(array, env),

            // Record literal: { field1 = e1; field2 = e2 }
            Expr::RecordLiteral { type_name, fields } => {
                self.infer_record_literal(type_name, fields, env)
            }

            // Record access: record.field
            Expr::RecordAccess { record, field } => self.infer_record_access(record, field, env),

            // Record update: { record with field = value }
            Expr::RecordUpdate { record, fields } => self.infer_record_update(record, fields, env),

            // Variant constructor: Some(42), None, etc.
            Expr::VariantConstruct {
                type_name,
                variant,
                fields,
            } => self.infer_variant_construct(type_name, variant, fields, env),

            // Pattern matching: match scrutinee with | pat1 -> e1 | pat2 -> e2
            Expr::Match { scrutinee, arms } => self.infer_match(scrutinee, arms, env),

            // Method call: obj.method(args...)
            Expr::MethodCall {
                receiver,
                method_name: _,
                args: _,
            } => {
                // For now, we infer method calls conservatively
                // Type check the receiver
                self.infer(receiver, env)?;
                // Return a fresh type variable since we don't know the method's return type
                Ok(Type::Var(self.fresh_var()))
            }

            // While loop: while cond do body
            Expr::While { cond, body } => {
                // Condition must be bool
                let cond_ty = self.infer(cond, env)?;
                self.unify(&cond_ty, &Type::Bool)?;
                // Type check the body
                self.infer(body, env)?;
                // While loops return unit
                Ok(Type::Unit)
            }

            // Break statement
            Expr::Break => {
                // Break has unit type but can only appear in loops
                // We'll let the compiler handle loop context validation
                Ok(Type::Unit)
            }

            // Continue statement
            Expr::Continue => {
                // Continue has unit type but can only appear in loops
                // We'll let the compiler handle loop context validation
                Ok(Type::Unit)
            }

            // Computation expression (stub implementation)
            Expr::ComputationExpr {
                builder: _,
                body: _,
            } => {
                // TODO: Implement proper type inference for computation expressions
                // For now, return a fresh type variable
                Ok(Type::Var(self.fresh_var()))
            }
        }
    }

    /// Infer the type of a literal value.
    fn infer_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::Bool(_) => Type::Bool,
            Literal::Str(_) => Type::String,
            Literal::Unit => Type::Unit,
        }
    }

    /// Infer the type of a variable by looking it up in the environment.
    fn infer_var(&mut self, name: &str, env: &TypeEnv) -> Result<Type, TypeError> {
        match env.lookup(name) {
            Some(scheme) => {
                // Instantiate the type scheme with fresh type variables
                Ok(env.instantiate(scheme, &mut || self.fresh_var()))
            }
            None => Err(TypeError::new(TypeErrorKind::UnboundVariable {
                name: name.to_string(),
            })),
        }
    }

    /// Infer the type of a lambda function.
    ///
    /// For `fun x -> body`, we:
    /// 1. Create a fresh type variable α for the parameter
    /// 2. Extend the environment with x: α
    /// 3. Infer the type β of the body
    /// 4. Return α -> β
    fn infer_lambda(&mut self, param: &str, body: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        let param_type = Type::Var(self.fresh_var());
        let param_scheme = TypeScheme::mono(param_type.clone());
        let extended_env = env.extend(param.to_string(), param_scheme);

        let body_type = self.infer(body, &extended_env)?;

        Ok(Type::Function(Box::new(param_type), Box::new(body_type)))
    }

    /// Infer the type of a function application.
    ///
    /// For `f arg`, we:
    /// 1. Infer the type tf of f
    /// 2. Infer the type targ of arg
    /// 3. Create a fresh type variable α for the result
    /// 4. Add constraint: tf = targ -> α
    /// 5. Return α
    fn infer_app(&mut self, func: &Expr, arg: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        let func_type = self.infer(func, env)?;
        let arg_type = self.infer(arg, env)?;
        let result_type = Type::Var(self.fresh_var());

        // Constraint: func_type = arg_type -> result_type
        let expected_func_type = Type::Function(Box::new(arg_type), Box::new(result_type.clone()));
        self.add_constraint(Constraint::Equal(func_type, expected_func_type));

        Ok(result_type)
    }

    /// Infer the type of a let-binding with automatic recursion detection.
    ///
    /// For `let x = value in body`:
    /// 1. Check if value references x (auto-detect recursion)
    /// 2. If recursive or is_recursive is true, infer with x in scope
    /// 3. Generalize the type (let-polymorphism)
    /// 4. Infer and return the type of body
    ///
    /// This implements issue #126: automatic recursive function detection
    /// for lambda expressions like `let factorial = fun n -> ... factorial ...`
    fn infer_let(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
        env: &TypeEnv,
        is_recursive: bool,
    ) -> Result<Type, TypeError> {
        // Auto-detect recursion: check if value references name
        let auto_recursive = !is_recursive && Self::expr_references_var(value, name);
        let treat_as_recursive = is_recursive || auto_recursive;

        let value_type = if treat_as_recursive {
            // For recursive bindings, assume a fresh type variable for the name
            let rec_var = Type::Var(self.fresh_var());
            let rec_scheme = TypeScheme::mono(rec_var.clone());
            let rec_env = env.extend(name.to_string(), rec_scheme);

            // Infer the value type in the extended environment
            let inferred = self.infer(value, &rec_env)?;

            // Add constraint: rec_var = inferred
            self.add_constraint(Constraint::Equal(rec_var, inferred.clone()));
            inferred
        } else {
            // Non-recursive: infer value in current environment
            self.infer(value, env)?
        };

        // Generalize the type (let-polymorphism)
        let value_scheme = env.generalize(&value_type);

        // Extend environment and infer body
        let extended_env = env.extend(name.to_string(), value_scheme);
        self.infer(body, &extended_env)
    }

    /// Infer the type of mutually recursive let-bindings.
    fn infer_let_rec_mutual(
        &mut self,
        bindings: &[(String, Expr)],
        body: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        // Create fresh type variables for all bindings
        let mut rec_env = env.clone();
        let mut binding_vars = Vec::new();

        for (name, _) in bindings {
            let var = Type::Var(self.fresh_var());
            rec_env.insert(name.clone(), TypeScheme::mono(var.clone()));
            binding_vars.push((name.clone(), var));
        }

        // Infer types for all bindings in the extended environment
        for ((_, expr), (_name, var)) in bindings.iter().zip(binding_vars.iter()) {
            let inferred = self.infer(expr, &rec_env)?;
            self.add_constraint(Constraint::Equal(var.clone(), inferred));
        }

        // Infer body type
        self.infer(body, &rec_env)
    }

    /// Infer the type of a conditional expression.
    ///
    /// For `if cond then t else e`:
    /// 1. Infer type of cond and constrain it to bool
    /// 2. Infer types of both branches
    /// 3. Constrain both branches to have the same type
    /// 4. Return the branch type
    fn infer_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let cond_type = self.infer(cond, env)?;
        self.add_constraint(Constraint::Equal(cond_type, Type::Bool));

        let then_type = self.infer(then_branch, env)?;
        let else_type = self.infer(else_branch, env)?;

        // Both branches must have the same type
        self.add_constraint(Constraint::Equal(then_type.clone(), else_type));

        Ok(then_type)
    }

    /// Infer the type of a binary operation.
    fn infer_binop(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let left_type = self.infer(left, env)?;
        let right_type = self.infer(right, env)?;

        if op.is_arithmetic() {
            // Arithmetic: both operands must be int or float, result is same type
            // For simplicity, we constrain to int (full implementation would support float)
            self.add_constraint(Constraint::Equal(left_type.clone(), Type::Int));
            self.add_constraint(Constraint::Equal(right_type, Type::Int));
            Ok(Type::Int)
        } else if op.is_comparison() {
            // Comparison: operands must have the same type, result is bool
            self.add_constraint(Constraint::Equal(left_type, right_type));
            Ok(Type::Bool)
        } else if op.is_logical() {
            // Logical: both operands must be bool, result is bool
            self.add_constraint(Constraint::Equal(left_type, Type::Bool));
            self.add_constraint(Constraint::Equal(right_type, Type::Bool));
            Ok(Type::Bool)
        } else {
            unreachable!("Unknown binary operator")
        }
    }

    /// Infer the type of a tuple.
    fn infer_tuple(&mut self, elements: &[Expr], env: &TypeEnv) -> Result<Type, TypeError> {
        let mut element_types = Vec::new();
        for element in elements {
            element_types.push(self.infer(element, env)?);
        }
        Ok(Type::Tuple(element_types))
    }

    /// Infer the type of a list.
    ///
    /// All elements must have the same type.
    fn infer_list(&mut self, elements: &[Expr], env: &TypeEnv) -> Result<Type, TypeError> {
        if elements.is_empty() {
            // Empty list has polymorphic type 'a list
            Ok(Type::List(Box::new(Type::Var(self.fresh_var()))))
        } else {
            let first_type = self.infer(&elements[0], env)?;
            for element in &elements[1..] {
                let element_type = self.infer(element, env)?;
                self.add_constraint(Constraint::Equal(first_type.clone(), element_type));
            }
            Ok(Type::List(Box::new(first_type)))
        }
    }

    /// Infer the type of cons operator (::).
    fn infer_cons(&mut self, head: &Expr, tail: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        let head_type = self.infer(head, env)?;
        let tail_type = self.infer(tail, env)?;

        let expected_tail = Type::List(Box::new(head_type.clone()));
        self.add_constraint(Constraint::Equal(tail_type, expected_tail));

        Ok(Type::List(Box::new(head_type)))
    }

    /// Infer the type of an array.
    fn infer_array(&mut self, elements: &[Expr], env: &TypeEnv) -> Result<Type, TypeError> {
        if elements.is_empty() {
            Ok(Type::Array(Box::new(Type::Var(self.fresh_var()))))
        } else {
            let first_type = self.infer(&elements[0], env)?;
            for element in &elements[1..] {
                let element_type = self.infer(element, env)?;
                self.add_constraint(Constraint::Equal(first_type.clone(), element_type));
            }
            Ok(Type::Array(Box::new(first_type)))
        }
    }

    /// Infer the type of array indexing.
    fn infer_array_index(
        &mut self,
        array: &Expr,
        index: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let array_type = self.infer(array, env)?;
        let index_type = self.infer(index, env)?;

        // Index must be int
        self.add_constraint(Constraint::Equal(index_type, Type::Int));

        // Array must be array type, extract element type
        let element_type = Type::Var(self.fresh_var());
        let expected_array_type = Type::Array(Box::new(element_type.clone()));
        self.add_constraint(Constraint::Equal(array_type, expected_array_type));

        Ok(element_type)
    }

    /// Infer the type of array update.
    fn infer_array_update(
        &mut self,
        array: &Expr,
        index: &Expr,
        value: &Expr,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let array_type = self.infer(array, env)?;
        let index_type = self.infer(index, env)?;
        let value_type = self.infer(value, env)?;

        // Index must be int
        self.add_constraint(Constraint::Equal(index_type, Type::Int));

        // Value type must match array element type
        let expected_array_type = Type::Array(Box::new(value_type));
        self.add_constraint(Constraint::Equal(array_type.clone(), expected_array_type));

        Ok(array_type)
    }

    /// Infer the type of array length.
    fn infer_array_length(&mut self, array: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        let array_type = self.infer(array, env)?;

        // Must be an array
        let element_type = Type::Var(self.fresh_var());
        let expected_array_type = Type::Array(Box::new(element_type));
        self.add_constraint(Constraint::Equal(array_type, expected_array_type));

        Ok(Type::Int)
    }

    /// Infer the type of a record literal.
    fn infer_record_literal(
        &mut self,
        type_name: &str,
        fields: &[(String, Box<Expr>)],
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        // If type_name is provided and we have a module registry, validate fields
        if !type_name.is_empty() {
            if let Some(ref registry) = self.module_registry {
                self.validate_record_fields(type_name, fields, registry)?;
            }
        }

        let mut field_types = HashMap::new();

        for (field_name, field_expr) in fields {
            let field_type = self.infer(field_expr, env)?;
            field_types.insert(field_name.clone(), field_type);
        }

        Ok(Type::Record(field_types))
    }

    /// Validate that record literal fields match the type definition.
    fn validate_record_fields(
        &self,
        type_name: &str,
        fields: &[(String, Box<Expr>)],
        registry: &ModuleRegistry,
    ) -> Result<(), TypeError> {
        // Try to find the type definition in the registry
        // First check if it's a qualified name (e.g., "Module.TypeName")
        let type_def = if type_name.contains('.') {
            let parts: Vec<&str> = type_name.splitn(2, '.').collect();
            if parts.len() == 2 {
                let (module_name, type_local_name) = (parts[0], parts[1]);
                registry
                    .get_module_types(module_name)
                    .and_then(|types| types.get(type_local_name))
            } else {
                None
            }
        } else {
            // Try to find in any module (search all modules)
            registry
                .module_names()
                .iter()
                .find_map(|module_name| {
                    registry
                        .get_module_types(module_name)
                        .and_then(|types| types.get(type_name))
                })
        };

        if let Some(type_def) = type_def {
            // Use the type provider's field validation
            let provided_fields: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();

            // Convert AST TypeDefinition to type provider TypeDefinition and validate
            let validation_result = match type_def {
                crate::modules::TypeDefinition::Record(record) => {
                    // Convert to type provider RecordDef
                    let provider_fields: Vec<(String, fusabi_type_providers::TypeExpr)> = record
                        .fields
                        .iter()
                        .map(|(name, ty)| {
                            (name.clone(), self.ast_type_expr_to_provider_type_expr(ty))
                        })
                        .collect();

                    let provider_record = fusabi_type_providers::RecordDef {
                        name: record.name.clone(),
                        fields: provider_fields,
                    };

                    let provider_type_def = fusabi_type_providers::TypeDefinition::Record(provider_record);
                    provider_type_def.validate_fields(&provided_fields)
                }
                crate::modules::TypeDefinition::Du(_) => {
                    // DU types don't support record literal syntax
                    return Err(TypeError::new(TypeErrorKind::Custom {
                        message: format!(
                            "Type '{}' is a discriminated union, not a record. Use variant constructors instead.",
                            type_name
                        ),
                    }));
                }
                crate::modules::TypeDefinition::Provider(_) => {
                    // Type provider declarations should already be resolved
                    return Err(TypeError::new(TypeErrorKind::Custom {
                        message: format!(
                            "Type '{}' is a type provider declaration. It should be resolved before type checking.",
                            type_name
                        ),
                    }));
                }
            };

            // Report validation errors
            if !validation_result.is_valid {
                if !validation_result.extra_fields.is_empty() {
                    // Get expected field names for suggestions
                    let expected_fields = match type_def {
                        crate::modules::TypeDefinition::Record(record) => {
                            record.fields.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>()
                        }
                        _ => vec![],
                    };

                    // For each extra field, compute suggestions
                    for extra_field in &validation_result.extra_fields {
                        let suggestions = Self::compute_field_suggestions(extra_field, &expected_fields);
                        return Err(TypeError::new(TypeErrorKind::ExtraField {
                            type_name: type_name.to_string(),
                            field: extra_field.clone(),
                            suggestions,
                        }));
                    }
                }

                // Report missing fields as a TypeProviderFieldMismatch
                if !validation_result.missing_fields.is_empty() || !validation_result.extra_fields.is_empty() {
                    return Err(TypeError::new(TypeErrorKind::TypeProviderFieldMismatch {
                        type_name: type_name.to_string(),
                        extra_fields: validation_result.extra_fields,
                        missing_fields: validation_result.missing_fields,
                    }));
                }
            }

            Ok(())
        } else {
            // Type not found in registry
            Err(TypeError::new(TypeErrorKind::UnknownType {
                type_name: type_name.to_string(),
            }))
        }
    }

    /// Convert AST TypeExpr to type provider TypeExpr.
    fn ast_type_expr_to_provider_type_expr(
        &self,
        ty: &crate::ast::TypeExpr,
    ) -> fusabi_type_providers::TypeExpr {
        match ty {
            crate::ast::TypeExpr::Named(name) => fusabi_type_providers::TypeExpr::Named(name.clone()),
            crate::ast::TypeExpr::Tuple(types) => fusabi_type_providers::TypeExpr::Tuple(
                types
                    .iter()
                    .map(|t| self.ast_type_expr_to_provider_type_expr(t))
                    .collect(),
            ),
            crate::ast::TypeExpr::Function(param, ret) => {
                fusabi_type_providers::TypeExpr::Function(
                    Box::new(self.ast_type_expr_to_provider_type_expr(param)),
                    Box::new(self.ast_type_expr_to_provider_type_expr(ret)),
                )
            }
        }
    }

    /// Compute field name suggestions based on Levenshtein distance.
    fn compute_field_suggestions(field: &str, expected_fields: &[String]) -> Vec<String> {
        let mut suggestions: Vec<(String, usize)> = expected_fields
            .iter()
            .map(|expected| {
                let distance = Self::levenshtein_distance(field, expected);
                (expected.clone(), distance)
            })
            .collect();

        // Sort by distance and keep only close matches (distance <= 3)
        suggestions.sort_by_key(|(_, dist)| *dist);
        suggestions
            .into_iter()
            .filter(|(_, dist)| *dist <= 3)
            .take(3) // Limit to top 3 suggestions
            .map(|(name, _)| name)
            .collect()
    }

    /// Compute Levenshtein distance between two strings.
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }

        matrix[len1][len2]
    }

    /// Infer the type of record field access.
    fn infer_record_access(
        &mut self,
        record: &Expr,
        field: &str,
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let record_type = self.infer(record, env)?;

        // Create a fresh type variable for the field
        let field_type = Type::Var(self.fresh_var());

        // Create a record type with at least this field
        let mut expected_fields = HashMap::new();
        expected_fields.insert(field.to_string(), field_type.clone());
        let expected_record = Type::Record(expected_fields);

        // Note: This is a simplified version. A full implementation would use
        // row polymorphism or structural typing for better record handling.
        self.add_constraint(Constraint::Equal(record_type, expected_record));

        Ok(field_type)
    }

    /// Infer the type of record update.
    fn infer_record_update(
        &mut self,
        record: &Expr,
        fields: &[(String, Box<Expr>)],
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let record_type = self.infer(record, env)?;

        // Infer types of updated fields
        let mut update_field_types = HashMap::new();
        for (field_name, field_expr) in fields {
            let field_type = self.infer(field_expr, env)?;
            update_field_types.insert(field_name.clone(), field_type);
        }

        // The result has the same type as the input record
        // (with potentially different field types for updated fields)
        Ok(record_type)
    }

    /// Infer the type of a variant constructor.
    fn infer_variant_construct(
        &mut self,
        _type_name: &str,
        variant: &str,
        fields: &[Box<Expr>],
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        // Infer types of all fields
        let mut field_types = Vec::new();
        for field in fields {
            field_types.push(self.infer(field, env)?);
        }

        // Create variant type
        Ok(Type::Variant(variant.to_string(), field_types))
    }

    /// Infer the type of a match expression.
    ///
    /// For `match scrutinee with | pat1 -> e1 | pat2 -> e2`:
    /// 1. Infer type of scrutinee
    /// 2. For each arm, check pattern matches scrutinee type
    /// 3. Infer type of each arm body
    /// 4. Constrain all arm bodies to have the same type
    /// 5. Return the arm type
    fn infer_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        if arms.is_empty() {
            return Err(TypeError::new(TypeErrorKind::Custom {
                message: "Match expression must have at least one arm".to_string(),
            }));
        }

        let scrutinee_type = self.infer(scrutinee, env)?;

        // Infer the type of the first arm as the result type
        let (_first_pattern_env, first_result_type) =
            self.infer_match_arm(&arms[0], &scrutinee_type, env)?;

        // Check remaining arms
        for arm in &arms[1..] {
            let (_, arm_type) = self.infer_match_arm(arm, &scrutinee_type, env)?;
            self.add_constraint(Constraint::Equal(first_result_type.clone(), arm_type));
        }

        Ok(first_result_type)
    }

    /// Infer the type of a single match arm.
    ///
    /// Returns the extended environment from pattern bindings and the body type.
    fn infer_match_arm(
        &mut self,
        arm: &MatchArm,
        scrutinee_type: &Type,
        env: &TypeEnv,
    ) -> Result<(TypeEnv, Type), TypeError> {
        // Check pattern against scrutinee type and get bindings
        let pattern_env = self.infer_pattern(&arm.pattern, scrutinee_type, env)?;

        // Infer body type in extended environment
        let body_type = self.infer(&arm.body, &pattern_env)?;

        Ok((pattern_env, body_type))
    }

    /// Infer pattern bindings and check pattern type matches scrutinee.
    ///
    /// Returns an extended environment with pattern variable bindings.
    pub fn infer_pattern(
        &mut self,
        pattern: &Pattern,
        scrutinee_ty: &Type,
        env: &TypeEnv,
    ) -> Result<TypeEnv, TypeError> {
        match pattern {
            // Wildcard matches anything, no bindings
            Pattern::Wildcard => Ok(env.clone()),

            // Variable binds the scrutinee value
            Pattern::Var(name) => {
                let scheme = TypeScheme::mono(scrutinee_ty.clone());
                Ok(env.extend(name.clone(), scheme))
            }

            // Literal must match scrutinee type exactly
            Pattern::Literal(lit) => {
                let lit_type = self.infer_literal(lit);
                self.add_constraint(Constraint::Equal(scrutinee_ty.clone(), lit_type));
                Ok(env.clone())
            }

            // Tuple pattern
            Pattern::Tuple(patterns) => {
                // Scrutinee must be a tuple with matching arity
                let mut pattern_types = Vec::new();
                for _ in patterns {
                    pattern_types.push(Type::Var(self.fresh_var()));
                }

                let expected_tuple = Type::Tuple(pattern_types.clone());
                self.add_constraint(Constraint::Equal(scrutinee_ty.clone(), expected_tuple));

                // Process each sub-pattern
                let mut extended_env = env.clone();
                for (pattern, pattern_type) in patterns.iter().zip(pattern_types.iter()) {
                    extended_env = self.infer_pattern(pattern, pattern_type, &extended_env)?;
                }

                Ok(extended_env)
            }

            // Variant pattern
            Pattern::Variant { variant, patterns } => {
                // Create types for variant fields
                let mut field_types = Vec::new();
                for _ in patterns {
                    field_types.push(Type::Var(self.fresh_var()));
                }

                let expected_variant = Type::Variant(variant.clone(), field_types.clone());
                self.add_constraint(Constraint::Equal(scrutinee_ty.clone(), expected_variant));

                // Process field patterns
                let mut extended_env = env.clone();
                for (pattern, field_type) in patterns.iter().zip(field_types.iter()) {
                    extended_env = self.infer_pattern(pattern, field_type, &extended_env)?;
                }

                Ok(extended_env)
            }
        }
    }

    /// Solve all accumulated constraints using unification.
    ///
    /// Returns a substitution that satisfies all constraints.
    pub fn solve_constraints(&mut self) -> Result<Substitution, TypeError> {
        let mut subst = Substitution::empty();

        for constraint in &self.constraints {
            match constraint {
                Constraint::Equal(t1, t2) => {
                    // Apply current substitution to both sides
                    let t1_subst = t1.apply(&subst);
                    let t2_subst = t2.apply(&subst);

                    // Unify and compose substitutions
                    let new_subst = self.unify(&t1_subst, &t2_subst)?;
                    subst = Substitution::compose(&new_subst, &subst);
                }
            }
        }

        Ok(subst)
    }

    /// Unify two types using Robinson's unification algorithm.
    ///
    /// Returns a substitution that makes the types equal, or an error if unification fails.
    #[allow(clippy::only_used_in_recursion)]
    pub fn unify(&self, t1: &Type, t2: &Type) -> Result<Substitution, TypeError> {
        match (t1, t2) {
            // Identical types unify trivially
            (Type::Int, Type::Int)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String)
            | (Type::Unit, Type::Unit)
            | (Type::Float, Type::Float) => Ok(Substitution::empty()),

            // Same type variable
            (Type::Var(v1), Type::Var(v2)) if v1 == v2 => Ok(Substitution::empty()),

            // Type variable unifies with any type (with occurs check)
            (Type::Var(v), t) | (t, Type::Var(v)) => {
                if t.occurs_check(v) {
                    Err(TypeError::new(TypeErrorKind::OccursCheck {
                        var: v.clone(),
                        in_type: t.clone(),
                    }))
                } else {
                    Ok(Substitution::singleton(v.clone(), t.clone()))
                }
            }

            // Function types unify if domain and codomain unify
            (Type::Function(a1, r1), Type::Function(a2, r2)) => {
                let subst1 = self.unify(a1, a2)?;
                let r1_subst = r1.apply(&subst1);
                let r2_subst = r2.apply(&subst1);
                let subst2 = self.unify(&r1_subst, &r2_subst)?;
                Ok(Substitution::compose(&subst2, &subst1))
            }

            // Tuple types unify if they have the same arity and elements unify
            (Type::Tuple(ts1), Type::Tuple(ts2)) => {
                if ts1.len() != ts2.len() {
                    return Err(TypeError::new(TypeErrorKind::Mismatch {
                        expected: t1.clone(),
                        got: t2.clone(),
                    }));
                }

                let mut subst = Substitution::empty();
                for (ty1, ty2) in ts1.iter().zip(ts2.iter()) {
                    let ty1_subst = ty1.apply(&subst);
                    let ty2_subst = ty2.apply(&subst);
                    let new_subst = self.unify(&ty1_subst, &ty2_subst)?;
                    subst = Substitution::compose(&new_subst, &subst);
                }
                Ok(subst)
            }

            // List types unify if element types unify
            (Type::List(t1), Type::List(t2)) => self.unify(t1, t2),

            // Array types unify if element types unify
            (Type::Array(t1), Type::Array(t2)) => self.unify(t1, t2),

            // Record types unify if they have the same fields with unifying types
            (Type::Record(fields1), Type::Record(fields2)) => {
                if fields1.len() != fields2.len() {
                    return Err(TypeError::new(TypeErrorKind::Mismatch {
                        expected: t1.clone(),
                        got: t2.clone(),
                    }));
                }

                let mut subst = Substitution::empty();
                for (name, ty1) in fields1 {
                    match fields2.get(name) {
                        Some(ty2) => {
                            let ty1_subst = ty1.apply(&subst);
                            let ty2_subst = ty2.apply(&subst);
                            let new_subst = self.unify(&ty1_subst, &ty2_subst)?;
                            subst = Substitution::compose(&new_subst, &subst);
                        }
                        None => {
                            return Err(TypeError::new(TypeErrorKind::FieldNotFound {
                                record_type: t1.clone(),
                                field: name.clone(),
                            }));
                        }
                    }
                }
                Ok(subst)
            }

            // Variant types unify if same variant name and field types unify
            (Type::Variant(name1, fields1), Type::Variant(name2, fields2)) => {
                if name1 != name2 {
                    return Err(TypeError::new(TypeErrorKind::Mismatch {
                        expected: t1.clone(),
                        got: t2.clone(),
                    }));
                }

                if fields1.len() != fields2.len() {
                    return Err(TypeError::new(TypeErrorKind::Mismatch {
                        expected: t1.clone(),
                        got: t2.clone(),
                    }));
                }

                let mut subst = Substitution::empty();
                for (ty1, ty2) in fields1.iter().zip(fields2.iter()) {
                    let ty1_subst = ty1.apply(&subst);
                    let ty2_subst = ty2.apply(&subst);
                    let new_subst = self.unify(&ty1_subst, &ty2_subst)?;
                    subst = Substitution::compose(&new_subst, &subst);
                }
                Ok(subst)
            }

            // All other cases are type mismatches
            _ => Err(TypeError::new(TypeErrorKind::Mismatch {
                expected: t1.clone(),
                got: t2.clone(),
            })),
        }
    }

    /// Convenience method: infer type and solve constraints in one step.
    ///
    /// This is the main entry point for type checking.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fusabi_frontend::inference::TypeInference;
    /// use fusabi_frontend::types::TypeEnv;
    /// use fusabi_frontend::ast::{Expr, Literal};
    ///
    /// let mut inference = TypeInference::new();
    /// let env = TypeEnv::new();
    /// let expr = Expr::Lit(Literal::Int(42));
    ///
    /// let ty = inference.infer_and_solve(&expr, &env).unwrap();
    /// ```
    pub fn infer_and_solve(&mut self, expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
        // Clear any previous constraints
        self.constraints.clear();

        // Infer the type (generating constraints)
        let ty = self.infer(expr, env)?;

        // Solve the constraints
        let subst = self.solve_constraints()?;

        // Apply the substitution to the result type
        Ok(ty.apply(&subst))
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create simple test expressions
    fn lit_int(n: i64) -> Expr {
        Expr::Lit(Literal::Int(n))
    }

    fn var(name: &str) -> Expr {
        Expr::Var(name.to_string())
    }

    fn lambda(param: &str, body: Expr) -> Expr {
        Expr::Lambda {
            param: param.to_string(),
            body: Box::new(body),
        }
    }

    fn app(func: Expr, arg: Expr) -> Expr {
        Expr::App {
            func: Box::new(func),
            arg: Box::new(arg),
        }
    }

    fn let_expr(name: &str, value: Expr, body: Expr) -> Expr {
        Expr::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    // ========================================================================
    // Basic Inference Tests
    // ========================================================================

    #[test]
    fn test_infer_literal_int() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        let expr = lit_int(42);

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_infer_literal_bool() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        let expr = Expr::Lit(Literal::Bool(true));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_infer_identity_function() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        // fun x -> x
        let expr = lambda("x", var("x"));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        // Should be 'a -> 'a (with some type variable)
        match ty {
            Type::Function(arg, ret) => match (*arg, *ret) {
                (Type::Var(v1), Type::Var(v2)) => assert_eq!(v1, v2),
                _ => panic!("Expected function with type variables"),
            },
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_infer_const_function() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        // fun x -> 42
        let expr = lambda("x", lit_int(42));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        // Should be 'a -> int
        match ty {
            Type::Function(_, ret) => assert_eq!(*ret, Type::Int),
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_infer_application() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        // (fun x -> x) 42
        let expr = app(lambda("x", var("x")), lit_int(42));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_infer_unbound_variable() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();
        let expr = var("x");

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_err());
        match result.unwrap_err().kind {
            TypeErrorKind::UnboundVariable { name } => assert_eq!(name, "x"),
            _ => panic!("Expected UnboundVariable error"),
        }
    }

    // ========================================================================
    // Auto-Recursive Detection Tests (Issue #126)
    // ========================================================================

    #[test]
    fn test_auto_recursive_lambda_factorial() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();

        // let factorial = fun n ->
        //     if n <= 1 then 1
        //     else n * factorial (n - 1)
        // in factorial 5
        let cond = Expr::BinOp {
            op: BinOp::Lte,
            left: Box::new(var("n")),
            right: Box::new(lit_int(1)),
        };
        let then_branch = lit_int(1);
        let else_branch = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(var("n")),
            right: Box::new(app(
                var("factorial"),
                Expr::BinOp {
                    op: BinOp::Sub,
                    left: Box::new(var("n")),
                    right: Box::new(lit_int(1)),
                },
            )),
        };
        let factorial_body = Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        };
        let factorial_lambda = lambda("n", factorial_body);
        let expr = let_expr(
            "factorial",
            factorial_lambda,
            app(var("factorial"), lit_int(5)),
        );

        // Should successfully infer type without needing explicit 'rec'
        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_auto_recursive_simple() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();

        // let f = fun x -> f x in f 42
        let f_body = app(var("f"), var("x"));
        let f_lambda = lambda("x", f_body);
        let expr = let_expr("f", f_lambda, app(var("f"), lit_int(42)));

        // Should infer type (may be polymorphic)
        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_recursive_lambda_still_works() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();

        // let double = fun x -> x * 2 in double 21
        let double_body = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(var("x")),
            right: Box::new(lit_int(2)),
        };
        let double_lambda = lambda("x", double_body);
        let expr = let_expr("double", double_lambda, app(var("double"), lit_int(21)));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn test_shadowing_prevents_auto_recursion() {
        let mut inf = TypeInference::new();
        let env = TypeEnv::new();

        // let f = fun f -> f in f 42
        // The parameter 'f' shadows the binding name, so this is not recursive
        // f is the identity function: 'a -> 'a
        // When applied to 42, it returns 42
        let f_lambda = lambda("f", var("f"));
        let expr = let_expr("f", f_lambda, app(var("f"), lit_int(42)));

        let ty = inf.infer_and_solve(&expr, &env).unwrap();
        // Result type should be int (identity function applied to int gives int)
        assert_eq!(ty, Type::Int);
    }

    // ========================================================================
    // Field Validation Tests (Issue #249)
    // ========================================================================

    #[test]
    fn test_record_literal_valid_fields() {
        use crate::ast::{RecordTypeDef, TypeExpr as AstTypeExpr};
        use crate::modules::{ModuleRegistry, TypeDefinition};

        let mut registry = ModuleRegistry::new();

        // Define a Person record type
        let person_type = TypeDefinition::Record(RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), AstTypeExpr::Named("string".to_string())),
                ("age".to_string(), AstTypeExpr::Named("int".to_string())),
            ],
        });

        let mut types = HashMap::new();
        types.insert("Person".to_string(), person_type);
        registry.register_module("Test".to_string(), HashMap::new(), types);

        let mut inf = TypeInference::with_module_registry(registry);
        let env = TypeEnv::new();

        // Valid record literal with all fields
        let expr = Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), Box::new(Expr::Lit(Literal::Str("Alice".to_string())))),
                ("age".to_string(), Box::new(lit_int(30))),
            ],
        };

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_record_literal_extra_field() {
        use crate::ast::{RecordTypeDef, TypeExpr as AstTypeExpr};
        use crate::modules::{ModuleRegistry, TypeDefinition};

        let mut registry = ModuleRegistry::new();

        // Define a Person record type
        let person_type = TypeDefinition::Record(RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), AstTypeExpr::Named("string".to_string())),
                ("age".to_string(), AstTypeExpr::Named("int".to_string())),
            ],
        });

        let mut types = HashMap::new();
        types.insert("Person".to_string(), person_type);
        registry.register_module("Test".to_string(), HashMap::new(), types);

        let mut inf = TypeInference::with_module_registry(registry);
        let env = TypeEnv::new();

        // Record literal with extra field "email"
        let expr = Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), Box::new(Expr::Lit(Literal::Str("Alice".to_string())))),
                ("age".to_string(), Box::new(lit_int(30))),
                ("email".to_string(), Box::new(Expr::Lit(Literal::Str("alice@example.com".to_string())))),
            ],
        };

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_err());

        match result.unwrap_err().kind {
            TypeErrorKind::ExtraField { field, .. } => {
                assert_eq!(field, "email");
            }
            _ => panic!("Expected ExtraField error"),
        }
    }

    #[test]
    fn test_record_literal_typo_suggestion() {
        use crate::ast::{RecordTypeDef, TypeExpr as AstTypeExpr};
        use crate::modules::{ModuleRegistry, TypeDefinition};

        let mut registry = ModuleRegistry::new();

        // Define a Person record type
        let person_type = TypeDefinition::Record(RecordTypeDef {
            name: "Person".to_string(),
            fields: vec![
                ("name".to_string(), AstTypeExpr::Named("string".to_string())),
                ("age".to_string(), AstTypeExpr::Named("int".to_string())),
            ],
        });

        let mut types = HashMap::new();
        types.insert("Person".to_string(), person_type);
        registry.register_module("Test".to_string(), HashMap::new(), types);

        let mut inf = TypeInference::with_module_registry(registry);
        let env = TypeEnv::new();

        // Record literal with typo "nam" instead of "name"
        let expr = Expr::RecordLiteral {
            type_name: "Person".to_string(),
            fields: vec![
                ("nam".to_string(), Box::new(Expr::Lit(Literal::Str("Alice".to_string())))),
                ("age".to_string(), Box::new(lit_int(30))),
            ],
        };

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_err());

        match result.unwrap_err().kind {
            TypeErrorKind::ExtraField { field, suggestions, .. } => {
                assert_eq!(field, "nam");
                assert!(suggestions.contains(&"name".to_string()));
            }
            _ => panic!("Expected ExtraField error with suggestions"),
        }
    }

    #[test]
    fn test_record_literal_unknown_type() {
        let registry = ModuleRegistry::new();
        let mut inf = TypeInference::with_module_registry(registry);
        let env = TypeEnv::new();

        // Record literal with unknown type
        let expr = Expr::RecordLiteral {
            type_name: "UnknownType".to_string(),
            fields: vec![
                ("field1".to_string(), Box::new(lit_int(42))),
            ],
        };

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_err());

        match result.unwrap_err().kind {
            TypeErrorKind::UnknownType { type_name } => {
                assert_eq!(type_name, "UnknownType");
            }
            _ => panic!("Expected UnknownType error"),
        }
    }

    #[test]
    fn test_record_literal_without_type_name_skips_validation() {
        // When type_name is empty, validation should be skipped
        let registry = ModuleRegistry::new();
        let mut inf = TypeInference::with_module_registry(registry);
        let env = TypeEnv::new();

        // Record literal without type annotation
        let expr = Expr::RecordLiteral {
            type_name: String::new(),
            fields: vec![
                ("field1".to_string(), Box::new(lit_int(42))),
                ("field2".to_string(), Box::new(Expr::Lit(Literal::Str("hello".to_string())))),
            ],
        };

        let result = inf.infer_and_solve(&expr, &env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(TypeInference::levenshtein_distance("", ""), 0);
        assert_eq!(TypeInference::levenshtein_distance("hello", "hello"), 0);
        assert_eq!(TypeInference::levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(TypeInference::levenshtein_distance("nam", "name"), 1);
        assert_eq!(TypeInference::levenshtein_distance("age", "aeg"), 2); // swap is 2 edits
        assert_eq!(TypeInference::levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_field_suggestions() {
        let expected = vec!["name".to_string(), "age".to_string(), "email".to_string()];

        // Exact match should still be suggested
        let suggestions = TypeInference::compute_field_suggestions("name", &expected);
        assert!(suggestions.contains(&"name".to_string()));

        // Close typo should be suggested
        let suggestions = TypeInference::compute_field_suggestions("nam", &expected);
        assert!(suggestions.contains(&"name".to_string()));

        // Multiple close matches
        let suggestions = TypeInference::compute_field_suggestions("ema", &expected);
        assert!(suggestions.contains(&"email".to_string()) || suggestions.contains(&"name".to_string()));

        // Too far should return empty or distant matches
        let suggestions = TypeInference::compute_field_suggestions("xyz", &expected);
        assert!(suggestions.is_empty() || suggestions.len() <= 3);
    }
}
