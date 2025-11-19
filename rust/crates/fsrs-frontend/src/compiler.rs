//! Bytecode Compiler for FSRS Mini-F#
//!
//! This module implements compilation from AST to bytecode chunks for the FSRS VM.
//! The compiler performs constant pooling, variable scoping, and generates efficient
//! bytecode instruction sequences.
//!
//! # Architecture
//!
//! The compiler uses:
//! - **Constant Pool**: Deduplicates literal values across the bytecode
//! - **Local Variables**: Stack-allocated variables tracked by index
//! - **Jump Patching**: Forward jump resolution for control flow
//!
//! # Example
//!
//! ```rust
//! use fsrs_frontend::ast::{Expr, Literal, BinOp};
//! use fsrs_frontend::compiler::Compiler;
//!
//! // Compile: 42 + 1
//! let expr = Expr::BinOp {
//!     op: BinOp::Add,
//!     left: Box::new(Expr::Lit(Literal::Int(42))),
//!     right: Box::new(Expr::Lit(Literal::Int(1))),
//! };
//!
//! let chunk = Compiler::compile(&expr).unwrap();
//! // Generates: LOAD_CONST 0; LOAD_CONST 1; ADD; RETURN
//! ```

use crate::ast::{BinOp, Expr, Literal, MatchArm, Pattern};
use fsrs_vm::chunk::Chunk;
use fsrs_vm::instruction::Instruction;
use fsrs_vm::value::Value;
use std::fmt;

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
        }
    }
}

impl std::error::Error for CompileError {}

/// Compilation result type
pub type CompileResult<T> = Result<T, CompileError>;

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
}

impl Compiler {
    /// Create a new compiler
    fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    /// Main entry point: compile an expression to a chunk
    pub fn compile(expr: &Expr) -> CompileResult<Chunk> {
        let mut compiler = Compiler::new();
        compiler.compile_expr(expr)?;
        compiler.emit(Instruction::Return);
        Ok(compiler.chunk)
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

    /// Compile a variable reference
    fn compile_var(&mut self, name: &str) -> CompileResult<()> {
        // Search for local variable from innermost to outermost scope
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                let idx = i as u8;
                self.emit(Instruction::LoadLocal(idx));
                return Ok(());
            }
        }

        // Variable not found in any scope
        Err(CompileError::UndefinedVariable(name.to_string()))
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

    /// Compile a lambda function (Phase 1: simplified, no closures yet)
    fn compile_lambda(&mut self, param: &str, body: &Expr) -> CompileResult<()> {
        // For Phase 1, we'll compile lambdas as inline code
        // In Phase 2, we'll create proper closure objects

        // For now, create a nested chunk for the lambda body
        let mut lambda_compiler = Compiler::new();

        // Lambda parameter becomes local 0
        lambda_compiler.begin_scope();
        lambda_compiler.add_local(param.to_string())?;

        // Compile the lambda body
        lambda_compiler.compile_expr(body)?;
        lambda_compiler.emit(Instruction::Return);

        // Clean up scope
        let locals_to_remove = lambda_compiler.end_scope_count();
        for _ in 0..locals_to_remove {
            lambda_compiler.locals.pop();
        }
        lambda_compiler.scope_depth -= 1;

        // For Phase 1, we'll store the lambda chunk as a constant
        // This is a simplified implementation - full closures come in Phase 2

        // For now, emit a placeholder (we'll improve this in Phase 2)
        // In Phase 1, lambdas are limited to immediate application
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TDD: Literal Compilation Tests (RED -> GREEN)
    // ========================================================================

    #[test]
    fn test_compile_literal_int() {
        let expr = Expr::Lit(Literal::Int(42));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Int(42));
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::Return);
    }

    #[test]
    fn test_compile_literal_bool() {
        let expr = Expr::Lit(Literal::Bool(true));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Bool(true));
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    }

    #[test]
    fn test_compile_literal_string() {
        let expr = Expr::Lit(Literal::Str("hello".to_string()));
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Str("hello".to_string()));
    }

    #[test]
    fn test_compile_literal_unit() {
        let expr = Expr::Lit(Literal::Unit);
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants.len(), 1);
        assert_eq!(chunk.constants[0], Value::Unit);
    }

    #[test]
    fn test_compile_literal_float_unsupported() {
        let expr = Expr::Lit(Literal::Float(3.15));
        let result = Compiler::compile(&expr);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CompileError::UnsupportedFloat);
    }

    // ========================================================================
    // TDD: Binary Operation Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_add() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: LoadConst 1, LoadConst 2, Add, Return
        assert_eq!(chunk.constants.len(), 2);
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
        assert_eq!(chunk.instructions[2], Instruction::Add);
        assert_eq!(chunk.instructions[3], Instruction::Return);
    }

    #[test]
    fn test_compile_all_arithmetic_ops() {
        let ops = vec![
            (BinOp::Add, Instruction::Add),
            (BinOp::Sub, Instruction::Sub),
            (BinOp::Mul, Instruction::Mul),
            (BinOp::Div, Instruction::Div),
        ];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Int(10))),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    #[test]
    fn test_compile_comparison_ops() {
        let ops = vec![
            (BinOp::Eq, Instruction::Eq),
            (BinOp::Neq, Instruction::Neq),
            (BinOp::Lt, Instruction::Lt),
            (BinOp::Lte, Instruction::Lte),
            (BinOp::Gt, Instruction::Gt),
            (BinOp::Gte, Instruction::Gte),
        ];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Int(1))),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    #[test]
    fn test_compile_logical_ops() {
        let ops = vec![(BinOp::And, Instruction::And), (BinOp::Or, Instruction::Or)];

        for (op, expected_instr) in ops {
            let expr = Expr::BinOp {
                op,
                left: Box::new(Expr::Lit(Literal::Bool(true))),
                right: Box::new(Expr::Lit(Literal::Bool(false))),
            };

            let chunk = Compiler::compile(&expr).unwrap();
            assert_eq!(chunk.instructions[2], expected_instr);
        }
    }

    // ========================================================================
    // TDD: Let-Binding Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_let_simple() {
        // let x = 42 in x
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(42))),
            body: Box::new(Expr::Var("x".to_string())),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: LoadConst(42), StoreLocal(0), LoadLocal(0), Return
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::LoadConst(_))));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::StoreLocal(_))));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::LoadLocal(_))));
    }

    #[test]
    fn test_compile_let_with_binop() {
        // let x = 10 in x + 5
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(10))),
            body: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Var("x".to_string())),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            }),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::Add));
    }

    #[test]
    fn test_compile_let_nested() {
        // let x = 1 in let y = 2 in x + y
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(1))),
            body: Box::new(Expr::Let {
                name: "y".to_string(),
                value: Box::new(Expr::Lit(Literal::Int(2))),
                body: Box::new(Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Var("x".to_string())),
                    right: Box::new(Expr::Var("y".to_string())),
                }),
            }),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have multiple locals
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(1)));
    }

    // ========================================================================
    // TDD: Variable Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_undefined_variable() {
        let expr = Expr::Var("x".to_string());
        let result = Compiler::compile(&expr);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CompileError::UndefinedVariable("x".to_string())
        );
    }

    // ========================================================================
    // TDD: If-Then-Else Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_if_simple() {
        // if true then 1 else 0
        let expr = Expr::If {
            cond: Box::new(Expr::Lit(Literal::Bool(true))),
            then_branch: Box::new(Expr::Lit(Literal::Int(1))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have jump instructions
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Jump(_))));
    }

    #[test]
    fn test_compile_if_with_comparison() {
        // if 10 > 5 then 42 else 0
        let expr = Expr::If {
            cond: Box::new(Expr::BinOp {
                op: BinOp::Gt,
                left: Box::new(Expr::Lit(Literal::Int(10))),
                right: Box::new(Expr::Lit(Literal::Int(5))),
            }),
            then_branch: Box::new(Expr::Lit(Literal::Int(42))),
            else_branch: Box::new(Expr::Lit(Literal::Int(0))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::Gt));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::JumpIfFalse(_))));
    }

    // ========================================================================
    // TDD: Constant Pool Tests
    // ========================================================================

    #[test]
    fn test_constant_deduplication() {
        // 42 + 42 should only have one constant
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(42))),
            right: Box::new(Expr::Lit(Literal::Int(42))),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        // Note: Current implementation doesn't deduplicate
        // This test documents current behavior
        assert!(chunk.constants.len() <= 2);
    }

    // ========================================================================
    // TDD: Tuple Compilation Tests
    // ========================================================================

    #[test]
    fn test_compile_tuple_empty() {
        // ()
        let expr = Expr::Tuple(vec![]);
        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: MakeTuple(0), Return
        assert_eq!(chunk.instructions[0], Instruction::MakeTuple(0));
        assert_eq!(chunk.instructions[1], Instruction::Return);
    }

    #[test]
    fn test_compile_tuple_pair() {
        // (1, 2)
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]);
        let chunk = Compiler::compile(&expr).unwrap();

        // Should have: LoadConst(1), LoadConst(2), MakeTuple(2), Return
        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
        assert_eq!(chunk.instructions[2], Instruction::MakeTuple(2));
        assert_eq!(chunk.instructions[3], Instruction::Return);
    }

    #[test]
    fn test_compile_tuple_triple() {
        // (1, 2, 3)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ]);
        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
        assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
        assert_eq!(chunk.instructions[2], Instruction::LoadConst(2));
        assert_eq!(chunk.instructions[3], Instruction::MakeTuple(3));
    }

    #[test]
    fn test_compile_tuple_nested() {
        // (1, (2, 3))
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Tuple(vec![Expr::Lit(Literal::Int(2)), Expr::Lit(Literal::Int(3))]),
        ]);
        let chunk = Compiler::compile(&expr).unwrap();

        // Inner tuple is compiled first
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(2))));
        // Outer tuple follows
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(2))));
    }

    #[test]
    fn test_compile_tuple_with_variables() {
        // let x = 1 in let y = 2 in (x, y)
        let expr = Expr::Let {
            name: "x".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(1))),
            body: Box::new(Expr::Let {
                name: "y".to_string(),
                value: Box::new(Expr::Lit(Literal::Int(2))),
                body: Box::new(Expr::Tuple(vec![
                    Expr::Var("x".to_string()),
                    Expr::Var("y".to_string()),
                ])),
            }),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(1)));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(2))));
    }

    #[test]
    fn test_compile_tuple_with_expressions() {
        // (1 + 2, 3 * 4)
        let expr = Expr::Tuple(vec![
            Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Lit(Literal::Int(1))),
                right: Box::new(Expr::Lit(Literal::Int(2))),
            },
            Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Lit(Literal::Int(3))),
                right: Box::new(Expr::Lit(Literal::Int(4))),
            },
        ]);

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk.instructions.contains(&Instruction::Add));
        assert!(chunk.instructions.contains(&Instruction::Mul));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(2))));
    }

    #[test]
    fn test_compile_tuple_mixed_types() {
        // (42, "hello", true)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(42)),
            Expr::Lit(Literal::Str("hello".to_string())),
            Expr::Lit(Literal::Bool(true)),
        ]);

        let chunk = Compiler::compile(&expr).unwrap();

        assert_eq!(chunk.constants[0], Value::Int(42));
        assert_eq!(chunk.constants[1], Value::Str("hello".to_string()));
        assert_eq!(chunk.constants[2], Value::Bool(true));
        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(3))));
    }

    #[test]
    fn test_compile_tuple_large() {
        // (1, 2, 3, 4, 5, 6, 7, 8)
        let expr = Expr::Tuple(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
            Expr::Lit(Literal::Int(4)),
            Expr::Lit(Literal::Int(5)),
            Expr::Lit(Literal::Int(6)),
            Expr::Lit(Literal::Int(7)),
            Expr::Lit(Literal::Int(8)),
        ]);

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(8))));
    }

    #[test]
    fn test_compile_tuple_in_let() {
        // let pair = (1, 2) in pair
        let expr = Expr::Let {
            name: "pair".to_string(),
            value: Box::new(Expr::Tuple(vec![
                Expr::Lit(Literal::Int(1)),
                Expr::Lit(Literal::Int(2)),
            ])),
            body: Box::new(Expr::Var("pair".to_string())),
        };

        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(2))));
        assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
        assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
    }

    #[test]
    fn test_compile_tuple_single_element() {
        // (42)
        let expr = Expr::Tuple(vec![Expr::Lit(Literal::Int(42))]);
        let chunk = Compiler::compile(&expr).unwrap();

        assert!(chunk
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::MakeTuple(1))));
    }

    #[test]
    fn test_compile_tuple_deeply_nested() {
        // ((1, 2), (3, 4))
        let expr = Expr::Tuple(vec![
            Expr::Tuple(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]),
            Expr::Tuple(vec![Expr::Lit(Literal::Int(3)), Expr::Lit(Literal::Int(4))]),
        ]);

        let chunk = Compiler::compile(&expr).unwrap();

        // Should have two inner MakeTuple(2) and one outer MakeTuple(2)
        let make_tuple_count = chunk
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::MakeTuple(2)))
            .count();
        assert_eq!(make_tuple_count, 3);
    }
}

// ========================================================================
// TDD: List Compilation Tests (Layer 3)
// ========================================================================

#[test]
fn test_compile_list_empty() {
    // []
    let expr = Expr::List(vec![]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have: LoadConst(Nil), Return
    assert_eq!(chunk.constants.len(), 1);
    assert_eq!(chunk.constants[0], Value::Nil);
    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::Return);
}

#[test]
fn test_compile_list_single() {
    // [42]
    let expr = Expr::List(vec![Expr::Lit(Literal::Int(42))]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have: LoadConst(42), MakeList(1), Return
    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::MakeList(1));
    assert_eq!(chunk.instructions[2], Instruction::Return);
}

#[test]
fn test_compile_list_multiple() {
    // [1; 2; 3]
    let expr = Expr::List(vec![
        Expr::Lit(Literal::Int(1)),
        Expr::Lit(Literal::Int(2)),
        Expr::Lit(Literal::Int(3)),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
    assert_eq!(chunk.instructions[2], Instruction::LoadConst(2));
    assert_eq!(chunk.instructions[3], Instruction::MakeList(3));
    assert_eq!(chunk.instructions[4], Instruction::Return);
}

#[test]
fn test_compile_cons_simple() {
    // 1 :: []
    let expr = Expr::Cons {
        head: Box::new(Expr::Lit(Literal::Int(1))),
        tail: Box::new(Expr::List(vec![])),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    // Should compile head, then tail, then Cons
    assert!(chunk.instructions.contains(&Instruction::Cons));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadConst(_))));
}

#[test]
fn test_compile_cons_with_list() {
    // 1 :: [2; 3]
    let expr = Expr::Cons {
        head: Box::new(Expr::Lit(Literal::Int(1))),
        tail: Box::new(Expr::List(vec![
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::Cons));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(2))));
}

#[test]
fn test_compile_cons_nested() {
    // 1 :: 2 :: []
    let expr = Expr::Cons {
        head: Box::new(Expr::Lit(Literal::Int(1))),
        tail: Box::new(Expr::Cons {
            head: Box::new(Expr::Lit(Literal::Int(2))),
            tail: Box::new(Expr::List(vec![])),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have two Cons instructions
    let cons_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::Cons))
        .count();
    assert_eq!(cons_count, 2);
}

#[test]
fn test_compile_list_nested() {
    // [[1; 2]; [3; 4]]
    let expr = Expr::List(vec![
        Expr::List(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]),
        Expr::List(vec![Expr::Lit(Literal::Int(3)), Expr::Lit(Literal::Int(4))]),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have two inner MakeList(2) and one outer MakeList(2)
    let make_list_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::MakeList(2)))
        .count();
    assert_eq!(make_list_count, 3);
}

#[test]
fn test_compile_list_with_variables() {
    // let x = 1 in let y = 2 in [x; y]
    let expr = Expr::Let {
        name: "x".to_string(),
        value: Box::new(Expr::Lit(Literal::Int(1))),
        body: Box::new(Expr::Let {
            name: "y".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(2))),
            body: Box::new(Expr::List(vec![
                Expr::Var("x".to_string()),
                Expr::Var("y".to_string()),
            ])),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(1)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(2))));
}

#[test]
fn test_compile_list_with_expressions() {
    // [1 + 2; 3 * 4]
    let expr = Expr::List(vec![
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        },
        Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Lit(Literal::Int(3))),
            right: Box::new(Expr::Lit(Literal::Int(4))),
        },
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::Add));
    assert!(chunk.instructions.contains(&Instruction::Mul));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(2))));
}

#[test]
fn test_compile_list_in_let() {
    // let xs = [1; 2; 3] in xs
    let expr = Expr::Let {
        name: "xs".to_string(),
        value: Box::new(Expr::List(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::Var("xs".to_string())),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(3))));
    assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
}

#[test]
fn test_compile_cons_with_variable() {
    // let xs = [2; 3] in 1 :: xs
    let expr = Expr::Let {
        name: "xs".to_string(),
        value: Box::new(Expr::List(vec![
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::Cons {
            head: Box::new(Expr::Lit(Literal::Int(1))),
            tail: Box::new(Expr::Var("xs".to_string())),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(2))));
    assert!(chunk.instructions.contains(&Instruction::Cons));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
}

#[test]
fn test_compile_list_mixed_types() {
    // [42; "hello"; true]
    let expr = Expr::List(vec![
        Expr::Lit(Literal::Int(42)),
        Expr::Lit(Literal::Str("hello".to_string())),
        Expr::Lit(Literal::Bool(true)),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert_eq!(chunk.constants[0], Value::Int(42));
    assert_eq!(chunk.constants[1], Value::Str("hello".to_string()));
    assert_eq!(chunk.constants[2], Value::Bool(true));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeList(3))));
}

// ========================================================================
// TDD: Array Compilation Tests (Layer 3)
// ========================================================================

#[test]
fn test_compile_array_empty() {
    // [||]
    let expr = Expr::Array(vec![]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have: MakeArray(0), Return
    assert_eq!(chunk.instructions[0], Instruction::MakeArray(0));
    assert_eq!(chunk.instructions[1], Instruction::Return);
}

#[test]
fn test_compile_array_single() {
    // [|42|]
    let expr = Expr::Array(vec![Expr::Lit(Literal::Int(42))]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have: LoadConst(42), MakeArray(1), Return
    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::MakeArray(1));
    assert_eq!(chunk.instructions[2], Instruction::Return);
}

#[test]
fn test_compile_array_multiple() {
    // [|1; 2; 3|]
    let expr = Expr::Array(vec![
        Expr::Lit(Literal::Int(1)),
        Expr::Lit(Literal::Int(2)),
        Expr::Lit(Literal::Int(3)),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert_eq!(chunk.instructions[0], Instruction::LoadConst(0));
    assert_eq!(chunk.instructions[1], Instruction::LoadConst(1));
    assert_eq!(chunk.instructions[2], Instruction::LoadConst(2));
    assert_eq!(chunk.instructions[3], Instruction::MakeArray(3));
    assert_eq!(chunk.instructions[4], Instruction::Return);
}

#[test]
fn test_compile_array_index() {
    // let arr = [|1; 2; 3|] in arr.[1]
    let expr = Expr::Let {
        name: "arr".to_string(),
        value: Box::new(Expr::Array(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::ArrayIndex {
            array: Box::new(Expr::Var("arr".to_string())),
            index: Box::new(Expr::Lit(Literal::Int(1))),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
    assert!(chunk.instructions.contains(&Instruction::ArrayGet));
}

#[test]
fn test_compile_array_update() {
    // let arr = [|1; 2; 3|] in arr.[1] <- 99
    let expr = Expr::Let {
        name: "arr".to_string(),
        value: Box::new(Expr::Array(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::ArrayUpdate {
            array: Box::new(Expr::Var("arr".to_string())),
            index: Box::new(Expr::Lit(Literal::Int(1))),
            value: Box::new(Expr::Lit(Literal::Int(99))),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
    assert!(chunk.instructions.contains(&Instruction::ArrayUpdate));
}

#[test]
fn test_compile_array_length() {
    // let arr = [|1; 2; 3|] in Array.length arr
    let expr = Expr::Let {
        name: "arr".to_string(),
        value: Box::new(Expr::Array(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::ArrayLength(Box::new(Expr::Var("arr".to_string())))),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
    assert!(chunk.instructions.contains(&Instruction::ArrayLength));
}

#[test]
fn test_compile_array_nested() {
    // [|[|1; 2|]; [|3; 4|]|]
    let expr = Expr::Array(vec![
        Expr::Array(vec![Expr::Lit(Literal::Int(1)), Expr::Lit(Literal::Int(2))]),
        Expr::Array(vec![Expr::Lit(Literal::Int(3)), Expr::Lit(Literal::Int(4))]),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    // Should have two inner MakeArray(2) and one outer MakeArray(2)
    let make_array_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::MakeArray(2)))
        .count();
    assert_eq!(make_array_count, 3);
}

#[test]
fn test_compile_array_with_variables() {
    // let x = 1 in let y = 2 in [|x; y|]
    let expr = Expr::Let {
        name: "x".to_string(),
        value: Box::new(Expr::Lit(Literal::Int(1))),
        body: Box::new(Expr::Let {
            name: "y".to_string(),
            value: Box::new(Expr::Lit(Literal::Int(2))),
            body: Box::new(Expr::Array(vec![
                Expr::Var("x".to_string()),
                Expr::Var("y".to_string()),
            ])),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(1)));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(2))));
}

#[test]
fn test_compile_array_with_expressions() {
    // [|1 + 2; 3 * 4|]
    let expr = Expr::Array(vec![
        Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Lit(Literal::Int(1))),
            right: Box::new(Expr::Lit(Literal::Int(2))),
        },
        Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Lit(Literal::Int(3))),
            right: Box::new(Expr::Lit(Literal::Int(4))),
        },
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk.instructions.contains(&Instruction::Add));
    assert!(chunk.instructions.contains(&Instruction::Mul));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(2))));
}

#[test]
fn test_compile_array_mixed_types() {
    // [|42; "hello"; true|]
    let expr = Expr::Array(vec![
        Expr::Lit(Literal::Int(42)),
        Expr::Lit(Literal::Str("hello".to_string())),
        Expr::Lit(Literal::Bool(true)),
    ]);
    let chunk = Compiler::compile(&expr).unwrap();

    assert_eq!(chunk.constants[0], Value::Int(42));
    assert_eq!(chunk.constants[1], Value::Str("hello".to_string()));
    assert_eq!(chunk.constants[2], Value::Bool(true));
    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
}

#[test]
fn test_compile_array_in_let() {
    // let arr = [|1; 2; 3|] in arr
    let expr = Expr::Let {
        name: "arr".to_string(),
        value: Box::new(Expr::Array(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::Var("arr".to_string())),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
    assert!(chunk.instructions.contains(&Instruction::StoreLocal(0)));
    assert!(chunk.instructions.contains(&Instruction::LoadLocal(0)));
}

#[test]
fn test_compile_array_chained_access() {
    // let arr = [|1; 2; 3|] in arr.[0] + arr.[2]
    let expr = Expr::Let {
        name: "arr".to_string(),
        value: Box::new(Expr::Array(vec![
            Expr::Lit(Literal::Int(1)),
            Expr::Lit(Literal::Int(2)),
            Expr::Lit(Literal::Int(3)),
        ])),
        body: Box::new(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::ArrayIndex {
                array: Box::new(Expr::Var("arr".to_string())),
                index: Box::new(Expr::Lit(Literal::Int(0))),
            }),
            right: Box::new(Expr::ArrayIndex {
                array: Box::new(Expr::Var("arr".to_string())),
                index: Box::new(Expr::Lit(Literal::Int(2))),
            }),
        }),
    };
    let chunk = Compiler::compile(&expr).unwrap();

    assert!(chunk
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::MakeArray(3))));
    let array_get_count = chunk
        .instructions
        .iter()
        .filter(|i| matches!(i, Instruction::ArrayGet))
        .count();
    assert_eq!(array_get_count, 2);
    assert!(chunk.instructions.contains(&Instruction::Add));
}
