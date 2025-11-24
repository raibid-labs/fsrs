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

use crate::ast::{BinOp, Expr, Literal, MatchArm, Pattern};
use crate::error::{TypeError, TypeErrorKind};
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
}

#[allow(clippy::result_large_err)]
impl TypeInference {
    /// Create a new type inference instance.
    pub fn new() -> Self {
        TypeInference {
            next_var_id: 0,
            constraints: Vec::new(),
        }
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

    /// Infer the type of a let-binding.
    ///
    /// For `let x = value in body`:
    /// 1. Infer the type t1 of value
    /// 2. Generalize t1 to a type scheme (for polymorphism)
    /// 3. Extend environment with x: scheme
    /// 4. Infer and return the type of body
    ///
    /// For recursive bindings, we add x to the environment before inferring value.
    fn infer_let(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
        env: &TypeEnv,
        is_recursive: bool,
    ) -> Result<Type, TypeError> {
        let value_type = if is_recursive {
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
        _type_name: &str,
        fields: &[(String, Box<Expr>)],
        env: &TypeEnv,
    ) -> Result<Type, TypeError> {
        let mut field_types = HashMap::new();

        for (field_name, field_expr) in fields {
            let field_type = self.infer(field_expr, env)?;
            field_types.insert(field_name.clone(), field_type);
        }

        Ok(Type::Record(field_types))
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
}
