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
//!     let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
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
pub mod error;

pub use crate::builder::GrammarBuilder;
use crate::grammar::Algorithm;
use crate::ast::AST;
use crate::load_grammar::load_grammar;
use error::ParserError;
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
    pub fn from_string(grammar: &str, parser_option: Arc<ParserOption>) -> Result<Self, ParserError> {
        #[cfg(feature = "debug")]
        let _grammar = match load_grammar(grammar, parser_option.clone()) {
            Ok(g) => g,
            Err(err) => return Err(err)
        };

        #[cfg(not(feature = "debug"))]
        let _grammar = match load_grammar(grammar) {
            Ok(g) => g,
            Err(err) => return Err(err)
        };

        Ok(Self {
            grammar_builder: GrammarBuilder::new(_grammar, parser_option.clone()),
        })
    }

    /// Constructs a parser from a grammar file path.
    ///
    /// Panics if the file cannot be read.
    pub fn from_file(file: String, parser_option: Arc<ParserOption>) -> Result<Self, ParserError> {
        let content = std::fs::read_to_string(file).unwrap();
        Self::from_string(content.as_str(), parser_option)
    }

    /// Parses the provided input text and returns the generated AST.
    pub fn parse(&self, text: &str) -> Result<AST, ParserError> {
        self.grammar_builder.parse(text)
    }
}