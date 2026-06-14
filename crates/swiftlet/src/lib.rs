//! *Swiftlet* is a high-performance text-parsing library for Rust, inspired by Python's [Lark](https://lark-parser.readthedocs.io/en/stable/index.html).
//!
//! # Example
//! ```
//! use swiftlet::preclude::*;
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
//!                     panic!("unexpected tree: {}", tree);
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
//!     let swiftlet = Swiftlet::from_str(grammar).expect("failed to load grammar");
//!     let parser = swiftlet.parser(ParserConfig::default());
//!     let text = "10 - 2 + 5 - 2";
//!
//!     match parser.parse(text) {
//!         Ok(tree) => {
//!             print!("AST: "); tree.print();
//!             println!("Total: {}", calculate(&tree));
//!         }
//!         Err(e) => println!("Error: {}", e),
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
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Controls how the Earley parser handles ambiguous grammars.
#[derive(Clone, Debug)]
pub enum Ambiguity {
    /// Resolve - return the first derivation found and discard the rest.
    Resolve,
    /// Explicit - return all derivations nested under an '_ambiguity' tree node.
    Explicit,
}

impl Display for Ambiguity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ambiguity::Resolve => write!(f, "resolve"),
            Ambiguity::Explicit => write!(f, "explicit"),
        }
    }
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
    fn default() -> Self {
        Self {
            start: "start".to_string(),
            algorithm: Algorithm::Earley,
            ambiguity: Ambiguity::Resolve,
            debug: false,
        }
    }
}

impl Display for ParserConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ParserConfig {{ start: {}, algorithm: {}, ambiguity: {}, debug: {} }}",
            self.start, self.algorithm, self.ambiguity, self.debug
        )
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

    /// Builds a parser instance for the given configuration.
    ///
    /// Accepts `ParserConfig` by value, by reference clone, or as `Arc<ParserConfig>`
    /// ``ìgnore
    /// swiftlet.parser(ParserConfig::default())            // by value
    /// swiftlet.parser(Arc::new(ParserConfig::default())   // existing Arc
    /// ```
    pub fn parser(&self, config: impl Into<Arc<ParserConfig>>) -> Parser {
        let config = config.into();
        #[cfg(feature = "debug")]
        if config.debug {
            println!("\nBNF Grammar");
            println!("===========");
            let rules = self.frontend.get_parser().rules.clone();

            for (_, v) in &rules {
                for rule in v {
                    println!("{}", rule);
                }
            }
        }

        Parser {
            parser_engine: ParserEngine::new(self.frontend.clone(), config),
        }
    }
}

/// Parser instance built from a validated Swiftlet grammar plus configuration.
pub struct Parser {
    parser_engine: ParserEngine,
}

impl Parser {
    /// Parses the provided input text and returns the generated AST.
    pub fn parse(&self, text: &str) -> Result<Ast, SwiftletError> {
        self.parser_engine.parse(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguity_display_works_for_both_variants() {
        assert_eq!(format!("{}", Ambiguity::Resolve), "resolve");
        assert_eq!(format!("{}", Ambiguity::Explicit), "explicit");
    }

    #[test]
    fn parser_config_display_contains_all_fields() {
        let config = ParserConfig::default();
        let s = format!("{}", config);
        assert!(s.contains("start"));
        assert!(s.contains("Earley"));
        assert!(s.contains("resolve"));
        assert!(s.contains("false"));
    }

    #[test]
    fn parser_config_debug_is_derivable() {
        let config = ParserConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ParserConfig"));
    }

    #[test]
    fn from_file_returns_error_for_missing_path() {
        let path = std::env::temp_dir()
            .join("swiftlet_nonexistent_test_grammar.lark")
            .to_string_lossy()
            .into_owned();
        let err = match Swiftlet::from_file(&path) {
            Ok(_) => panic!("expected error for missing file"),
            Err(e) => e,
        };
        assert!(matches!(
            err,
            error::SwiftletError::GrammarFileReadError { .. }
        ));
    }

    #[test]
    fn from_file_loads_valid_grammar_file() {
        use std::io::Write;
        let grammar = "start: WORD\nWORD: /\\w+/\n";
        let path = std::env::temp_dir().join("swiftlet_test_grammar_valid.lark");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(grammar.as_bytes()).unwrap();

        let swiftlet = Swiftlet::from_file(path.to_str().unwrap())
            .expect("valid grammar file should load");
        let result = swiftlet
            .parser(ParserConfig::default())
            .parse("hello");
        assert!(result.is_ok());
        std::fs::remove_file(path).ok();
    }
}
