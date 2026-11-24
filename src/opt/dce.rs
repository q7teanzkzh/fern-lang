//! Dead code elimination.
//!
//! Liveness is computed backwards from every `ret`; unreached definitions are
//! dropped and the surviving values are renumbered so the module stays dense.
//! Renumbering invalidates previously handed-out ids, so run this after the
//! passes that rely on stable indices.

use crate::ir::{Function, Inst, ValueId};

/// Remove instructions that no `ret` transitively uses.
///
/// Returns the number of instructions removed.
pub fn remove_unused(function: &mut Function) -> usize {
    let mut live = vec![false; function.insts.len()];
    let mut work: Vec<ValueId> = function
        .insts
        .iter()
        .filter_map(|inst| match inst {
            Inst::Ret(src) => Some(*src),
            _ => None,
        })
        .collect();

    while let Some(id) = work.pop() {
        let Some(slot) = live.get_mut(id as usize) else {
            continue;
        };
        if *slot {
            continue;
        }
        *slot = true;
        match &function.insts[id as usize] {
            Inst::Bin(_, lhs, rhs) => {
                work.push(*lhs);
                work.push(*rhs);
            }
            Inst::Copy(src) => work.push(*src),
            Inst::Const(_) | Inst::Ret(_) => {}
        }
    }

    // `ret` is never a definition and stays in place; only unused values count.
    let removed = function
        .insts
        .iter()
        .enumerate()
        .filter(|(id, inst)| !matches!(inst, Inst::Ret(_)) && !live[*id])
        .count();
    if removed == 0 {
        return 0;
    }

    let mut remap = vec![0 as ValueId; function.insts.len()];
    let mut next = 0 as ValueId;
    for (id, keep) in live.iter().enumerate() {
        if *keep {
            remap[id] = next;
            next += 1;
        }
    }

    let mut rebuilt = Vec::with_capacity(function.insts.len() - removed);
    for (id, inst) in function.insts.iter().enumerate() {
        if !live[id] && !matches!(inst, Inst::Ret(_)) {
            continue;
        }
        rebuilt.push(match inst {
            Inst::Const(value) => Inst::Const(*value),
            Inst::Bin(op, lhs, rhs) => Inst::Bin(*op, remap_id(&remap, *lhs), remap_id(&remap, *rhs)),
            Inst::Copy(src) => Inst::Copy(remap_id(&remap, *src)),
            Inst::Ret(src) => Inst::Ret(remap_id(&remap, *src)),
        });
    }
    function.insts = rebuilt;
    removed
}

/// Ids outside the table cannot be renumbered; keep them so the module still
/// round-trips and a later pass can report the dangling use.
fn remap_id(remap: &[ValueId], id: ValueId) -> ValueId {
    remap.get(id as usize).copied().unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::BinOp;

    #[test]
    fn drops_an_unused_constant_and_renumbers() {
        let mut function = Function {
            name: "drop".to_owned(),
            insts: vec![
                Inst::Const(1),
                Inst::Const(2),
                Inst::Copy(0),
                Inst::Ret(2),
            ],
        };
        assert_eq!(remove_unused(&mut function), 1);
        assert_eq!(function.to_text(), "fn drop\nv0 = const 1\nv1 = copy v0\nret v1\n");
    }

    #[test]
    fn keeps_a_shared_operand() {
        let mut function = Function {
            name: "keep".to_owned(),
            insts: vec![
                Inst::Const(4),
                Inst::Const(5),
                Inst::Bin(BinOp::Add, 0, 1),
                Inst::Bin(BinOp::Mul, 2, 1),
                Inst::Ret(3),
            ],
        };
        assert_eq!(remove_unused(&mut function), 0);
        assert_eq!(function.insts.len(), 5);
    }

    #[test]
    fn removes_a_long_dead_chain() {
        let mut function = Function {
            name: "chain".to_owned(),
            insts: vec![
                Inst::Const(9),
                Inst::Const(3),
                Inst::Bin(BinOp::Sub, 0, 1),
                Inst::Const(8),
                Inst::Ret(3),
            ],
        };
        assert_eq!(remove_unused(&mut function), 3);
        assert_eq!(function.insts, vec![Inst::Const(8), Inst::Ret(0)]);
    }
}
