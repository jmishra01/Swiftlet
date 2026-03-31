//! *Swiftlet* is a high-performance text-parsing library for Rust, inspired by Python’s [Lark](https://lark-parser.readthedocs.io/en/stable/index.html).
//!
//! # Example
//! ```
//! use swiftlet::preclude::*;
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
pub mod ast;
mod builder;
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

use crate::ast::AST;
pub use crate::builder::GrammarBuilder;
use crate::grammar::Algorithm;
use crate::lexer::Token;
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

/// Controls how the parser obtains terminals from input text.
#[derive(Clone, Debug)]
pub enum LexerMode {
    /// Use the current global tokenizer before parsing.
    Basic,
    /// Let the Earley parser request only the terminals expected at each position.
    Dynamic,
    /// Parse terminals directly inside Earley without relying on the pre-tokenized stream.
    Scannerless,
}

/// Configures parser construction and runtime behavior.
#[derive(Debug, Clone)]
pub struct ParserOption {
    pub start: String,
    pub algorithm: Algorithm,
    pub ambiguity: Ambiguity,
    pub lexer_mode: LexerMode,
    pub debug: bool,
}

impl Default for ParserOption {
    /// Returns default parser options used by `Swiftlet`.
    fn default() -> Self {
        Self {
            start: "start".to_string(),
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Resolve,
            lexer_mode: LexerMode::Basic,
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
    pub fn from_string(
        grammar: &str,
        parser_option: Arc<ParserOption>,
    ) -> Result<Self, ParserError> {
        #[cfg(feature = "debug")]
        let _grammar = match load_grammar(grammar, parser_option.clone()) {
            Ok(g) => g,
            Err(err) => return Err(err),
        };

        #[cfg(not(feature = "debug"))]
        let _grammar = match load_grammar(grammar) {
            Ok(g) => g,
            Err(err) => return Err(err),
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

    /// Tokenizes input text and returns the resulting token stream.
    pub fn tokens(&self, text: &str) -> Vec<Token> {
        self.grammar_builder.tokens(text)
    }

    /// Prints a readable debug view of the token stream for `text`.
    pub fn print_tokens(&self, text: &str) {
        for token in self.tokens(text) {
            println!("{}", format_token_debug(&token));
        }
    }
}

fn format_token_debug(token: &Token) -> String {
    format!(
        "{} -> {:?} @ {}..{}",
        token.get_terminal(),
        token.word(),
        token.get_start(),
        token.get_end()
    )
}
