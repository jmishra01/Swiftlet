use crate::ast::Ast;
use crate::error::SwiftletError;
use crate::lexer::Tokenizer;
use crate::parser_frontends::GrammarRuntime;
use std::sync::Arc;

pub mod clr;
pub mod earley;
mod utils;

/// Common interface implemented by concrete parser backends.
pub trait ParserBackend {
    /// Returns parser frontend containing lexer and grammar configuration.
    fn get_parser_frontend(&self) -> Arc<GrammarRuntime>;

    /// Parses token stream into AST.
    fn parse(&self, token: &mut Tokenizer) -> Result<Ast, SwiftletError>;
}
