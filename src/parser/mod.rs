use crate::lexer::{Tokenizer, AST};
use crate::parser::error::ParserError;
use crate::parser_frontends::ParserFrontend;
use std::sync::Arc;

pub mod clr;
pub mod earley;
mod utils;
pub mod error;

pub trait Parser {
    /// Returns parser frontend containing lexer and grammar configuration.
    fn get_parser_frontend(&self) -> Arc<ParserFrontend>;

    /// Parses token stream into AST.
    fn parse(&self, token: Tokenizer) -> Result<AST, ParserError>;
}
