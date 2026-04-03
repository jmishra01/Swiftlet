use crate::lexer::Symbol;
use crate::parser::clr::ActionTable;
use indexmap::IndexSet;
use std::sync::Arc;
use thiserror::Error;

/// Enumerates parser construction and runtime failures.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("{conflict} conflict: {lr_table:?}\nFor more information, run with debug: true.")]
    Conflict {lr_table: IndexSet<ActionTable>, conflict: String},
    #[error("Didn't find transition for non-terminal: {0:?}")]
    TransitionError(Arc<Symbol>),
    #[error("Failed to parser input text: \"{0}\"")]
    FailedToParse(String),
    #[error("Failed to parse grammar: {0}")]
    GrammarParseError(String),
    #[error(
        "Tokenization failed at text {location} (line {line}, column {column}). Expected one of: {expected:?}\n{text}\n{caret}"
    )]
    TokenizationError {
        location: usize,
        line: usize,
        column: usize,
        expected: Vec<String>,
        text: String,
        caret: String,
    },
    #[error("Tokenization State Error: Something went wrong at state {0}")]
    TokenizationStateError (String),
    #[error("Didn't find any rule for word: \"{0}\" in the given grammar.")]
    RuleNotFound(String),
    #[error("Rule '{0}' is used, but production rules are not defined.")]
    RuleProductionNotFound(String),
}
