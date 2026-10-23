//! Language IR: HIR → MIR-like IR (references first-class).

use kowari_diagnostics::Diag;
use kowari_typeck::Hir;

pub struct Mir {
    // TODO
}

pub fn lower(_hir: Hir) -> Result<Mir, Vec<Diag>> {
    unimplemented!("IR lowering not implemented yet")
}
