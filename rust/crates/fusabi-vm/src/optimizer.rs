// Fusabi VM Bytecode Optimizer
// Performs compile-time optimizations on bytecode chunks

use crate::chunk::{Chunk, SourceSpan};
use crate::instruction::Instruction;
use crate::value::Value;

pub fn optimize_chunk(chunk: &mut Chunk) {
    fold_constants(chunk);
    eliminate_dead_code(chunk);
}

fn fold_constants(chunk: &mut Chunk) {
    let mut changed = true;
    while changed {
        changed = false;
        changed |= fold_arithmetic(chunk);
        changed |= fold_boolean(chunk);
        changed |= fold_comparison(chunk);
    }
}

fn fold_arithmetic(chunk: &mut Chunk) -> bool {
    let mut i = 0;
    let mut changed = false;

    while i + 2 < chunk.instructions.len() {
        let folded = match (
            &chunk.instructions[i],
            &chunk.instructions[i + 1],
            &chunk.instructions[i + 2],
        ) {
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Add) => {
                fold_binary_int(chunk, *a_idx, *b_idx, |a, b| a.checked_add(b))
                    .or_else(|| fold_binary_float(chunk, *a_idx, *b_idx, |a, b| a + b))
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Sub) => {
                fold_binary_int(chunk, *a_idx, *b_idx, |a, b| a.checked_sub(b))
                    .or_else(|| fold_binary_float(chunk, *a_idx, *b_idx, |a, b| a - b))
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Mul) => {
                fold_binary_int(chunk, *a_idx, *b_idx, |a, b| a.checked_mul(b))
                    .or_else(|| fold_binary_float(chunk, *a_idx, *b_idx, |a, b| a * b))
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Div) => {
                match (chunk.constant_at(*a_idx), chunk.constant_at(*b_idx)) {
                    (Some(Value::Int(_)), Some(Value::Int(0))) => None,
                    (Some(Value::Float(_)), Some(Value::Float(b))) if *b == 0.0 => None,
                    _ => fold_binary_int(chunk, *a_idx, *b_idx, |a, b| a.checked_div(b))
                        .or_else(|| fold_binary_float(chunk, *a_idx, *b_idx, |a, b| a / b)),
                }
            }
            _ => None,
        };

        if let Some((result_value, span)) = folded {
            let new_idx = chunk.add_constant(result_value);
            chunk.instructions[i] = Instruction::LoadConst(new_idx);
            chunk.instructions.remove(i + 2);
            chunk.spans.remove(i + 2);
            chunk.instructions.remove(i + 1);
            chunk.spans.remove(i + 1);
            chunk.spans[i] = span;
            changed = true;
        } else {
            i += 1;
        }
    }

    changed
}

fn fold_binary_int<F>(chunk: &Chunk, a_idx: u16, b_idx: u16, op: F) -> Option<(Value, SourceSpan)>
where
    F: FnOnce(i64, i64) -> Option<i64>,
{
    match (chunk.constant_at(a_idx), chunk.constant_at(b_idx)) {
        (Some(Value::Int(a)), Some(Value::Int(b))) => {
            op(*a, *b).map(|result| (Value::Int(result), SourceSpan::unknown()))
        }
        _ => None,
    }
}

fn fold_binary_float<F>(chunk: &Chunk, a_idx: u16, b_idx: u16, op: F) -> Option<(Value, SourceSpan)>
where
    F: FnOnce(f64, f64) -> f64,
{
    match (chunk.constant_at(a_idx), chunk.constant_at(b_idx)) {
        (Some(Value::Float(a)), Some(Value::Float(b))) => {
            Some((Value::Float(op(*a, *b)), SourceSpan::unknown()))
        }
        _ => None,
    }
}

fn fold_boolean(chunk: &mut Chunk) -> bool {
    let mut i = 0;
    let mut changed = false;

    while i + 1 < chunk.instructions.len() {
        let folded = match (&chunk.instructions[i], &chunk.instructions[i + 1]) {
            (Instruction::LoadConst(idx), Instruction::Not) => match chunk.constant_at(*idx) {
                Some(Value::Bool(b)) => Some(Value::Bool(!b)),
                _ => None,
            },
            _ => None,
        };

        if let Some(result_value) = folded {
            let new_idx = chunk.add_constant(result_value);
            chunk.instructions[i] = Instruction::LoadConst(new_idx);
            chunk.instructions.remove(i + 1);
            chunk.spans.remove(i + 1);
            changed = true;
        } else {
            i += 1;
        }
    }

    changed
}

fn fold_comparison(chunk: &mut Chunk) -> bool {
    let mut i = 0;
    let mut changed = false;

    while i + 2 < chunk.instructions.len() {
        let folded = match (
            &chunk.instructions[i],
            &chunk.instructions[i + 1],
            &chunk.instructions[i + 2],
        ) {
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Eq) => {
                fold_equality(chunk, *a_idx, *b_idx)
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Neq) => {
                fold_equality(chunk, *a_idx, *b_idx).map(|v| match v {
                    Value::Bool(b) => Value::Bool(!b),
                    _ => v,
                })
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Lt) => {
                fold_ordering(chunk, *a_idx, *b_idx, |ord| ord.is_lt())
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Lte) => {
                fold_ordering(chunk, *a_idx, *b_idx, |ord| ord.is_le())
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Gt) => {
                fold_ordering(chunk, *a_idx, *b_idx, |ord| ord.is_gt())
            }
            (Instruction::LoadConst(a_idx), Instruction::LoadConst(b_idx), Instruction::Gte) => {
                fold_ordering(chunk, *a_idx, *b_idx, |ord| ord.is_ge())
            }
            _ => None,
        };

        if let Some(result_value) = folded {
            let new_idx = chunk.add_constant(result_value);
            chunk.instructions[i] = Instruction::LoadConst(new_idx);
            chunk.instructions.remove(i + 2);
            chunk.spans.remove(i + 2);
            chunk.instructions.remove(i + 1);
            chunk.spans.remove(i + 1);
            changed = true;
        } else {
            i += 1;
        }
    }

    changed
}

fn fold_equality(chunk: &Chunk, a_idx: u16, b_idx: u16) -> Option<Value> {
    match (chunk.constant_at(a_idx), chunk.constant_at(b_idx)) {
        (Some(Value::Int(a)), Some(Value::Int(b))) => Some(Value::Bool(a == b)),
        (Some(Value::Float(a)), Some(Value::Float(b))) => Some(Value::Bool(a == b)),
        (Some(Value::Bool(a)), Some(Value::Bool(b))) => Some(Value::Bool(a == b)),
        (Some(Value::Str(a)), Some(Value::Str(b))) => Some(Value::Bool(a == b)),
        (Some(Value::Unit), Some(Value::Unit)) => Some(Value::Bool(true)),
        _ => None,
    }
}

fn fold_ordering<F>(chunk: &Chunk, a_idx: u16, b_idx: u16, cmp: F) -> Option<Value>
where
    F: FnOnce(std::cmp::Ordering) -> bool,
{
    match (chunk.constant_at(a_idx), chunk.constant_at(b_idx)) {
        (Some(Value::Int(a)), Some(Value::Int(b))) => Some(Value::Bool(cmp(a.cmp(b)))),
        (Some(Value::Float(a)), Some(Value::Float(b))) => {
            a.partial_cmp(b).map(|ord| Value::Bool(cmp(ord)))
        }
        _ => None,
    }
}

fn eliminate_dead_code(chunk: &mut Chunk) {
    remove_unreachable_after_jumps(chunk);
    combine_consecutive_pops(chunk);
}

fn remove_unreachable_after_jumps(chunk: &mut Chunk) {
    let mut i = 0;
    while i < chunk.instructions.len() {
        if matches!(chunk.instructions[i], Instruction::Jump(_)) {
            let jump_target = calculate_jump_target(i, &chunk.instructions[i]);
            let j = i + 1;
            while j < chunk.instructions.len() {
                if is_jump_target(j, chunk) || Some(j) == jump_target {
                    break;
                }
                chunk.instructions.remove(j);
                chunk.spans.remove(j);
            }
        }
        i += 1;
    }
}

fn calculate_jump_target(current: usize, instr: &Instruction) -> Option<usize> {
    match instr {
        Instruction::Jump(offset) => {
            let target = current as i32 + 1 + *offset as i32;
            if target >= 0 {
                Some(target as usize)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_jump_target(offset: usize, chunk: &Chunk) -> bool {
    for (i, instr) in chunk.instructions.iter().enumerate() {
        match instr {
            Instruction::Jump(delta) | Instruction::JumpIfFalse(delta) => {
                let target = i as i32 + 1 + *delta as i32;
                if target == offset as i32 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn combine_consecutive_pops(chunk: &mut Chunk) {
    let mut i = 0;
    while i + 1 < chunk.instructions.len() {
        if matches!(chunk.instructions[i], Instruction::Pop)
            && matches!(chunk.instructions[i + 1], Instruction::Pop)
        {
            chunk.instructions.remove(i + 1);
            chunk.spans.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_add_integers() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(10));
        let b = chunk.add_constant(Value::Int(32));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Add);
        chunk.emit(Instruction::Return);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 2);
        assert!(matches!(chunk.instructions[0], Instruction::LoadConst(_)));
        assert_eq!(chunk.instructions[1], Instruction::Return);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }

    #[test]
    fn test_fold_sub_integers() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(50));
        let b = chunk.add_constant(Value::Int(8));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Sub);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }

    #[test]
    fn test_fold_mul_integers() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(6));
        let b = chunk.add_constant(Value::Int(7));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Mul);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }

    #[test]
    fn test_fold_div_integers() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(84));
        let b = chunk.add_constant(Value::Int(2));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Div);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }

    #[test]
    fn test_no_fold_div_by_zero() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(42));
        let b = chunk.add_constant(Value::Int(0));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Div);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 3);
    }

    #[test]
    fn test_fold_not_true() {
        let mut chunk = Chunk::new();
        let t = chunk.add_constant(Value::Bool(true));
        chunk.emit(Instruction::LoadConst(t));
        chunk.emit(Instruction::Not);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Bool(false)));
        }
    }

    #[test]
    fn test_fold_not_false() {
        let mut chunk = Chunk::new();
        let f = chunk.add_constant(Value::Bool(false));
        chunk.emit(Instruction::LoadConst(f));
        chunk.emit(Instruction::Not);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Bool(true)));
        }
    }

    #[test]
    fn test_fold_eq_integers() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(42));
        let b = chunk.add_constant(Value::Int(42));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Eq);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Bool(true)));
        }
    }

    #[test]
    fn test_fold_eq_integers_false() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(42));
        let b = chunk.add_constant(Value::Int(43));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Eq);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Bool(false)));
        }
    }

    #[test]
    fn test_fold_floats() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Float(3.0));
        let b = chunk.add_constant(Value::Float(14.0));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Add);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Float(17.0)));
        }
    }

    #[test]
    fn test_chained_folding() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(2));
        let b = chunk.add_constant(Value::Int(3));
        let c = chunk.add_constant(Value::Int(7));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Mul);
        chunk.emit(Instruction::LoadConst(c));
        chunk.emit(Instruction::Mul);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }

    #[test]
    fn test_combine_consecutive_pops() {
        let mut chunk = Chunk::new();
        chunk.emit(Instruction::Pop);
        chunk.emit(Instruction::Pop);
        chunk.emit(Instruction::Pop);
        chunk.emit(Instruction::Return);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 2);
        assert_eq!(chunk.instructions[0], Instruction::Pop);
        assert_eq!(chunk.instructions[1], Instruction::Return);
    }

    #[test]
    fn test_fold_lt() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(1));
        let b = chunk.add_constant(Value::Int(2));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Lt);

        optimize_chunk(&mut chunk);

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Bool(true)));
        }
    }

    #[test]
    fn test_chunk_optimize_method() {
        let mut chunk = Chunk::new();
        let a = chunk.add_constant(Value::Int(10));
        let b = chunk.add_constant(Value::Int(32));
        chunk.emit(Instruction::LoadConst(a));
        chunk.emit(Instruction::LoadConst(b));
        chunk.emit(Instruction::Add);

        chunk.optimize();

        assert_eq!(chunk.instructions.len(), 1);
        if let Instruction::LoadConst(idx) = chunk.instructions[0] {
            assert_eq!(chunk.constant_at(idx), Some(&Value::Int(42)));
        }
    }
}
