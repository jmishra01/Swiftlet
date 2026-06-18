use crate::error::SwiftletError;
use fancy_regex::{Regex, RegexBuilder};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use re_parser;

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
    #[inline]
    pub fn get_value(&self) -> String {
        self.as_str().to_string()
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

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for Symbol {
    /// Formats symbol as `Terminal(name)` or `NonTerminal(name)`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Terminal(value) => write!(f, "Terminal({})", value),
            Symbol::NonTerminal(value) => write!(f, "NonTerminal({})", value),
        }
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

/// Regex compilation flags passed to [`TerminalDef::with_regex`]
///
/// Each field corresponds to a standard regex modifier flag.
/// The default enables only Unicode mode (`u = true`).
#[derive(Debug, Clone)]
pub(crate) struct RegexFlag {
    pub(crate) i: bool, // Case-insensitive
    pub(crate) m: bool, // Multi-line
    pub(crate) s: bool, // Dot matches all
    pub(crate) u: bool, // Unicode matching
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
    pub(crate) min_width: usize,
    pub(crate) max_width: usize,
    pub(crate) priority: usize,
}

impl TerminalDef {
    /// Creates a literal-string terminal definition.
    pub(crate) fn with_string(name: &str, value: &str, priority: usize) -> Self {
        let name = Arc::new(Symbol::Terminal(name.to_string()));

        Self {
            name,
            value: value.to_string(),
            pattern: Pattern::PatternStr(value.to_string()),
            min_width: value.len(),
            max_width: value.len(),
            priority,
        }
    }

    /// Creates a regex-based terminal definition using the provided flags.
    pub(crate) fn with_regex(
        name: &str,
        value: &str,
        regex_flag: RegexFlag,
        priority: usize,
    ) -> Self {
        let name = Arc::new(Symbol::Terminal(name.to_string()));
        let (pattern, min_width, max_width) = {
            let rb = RegexBuilder::new((r"^".to_string() + value).as_str())
                .case_insensitive(regex_flag.i)
                .multi_line(regex_flag.m)
                .dot_matches_new_line(regex_flag.s)
                .unicode_mode(regex_flag.u)
                .verbose_mode(regex_flag.x)
                .build()
                .unwrap();
            let pr = Pattern::PatternRegex(rb);

            let (min, max) = match re_parser::parse(value) {
                Ok(re_ast) => {
                    (re_ast.min_width(), re_ast.max_width().unwrap_or_else(|| usize::MAX))
                },
                Err(_) => {
                    if value.contains("+") || value.contains("*") {
                        (value.len(), usize::MAX)
                    } else {
                        (value.len(), value.len())
                    }
                }
            };
            (pr, min, max)
        };

        Self {
            name,
            value: value.to_string(),
            pattern,
            min_width,
            max_width,
            priority,
        }
    }

    /// Returns a reference to the terminal symbol name (avoids Arc clone at call site).
    pub fn get_name(&self) -> &Arc<Symbol> {
        &self.name
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
#[derive(Clone, Debug)]
pub struct Token {
    source: Arc<str>,
    start: usize,
    end: usize,
    line: usize,
    pub terminal: Arc<Symbol>,
    /// Precomputed: `terminal` starts with `_` but not `__` (hidden - suppressed from AST).
    pub terminal_is_hidden: bool,
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
        let name = terminal.as_str();
        let terminal_is_hidden = name.starts_with('_') && !name.starts_with("__");
        Self {
            source: source.into(),
            start,
            end,
            line,
            terminal,
            terminal_is_hidden,
        }
    }

    /// Returns the token start byte offset.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the token end byte offset.
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the zero-based source line where the token starts.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the terminal name associated with this token.
    pub fn terminal(&self) -> &str {
        self.terminal.as_str()
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

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = format!(
            "Token {{ word: {}, start: {}, end: {}, line: {}, terminal: {:?}  }}",
            self.word(),
            self.start,
            self.end,
            self.line,
            self.terminal
        );
        f.write_str(&text)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.word() == other.word() && self.line == other.line && self.terminal == other.terminal
    }
}

impl Eq for Token {}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.word().hash(state);
        self.line().hash(state);
        self.terminal().hash(state);
    }
}

/// Stores a side-effect-free token probe together with the tokenizer state after commit.
#[derive(Clone, Debug)]
pub(crate) struct TokenMatch {
    pub(crate) token: Arc<Token>,
    pub(crate) next_start: usize,
    pub(crate) next_line: usize,
}

/// Allocation-free result of a token peek: just the offsets and metadata needed to
/// rank candidates and, for the winner, build the `Token` afterward.
///
/// In the Earley parser many terminals are peeked at one position but only the
/// priority-sort winner is consumed, so deferring the `Arc<Token>` allocation until
/// after selection avoids building tokens that are immediately discarded.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TokenProbe {
    /// Byte offset where the match starts (after skipping ignored terminals).
    pub(crate) start: usize,
    /// Source line at `start`.
    pub(crate) line: usize,
    /// Byte offset immediately after the match (also the token's end offset).
    pub(crate) next_start: usize,
    /// Source line after consuming the match.
    pub(crate) next_line: usize,
    /// Matched terminal's priority.
    pub(crate)  priority: usize,
}


/// Tokenizes input text using the configured terminal definitions.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    text: Arc<str>,
    start: usize,
    line: usize,
    len: usize,
    sym_terminal_def: Arc<FxHashMap<Arc<Symbol>, Arc<TerminalDef>>>,
    ignore_terminals: Arc<[Arc<TerminalDef>]>,
    /// Memoized list of `(offset, line)` positions at which a terminal should be probed,
    /// starting from `self.start` after greedily skipping leading ignored terminals.
    ///
    /// The trajectory depends only on `start` / `line` and the (fixed) ignore set -- never on
    /// which terminal is being peeked -- so it is shared across all peeks at the current
    /// position and rebuild only when `commit_token_match` advances the cursor. This avoids
    /// re-running the whitespace/ignores regexes once per expected terminal in every column.
    skip_cache: RefCell<Option<Vec<(usize, usize)>>>,
}

impl Tokenizer {
    /// Creates a tokenizer from input text, terminal definitions, and ignored terminal symbols.
    pub(crate) fn new(
        text: Arc<str>,
        sym_terminal_def: Arc<FxHashMap<Arc<Symbol>, Arc<TerminalDef>>>,
        ignore_terminals: Arc<[Arc<TerminalDef>]>,
    ) -> Self {
        let len = text.len();

        Self {
            text,
            start: 0usize,
            line: 0usize,
            len,
            sym_terminal_def,
            ignore_terminals,
            skip_cache: RefCell::new(None),
        }
    }

    /// Returns the current cursor byte offset in the source text.
    pub(crate) fn get_start(&self) -> usize {
        self.start
    }

    /// Returns the `(line, column)` pair for the current cursor position.
    pub(crate) fn get_line_column(&self) -> (usize, usize) {
        self.line_column(self.start)
    }

    /// Looks up a terminal definition by symbol reference, returning `None` if not registered.
    pub(crate) fn get_terminal_def(&self, name: &Arc<Symbol>) -> Option<&Arc<TerminalDef>> {
        self.sym_terminal_def.get(name)
    }

    /// Returns the original tokenizer input text.
    pub(crate) fn get_text(&self) -> &str {
        &self.text
    }

    /// Advances the cursor to the positioned recorded in `token_match` and invalidates the skip cache.
    pub(crate) fn commit_token_match(&mut self, token_match: &TokenMatch) {
        self.commit_position(token_match.next_start, token_match.next_line);
    }

    /// Advances the cursor to `(next_start, next_line)` and invalidates the skip cache.
    pub(crate) fn commit_position(&mut self, next_start: usize, next_end: usize) {
        self.start = next_start;
        self.line = next_end;
        // Cursor moved -- the cached skip trajectory no longer applies.
        *self.skip_cache.get_mut() = None;
    }

    /// Builds an owned token over the shared source buffer (cheap `Arc` refcount bumps only).
    pub(crate) fn build_token(
        &self,
        start: usize,
        end: usize,
        line: usize,
        terminal: &Arc<Symbol>,
    ) -> Arc<Token> {
        Arc::new(Token::new(
            self.text.clone(),
            start,
            end,
            line,
            terminal.clone(),
        ))
    }

    /// Builds the probe trajectory from `self.start`: every `(offset, line)` at which a
    /// terminal is attempted, walking forward past leading ignored terminals. Mirrors the
    /// original per-call skip loop exactly, but is computed once and cached.
    fn skip_trajectory(&self) -> Vec<(usize, usize)> {
        let mut trajectory = vec![];
        let mut start = self.start;
        let mut line = self.line;

        while start < self.len {
            trajectory.push((start, line));

            let slice_text = &self.text[start..self.len];
            let mut advanced = false;
            for term_def in self.ignore_terminals.iter() {
                if let Some((mt_end, _)) = term_def.pattern.capture(slice_text) {
                    if term_def.name.as_ref().as_str() == "_NL" {
                        line += 1;
                    }
                    start += mt_end;
                    advanced = true;
                    break;
                }
            }

            if !advanced {
                break;
            }
        }
        trajectory
    }



    fn line_column(&self, location: usize) -> (usize, usize) {
        let prefix = &self.text[..location];
        let line = prefix.chars().filter(|ch| *ch == '\n').count() + 1;
        let column = prefix
            .rsplit('\n')
            .next()
            .map(|segment| segment.chars().count() + 1)
            .unwrap_or(1);
        (line, column)
    }

    /// Probes for `next_symbols` at the current cursor without allocating a `Token`.
    ///
    /// Returns the match offsets and metadata, or `None` if the terminal does not match.
    /// The skip trajectory (post-ignore probe positions) is computed once per cursor position
    /// and shared all terminals peeked there.
    pub(crate) fn peek_probe(
        &self,
        next_symbols: &Arc<Symbol>,
    ) -> Option<TokenProbe> {
        let terminal = self.sym_terminal_def.get(next_symbols)?;
        if self.skip_cache.borrow().is_none() {
            let trajectory = self.skip_trajectory();
            *self.skip_cache.borrow_mut() = Some(trajectory);
        }

        let cache = self.skip_cache.borrow();
        let trajectory = cache.as_ref().expect("skip trajectory just populated");

        for &(start, line) in trajectory.iter() {
            if let Some((mt_end, _)) = terminal.capture(&self.text[start..]) {
                let next_start = start + mt_end;
                let next_line = if terminal.name.as_ref().as_str() == "_NL" {
                    line + 1
                } else {
                    line
                };
                return Some(TokenProbe {
                    start,
                    line,
                    next_start,
                    next_line,
                    priority: terminal.priority
                });
            }
        }
        None
    }

    /// Probes for `next_symbols` at the current cursor, returning a full `TokenMatch` on success.
    ///
    /// Unlike [`peek_probe`](Self::peek_probe), this allocates the `Arc<Token>` and bundles it with
    /// cursor metadata. Prefer `peek_probe` in hot paths where only the winning terminal will be
    /// committed.
    pub(crate) fn peek_token_with_next_symbol(
        &self,
        next_symbols: &Arc<Symbol>) -> Result<Option<TokenMatch>, SwiftletError> {
        Ok(
            self.peek_probe(next_symbols).map(|probe| TokenMatch {
                token: self.build_token(probe.start, probe.next_start, probe.line, next_symbols),
                next_start: probe.next_start,
                next_line: probe.next_line
            })
        )
    }
}

/// Holds the symbol-to-[`TerminalDef`] lookup table used to configure a [`Tokenizer`].
#[derive(Debug)]
pub(crate) struct LexerConf {
    sym_terminal_def: Arc<FxHashMap<Arc<Symbol>, Arc<TerminalDef>>>,
}

impl LexerConf {
    /// Creates lexer configuration from terminal definitions.
    pub fn new(terminals: Vec<Arc<TerminalDef>>) -> Self {
        let it = terminals
            .iter()
            .map(|terminal_def| (terminal_def.name.clone(), terminal_def.clone()));
        let sym_terminal_def = FxHashMap::from_iter(it);

        Self {
            sym_terminal_def: Arc::new(sym_terminal_def),
        }
    }

    /// Returns the [`TerminalDef`] registered for `name`, or `None` if unknown.
    pub(crate) fn get_terminal_def(&self, name: &Arc<Symbol>) -> Option<&Arc<TerminalDef>> {
        self.sym_terminal_def.get(name)
    }

    /// Creates a tokenizer over `text` with a provided ignore-symbol set.
    pub fn tokenize(&self, text: &str, ignore_terminals: Arc<[Arc<TerminalDef>]>) -> Tokenizer {
        Tokenizer::new(
            Arc::<str>::from(text),
            self.sym_terminal_def.clone(),
            ignore_terminals,
        )
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
        let st = TerminalDef::with_string("PLUS", "+", 0);
        let rg = TerminalDef::with_regex("INT", r"\d+", RegexFlag::default(), 0);

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
    fn token_accessors_and_terminal_name_work() {
        let tk = Token::new(
            Arc::<str>::from("alpha beta"),
            6,
            10,
            3,
            Arc::new(Symbol::Terminal("WORD".to_string())),
        );

        assert_eq!(tk.start(), 6);
        assert_eq!(tk.end(), 10);
        assert_eq!(tk.line(), 3);
        assert_eq!(tk.terminal(), "WORD".to_string());
        assert_eq!(tk.word(), "beta");
    }

    #[test]
    fn token_word_falls_back_to_source_for_invalid_bounds() {
        let reversed = Token::new(
            Arc::<str>::from("hello"),
            4,
            2,
            0,
            Arc::new(Symbol::Terminal("TEXT".to_string())),
        );
        let beyond_end = Token::new(
            Arc::<str>::from("hello"),
            0,
            10,
            0,
            Arc::new(Symbol::Terminal("TEXT".to_string())),
        );

        assert_eq!(reversed.word(), "hello");
        assert_eq!(beyond_end.word(), "hello");
    }

    #[test]
    fn token_display_contains_core_metadata() {
        let tk = Token::new(
            Arc::<str>::from("sum"),
            0,
            3,
            1,
            Arc::new(Symbol::Terminal("IDENT".to_string())),
        );

        assert_eq!(
            tk.to_string(),
            "Token { word: sum, start: 0, end: 3, line: 1, terminal: Terminal(IDENT)  }"
        );
    }

    #[test]
    fn token_equality_depends_on_word_line_and_terminal() {
        let lhs = Token::new(
            Arc::<str>::from("abc def"),
            0,
            3,
            2,
            Arc::new(Symbol::Terminal("IDENT".to_string())),
        );
        let same = Token::new(
            Arc::<str>::from("abc xyz"),
            0,
            3,
            2,
            Arc::new(Symbol::Terminal("IDENT".to_string())),
        );
        let different_line = Token::new(
            Arc::<str>::from("abc def"),
            0,
            3,
            4,
            Arc::new(Symbol::Terminal("IDENT".to_string())),
        );
        let different_terminal = Token::new(
            Arc::<str>::from("abc def"),
            0,
            3,
            2,
            Arc::new(Symbol::Terminal("NUMBER".to_string())),
        );

        assert_eq!(lhs, same);
        assert_ne!(lhs, different_line);
        assert_ne!(lhs, different_terminal);
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

    #[test]
    fn symbol_terminal_starts_with_works() {
        let t = Symbol::Terminal("INT".to_string());
        assert!(t.starts_with("IN"));
        assert!(!t.starts_with("int"));
    }

    #[test]
    fn symbol_display_works_for_both_variants() {
        let term = Symbol::Terminal("INT".to_string());
        let non_term = Symbol::NonTerminal("expr".to_string());
        assert_eq!(format!("{}", term), "INT");
        assert_eq!(format!("{}", non_term), "expr");
    }

    #[test]
    fn symbol_debug_formats_non_terminal_correctly() {
        let non_term = Symbol::NonTerminal("expr".to_string());
        assert_eq!(format!("{:?}", non_term), "NonTerminal(expr)");
    }

    #[test]
    fn terminal_def_partial_eq_compares_name_and_value() {
        let a = TerminalDef::with_string("PLUS", "+", 0);
        let b = TerminalDef::with_string("PLUS", "+", 0);
        let different_name = TerminalDef::with_string("MINUS", "+", 0);
        let different_value = TerminalDef::with_string("PLUS", "-", 0);
        assert_eq!(a, b);
        assert_ne!(a, different_name);
        assert_ne!(a, different_value);
    }

    #[test]
    fn token_can_be_used_as_hash_map_key() {
        use std::collections::HashSet;
        let source = Arc::<str>::from("hello world");
        let sym = Arc::new(Symbol::Terminal("WORD".to_string()));
        let tok1 = Token::new(source.clone(), 0, 5, 0, sym.clone());
        let tok2 = Token::new(source.clone(), 0, 5, 0, sym.clone());
        let tok3 = Token::new(source.clone(), 6, 11, 0, sym.clone());

        let mut set = HashSet::new();
        set.insert(tok1);
        set.insert(tok2); // duplicate — should not grow the set
        set.insert(tok3);
        assert_eq!(set.len(), 2);
    }
}
