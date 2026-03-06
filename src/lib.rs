//! Barat — a context-free grammar parser inspired by Python's Lark.
//!
mod builder;
mod common;
pub mod grammar;
pub mod lexer;
pub mod load_grammar;
mod macros;
pub mod parser;
pub mod parser_frontends;
mod transform;

pub use crate::builder::GrammarBuilder;
use crate::grammar::Algorithm;
use crate::lexer::AST;
use crate::load_grammar::load_grammar;
use crate::parser::error::ParserError;
use std::sync::Arc;


/// Ambiguity Enum
/// used to decide how to handle ambiguity in the parse. Relevant to Earley algorithm
#[derive(Clone, Debug)]
pub enum Ambiguity {
    /// Resolve - return first derivation.
    Resolve,
    /// Explicit - return all derivation under '_ambiguity' tree node
    Explicit,
}

#[derive(Debug, Clone)]
pub struct ParserOption {
    pub start: String,
    pub algorithm: Algorithm,
    pub ambiguity: Ambiguity,
    pub debug: bool,
}

impl Default for ParserOption {
    fn default() -> Self {
        Self {
            start: "start".to_string(),
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Resolve,
            debug: false,
        }
    }
}

pub struct Barat {
    grammar_builder: GrammarBuilder,
}

impl Barat {
    pub fn from_string(grammar: String, parser_conf: Arc<ParserOption>) -> Self {
        Self {
            grammar_builder: GrammarBuilder::new(load_grammar(grammar), parser_conf.clone()),
        }
    }

    pub fn from_file(file: String, parser_conf: Arc<ParserOption>) -> Self {
        let content = std::fs::read_to_string(file).unwrap();
        Self::from_string(content, parser_conf)
    }

    pub fn parse(&mut self, text: &str) -> Result<AST, ParserError> {
        self.grammar_builder.parse(text)
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar::Algorithm;
    use crate::{Barat, ParserOption};
    use std::sync::Arc;

    #[test]
    fn test_grammar_1() {
        let text = r#"
        start: expr
        expr: (expr "+")? t
        t: INT
        %import (WS, INT)
        %ignore WS
        "#;

        let parser_option = ParserOption { algorithm: Algorithm::CLR, ..ParserOption::default() };
        let mut tp = Barat::from_string(text.to_string(), Arc::from(parser_option));
        assert!(tp.parse("1 + 2").is_ok());
    }
}
