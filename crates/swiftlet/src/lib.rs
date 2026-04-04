//! *Swiftlet* is a high-performance text-parsing library for Rust, inspired by Python’s [Lark](https://lark-parser.readthedocs.io/en/stable/index.html).
//!
//! # Example
//! ```
//! use swiftlet::preclude::*;
//! use std::sync::Arc;
//!
//!
//! fn calculate(ast: &Ast) -> i32 {
//!     match ast {
//!         Ast::Token(token) => {
//!             token.word().parse::<i32>().unwrap()
//!         }
//!         Ast::Tree(tree, children) => {
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
//!     let conf = Arc::new(ParserConfig::default());
//!     let swiftlet = Swiftlet::from_str(grammar).expect("failed to load grammar");
//!     let parser = swiftlet.parser(conf);
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
pub mod ast;
mod engine;
mod common;
pub mod error;
pub mod grammar;
pub mod lexer;
pub mod load_grammar;
mod macros;
pub mod parser;
pub mod parser_frontends;
pub mod preclude;
mod transform;

use crate::ast::Ast;
pub use crate::engine::ParserEngine;
use crate::grammar::Algorithm;
use crate::load_grammar::load_grammar;
use crate::parser_frontends::GrammarRuntime;
use error::SwiftletError;
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
pub struct ParserConfig {
    pub start: String,
    pub algorithm: Algorithm,
    pub ambiguity: Ambiguity,
    pub debug: bool,
}

impl Default for ParserConfig {
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

fn normalize_grammar(grammar: &str) -> String {
    format!(
        r#"{}
        "#,
        grammar.trim()
    )
}

/// Reusable loaded grammar that can build multiple parser instances.
pub struct Swiftlet {
    frontend: Arc<GrammarRuntime>,
}

impl Swiftlet {
    /// Loads and validates a grammar from inline text.
    pub fn from_str(grammar: &str) -> Result<Self, SwiftletError> {
        let grammar = normalize_grammar(grammar);

        #[cfg(feature = "debug")]
        let frontend = load_grammar(&grammar, Arc::new(ParserConfig::default()))?;

        #[cfg(not(feature = "debug"))]
        let frontend = load_grammar(&grammar)?;

        Ok(Self { frontend })
    }

    /// Loads and validates a grammar from a file path.
    pub fn from_file(path: &str) -> Result<Self, SwiftletError> {
        let content = std::fs::read_to_string(path).map_err(|source| {
            SwiftletError::GrammarFileReadError {
                path: path.to_string(),
                source,
            }
        })?;

        Self::from_str(&content)
    }

    /// Builds a parser instance for the given parser options.
    pub fn parser(&self, parser_option: Arc<ParserConfig>) -> Parser {
        Parser {
            parser_engine: ParserEngine::new(self.frontend.clone(), parser_option),
        }
    }
}

/// Parser instance built from a validated Swiftlet grammar plus parser options.
pub struct Parser {
    parser_engine: ParserEngine,
}

impl Parser {
    /// Parses the provided input text and returns the generated AST.
    pub fn parse(&self, text: &str) -> Result<Ast, SwiftletError> {
        self.parser_engine.parse(text)
    }
}
