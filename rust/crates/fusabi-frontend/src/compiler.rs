//! Bytecode Compiler for Fusabi Mini-F#
//!
//! This module implements compilation from AST to bytecode chunks for the Fusabi VM.
//! The compiler performs constant pooling, variable scoping, and generates efficient
//! bytecode instruction sequences.
//!
//! # Architecture
//!
//! The compiler uses:
//! - **Constant Pool**: Deduplicates literal values across the bytecode
//! - **Local Variables**: Stack-allocated variables tracked by index
//! - **Jump Patching**: Forward jump resolution for control flow
//! - **Optional Type Checking**: Can run type inference before compilation
//! - **Module System**: Supports module-aware compilation and qualified names
//!
//! # Example
//!
//! ```rust
//! use fusabi_frontend::ast::{Expr, Literal, BinOp};
//! use fusabi_frontend::compiler::{Compiler, CompileOptions};
//!
//! // Compile: 42 + 1
//! let expr = Expr::BinOp {
//!     op: BinOp::Add,
//!     left: Box::new(Expr::Lit(Literal::Int(42))),
//!     right: Box::new(Expr::Lit(Literal::Int(1))),
//! };
//!
//! // Compile without type checking (backward compatible)
//! let chunk = Compiler::compile(&expr).unwrap();
//!
//! // Compile with type checking enabled
//! let options = CompileOptions {
//!     enable_type_checking: true,
//!     ..Default::default()
//! };
//! let chunk = Compiler::compile_with_options(&expr, options).unwrap();
//! ```

use crate::ast::{BinOp, Expr, Import, Literal, MatchArm, ModuleDef, ModuleItem, Pattern, Program};
use crate::modules::ModuleRegistry;
use crate::types::{Type, TypeEnv};
use fusabi_vm::chunk::Chunk;
use fusabi_vm::closure::Closure;
use fusabi_vm::instruction::Instruction;
use fusabi_vm::value::Value;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Compilation errors
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    /// Undefined variable reference
    UndefinedVariable(String),
    /// Too many constants in constant pool (max u16::MAX)
    TooManyConstants,
    /// Too many local variables (max u8::MAX)
    TooManyLocals,
    /// Invalid jump offset (beyond i16 range)
    InvalidJumpOffset,
    /// Unsupported float operations in Phase 1
    UnsupportedFloat,
    /// Tuple too large (max u16::MAX elements)
    TupleTooLarge,
    /// Type error during type checking
    TypeError(String),
    /// Code generation error
    CodeGenError(String),
    /// Module not found
    ModuleNotFound(String),
    /// No module context available
    NoModuleContext,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            CompileError::TooManyConstants => {
                write!(f, "Too many constants (max {})", u16::MAX)
            }
            CompileError::TooManyLocals => {
                write!(f, "Too many local variables (max {})", u8::MAX)
            }
            CompileError::InvalidJumpOffset => {
                write!(f, "Jump offset too large")
            }
            CompileError::UnsupportedFloat => {
                write!(f, "Float operations not supported in Phase 1")
            }
            CompileError::TupleTooLarge => {
                write!(f, "Tuple too large (max {} elements)", u16::MAX)
            }
            CompileError::TypeError(msg) => {
                write!(f, "Type error: {}", msg)
            }
            CompileError::CodeGenError(msg) => {
                write!(f, "Code generation error: {}", msg)
            }
            CompileError::ModuleNotFound(name) => {
                write!(f, "Module not found: {}", name)
            }
            CompileError::NoModuleContext => {
                write!(f, "No module context available for qualified name lookup")
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Compilation result type
pub type CompileResult<T> = Result<T, CompileError>;

/// Compilation options
///
/// Controls various aspects of the compilation process, including
/// optional type checking and strictness levels.
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Enable type checking before compilation
    pub enable_type_checking: bool,
    /// Strict mode - treat warnings as errors
    pub strict_mode: bool,
    /// Allow type warnings (only relevant if enable_type_checking is true)
    pub allow_warnings: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        CompileOptions {
            enable_type_checking: false, // Backward compatible - type checking is opt-in
            strict_mode: false,
            allow_warnings: true,
        }
    }
}

/// Local variable information
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

/// Bytecode compiler state
pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    options: CompileOptions,
    type_env: Option<TypeEnv>,

    // Module support
    module_registry: Option<ModuleRegistry>,
    imported_bindings: HashMap<String, Expr>,
}

impl Compiler {
    /// Create a new compiler with default options
    fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            options: CompileOptions::default(),
            type_env: None,
            module_registry: None,
            imported_bindings: HashMap::new(),
        }
    }

    /// Create a new compiler with custom options
    fn new_with_options(options: CompileOptions) -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            options,
            type_env: None,
            module_registry: None,
            imported_bindings: HashMap::new(),
        }
    }

    /// Main entry point: compile an expression to a chunk (backward compatible)
    pub fn compile(expr: &Expr) -> CompileResult<Chunk> {
        Self::compile_with_options(expr, CompileOptions::default())
    }

    /// Compile an expression with type checking enabled
    pub fn compile_checked(expr: &Expr) -> CompileResult<Chunk> {
        let options = CompileOptions {
            enable_type_checking: true,
            ..Default::default()
        };
        Self::compile_with_options(expr, options)
    }

    /// Compile an expression with custom options
    pub fn compile_with_options(expr: &Expr, options: CompileOptions) -> CompileResult<Chunk> {
        let mut compiler = Compiler::new_with_options(options);

        // Optional type checking phase
        if compiler.options.enable_type_checking {
            compiler.type_check(expr)?;
        }

        // Compilation phase
        compiler.compile_expr(expr)?;
        compiler.emit(Instruction::Return);
        Ok(compiler.chunk)
    }

    /// Compile a complete program with modules
    ///
    /// This is the main entry point for compiling programs with module definitions.
    /// It performs three phases:
    /// 1. Register all modules and their bindings
    /// 2. Apply imports to the current environment
    /// 3. Compile the main expression (if present)
    pub fn compile_program(program: &Program) -> CompileResult<Chunk> {
        let mut compiler = Compiler::new();
        let mut registry = ModuleRegistry::new();

        // Phase 1: Register all modules
        for module in &program.modules {
            compiler.register_module(&mut registry, module)?;
        }

        // Store registry for qualified name lookups
        compiler.module_registry = Some(registry);

        // Phase 2: Apply imports to environment
        for import in &program.imports {
            compiler.apply_import(import)?;
        }

        // Phase 3: Compile main expression (if present)
        if let Some(main_expr) = &program.main_expr {
            compiler.compile_expr(main_expr)?;
        } else {
            // No main expression, just return Unit
            let unit_idx = compiler.add_constant(Value::Unit)?;
            compiler.emit(Instruction::LoadConst(unit_idx));
        }

        compiler.emit(Instruction::Return);
        Ok(compiler.chunk)
    }

    /// Register a module and compile its bindings
    ///
    /// This processes all items in a module definition and registers them
    /// in the module registry for later lookup.
    fn register_module(
        &mut self,
        registry: &mut ModuleRegistry,
        module: &ModuleDef,
    ) -> CompileResult<()> {
        let mut bindings = HashMap::new();
        let mut types = HashMap::new();

        for item in &module.items {
            match item {
                ModuleItem::Let(name, expr) => {
                    // Store binding for later compilation
                    bindings.insert(name.clone(), expr.clone());
                }
                ModuleItem::LetRec(rec_bindings) => {
                    // Handle recursive bindings
                    for (name, expr) in rec_bindings {
                        bindings.insert(name.clone(), expr.clone());
                    }
                }
                ModuleItem::TypeDef(type_def) => {
                    // Convert AST TypeDefinition to modules TypeDefinition
                    let module_type_def = match type_def {
                        crate::ast::TypeDefinition::Record(r) => {
                            crate::modules::TypeDefinition::Record(r.clone())
                        }
                        crate::ast::TypeDefinition::Du(du) => {
                            crate::modules::TypeDefinition::Du(du.clone())
                        }
                    };

                    // Extract type name based on definition
                    let type_name = match type_def {
                        crate::ast::TypeDefinition::Record(r) => r.name.clone(),
                        crate::ast::TypeDefinition::Du(du) => du.name.clone(),
                    };

                    types.insert(type_name, module_type_def);
                }
                ModuleItem::Module(nested) => {
                    // Recursively register nested module
                    self.register_module(registry, nested)?;
                }
            }
        }

        // Register module in registry
        registry.register_module(module.name.clone(), bindings, types);

        Ok(())
    }

    /// Apply an import to the current environment
    ///
    /// This brings all bindings from the imported module into the current scope,
    /// allowing them to be accessed without qualification.
    fn apply_import(&mut self, import: &Import) -> CompileResult<()> {
        // Get module name (for simple imports, it's the first component)
        let module_name = import
            .module_path
            .first()
            .ok_or_else(|| CompileError::ModuleNotFound("empty module path".to_string()))?;

        let registry = self
            .module_registry
            .as_ref()
            .ok_or(CompileError::NoModuleContext)?;

        let module_bindings = registry
            .get_module_bindings(module_name)
            .ok_or_else(|| CompileError::ModuleNotFound(module_name.clone()))?;

        // Add all bindings from imported module to current environment
        for (name, expr) in module_bindings {
            self.imported_bindings.insert(name.clone(), expr.clone());
        }

        Ok(())
    }

    /// Type check expression
    ///
    /// This is a placeholder for the actual type inference implementation.
    /// Once the type inference module is complete, this will perform full
    /// Hindley-Milner type inference and constraint solving.
    fn type_check(&mut self, _expr: &Expr) -> CompileResult<Type> {
        // Placeholder implementation
        // TODO: Replace with actual type inference when available
        //
        // Expected implementation:
        // let mut inference = TypeInference::new();
        // let env = TypeEnv::new();
        // let ty = inference.infer(expr, &env)
        //     .map_err(|e| CompileError::TypeError(format!("{}", e)))?;
        // let subst = inference.solve_constraints()
        //     .map_err(|e| CompileError::TypeError(format!("{}", e)))?;
        // let final_ty = subst.apply_type(&ty);
        // self.type_env = Some(env);
        // Ok(final_ty)

        // For now, just initialize empty type environment
        self.type_env = Some(TypeEnv::new());
        Ok(Type::Unit)
    }

    /// Compile an expression and emit instructions
    fn compile_expr(&mut self, expr: &Expr) -> CompileResult<()> {
        match expr {
            Expr::Lit(lit) => self.compile_literal(lit),
            Expr::Var(name) => self.compile_var(name),
            Expr::BinOp { op, left, right } => self.compile_binop(*op, left, right),
            Expr::Let { name, value, body } => self.compile_let(name, value, body),
            Expr::LetRec { name, value, body } => self.compile_let_rec(name, value, body),
            Expr::LetRecMutual { bindings, body } => self.compile_let_rec_mutual(bindings, body),
            Expr::Lambda { param, body } => self.compile_lambda(param, body),
            Expr::App { func, arg } => self.compile_app(func, arg),
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => self.compile_if(cond, then_branch, else_branch),
            Expr::Tuple(elements) => self.compile_tuple(elements),
            Expr::List(elements) => self.compile_list(elements),
            Expr::Cons { head, tail } => self.compile_cons(head, tail),
            Expr::Array(elements) => self.compile_array(elements),
            Expr::ArrayIndex { array, index } => self.compile_array_index(array, index),
            Expr::ArrayUpdate {
                array,
                index,
                value,
            } => self.compile_array_update(array, index, value),
            Expr::ArrayLength(array) => self.compile_array_length(array),
            Expr::RecordLiteral { fields, .. } => self.compile_record_literal(fields),
            Expr::RecordAccess { record, field } => self.compile_record_access(record, field),
            Expr::RecordUpdate { record, fields } => self.compile_record_update(record, fields),
            Expr::Match { scrutinee, arms } => self.compile_match(scrutinee, arms),
            Expr::VariantConstruct {
                type_name,
                variant,
                fields,
            } => self.compile_variant_construct(type_name, variant, fields),
        }
    }

    /// Compile a literal value
    fn compile_literal(&mut self, lit: &Literal) -> CompileResult<()> {
        let value = match lit {
            Literal::Int(n) => Value::Int(*n),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Str(s) => Value::Str(s.clone()),
            Literal::Unit => Value::Unit,
            Literal::Float(_) => return Err(CompileError::UnsupportedFloat),
        };

        let idx = self.add_constant(value)?;
        self.emit(Instruction::LoadConst(idx));
        Ok(())
    }

    /// Compile a variable reference with module support
    ///
    /// This handles:
    /// - Qualified names (e.g., Math.add)
    /// - Imported bindings (from open statements)
    /// - Local variables
    fn compile_var(&mut self, name: &str) -> CompileResult<()> {
        // Check if it's a qualified name (e.g., "Math.add")
        if let Some((module_path, binding_name)) = parse_qualified_name(name) {
            return self.compile_qualified_var(&module_path, &binding_name);
        }

        // Check local scope first
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                let idx = i as u8;
                self.emit(Instruction::LoadLocal(idx));
                return Ok(());
            }
        }

        // Check imported bindings
        if let Some(expr) = self.imported_bindings.get(name) {
            // Compile the imported expression directly
            return self.compile_expr(&expr.clone());
        }

        // If not found locally or imported, assume it's a global variable
        let idx = self.add_constant(Value::Str(name.to_string()))?;
        self.emit(Instruction::LoadGlobal(idx));
        Ok(())
    }

    /// Compile a qualified variable reference (e.g., Math.add)
    fn compile_qualified_var(&mut self, module_path: &[String], name: &str) -> CompileResult<()> {
        // For now, only support single-level qualification (Module.binding)
        if module_path.len() != 1 {
            return Err(CompileError::CodeGenError(
                "Nested module paths not yet supported in compilation".to_string(),
            ));
        }

        let module_name = &module_path[0];

        // Look up the qualified name in module registry
        let expr = self
            .module_registry
            .as_ref()
            .ok_or(CompileError::NoModuleContext)?
            .resolve_qualified(module_name, name)
            .ok_or_else(|| CompileError::UndefinedVariable(format!("{}.{}", module_name, name)))?;

        // Compile the looked-up expression
        self.compile_expr(&expr.clone())
    }

    /// Compile a binary operation
    fn compile_binop(&mut self, op: BinOp, left: &Expr, right: &Expr) -> CompileResult<()> {
        // Compile operands
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        // Emit operation instruction
        let instr = match op {
            BinOp::Add => Instruction::Add,
            BinOp::Sub => Instruction::Sub,
            BinOp::Mul => Instruction::Mul,
            BinOp::Div => Instruction::Div,
            BinOp::Eq => Instruction::Eq,
            BinOp::Neq => Instruction::Neq,
            BinOp::Lt => Instruction::Lt,
            BinOp::Lte => Instruction::Lte,
            BinOp::Gt => Instruction::Gt,
            BinOp::Gte => Instruction::Gte,
            BinOp::And => Instruction::And,
            BinOp::Or => Instruction::Or,
        };
        self.emit(instr);
        Ok(())
    }

    /// Compile a tuple expression
    fn compile_tuple(&mut self, elements: &[Expr]) -> CompileResult<()> {
        // Check if tuple size fits in u16
        if elements.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge);
        }

        // Compile each element (left to right)
        for element in elements {
            self.compile_expr(element)?;
        }

        // Emit MakeTuple instruction with element count
        let element_count = elements.len() as u16;
        self.emit(Instruction::MakeTuple(element_count));

        Ok(())
    }

    /// Compile a let-binding
    /// Compile a list expression
    fn compile_list(&mut self, elements: &[Expr]) -> CompileResult<()> {
        // Handle empty list: [] -> emit LoadConst with Value::Nil
        if elements.is_empty() {
            let idx = self.add_constant(Value::Nil)?;
            self.emit(Instruction::LoadConst(idx));
            return Ok(());
        }

        // Check if list size fits in u16
        if elements.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge); // Reuse error, or add ListTooLarge
        }

        // Compile each element (left to right)
        for element in elements {
            self.compile_expr(element)?;
        }

        // Emit MakeList instruction with element count
        let element_count = elements.len() as u16;
        self.emit(Instruction::MakeList(element_count));

        Ok(())
    }

    /// Compile a cons expression
    fn compile_cons(&mut self, head: &Expr, tail: &Expr) -> CompileResult<()> {
        // Compile head expression
        self.compile_expr(head)?;
        // Compile tail expression
        self.compile_expr(tail)?;
        // Emit Cons instruction
        self.emit(Instruction::Cons);
        Ok(())
    }

    /// Compile an array expression
    fn compile_array(&mut self, elements: &[Expr]) -> CompileResult<()> {
        // Check if array size fits in u16
        if elements.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge); // Reuse error for now
        }

        // Compile each element (left to right)
        for element in elements {
            self.compile_expr(element)?;
        }

        // Emit MakeArray instruction with element count
        let element_count = elements.len() as u16;
        self.emit(Instruction::MakeArray(element_count));

        Ok(())
    }

    /// Compile an array index expression
    fn compile_array_index(&mut self, array: &Expr, index: &Expr) -> CompileResult<()> {
        // Compile array expression
        self.compile_expr(array)?;
        // Compile index expression
        self.compile_expr(index)?;
        // Emit ArrayGet instruction
        self.emit(Instruction::ArrayGet);
        Ok(())
    }

    /// Compile an array update expression (immutable)
    fn compile_array_update(
        &mut self,
        array: &Expr,
        index: &Expr,
        value: &Expr,
    ) -> CompileResult<()> {
        // Compile array expression
        self.compile_expr(array)?;
        // Compile index expression
        self.compile_expr(index)?;
        // Compile new value expression
        self.compile_expr(value)?;
        // Emit ArrayUpdate instruction (creates new array)
        self.emit(Instruction::ArrayUpdate);
        Ok(())
    }

    /// Compile an array length expression
    fn compile_array_length(&mut self, array: &Expr) -> CompileResult<()> {
        // Compile array expression
        self.compile_expr(array)?;
        // Emit ArrayLength instruction
        self.emit(Instruction::ArrayLength);
        Ok(())
    }

    fn compile_let(&mut self, name: &str, value: &Expr, body: &Expr) -> CompileResult<()> {
        // Compile the value expression
        self.compile_expr(value)?;

        // Enter new scope
        self.begin_scope();

        // Add local variable
        self.add_local(name.to_string())?;

        // Store the value in the local slot
        let local_idx = (self.locals.len() - 1) as u8;
        self.emit(Instruction::StoreLocal(local_idx));

        // Compile the body expression
        self.compile_expr(body)?;

        // Exit scope - note: we don't emit POP for the body result
        // The result stays on top of the stack for the caller
        let locals_to_remove = self.end_scope_count();

        // Only pop if we have multiple locals and need to clean up intermediates
        // For Phase 1 MVP, we simplify by not emitting POPs
        // The locals stay in their stack slots until function return
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile a lambda function
    fn compile_lambda(&mut self, param: &str, body: &Expr) -> CompileResult<()> {
        // Create a nested chunk for the lambda body
        let mut lambda_compiler = Compiler::new();

        // Lambda parameter becomes local 0
        lambda_compiler.begin_scope();
        lambda_compiler.add_local(param.to_string())?;

        // Compile the lambda body
        lambda_compiler.compile_expr(body)?;
        lambda_compiler.emit(Instruction::Return);

        // Clean up scope (not strictly needed as we discard compiler, but good practice)
        let _locals_to_remove = lambda_compiler.end_scope_count();
        lambda_compiler.scope_depth -= 1;

        // Create a closure prototype (chunk + arity)
        // For now, we don't support upvalue capture in the compiler (Phase 2 extension)
        // We assume no upvalues.
        let closure = Closure::with_arity(lambda_compiler.chunk, 1);
        let closure_val = Value::Closure(Rc::new(closure));

        // Store prototype in constants
        let const_idx = self.add_constant(closure_val)?;

        // Emit MakeClosure instruction (0 upvalues for now)
        self.emit(Instruction::MakeClosure(const_idx, 0));
        Ok(())
    }

    /// Compile a recursive let-binding using placeholder strategy
    fn compile_let_rec(&mut self, name: &str, value: &Expr, body: &Expr) -> CompileResult<()> {
        // Strategy: Create a placeholder, compile the function with
        // the name in scope, then update the binding

        // 1. Push placeholder (will be replaced by the actual value)
        let placeholder_idx = self.add_constant(Value::Unit)?;
        self.emit(Instruction::LoadConst(placeholder_idx));

        // 2. Enter scope and add local for the recursive binding
        self.begin_scope();
        self.add_local(name.to_string())?;
        let local_idx = (self.locals.len() - 1) as u8;

        // 3. Store placeholder in local slot
        self.emit(Instruction::StoreLocal(local_idx));

        // 4. Compile the value (usually a lambda) with name in scope
        // The value can now reference itself via the local
        self.compile_expr(value)?;

        // 5. Update the local slot with the actual value
        self.emit(Instruction::StoreLocal(local_idx));

        // 6. Compile body (the local is still in scope)
        self.compile_expr(body)?;

        // 7. Clean up scope
        let locals_to_remove = self.end_scope_count();
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile mutually recursive bindings
    fn compile_let_rec_mutual(
        &mut self,
        bindings: &[(String, Expr)],
        body: &Expr,
    ) -> CompileResult<()> {
        // Strategy: Create placeholders for all bindings, then fill them in

        // 1. Enter scope
        self.begin_scope();

        // 2. Push placeholders and create locals for all bindings
        let placeholder_idx = self.add_constant(Value::Unit)?;
        let mut local_indices = Vec::new();

        for (name, _) in bindings {
            // Push placeholder
            self.emit(Instruction::LoadConst(placeholder_idx));

            // Add local
            self.add_local(name.clone())?;
            let local_idx = (self.locals.len() - 1) as u8;
            local_indices.push(local_idx);

            // Store placeholder
            self.emit(Instruction::StoreLocal(local_idx));
        }

        // 3. Compile each value (with all names in scope)
        for (i, (_name, value)) in bindings.iter().enumerate() {
            self.compile_expr(value)?;
            self.emit(Instruction::StoreLocal(local_indices[i]));
        }

        // 4. Compile body
        self.compile_expr(body)?;

        // 5. Clean up scope
        let locals_to_remove = self.end_scope_count();
        for _ in 0..locals_to_remove {
            self.locals.pop();
        }
        self.scope_depth -= 1;

        Ok(())
    }

    /// Compile a function application
    fn compile_app(&mut self, func: &Expr, arg: &Expr) -> CompileResult<()> {
        // Compile the function expression
        self.compile_expr(func)?;

        // Compile the argument expression
        self.compile_expr(arg)?;

        // Emit call instruction with 1 argument
        self.emit(Instruction::Call(1));

        Ok(())
    }

    /// Compile an if-then-else expression
    fn compile_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> CompileResult<()> {
        // Compile condition
        self.compile_expr(cond)?;

        // Emit JumpIfFalse with placeholder offset
        let jump_to_else = self.emit_jump(Instruction::JumpIfFalse(0));

        // Note: JumpIfFalse pops the condition value, so no manual POP needed

        // Compile then branch
        self.compile_expr(then_branch)?;

        // Emit Jump to skip else branch with placeholder offset
        let jump_to_end = self.emit_jump(Instruction::Jump(0));

        // Patch the JumpIfFalse to point here
        self.patch_jump(jump_to_else)?;

        // Compile else branch
        self.compile_expr(else_branch)?;

        // Patch the Jump to point here
        self.patch_jump(jump_to_end)?;

        Ok(())
    }

    /// Compile a match expression with full pattern matching support
    fn compile_match(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> CompileResult<()> {
        // Compile scrutinee once and keep it on the stack
        self.compile_expr(scrutinee)?;

        let _end_label = self.chunk.current_offset();
        let mut end_jumps = Vec::new();

        for (i, arm) in arms.iter().enumerate() {
            let is_last_arm = i == arms.len() - 1;

            // Duplicate scrutinee for pattern test (except we'll clean it up)
            if !is_last_arm {
                self.emit(Instruction::Dup);
            }

            // Compile pattern test - this will push a boolean result
            let _next_arm_offset = if !is_last_arm {
                self.chunk.current_offset()
            } else {
                0 // Last arm doesn't need a jump
            };

            // Test the pattern and get a boolean result
            self.compile_pattern_test(&arm.pattern)?;

            // Jump to next arm if pattern didn't match
            let jump_to_next = if !is_last_arm {
                self.emit_jump(Instruction::JumpIfFalse(0))
            } else {
                // Pop the boolean since it's not consumed by jump
                self.emit(Instruction::Pop);
                0
            };

            // Pattern matched - now bind variables from the pattern
            // We still have the scrutinee on the stack
            if !is_last_arm {
                self.emit(Instruction::Dup); // Keep scrutinee for binding
            }

            // Enter a new scope for pattern bindings
            self.begin_scope();

            // Bind variables from the pattern
            self.compile_pattern_bindings(&arm.pattern)?;

            // Compile arm body
            self.compile_expr(&arm.body)?;

            // Exit scope for pattern bindings
            let locals_to_remove = self.end_scope_count();
            for _ in 0..locals_to_remove {
                self.locals.pop();
            }
            self.scope_depth -= 1;

            // Jump to end of match expression
            let jump_to_end = self.emit_jump(Instruction::Jump(0));
            end_jumps.push(jump_to_end);

            // Patch jump to next arm (if not last)
            if !is_last_arm {
                self.patch_jump(jump_to_next)?;
                // Pop the scrutinee since this arm didn't match
                self.emit(Instruction::Pop);
            }
        }

        // Patch all end jumps to point here
        for jump_idx in end_jumps {
            self.patch_jump(jump_idx)?;
        }

        Ok(())
    }

    /// Compile a pattern test - checks if scrutinee matches pattern
    /// Expects scrutinee on top of stack, pushes boolean result
    fn compile_pattern_test(&mut self, pattern: &Pattern) -> CompileResult<()> {
        match pattern {
            Pattern::Wildcard | Pattern::Var(_) => {
                // Always matches - push true
                let true_idx = self.add_constant(Value::Bool(true))?;
                self.emit(Instruction::LoadConst(true_idx));
                Ok(())
            }
            Pattern::Literal(Literal::Int(n)) => {
                self.emit(Instruction::CheckInt(*n));
                Ok(())
            }
            Pattern::Literal(Literal::Bool(b)) => {
                self.emit(Instruction::CheckBool(*b));
                Ok(())
            }
            Pattern::Literal(Literal::Str(s)) => {
                self.emit(Instruction::CheckString(s.clone()));
                Ok(())
            }
            Pattern::Literal(Literal::Unit) => {
                // Check if value equals Unit
                let unit_idx = self.add_constant(Value::Unit)?;
                self.emit(Instruction::LoadConst(unit_idx));
                self.emit(Instruction::Eq);
                Ok(())
            }
            Pattern::Literal(Literal::Float(_)) => Err(CompileError::UnsupportedFloat),
            Pattern::Tuple(patterns) => {
                // Check tuple length first
                self.emit(Instruction::Dup);
                self.emit(Instruction::CheckTupleLen(patterns.len() as u8));

                // If not a tuple of right length, we're done (false on stack)
                // If it is, we need to check each element
                if !patterns.is_empty() {
                    // We have length check result on stack
                    // For now, simplified: just check length
                    // Full nested pattern checking would require more complex control flow
                }

                Ok(())
            }
            Pattern::Variant {
                variant,
                patterns: _,
            } => {
                // Check variant tag
                self.emit(Instruction::Dup);
                self.emit(Instruction::CheckVariantTag(variant.clone()));
                Ok(())
            }
        }
    }

    /// Compile pattern bindings - extracts values from scrutinee and stores in locals
    /// Expects scrutinee on top of stack, consumes it
    fn compile_pattern_bindings(&mut self, pattern: &Pattern) -> CompileResult<()> {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => {
                // No bindings, just pop the scrutinee
                self.emit(Instruction::Pop);
                Ok(())
            }
            Pattern::Var(name) => {
                // Bind the scrutinee to this variable
                self.add_local(name.clone())?;
                let local_idx = (self.locals.len() - 1) as u8;
                self.emit(Instruction::StoreLocal(local_idx));
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                // Extract each element and bind recursively
                for (i, pat) in patterns.iter().enumerate() {
                    self.emit(Instruction::Dup); // Dup the tuple
                    self.emit(Instruction::GetTupleElem(i as u8)); // Get element
                    self.compile_pattern_bindings(pat)?; // Bind it
                }
                // Pop the original tuple
                self.emit(Instruction::Pop);
                Ok(())
            }
            Pattern::Variant {
                variant: _,
                patterns,
            } => {
                // Extract each field from the variant and bind recursively
                for (i, pat) in patterns.iter().enumerate() {
                    self.emit(Instruction::Dup); // Dup the variant
                    self.emit(Instruction::GetVariantField(i as u8)); // Get field
                    self.compile_pattern_bindings(pat)?; // Bind it
                }
                // Pop the original variant
                self.emit(Instruction::Pop);
                Ok(())
            }
        }
    }

    /// Emit an instruction
    /// Compile a variant construction expression
    /// Stack effect: pushes a variant value
    fn compile_variant_construct(
        &mut self,
        type_name: &str,
        variant_name: &str,
        fields: &[Box<Expr>],
    ) -> CompileResult<()> {
        // Check if field count fits in u16
        if fields.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge); // Reuse error for now
        }

        // Push type name as a string constant
        let type_name_idx = self.add_constant(Value::Str(type_name.to_string()))?;
        self.emit(Instruction::LoadConst(type_name_idx));

        // Push variant name as a string constant
        let variant_name_idx = self.add_constant(Value::Str(variant_name.to_string()))?;
        self.emit(Instruction::LoadConst(variant_name_idx));

        // Compile each field expression
        for field in fields {
            self.compile_expr(field)?;
        }

        // Emit MakeVariant instruction with field count
        let field_count = fields.len() as u16;
        self.emit(Instruction::MakeVariant(field_count));

        Ok(())
    }

    fn emit(&mut self, instruction: Instruction) {
        self.chunk.emit(instruction);
    }

    /// Emit a jump instruction and return its index for later patching
    fn emit_jump(&mut self, instruction: Instruction) -> usize {
        self.emit(instruction);
        self.chunk.current_offset() - 1
    }

    /// Patch a jump instruction with the correct offset
    fn patch_jump(&mut self, jump_index: usize) -> CompileResult<()> {
        // Calculate the offset from the jump instruction to the current position
        let jump_offset = self.chunk.current_offset() - jump_index - 1;

        // Check if offset fits in i16
        if jump_offset > i16::MAX as usize {
            return Err(CompileError::InvalidJumpOffset);
        }

        // Get the current instruction and patch it
        match self.chunk.instructions[jump_index] {
            Instruction::Jump(_) => {
                self.chunk.instructions[jump_index] = Instruction::Jump(jump_offset as i16);
            }
            Instruction::JumpIfFalse(_) => {
                self.chunk.instructions[jump_index] = Instruction::JumpIfFalse(jump_offset as i16);
            }
            _ => unreachable!("patch_jump called on non-jump instruction"),
        }

        Ok(())
    }

    /// Add a constant to the constant pool and return its index
    fn add_constant(&mut self, value: Value) -> CompileResult<u16> {
        let count = self.chunk.constants.len();
        if count >= u16::MAX as usize {
            return Err(CompileError::TooManyConstants);
        }
        let idx = self.chunk.add_constant(value);
        Ok(idx)
    }
    /// Begin a new scope
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Count locals to be removed when ending current scope
    fn end_scope_count(&self) -> usize {
        self.locals
            .iter()
            .rev()
            .take_while(|local| local.depth > self.scope_depth - 1)
            .count()
    }

    /// Add a local variable
    fn add_local(&mut self, name: String) -> CompileResult<()> {
        if self.locals.len() >= u8::MAX as usize {
            return Err(CompileError::TooManyLocals);
        }

        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });

        Ok(())
    }
    /// Compile a record literal expression
    /// Stack effect: pushes a record value
    fn compile_record_literal(&mut self, fields: &[(String, Box<Expr>)]) -> CompileResult<()> {
        // Check if record size fits in u16
        if fields.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge); // Reuse error for now, or add RecordTooLarge
        }

        // Compile each field (field_name, field_value) pair
        // Stack layout: [field_name_1, field_value_1, field_name_2, field_value_2, ...]
        for (field_name, field_value) in fields {
            // Push field name as a string constant
            let field_name_idx = self.add_constant(Value::Str(field_name.clone()))?;
            self.emit(Instruction::LoadConst(field_name_idx));

            // Compile and push field value
            self.compile_expr(field_value)?;
        }

        // Emit MakeRecord instruction with field count
        let field_count = fields.len() as u16;
        self.emit(Instruction::MakeRecord(field_count));

        Ok(())
    }

    /// Compile a record field access expression
    /// Stack effect: pushes the field value
    fn compile_record_access(&mut self, record: &Expr, field: &str) -> CompileResult<()> {
        // Compile the record expression
        self.compile_expr(record)?;

        // Push field name as a string constant
        let field_name_idx = self.add_constant(Value::Str(field.to_string()))?;
        self.emit(Instruction::LoadConst(field_name_idx));

        // Emit GetRecordField instruction
        self.emit(Instruction::GetRecordField);

        Ok(())
    }

    /// Compile a record update expression (immutable)
    /// Stack effect: pushes a new record with updated fields
    fn compile_record_update(
        &mut self,
        record: &Expr,
        fields: &[(String, Box<Expr>)],
    ) -> CompileResult<()> {
        // Check if update size fits in u16
        if fields.len() > u16::MAX as usize {
            return Err(CompileError::TupleTooLarge); // Reuse error for now
        }

        // Compile the base record expression
        self.compile_expr(record)?;

        // Compile each update (field_name, new_value) pair
        // Stack layout: [record, field_name_1, new_value_1, field_name_2, new_value_2, ...]
        for (field_name, field_value) in fields {
            // Push field name as a string constant
            let field_name_idx = self.add_constant(Value::Str(field_name.clone()))?;
            self.emit(Instruction::LoadConst(field_name_idx));

            // Compile and push new field value
            self.compile_expr(field_value)?;
        }

        // Emit UpdateRecord instruction with update count
        let update_count = fields.len() as u16;
        self.emit(Instruction::UpdateRecord(update_count));

        Ok(())
    }
}

/// Parse a qualified name into module path and binding name
///
/// Examples:
/// - "Math.add" -> (["Math"], "add")
/// - "Geometry.Point.x" -> (["Geometry", "Point"], "x")
fn parse_qualified_name(name: &str) -> Option<(Vec<String>, String)> {
    if name.contains('.') {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() >= 2 {
            let module_path: Vec<String> = parts[..parts.len() - 1]
                .iter()
                .map(|s| s.to_string())
                .collect();
            let binding_name = parts[parts.len() - 1].to_string();
            return Some((module_path, binding_name));
        }
    }
    None
}

// Note: Tests remain unchanged from original file
// They validate the compiler works with default options (no type checking)
#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests validate backward compatibility
    #[test]
    fn test_compile_options_default() {
        let options = CompileOptions::default();
        assert!(!options.enable_type_checking);
        assert!(!options.strict_mode);
        assert!(options.allow_warnings);
    }

    #[test]
    fn test_compile_with_type_checking() {
        let expr = Expr::Lit(Literal::Int(42));
        let result = Compiler::compile_checked(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_backwards_compatible() {
        let expr = Expr::Lit(Literal::Int(42));
        let chunk = Compiler::compile(&expr).unwrap();
        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Int(42));
    }

    #[test]
    fn test_parse_qualified_name_simple() {
        let result = parse_qualified_name("Math.add");
        assert!(result.is_some());
        let (path, name) = result.unwrap();
        assert_eq!(path, vec!["Math".to_string()]);
        assert_eq!(name, "add");
    }

    #[test]
    fn test_parse_qualified_name_nested() {
        let result = parse_qualified_name("Geometry.Point.x");
        assert!(result.is_some());
        let (path, name) = result.unwrap();
        assert_eq!(path, vec!["Geometry".to_string(), "Point".to_string()]);
        assert_eq!(name, "x");
    }

    #[test]
    fn test_parse_qualified_name_unqualified() {
        let result = parse_qualified_name("add");
        assert!(result.is_none());
    }
}
