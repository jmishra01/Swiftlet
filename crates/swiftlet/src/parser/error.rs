use crate::lexer::Symbol;
use crate::parser::clr::ActionTable;
use indexmap::IndexSet;
use std::sync::Arc;
use thiserror::Error;

/// Enumerates parser construction and runtime failures.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("LR Table: {lr_table:?}\n{conflict} conflict.")]
    Conflict {
        lr_table: IndexSet<ActionTable>,
        conflict: String,
    },
    #[error("Didn't find transition for non-terminal: {0:?}")]
    TransitionError(Arc<Symbol>),
    #[error("Failed to parser input text: \"{0}\"")]
    FailedToParse(String),
    #[error("Didn't find any rule for word: \"{0}\" in the given grammar.")]
    RuleNotFound(String),
}
