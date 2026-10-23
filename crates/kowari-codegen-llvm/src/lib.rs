//! IR → LLVM IR → object file.

use kowari_diagnostics::Diag;
use kowari_ir::Mir;

pub struct Object {
    pub bytes: Vec<u8>,
}

pub fn compile(_mir: Mir, _opts: ()) -> Result<Object, Diag> {
    unimplemented!("codegen not implemented yet")
}
