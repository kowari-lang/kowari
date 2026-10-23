//! Parser: tokens → AST.

use kowari_ast::Ast;
use kowari_diagnostics::Diag;
use kowari_lexer::Token;

pub fn parse(_tokens: Vec<Token>) -> Result<Ast, Vec<Diag>> {
    unimplemented!("parser not implemented yet")
}
