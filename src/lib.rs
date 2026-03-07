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
    /// Returns default parser options used by `Barat`.
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
    /// Constructs a parser from grammar text.
    pub fn from_string(grammar: String, parser_option: Arc<ParserOption>) -> Self {
        Self {
            grammar_builder: GrammarBuilder::new(load_grammar(grammar, parser_option.clone()), parser_option.clone()),
        }
    }

    /// Constructs a parser from a grammar file path.
    ///
    /// Panics if the file cannot be read.
    pub fn from_file(file: String, parser_conf: Arc<ParserOption>) -> Self {
        let content = std::fs::read_to_string(file).unwrap();
        Self::from_string(content, parser_conf)
    }

    /// Parses the provided input text and returns the generated AST.
    pub fn parse(&mut self, text: &str) -> Result<AST, ParserError> {
        self.grammar_builder.parse(text)
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar::Algorithm;
    use crate::{Ambiguity, Barat, ParserOption};
    use std::fs;
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

    #[test]
    fn parser_option_default_values() {
        let opt = ParserOption::default();
        assert_eq!(opt.start, "start".to_string());
        assert!(matches!(opt.algorithm, Algorithm::Earley));
        assert!(matches!(opt.ambiguity, Ambiguity::Resolve));
        assert!(!opt.debug);
    }

    #[test]
    fn barat_from_file_parses_input() {
        let grammar = r#"
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        "#;
        let path = std::env::temp_dir().join("barat_test_grammar.lark");
        fs::write(&path, grammar).unwrap();

        let parser_option = Arc::new(ParserOption {
            algorithm: Algorithm::CLR,
            ..ParserOption::default()
        });
        let mut parser = Barat::from_file(path.to_string_lossy().to_string(), parser_option);
        assert!(parser.parse("10").is_ok());

        let _ = fs::remove_file(path);
    }
}
