//! Constant folding.
//!
//! Because the IR is SSA and definitions precede uses, a single forward
//! sweep reaches the fixpoint: a slot folded in this sweep is visible to
//! every later operand lookup.

use crate::ir::{Function, Inst, ValueId};

/// Fold every instruction whose operands are already known constants.
///
/// Returns how many instructions were rewritten. `Const` instructions are
/// left alone (they are already folded), and operations that do not have a
/// defined constant result stay in place.
pub fn fold_constants(function: &mut Function) -> usize {
    let mut known: Vec<Option<u64>> = Vec::with_capacity(function.insts.len());
    let mut folded = 0;
    for idx in 0..function.insts.len() {
        let value = match function.insts[idx] {
            Inst::Const(value) => Some(value),
            Inst::Bin(op, lhs, rhs) => lookup(&known, lhs)
                .zip(lookup(&known, rhs))
                .and_then(|(lhs, rhs)| op.eval(lhs, rhs)),
            Inst::Copy(src) => lookup(&known, src),
            Inst::Ret(_) => None,
        };
        if let Some(value) = value {
            if function.insts[idx] != Inst::Const(value) {
                function.insts[idx] = Inst::Const(value);
                folded += 1;
            }
        }
        known.push(match function.insts[idx] {
            Inst::Const(value) => Some(value),
            _ => None,
        });
    }
    folded
}

fn lookup(known: &[Option<u64>], id: ValueId) -> Option<u64> {
    known.get(id as usize).copied().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::BinOp;

    #[test]
    fn folds_through_a_chain_transitively() {
        let mut function = Function {
            name: "chain".to_owned(),
            insts: vec![
                Inst::Const(2),
                Inst::Const(3),
                Inst::Bin(BinOp::Add, 0, 1),
                Inst::Bin(BinOp::Mul, 2, 1),
                Inst::Ret(3),
            ],
        };
        assert_eq!(fold_constants(&mut function), 2);
        assert_eq!(function.insts[2], Inst::Const(5));
        assert_eq!(function.insts[3], Inst::Const(15));
    }

    #[test]
    fn keeps_division_by_zero() {
        let mut function = Function {
            name: "divzero".to_owned(),
            insts: vec![Inst::Const(7), Inst::Const(0), Inst::Bin(BinOp::Div, 0, 1), Inst::Ret(2)],
        };
        assert_eq!(fold_constants(&mut function), 0);
        assert_eq!(function.insts[2], Inst::Bin(BinOp::Div, 0, 1));
    }

    #[test]
    fn folds_copy_of_constant() {
        let mut function = Function {
            name: "copy".to_owned(),
            insts: vec![Inst::Const(9), Inst::Copy(0), Inst::Ret(1)],
        };
        assert_eq!(fold_constants(&mut function), 1);
        assert_eq!(function.insts[1], Inst::Const(9));
    }
}
