#[macro_export]
macro_rules! terminal_def {
    ($x:expr, $y:expr, $z:expr, $w:expr) => {
        Arc::new(TerminalDef::new($x, $y, $z, $w))
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
