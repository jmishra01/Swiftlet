use std::io;
use thiserror::Error;

/// Errors produced while loading or transforming grammars.
#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Failed to parse grammar: {0}")]
    Parse(String),
    #[error("Rule '{0}' is used, but production rules are not defined.")]
    RuleProductionNotFound(String),
}

/// Errors produced while tokenizing parser input.
#[derive(Debug, Error)]
pub enum LexerError {
    #[error(
        "Tokenization failed at text {location} (line {line}, column {column}). Expected one of: {expected:?}\n{text}\n{caret}"
    )]
    Tokenization {
        location: usize,
        line: usize,
        column: usize,
        expected: Vec<String>,
        text: String,
        caret: String,
    },
    #[error("Tokenization State Error: Something went wrong at state {0}")]
    State(String),
}

/// Errors produced while constructing parse tables or parsing token streams.
#[derive(Debug, Error)]
pub enum ParseError {
    /// A Shift/reduce or reduce/reduce conflict was found in the grammar.
    /// `description` contains the conflict type and the involved actions formatted as a string.
    /// For full detail, rebuild with `debug: true`
    #[error("Parser conflict: {description:?}\nFor more information, run with debug: true.")]
    Conflict { description: String },
    #[error("Didn't find transition for non-terminal: {0:?}")]
    Transition(String),
    #[error("Failed to parse input text: \"{0}\"")]
    FailedToParse(String),
    #[error("Didn't find any rule for word: \"{0}\" in the given grammar.")]
    RuleNotFound(String),
}

/// Public crate-level error type returned by Swiftlet APIs.
#[derive(Debug, Error)]
pub enum SwiftletError {
    #[error(transparent)]
    Grammar(#[from] GrammarError),
    #[error("Failed to read grammar file '{path}': {source}")]
    GrammarFileReadError { path: String, source: io::Error },
    #[error(transparent)]
    Lexer(#[from] LexerError),
    #[error(transparent)]
    Parse(#[from] ParseError),
}
