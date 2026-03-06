#[macro_export]
macro_rules! terminal_def {
    ($x:expr, $y:expr) => {
        Arc::new(TerminalDef::with_string($x, $y))
    };

    ($x:expr, $y:expr, $r:expr) => {
        Arc::new(TerminalDef::with_regex($x, $y, $r))
    };
}

#[macro_export]
macro_rules! terms {
    ($t:expr) => {
        Arc::new(Symbol::Terminal($t.to_string()))
    };
}

#[macro_export]
macro_rules! non_terms {
    ($t:expr) => {
        Arc::new(Symbol::NonTerminal($t.to_string()))
    };
}
