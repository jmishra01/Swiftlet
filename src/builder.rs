use crate::grammar::Algorithm;
use crate::lexer::{Tokenizer, AST};
use crate::parser::clr::Clr;
use crate::parser::earley::EarleyParser;
use crate::parser::error::ParserError;
use crate::parser::Parser;
use crate::parser_frontends::ParserFrontend;
use crate::ParserOption;
use std::sync::Arc;

pub struct GrammarBuilder {
    parser: Box<dyn Parser + Send + Sync>,
}

impl GrammarBuilder {
    pub fn new(parser_frontend: Arc<ParserFrontend>, parser_conf: Arc<ParserOption>) -> Self {
        match parser_conf.algorithm {
            Algorithm::Earley => Self {
                parser: Box::new(EarleyParser::new(parser_frontend, parser_conf)),
            },
            Algorithm::CLR => Self {
                parser: Box::new(Clr::new(parser_frontend, parser_conf)),
            },
        }
    }

    pub fn get_tokens(&self, text: &str, ignore: &[String]) -> Tokenizer {
        self.parser.get_parser_frontend().tokenizer(text, ignore)
    }

    pub fn parse(&self, text: &str) -> Result<AST, ParserError> {
        let tokens = self.get_tokens(
            text,
            self.parser.get_parser_frontend().get_parser().get_ignores(),
        );
        self.parser.parse(tokens)
    }
}
