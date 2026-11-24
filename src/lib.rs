//! fern-lang: a tiny SSA IR with a constant-folding pass.
//!
//! The crate is a teaching playground: the IR is intentionally small, the
//! pass is a single sweep, and the tests pin the cases where folding must
//! and must not fire.

pub mod fold;
pub mod ir;

use ir::{Function, ParseError};

/// Parse `text`, fold it, and return the rewritten module plus the number of
/// instructions that were folded.
pub fn fold_text(text: &str) -> Result<(String, usize), ParseError> {
    let mut function = Function::parse(text)?;
    let folded = fold::fold_constants(&mut function);
    Ok((function.to_text(), folded))
}
