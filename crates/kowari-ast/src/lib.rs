//! Shared AST type definitions (data only).

pub mod expr;
pub mod item;
pub mod literal;
pub mod pat;
pub mod stmt;
pub mod ty;

#[derive(Debug)]
pub struct Ast {
    // TODO: fill in during M1a
}

#[derive(Clone, Debug)]
pub enum Visibility {
    Public,
    Private,
}
