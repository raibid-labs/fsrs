// Fusabi VM Bytecode Instructions
// Defines the instruction set for the stack-based VM

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// Bytecode instruction for the Fusabi VM
///
/// The VM is stack-based, with instructions operating on a value stack
/// and accessing locals, constants, and upvalues.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Instruction {
    // ===== Stack Operations =====
    /// Push constants[`idx`] onto stack
    LoadConst(u16),

    /// Push locals[`idx`] onto stack
    LoadLocal(u8),

    /// Pop stack top into locals[`idx`]
    StoreLocal(u8),

    /// Push upvalues[`idx`] onto stack
    LoadUpvalue(u8),

    /// Pop stack top into upvalues[`idx`]
    StoreUpvalue(u8),

    /// Load global variable by name (name is at constants[`idx`])
    LoadGlobal(u16),

    /// Pop top of stack and discard
    Pop,
    /// Duplicate top of stack
    Dup,

    // ===== Pattern Matching Operations =====
    /// Check if top of stack equals int, push bool
    CheckInt(i64),

    /// Check if top of stack equals bool, push bool
    CheckBool(bool),

    /// Check if top of stack equals string, push bool
    CheckString(String),

    /// Check if top of stack is tuple with N elements, push bool
    CheckTupleLen(u8),

    /// Get tuple element by index (leaves tuple on stack)
    GetTupleElem(u8),

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

    // ===== Tuple Operations =====
    /// Create tuple from N stack values
    /// Pop N values from stack, create tuple, push tuple
    MakeTuple(u16),

    /// Extract field from tuple by index
    /// Pop tuple from stack, push field at index
    GetTupleField(u8),

    // ===== List Operations =====
    /// Create list from N stack values [e1; e2; ...]
    /// Pop N values from stack (in reverse order), build cons list, push list
    MakeList(u16),

    /// Cons: Pop tail, pop head, push (head :: tail)
    Cons,

    /// ListHead: Pop list, push head (error if empty)
    ListHead,

    /// ListTail: Pop list, push tail (error if empty)
    ListTail,

    /// IsNil: Pop list, push bool (true if empty)
    IsNil,

    // ===== Array Operations =====
    /// Create array from N stack values [|e1; e2; e3|]
    /// Pop N values from stack (in reverse order), build array, push array
    MakeArray(u16),

    /// ArrayGet: Pop index, pop array, push element
    ArrayGet,

    /// ArraySet: Pop value, pop index, pop array (mutates in place), push unit
    ArraySet,

    /// ArrayLength: Pop array, push length as Int
    ArrayLength,

    /// ArrayUpdate: Create new array with updated element (immutable)
    /// Pop value, pop index, pop array, push new array
    ArrayUpdate,

    // ===== Record Operations =====
    /// Create record from N field-value pairs
    /// Stack layout: [field_name_0, value_0, field_name_1, value_1, ..., field_name_N-1, value_N-1]
    /// Pop 2*N values (N field-value pairs), create record, push record
    MakeRecord(u16),

    /// Get record field by name
    /// Pop field_name (String), pop record, push field value
    GetRecordField,

    /// Update record (immutable - creates new record)
    /// Stack layout: [..., field_name_0, value_0, ..., record, N (count)]
    /// Pop count, pop record, pop N field-value pairs, create new record, push new record
    UpdateRecord(u16),

    // ===== Discriminated Union Operations =====
    /// Create discriminated union variant
    /// Stack layout: [type_name (String), variant_name (String), field_0, field_1, ..., field_N-1]
    /// Pop N+2 values (type_name, variant_name, N fields), create variant, push variant
    MakeVariant(u16),

    /// Check if variant has specific tag/name
    /// Pop variant from stack, push bool (true if variant name matches)
    CheckVariantTag(String),

    /// Get variant field by index
    /// Pop variant from stack, push field value at index
    GetVariantField(u8),

    // ===== Closure Operations =====
    /// Create closure from constant index (which points to a Chunk/Function)
    /// Pops upvalues from stack based on the function's upvalue count
    /// Args: (constant_index, upvalue_count)
    MakeClosure(u16, u8),

    /// Close upvalues on the stack up to a given stack slot
    CloseUpvalue(u8),
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
            Instruction::LoadGlobal(idx) => write!(f, "LOAD_GLOBAL {}", idx),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Dup => write!(f, "DUP"),

            // Pattern matching
            Instruction::CheckInt(n) => write!(f, "CHECK_INT {}", n),
            Instruction::CheckBool(b) => write!(f, "CHECK_BOOL {}", b),
            Instruction::CheckString(s) => write!(f, "CHECK_STRING \"{}\"", s),
            Instruction::CheckTupleLen(n) => write!(f, "CHECK_TUPLE_LEN {}", n),
            Instruction::GetTupleElem(idx) => write!(f, "GET_TUPLE_ELEM {}", idx),

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

            // Tuple operations
            Instruction::MakeTuple(n) => write!(f, "MAKE_TUPLE {}", n),
            Instruction::GetTupleField(idx) => write!(f, "GET_TUPLE_FIELD {}", idx),

            // List operations
            Instruction::MakeList(n) => write!(f, "MAKE_LIST {}", n),
            Instruction::Cons => write!(f, "CONS"),
            Instruction::ListHead => write!(f, "LIST_HEAD"),
            Instruction::ListTail => write!(f, "LIST_TAIL"),
            Instruction::IsNil => write!(f, "IS_NIL"),

            // Array operations
            Instruction::MakeArray(n) => write!(f, "MAKE_ARRAY {}", n),
            Instruction::ArrayGet => write!(f, "ARRAY_GET"),
            Instruction::ArraySet => write!(f, "ARRAY_SET"),
            Instruction::ArrayLength => write!(f, "ARRAY_LENGTH"),
            Instruction::ArrayUpdate => write!(f, "ARRAY_UPDATE"),

            // Record operations
            Instruction::MakeRecord(n) => write!(f, "MAKE_RECORD {}", n),
            Instruction::GetRecordField => write!(f, "GET_RECORD_FIELD"),
            Instruction::UpdateRecord(n) => write!(f, "UPDATE_RECORD {}", n),

            // Discriminated union operations
            Instruction::MakeVariant(n) => write!(f, "MAKE_VARIANT {}", n),
            Instruction::CheckVariantTag(tag) => write!(f, "CHECK_VARIANT_TAG \"{}\"", tag),
            Instruction::GetVariantField(idx) => write!(f, "GET_VARIANT_FIELD {}", idx),

            // Closure operations
            Instruction::MakeClosure(idx, count) => write!(f, "MAKE_CLOSURE {} {}", idx, count),
            Instruction::CloseUpvalue(idx) => write!(f, "CLOSE_UPVALUE {}", idx),
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

    #[test]
    fn test_instruction_make_tuple() {
        let instr = Instruction::MakeTuple(3);
        assert_eq!(instr, Instruction::MakeTuple(3));
    }

    #[test]
    fn test_instruction_get_tuple_field() {
        let instr = Instruction::GetTupleField(1);
        assert_eq!(instr, Instruction::GetTupleField(1));
    }

    // ========== List Instruction Tests ==========

    #[test]
    fn test_instruction_make_list() {
        let instr = Instruction::MakeList(3);
        assert_eq!(instr, Instruction::MakeList(3));
    }

    #[test]
    fn test_instruction_cons() {
        let instr = Instruction::Cons;
        assert_eq!(instr, Instruction::Cons);
    }

    #[test]
    fn test_instruction_list_head() {
        let instr = Instruction::ListHead;
        assert_eq!(instr, Instruction::ListHead);
    }

    #[test]
    fn test_instruction_list_tail() {
        let instr = Instruction::ListTail;
        assert_eq!(instr, Instruction::ListTail);
    }

    #[test]
    fn test_instruction_is_nil() {
        let instr = Instruction::IsNil;
        assert_eq!(instr, Instruction::IsNil);
    }

    // ========== Array Instruction Tests ==========

    #[test]
    fn test_instruction_make_array() {
        let instr = Instruction::MakeArray(3);
        assert_eq!(instr, Instruction::MakeArray(3));
    }

    #[test]
    fn test_instruction_array_get() {
        let instr = Instruction::ArrayGet;
        assert_eq!(instr, Instruction::ArrayGet);
    }

    #[test]
    fn test_instruction_array_set() {
        let instr = Instruction::ArraySet;
        assert_eq!(instr, Instruction::ArraySet);
    }

    #[test]
    fn test_instruction_array_length() {
        let instr = Instruction::ArrayLength;
        assert_eq!(instr, Instruction::ArrayLength);
    }

    #[test]
    fn test_instruction_array_update() {
        let instr = Instruction::ArrayUpdate;
        assert_eq!(instr, Instruction::ArrayUpdate);
    }

    // ========== Record Instruction Tests ==========

    #[test]
    fn test_instruction_make_record() {
        let instr = Instruction::MakeRecord(3);
        assert_eq!(instr, Instruction::MakeRecord(3));
    }

    #[test]
    fn test_instruction_get_record_field() {
        let instr = Instruction::GetRecordField;
        assert_eq!(instr, Instruction::GetRecordField);
    }

    #[test]
    fn test_instruction_update_record() {
        let instr = Instruction::UpdateRecord(2);
        assert_eq!(instr, Instruction::UpdateRecord(2));
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

    #[test]
    fn test_display_make_tuple() {
        assert_eq!(format!("{}", Instruction::MakeTuple(2)), "MAKE_TUPLE 2");
        assert_eq!(format!("{}", Instruction::MakeTuple(5)), "MAKE_TUPLE 5");
    }

    #[test]
    fn test_display_get_tuple_field() {
        assert_eq!(
            format!("{}", Instruction::GetTupleField(0)),
            "GET_TUPLE_FIELD 0"
        );
        assert_eq!(
            format!("{}", Instruction::GetTupleField(3)),
            "GET_TUPLE_FIELD 3"
        );
    }

    #[test]
    fn test_display_make_list() {
        assert_eq!(format!("{}", Instruction::MakeList(3)), "MAKE_LIST 3");
        assert_eq!(format!("{}", Instruction::MakeList(0)), "MAKE_LIST 0");
    }

    #[test]
    fn test_display_cons() {
        assert_eq!(format!("{}", Instruction::Cons), "CONS");
    }

    #[test]
    fn test_display_list_head() {
        assert_eq!(format!("{}", Instruction::ListHead), "LIST_HEAD");
    }

    #[test]
    fn test_display_list_tail() {
        assert_eq!(format!("{}", Instruction::ListTail), "LIST_TAIL");
    }

    #[test]
    fn test_display_is_nil() {
        assert_eq!(format!("{}", Instruction::IsNil), "IS_NIL");
    }

    #[test]
    fn test_display_make_array() {
        assert_eq!(format!("{}", Instruction::MakeArray(3)), "MAKE_ARRAY 3");
        assert_eq!(format!("{}", Instruction::MakeArray(0)), "MAKE_ARRAY 0");
    }

    #[test]
    fn test_display_array_get() {
        assert_eq!(format!("{}", Instruction::ArrayGet), "ARRAY_GET");
    }

    #[test]
    fn test_display_array_set() {
        assert_eq!(format!("{}", Instruction::ArraySet), "ARRAY_SET");
    }

    #[test]
    fn test_display_array_length() {
        assert_eq!(format!("{}", Instruction::ArrayLength), "ARRAY_LENGTH");
    }

    #[test]
    fn test_display_array_update() {
        assert_eq!(format!("{}", Instruction::ArrayUpdate), "ARRAY_UPDATE");
    }

    #[test]
    fn test_display_make_record() {
        assert_eq!(format!("{}", Instruction::MakeRecord(3)), "MAKE_RECORD 3");
        assert_eq!(format!("{}", Instruction::MakeRecord(0)), "MAKE_RECORD 0");
    }

    #[test]
    fn test_display_get_record_field() {
        assert_eq!(
            format!("{}", Instruction::GetRecordField),
            "GET_RECORD_FIELD"
        );
    }

    #[test]
    fn test_display_update_record() {
        assert_eq!(
            format!("{}", Instruction::UpdateRecord(2)),
            "UPDATE_RECORD 2"
        );
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

    #[test]
    fn test_clone_make_tuple() {
        let instr1 = Instruction::MakeTuple(3);
        let instr2 = instr1.clone();
        assert_eq!(instr1, instr2);
    }

    #[test]
    fn test_clone_make_list() {
        let instr1 = Instruction::MakeList(5);
        let instr2 = instr1.clone();
        assert_eq!(instr1, instr2);
    }

    #[test]
    fn test_clone_make_array() {
        let instr1 = Instruction::MakeArray(5);
        let instr2 = instr1.clone();
        assert_eq!(instr1, instr2);
    }

    #[test]
    fn test_clone_make_record() {
        let instr1 = Instruction::MakeRecord(5);
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

    #[test]
    fn test_debug_make_tuple() {
        let instr = Instruction::MakeTuple(2);
        assert_eq!(format!("{:?}", instr), "MakeTuple(2)");
    }

    #[test]
    fn test_debug_get_tuple_field() {
        let instr = Instruction::GetTupleField(1);
        assert_eq!(format!("{:?}", instr), "GetTupleField(1)");
    }

    #[test]
    fn test_debug_make_list() {
        let instr = Instruction::MakeList(3);
        assert_eq!(format!("{:?}", instr), "MakeList(3)");
    }

    #[test]
    fn test_debug_cons() {
        let instr = Instruction::Cons;
        assert_eq!(format!("{:?}", instr), "Cons");
    }

    #[test]
    fn test_debug_make_array() {
        let instr = Instruction::MakeArray(3);
        assert_eq!(format!("{:?}", instr), "MakeArray(3)");
    }

    #[test]
    fn test_debug_array_get() {
        let instr = Instruction::ArrayGet;
        assert_eq!(format!("{:?}", instr), "ArrayGet");
    }

    #[test]
    fn test_debug_make_record() {
        let instr = Instruction::MakeRecord(3);
        assert_eq!(format!("{:?}", instr), "MakeRecord(3)");
    }

    #[test]
    fn test_debug_get_record_field() {
        let instr = Instruction::GetRecordField;
        assert_eq!(format!("{:?}", instr), "GetRecordField");
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

    #[test]
    fn test_equality_tuple_instructions() {
        assert_eq!(Instruction::MakeTuple(2), Instruction::MakeTuple(2));
        assert_ne!(Instruction::MakeTuple(2), Instruction::MakeTuple(3));
        assert_eq!(Instruction::GetTupleField(0), Instruction::GetTupleField(0));
        assert_ne!(Instruction::GetTupleField(0), Instruction::GetTupleField(1));
    }

    #[test]
    fn test_equality_list_instructions() {
        assert_eq!(Instruction::MakeList(3), Instruction::MakeList(3));
        assert_ne!(Instruction::MakeList(3), Instruction::MakeList(5));
        assert_eq!(Instruction::Cons, Instruction::Cons);
        assert_eq!(Instruction::ListHead, Instruction::ListHead);
        assert_eq!(Instruction::ListTail, Instruction::ListTail);
        assert_eq!(Instruction::IsNil, Instruction::IsNil);
    }

    #[test]
    fn test_equality_array_instructions() {
        assert_eq!(Instruction::MakeArray(3), Instruction::MakeArray(3));
        assert_ne!(Instruction::MakeArray(3), Instruction::MakeArray(5));
        assert_eq!(Instruction::ArrayGet, Instruction::ArrayGet);
        assert_eq!(Instruction::ArraySet, Instruction::ArraySet);
        assert_eq!(Instruction::ArrayLength, Instruction::ArrayLength);
        assert_eq!(Instruction::ArrayUpdate, Instruction::ArrayUpdate);
    }

    #[test]
    fn test_equality_record_instructions() {
        assert_eq!(Instruction::MakeRecord(3), Instruction::MakeRecord(3));
        assert_ne!(Instruction::MakeRecord(3), Instruction::MakeRecord(5));
        assert_eq!(Instruction::GetRecordField, Instruction::GetRecordField);
        assert_eq!(Instruction::UpdateRecord(2), Instruction::UpdateRecord(2));
        assert_ne!(Instruction::UpdateRecord(2), Instruction::UpdateRecord(3));
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

    #[test]
    fn test_max_tuple_size() {
        let instr = Instruction::MakeTuple(u16::MAX);
        assert_eq!(format!("{}", instr), format!("MAKE_TUPLE {}", u16::MAX));
    }

    #[test]
    fn test_max_tuple_field_index() {
        let instr = Instruction::GetTupleField(u8::MAX);
        assert_eq!(format!("{}", instr), format!("GET_TUPLE_FIELD {}", u8::MAX));
    }

    #[test]
    fn test_max_list_size() {
        let instr = Instruction::MakeList(u16::MAX);
        assert_eq!(format!("{}", instr), format!("MAKE_LIST {}", u16::MAX));
    }

    #[test]
    fn test_max_array_size() {
        let instr = Instruction::MakeArray(u16::MAX);
        assert_eq!(format!("{}", instr), format!("MAKE_ARRAY {}", u16::MAX));
    }

    #[test]
    fn test_max_record_size() {
        let instr = Instruction::MakeRecord(u16::MAX);
        assert_eq!(format!("{}", instr), format!("MAKE_RECORD {}", u16::MAX));
    }
}
