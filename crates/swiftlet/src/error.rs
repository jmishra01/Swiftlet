use crate::lexer::Symbol;
use crate::parser::clr::ActionTable;
use indexmap::IndexSet;
use std::sync::Arc;
use thiserror::Error;

/// Enumerates parser construction and runtime failures.
#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Conflict: Ambiguous Grammar Detected ({conflict}) | {message} | {lr_table:?}\nRun with debug: true, for more details.")]
    Conflict {
        lr_table: IndexSet<ActionTable>,
        conflict: String,
        message: String,
    },
    #[error("Didn't find transition for non-terminal: {0:?}")]
    TransitionError(Arc<Symbol>),
    #[error("Failed to parser input text: \"{0}\"")]
    FailedToParse(String),
    #[error("Didn't find any rule for word: \"{0}\" in the given grammar.")]
    RuleNotFound(String),
    #[error("Rule '{0}' is used, but production rules are not defined.")]
    RuleProductionNotFound(String),
}
