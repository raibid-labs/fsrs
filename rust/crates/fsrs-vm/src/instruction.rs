// FSRS VM Bytecode Instructions
// Defines the instruction set for the stack-based VM

use std::fmt;

/// Bytecode instruction for the FSRS VM
///
/// The VM is stack-based, with instructions operating on a value stack
/// and accessing locals, constants, and upvalues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // ===== Stack Operations =====
    /// Push constants[idx] onto stack
    LoadConst(u16),

    /// Push locals[idx] onto stack
    LoadLocal(u8),

    /// Pop stack top into locals[idx]
    StoreLocal(u8),

    /// Push upvalues[idx] onto stack
    LoadUpvalue(u8),

    /// Pop stack top into upvalues[idx]
    StoreUpvalue(u8),

    /// Pop top of stack and discard
    Pop,

    // ===== Arithmetic Operations =====
    /// Pop two integers, push sum (a + b)
    Add,

    /// Pop two integers, push difference (a - b)
    Sub,

    /// Pop two integers, push product (a * b)
    Mul,

    /// Pop two integers, push quotient (a / b)
    Div,

    // ===== Comparison Operations =====
    /// Pop two values, push equality result (a == b)
    Eq,

    /// Pop two values, push inequality result (a != b)
    Neq,

    /// Pop two values, push less-than result (a < b)
    Lt,

    /// Pop two values, push less-than-or-equal result (a <= b)
    Lte,

    /// Pop two values, push greater-than result (a > b)
    Gt,

    /// Pop two values, push greater-than-or-equal result (a >= b)
    Gte,

    // ===== Logical Operations =====
    /// Pop two booleans, push logical AND (a && b)
    And,

    /// Pop two booleans, push logical OR (a || b)
    Or,

    /// Pop one boolean, push logical NOT (!a)
    Not,

    // ===== Control Flow =====
    /// Unconditional jump by signed offset
    Jump(i16),

    /// Pop stack top; if false, jump by signed offset
    JumpIfFalse(i16),

    /// Call function with N arguments
    Call(u8),

    /// Tail call optimization for recursive functions
    TailCall(u8),

    /// Return from current function
    Return,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Stack operations
            Instruction::LoadConst(idx) => write!(f, "LOAD_CONST {}", idx),
            Instruction::LoadLocal(idx) => write!(f, "LOAD_LOCAL {}", idx),
            Instruction::StoreLocal(idx) => write!(f, "STORE_LOCAL {}", idx),
            Instruction::LoadUpvalue(idx) => write!(f, "LOAD_UPVALUE {}", idx),
            Instruction::StoreUpvalue(idx) => write!(f, "STORE_UPVALUE {}", idx),
            Instruction::Pop => write!(f, "POP"),

            // Arithmetic
            Instruction::Add => write!(f, "ADD"),
            Instruction::Sub => write!(f, "SUB"),
            Instruction::Mul => write!(f, "MUL"),
            Instruction::Div => write!(f, "DIV"),

            // Comparison
            Instruction::Eq => write!(f, "EQ"),
            Instruction::Neq => write!(f, "NEQ"),
            Instruction::Lt => write!(f, "LT"),
            Instruction::Lte => write!(f, "LTE"),
            Instruction::Gt => write!(f, "GT"),
            Instruction::Gte => write!(f, "GTE"),

            // Logical
            Instruction::And => write!(f, "AND"),
            Instruction::Or => write!(f, "OR"),
            Instruction::Not => write!(f, "NOT"),

            // Control flow
            Instruction::Jump(offset) => write!(f, "JUMP {}", offset),
            Instruction::JumpIfFalse(offset) => write!(f, "JUMP_IF_FALSE {}", offset),
            Instruction::Call(argc) => write!(f, "CALL {}", argc),
            Instruction::TailCall(argc) => write!(f, "TAIL_CALL {}", argc),
            Instruction::Return => write!(f, "RETURN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Instruction Construction Tests ==========

    #[test]
    fn test_instruction_load_const() {
        let instr = Instruction::LoadConst(42);
        assert_eq!(instr, Instruction::LoadConst(42));
    }

    #[test]
    fn test_instruction_load_local() {
        let instr = Instruction::LoadLocal(5);
        assert_eq!(instr, Instruction::LoadLocal(5));
    }

    #[test]
    fn test_instruction_store_local() {
        let instr = Instruction::StoreLocal(3);
        assert_eq!(instr, Instruction::StoreLocal(3));
    }

    #[test]
    fn test_instruction_arithmetic() {
        assert_eq!(Instruction::Add, Instruction::Add);
        assert_eq!(Instruction::Sub, Instruction::Sub);
        assert_eq!(Instruction::Mul, Instruction::Mul);
        assert_eq!(Instruction::Div, Instruction::Div);
    }

    #[test]
    fn test_instruction_comparison() {
        assert_eq!(Instruction::Eq, Instruction::Eq);
        assert_eq!(Instruction::Neq, Instruction::Neq);
        assert_eq!(Instruction::Lt, Instruction::Lt);
        assert_eq!(Instruction::Lte, Instruction::Lte);
        assert_eq!(Instruction::Gt, Instruction::Gt);
        assert_eq!(Instruction::Gte, Instruction::Gte);
    }

    #[test]
    fn test_instruction_logical() {
        assert_eq!(Instruction::And, Instruction::And);
        assert_eq!(Instruction::Or, Instruction::Or);
        assert_eq!(Instruction::Not, Instruction::Not);
    }

    #[test]
    fn test_instruction_control_flow() {
        assert_eq!(Instruction::Jump(10), Instruction::Jump(10));
        assert_eq!(Instruction::JumpIfFalse(-5), Instruction::JumpIfFalse(-5));
        assert_eq!(Instruction::Call(3), Instruction::Call(3));
        assert_eq!(Instruction::Return, Instruction::Return);
    }

    // ========== Display Formatting Tests ==========

    #[test]
    fn test_display_load_const() {
        let instr = Instruction::LoadConst(42);
        assert_eq!(format!("{}", instr), "LOAD_CONST 42");
    }

    #[test]
    fn test_display_load_local() {
        let instr = Instruction::LoadLocal(5);
        assert_eq!(format!("{}", instr), "LOAD_LOCAL 5");
    }

    #[test]
    fn test_display_store_local() {
        let instr = Instruction::StoreLocal(3);
        assert_eq!(format!("{}", instr), "STORE_LOCAL 3");
    }

    #[test]
    fn test_display_arithmetic() {
        assert_eq!(format!("{}", Instruction::Add), "ADD");
        assert_eq!(format!("{}", Instruction::Sub), "SUB");
        assert_eq!(format!("{}", Instruction::Mul), "MUL");
        assert_eq!(format!("{}", Instruction::Div), "DIV");
    }

    #[test]
    fn test_display_comparison() {
        assert_eq!(format!("{}", Instruction::Eq), "EQ");
        assert_eq!(format!("{}", Instruction::Neq), "NEQ");
        assert_eq!(format!("{}", Instruction::Lt), "LT");
        assert_eq!(format!("{}", Instruction::Lte), "LTE");
        assert_eq!(format!("{}", Instruction::Gt), "GT");
        assert_eq!(format!("{}", Instruction::Gte), "GTE");
    }

    #[test]
    fn test_display_logical() {
        assert_eq!(format!("{}", Instruction::And), "AND");
        assert_eq!(format!("{}", Instruction::Or), "OR");
        assert_eq!(format!("{}", Instruction::Not), "NOT");
    }

    #[test]
    fn test_display_jump() {
        assert_eq!(format!("{}", Instruction::Jump(10)), "JUMP 10");
        assert_eq!(format!("{}", Instruction::Jump(-5)), "JUMP -5");
    }

    #[test]
    fn test_display_jump_if_false() {
        assert_eq!(
            format!("{}", Instruction::JumpIfFalse(20)),
            "JUMP_IF_FALSE 20"
        );
        assert_eq!(
            format!("{}", Instruction::JumpIfFalse(-10)),
            "JUMP_IF_FALSE -10"
        );
    }

    #[test]
    fn test_display_call() {
        assert_eq!(format!("{}", Instruction::Call(3)), "CALL 3");
    }

    #[test]
    fn test_display_return() {
        assert_eq!(format!("{}", Instruction::Return), "RETURN");
    }

    // ========== Clone Tests ==========

    #[test]
    fn test_clone_load_const() {
        let instr1 = Instruction::LoadConst(42);
        let instr2 = instr1.clone();
        assert_eq!(instr1, instr2);
    }

    #[test]
    fn test_clone_jump() {
        let instr1 = Instruction::Jump(10);
        let instr2 = instr1.clone();
        assert_eq!(instr1, instr2);
    }

    // ========== Debug Formatting Tests ==========

    #[test]
    fn test_debug_load_const() {
        let instr = Instruction::LoadConst(42);
        assert_eq!(format!("{:?}", instr), "LoadConst(42)");
    }

    #[test]
    fn test_debug_add() {
        let instr = Instruction::Add;
        assert_eq!(format!("{:?}", instr), "Add");
    }

    // ========== Equality Tests ==========

    #[test]
    fn test_equality_same_variant() {
        assert_eq!(Instruction::LoadConst(42), Instruction::LoadConst(42));
        assert_ne!(Instruction::LoadConst(42), Instruction::LoadConst(43));
    }

    #[test]
    fn test_equality_different_variants() {
        assert_ne!(Instruction::Add, Instruction::Sub);
        assert_ne!(Instruction::Jump(5), Instruction::JumpIfFalse(5));
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_max_const_index() {
        let instr = Instruction::LoadConst(u16::MAX);
        assert_eq!(format!("{}", instr), format!("LOAD_CONST {}", u16::MAX));
    }

    #[test]
    fn test_max_local_index() {
        let instr = Instruction::LoadLocal(u8::MAX);
        assert_eq!(format!("{}", instr), format!("LOAD_LOCAL {}", u8::MAX));
    }

    #[test]
    fn test_jump_offsets() {
        assert_eq!(
            format!("{}", Instruction::Jump(i16::MAX)),
            format!("JUMP {}", i16::MAX)
        );
        assert_eq!(
            format!("{}", Instruction::Jump(i16::MIN)),
            format!("JUMP {}", i16::MIN)
        );
    }
}
