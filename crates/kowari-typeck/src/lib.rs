//! Type checker + affine resource analysis.

use kowari_ast::Ast;
use kowari_diagnostics::Diag;

/// Typed HIR. Populated by typeck; consumed by `kowari-ir`.
pub struct Hir {
    // TODO
}

pub fn check(_ast: Ast) -> Result<Hir, Vec<Diag>> {
    unimplemented!("typeck not implemented yet")
}