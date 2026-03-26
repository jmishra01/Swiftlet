//! *Swiftlet* is a high-performance text-parsing library for Rust, inspired by Python’s [Lark](https://lark-parser.readthedocs.io/en/stable/index.html).
//!
//! # Example
//! ```
//! use swiftlet::{Swiftlet, ParserOption, ast::AST};
//! use std::sync::Arc;
//!
//!
//! fn calculate(ast: &AST) -> i32 {
//!     match ast {
//!         AST::Token(token) => {
//!             token.word().parse::<i32>().unwrap()
//!         }
//!         AST::Tree(tree, children) => {
//!             match tree.as_str() {
//!                 "start" | "expr" => calculate(&children[0]),
//!                 "add" => calculate(&children[0]) + calculate(&children[2]),
//!                 "sub" => calculate(&children[0]) - calculate(&children[2]),
//!                 _ => {
//!                     panic!("Invalid tree: {}", tree);
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let grammar = r#"
//!         start: expr
//!         expr: expr "+" INT -> add
//!             | expr "-" INT -> sub
//!             | INT
//!         %import (WS, INT)
//!         %ignore WS
//!         "#;
//!
//!     let conf = Arc::new(ParserOption::default());
//!     let mut parser = Swiftlet::from_string(grammar, conf);
//!     let text = "10 - 2 + 5 - 2";
//!
//!     match parser.parse(text) {
//!         Ok(tree) => {
//!             print!("AST: "); tree.print();
//!             println!("Total: {}", calculate(&tree));
//!         }
//!         Err(e) => {
//!             println!("Error: {}", e);
//!         }
//!     }
//! }
//! ```
mod builder;
mod common;
pub mod grammar;
pub mod lexer;
pub mod load_grammar;
mod macros;
pub mod parser;
pub mod parser_frontends;
mod transform;
pub mod ast;
pub mod preclude;

pub use crate::builder::GrammarBuilder;
use crate::grammar::Algorithm;
use crate::ast::AST;
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

/// Configures parser construction and runtime behavior.
#[derive(Debug, Clone)]
pub struct ParserOption {
    pub start: String,
    pub algorithm: Algorithm,
    pub ambiguity: Ambiguity,
    pub debug: bool,
}

impl Default for ParserOption {
    /// Returns default parser options used by `Swiftlet`.
    fn default() -> Self {
        Self {
            start: "start".to_string(),
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Resolve,
            debug: false,
        }
    }
}

/// High-level parser entry point built from a grammar definition.
pub struct Swiftlet {
    grammar_builder: GrammarBuilder,
}

impl Swiftlet {
    /// Constructs a parser from grammar text.
    pub fn from_string(grammar: &str, parser_option: Arc<ParserOption>) -> Self {
        #[cfg(feature = "debug")]
        let _grammar = load_grammar(grammar, parser_option.clone());

        #[cfg(not(feature = "debug"))]
        let _grammar = load_grammar(grammar);

        Self {
            grammar_builder: GrammarBuilder::new(_grammar, parser_option.clone()),
        }
    }

    /// Constructs a parser from a grammar file path.
    ///
    /// Panics if the file cannot be read.
    pub fn from_file(file: String, parser_option: Arc<ParserOption>) -> Self {
        let content = std::fs::read_to_string(file).unwrap();
        Self::from_string(content.as_str(), parser_option)
    }

    /// Parses the provided input text and returns the generated AST.
    pub fn parse(&self, text: &str) -> Result<AST, ParserError> {
        self.grammar_builder.parse(text)
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar::Algorithm;
    use crate::{Ambiguity, ParserOption, Swiftlet};
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

        let parser_option = ParserOption {
            algorithm: Algorithm::CLR,
            ..ParserOption::default()
        };
        let tp = Swiftlet::from_string(text, Arc::from(parser_option));
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
    fn swiftlet_from_file_parses_input() {
        let grammar = r#"
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        "#;
        let path = std::env::temp_dir().join("swiftlet_test_grammar.lark");
        fs::write(&path, grammar).unwrap();

        let parser_option = Arc::new(ParserOption {
            algorithm: Algorithm::CLR,
            ..ParserOption::default()
        });
        let parser = Swiftlet::from_file(path.to_string_lossy().to_string(), parser_option);
        assert!(parser.parse("10").is_ok());

        let _ = fs::remove_file(path);
    }
}
