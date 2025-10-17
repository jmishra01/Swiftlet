use crate::lexer::{Tokenizer, AST};
use crate::parser::error::ParserError;
use crate::parser_frontends::ParserFrontend;
use std::sync::Arc;

pub mod clr;
pub mod earley;
mod utils;
pub mod error;

pub trait Parser {
    fn get_parser_frontend(&self) -> Arc<ParserFrontend>;

    fn parse(&self, token: Tokenizer) -> Result<AST, ParserError>;
}
