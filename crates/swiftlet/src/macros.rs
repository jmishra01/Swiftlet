#[macro_export]
macro_rules! terminal_def {
    ($x:expr, $y:expr, $n:expr) => {
        Arc::new(TerminalDef::with_string($x, $y, $n))
    };

    ($x:expr, $y:expr, $r:expr, $n:expr) => {
        Arc::new(TerminalDef::with_regex($x, $y, $r, $n))
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
