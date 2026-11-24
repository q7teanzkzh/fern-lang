//! fern-lang: a tiny SSA IR with a constant-folding pass.
//!
//! The crate is a teaching playground: the IR is intentionally small, the
//! pass is a single sweep, and the tests pin the cases where folding must
//! and must not fire.

pub mod fold;
pub mod ir;
pub mod opt;

use ir::{Function, ParseError};

/// Parse `text`, fold it, and return the rewritten module plus the number of
/// instructions that were folded.
pub fn fold_text(text: &str) -> Result<(String, usize), ParseError> {
    let mut function = Function::parse(text)?;
    let folded = fold::fold_constants(&mut function);
    Ok((function.to_text(), folded))
}

/// Parse `text`, run the full [`opt::optimize`] pipeline, and return the
/// rewritten module plus its [`opt::Report`].
pub fn optimize_text(text: &str) -> Result<(String, opt::Report), ParseError> {
    let mut function = Function::parse(text)?;
    let report = opt::optimize(&mut function);
    Ok((function.to_text(), report))
}
