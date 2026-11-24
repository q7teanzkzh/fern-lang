//! Optimization pipelines.
//!
//! Folding first is what makes compaction worthwhile: it leaves constant
//! definitions with no remaining users, which [`dce`] then drops.

pub mod dce;

use crate::ir::Function;

/// Counts reported by [`optimize`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub folded: usize,
    pub removed: usize,
}

/// Fold constants, then drop the instructions left without users.
pub fn optimize(function: &mut Function) -> Report {
    let folded = crate::fold::fold_constants(function);
    let removed = dce::remove_unused(function);
    Report { folded, removed }
}
