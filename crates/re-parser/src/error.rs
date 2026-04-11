use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum ParseError {
    #[error("unexpected end of pattern")]
    UnexpectedEnd,

    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),

    #[error("unmatched '(' at position {0}")]
    UnmatchedOpenParen(usize),

    #[error("unmatched ')' at position {0}")]
    UnmatchedCloseParen(usize),

    #[error("unmatched '[' at position {0}")]
    UnmatchedOpenBracket(usize),

    #[error("invalid quantifier at position {0}: {1}")]
    InvalidQuantifier(usize, String),

    #[error("invalid escape sequence '\\{0}' at position {1}")]
    InvalidEscape(char, usize),

    #[error("invalid range '{0}-{1}' in character class: start must be <= end")]
    InvalidRange(char, char),

    #[error("invalid group syntax at position {0}: {1}")]
    InvalidGroup(usize, String),

    #[error("named group '{0}' contains invalid characters")]
    InvalidGroupName(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;
