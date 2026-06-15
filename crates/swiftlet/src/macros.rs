/// Creates an `Arc<TerminalDef>` from either a literal string or a regex pattern.
///
/// **String form** (`name`, `value`, `priority`)
/// ```ignore
/// let plus = terminal_def!("PLUS", "+", 0);
/// ```
///
/// **Regex form** (`name`, `pattern`, `flags`, `priority`);
/// ```ignore
/// let int = terminal_def!("INT", r"\d+", RegexFlag::default(), 0);
/// ```
#[macro_export]
macro_rules! terminal_def {
    ($x:expr, $y:expr, $n:expr) => {
        Arc::new(TerminalDef::with_string($x, $y, $n))
    };

    ($x:expr, $y:expr, $r:expr, $n:expr) => {
        Arc::new(TerminalDef::with_regex($x, $y, $r, $n))
    };
}

/// Wraps a string in `Ac<Symbol::Terminal>`.
///
/// ```ignore
/// let sym = terms!("INT");
/// assert!(sym.is_terminal());
/// ```
#[macro_export]
macro_rules! terms {
    ($t:expr) => {
        Arc::new(Symbol::Terminal($t.to_string()))
    };
}

/// Wraps a string in `Arc<Symbol::NonTerminal>`.
///
/// ```ignore
/// let sym = non_terms!("expr");
/// assert!(!sym.is_terminal());
/// ```
#[macro_export]
macro_rules! non_terms {
    ($t:expr) => {
        Arc::new(Symbol::NonTerminal($t.to_string()))
    };
}
