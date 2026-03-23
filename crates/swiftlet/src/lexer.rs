use fancy_regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::Arc;

/// Distinguishes grammar terminals from non-terminals.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Symbol {
    Terminal(String),
    NonTerminal(String),
}

impl Symbol {
    /// Returns the underlying symbol text as a borrowed string slice.
    pub fn as_str(&self) -> &str {
        match self {
            Symbol::Terminal(value) => value.as_str(),
            Symbol::NonTerminal(value) => value.as_str(),
        }
    }

    /// Returns the underlying symbol text as an owned string.
    pub fn get_value(&self) -> String {
        match self {
            Symbol::Terminal(value) => value.clone(),
            Symbol::NonTerminal(value) => value.clone(),
        }
    }

    #[inline(always)]
    /// Returns `true` when this symbol is terminal.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Symbol::Terminal(_))
    }

    /// Returns whether the symbol text starts with `prefix`.
    pub fn starts_with(&self, prefix: &str) -> bool {
        match self {
            Symbol::Terminal(value) => value.starts_with(prefix),
            Symbol::NonTerminal(value) => value.starts_with(prefix),
        }
    }
}

impl Debug for Symbol {
    /// Formats symbol as `Terminal(name)` or `NonTerminal(name)`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Symbol::NonTerminal(name) => format!("NonTerminal({name})"),
                Symbol::Terminal(name) => format!("Terminal({name})"),
            }
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    PatternStr(String),
    PatternRegex(Regex),
}

impl Pattern {
    /// Attempts to capture a token match at the beginning of `text`.
    pub(crate) fn capture(&self, text: &str) -> Option<(usize, usize)> {
        match self {
            Pattern::PatternStr(name) => {
                if text.starts_with(name) {
                    return Some((name.len(), name.len()));
                }
                None
            }
            Pattern::PatternRegex(regex) => {
                if let Ok(Some(caps)) = regex.captures(text)
                    && caps.len() > 0
                    && let Some(mt) = caps.get(0)
                {
                    return Some((mt.end(), mt.as_str().len()));
                }
                None
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RegexFlag {
    pub(crate) i: bool, // Case-insensitive
    pub(crate) m: bool, // Multi-line
    pub(crate) s: bool, // Dot matches all
    pub(crate) u: bool, // unicode matching
    pub(crate) x: bool, // verbose
}

impl Default for RegexFlag {
    /// Returns default regex flags used for terminal definitions.
    fn default() -> Self {
        Self {
            i: false,
            m: false,
            s: false,
            u: true,
            x: false,
        }
    }
}

/// Defines a terminal token and its matching pattern.
#[derive(Debug, Clone)]
pub struct TerminalDef {
    pub(crate) name: Arc<Symbol>,
    #[allow(dead_code)]
    pub(crate) value: String,
    pub(crate) pattern: Pattern,
    pub(crate) max_width: usize,
}

impl TerminalDef {
    /// Creates a literal-string terminal definition.
    pub(crate) fn with_string(name: &str, value: &str) -> Self {
        let name = Arc::new(Symbol::Terminal(name.to_string()));

        Self {
            name,
            value: value.to_string(),
            pattern: Pattern::PatternStr(value.to_string()),
            max_width: value.len(),
        }
    }

    /// Creates a regex-based terminal definition using the provided flags.
    pub(crate) fn with_regex(name: &str, value: &str, regex_flag: RegexFlag) -> Self {
        let name = Arc::new(Symbol::Terminal(name.to_string()));
        let (pattern, max_width) = {
            let rb = RegexBuilder::new((r"^".to_string() + value).as_str())
                .case_insensitive(regex_flag.i)
                .multi_line(regex_flag.m)
                .dot_matches_new_line(regex_flag.s)
                .unicode_mode(regex_flag.u)
                .verbose_mode(regex_flag.x)
                .build()
                .unwrap();
            let pr = Pattern::PatternRegex(rb);
            let max = if value.contains("+") || value.contains("*") {
                usize::MAX
            } else {
                value.len()
            };
            (pr, max)
        };

        Self {
            name,
            value: value.to_string(),
            pattern,
            max_width,
        }
    }

    /// Returns terminal symbol name.
    pub fn get_name(&self) -> Arc<Symbol> {
        self.name.clone()
    }

    /// Attempts to match this terminal at the beginning of `text`.
    fn capture(&self, text: &str) -> Option<(usize, usize)> {
        self.pattern.capture(text)
    }
}

impl PartialEq for TerminalDef {
    /// Compares terminal definitions by name and source pattern text.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

/// Stores a concrete token slice and its source location metadata.
#[derive(Debug, Clone)]
pub struct Token {
    source: Arc<str>,
    start: usize,
    end: usize,
    line: usize,
    pub terminal: Arc<Symbol>,
}

impl Token {
    /// Creates a token with source position metadata.
    pub fn new(
        source: impl Into<Arc<str>>,
        start: usize,
        end: usize,
        line: usize,
        terminal: Arc<Symbol>,
    ) -> Self {
        Self {
            source: source.into(),
            start,
            end,
            line,
            terminal,
        }
    }

    /// Returns the token start byte offset.
    pub fn get_start(&self) -> usize {
        self.start
    }

    /// Returns the token end byte offset.
    pub fn get_end(&self) -> usize {
        self.end
    }

    /// Returns the zero-based source line where the token starts.
    pub fn get_line(&self) -> usize {
        self.line
    }

    /// Returns the terminal name associated with this token.
    pub fn get_terminal(&self) -> String {
        self.terminal.get_value()
    }

    /// Returns the token text from the shared source buffer.
    pub fn word(&self) -> &str {
        if self.start <= self.end && self.end <= self.source.len() {
            &self.source[self.start..self.end]
        } else {
            &self.source
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.word() == other.word() && self.line == other.line && self.terminal == other.terminal
    }
}

impl Eq for Token {}

/// Tokenizes input text using the configured terminal definitions.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    text: Arc<str>,
    start: usize,
    line: usize,
    len: usize,
    terminals: Vec<Arc<TerminalDef>>,
    ignore: Arc<HashSet<Arc<Symbol>>>,
}

impl Tokenizer {
    /// Creates a tokenizer from input text, terminal definitions, and ignored terminal symbols.
    pub(crate) fn new(
        text: Arc<str>,
        terminals: &[Arc<TerminalDef>],
        ignore: Arc<HashSet<Arc<Symbol>>>,
    ) -> Self {
        let len = text.len();
        Self {
            text,
            start: 0usize,
            line: 0usize,
            len,
            terminals: terminals.to_owned(),
            ignore,
        }
    }

    /// Returns the original tokenizer input text.
    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }
}

impl Iterator for Tokenizer {
    type Item = Arc<Token>;

    /// Produces the next token matched at the current cursor.
    ///
    /// Panics when no terminal matches the current input position.
    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.len {
            let slice_text = &self.text[self.start..];

            let previous_start = self.start;

            for terminal in self.terminals.iter() {
                if let Some((mt_end, mt_len)) = terminal.capture(slice_text) {
                    return if !self.ignore.contains(&terminal.name) {
                        let token = Arc::new(Token::new(
                            self.text.clone(),
                            self.start,
                            mt_end + self.start,
                            self.line,
                            terminal.name.clone(),
                        ));

                        if terminal.name.as_ref().as_str() == "_NL" {
                            self.line += 1;
                            if self.start == 0 {
                                self.start += mt_end;
                                return self.next();
                            }
                        }
                        self.start += mt_len;

                        Some(token)
                    } else {
                        self.start += mt_end;
                        self.next()
                    };
                }
            }

            if previous_start == self.start {
                let expected_next_token = self
                    .terminals
                    .iter()
                    .map(|x| match x.pattern.clone() {
                        Pattern::PatternRegex(_) => x.get_name().as_ref().as_str().to_string(),
                        Pattern::PatternStr(name) => name,
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                panic!(
                    "Failed during tokenization at location {} of input text: \"{}\", expecting one of the following terminals: ({}).",
                    previous_start, self.text, expected_next_token
                );
            }
        }
        None
    }
}

#[derive(Debug)]
pub(crate) struct LexerConf {
    pub terminals: Vec<Arc<TerminalDef>>,
}

impl LexerConf {
    /// Creates lexer configuration from terminal definitions.
    pub fn new(terminals: Vec<Arc<TerminalDef>>) -> Self {
        Self { terminals }
    }

    /// Creates a tokenizer over `text` with a provided ignore-symbol set.
    pub fn tokenize(&self, text: &str, ignore: Arc<HashSet<Arc<Symbol>>>) -> Tokenizer {
        Tokenizer::new(Arc::<str>::from(text), &self.terminals, ignore)
    }
}

/// Infers whether a symbol name is terminal or non-terminal from casing.
pub fn get_symbol(word: &str) -> Arc<Symbol> {
    if word.chars().any(|x| x.is_ascii_lowercase()) {
        return Arc::new(Symbol::NonTerminal(word.to_string()));
    }
    Arc::new(Symbol::Terminal(word.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn symbol_helpers_work() {
        let t = Symbol::Terminal("INT".to_string());
        let nt = Symbol::NonTerminal("expr".to_string());
        assert_eq!(t.as_str(), "INT");
        assert_eq!(nt.get_value(), "expr".to_string());
        assert!(t.is_terminal());
        assert!(!nt.is_terminal());
        assert!(nt.starts_with("ex"));
    }

    #[test]
    fn regex_flag_default_is_expected() {
        let f = RegexFlag::default();
        assert!(!f.i);
        assert!(!f.m);
        assert!(!f.s);
        assert!(f.u);
        assert!(!f.x);
    }

    #[test]
    fn terminal_def_with_string_and_regex_capture() {
        let st = TerminalDef::with_string("PLUS", "+");
        let rg = TerminalDef::with_regex("INT", r"\d+", RegexFlag::default());

        assert_eq!(st.get_name().as_ref().as_str(), "PLUS");
        assert_eq!(st.capture("+1").unwrap().0, 1);
        assert_eq!(rg.capture("123abc").unwrap().0, 3);
    }

    #[test]
    fn token_new_sets_fields() {
        let tk = Token::new(
            Arc::<str>::from("xabc"),
            1,
            4,
            2,
            Arc::new(Symbol::Terminal("ID".to_string())),
        );
        assert_eq!(tk.word(), "abc");
        assert_eq!(tk.start, 1);
        assert_eq!(tk.end, 4);
        assert_eq!(tk.line, 2);
    }

    #[test]
    fn tokenizer_and_lexer_conf_tokenize_with_ignore() {
        let terminals = vec![
            Arc::new(TerminalDef::with_regex("_NL", r"\n+", RegexFlag::default())),
            Arc::new(TerminalDef::with_regex("WS", r"[ ]+", RegexFlag::default())),
            Arc::new(TerminalDef::with_regex("INT", r"\d+", RegexFlag::default())),
        ];

        let lexer = LexerConf::new(terminals);
        let mut tokenizer = lexer.tokenize(
            "12 34\n56",
            Arc::new(
                [
                    Arc::new(Symbol::Terminal("WS".to_string())),
                    Arc::new(Symbol::Terminal("_NL".to_string())),
                ]
                .into_iter()
                .collect(),
            ),
        );
        let words = tokenizer
            .by_ref()
            .map(|x| x.word().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            words,
            vec!["12".to_string(), "34".to_string(), "56".to_string()]
        );
        assert_eq!(tokenizer.get_text(), "12 34\n56");
    }

    #[test]
    fn tokenizer_panics_on_unmatched_input() {
        let terminals = vec![Arc::new(TerminalDef::with_string("A", "a"))];
        let mut tokenizer =
            Tokenizer::new(Arc::<str>::from("x"), &terminals, Arc::new(HashSet::new()));
        let panicked = std::panic::catch_unwind(move || {
            let _ = tokenizer.next();
        });
        assert!(panicked.is_err());
    }

    #[test]
    fn get_symbol_classifies_terminal_vs_non_terminal() {
        let nt = get_symbol("expr");
        let t = get_symbol("INT");
        assert_eq!(nt.as_ref().as_str(), "expr");
        assert_eq!(t.as_ref().as_str(), "INT");
        assert!(!nt.is_terminal());
        assert!(t.is_terminal());
    }
}
