/// A parsed regex AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    /// Matches a single literal character, e.g. `a`.
    Literal(char),

    /// Matches any character except newline (or including it with `s` flag): `.`
    AnyChar,

    /// Zero-width assertion: `^`, `$`, `\b`, `\B`.
    Anchor(Anchor),

    /// Predefined character class shorthand: `\d`, `\w`, `\s` and their negations.
    EscapeClass(EscapeClass),

    /// A character class expression: `[abc]`, `[^a-z]`, `[\d\w]`.
    CharClass(CharClass),

    /// A group: capturing, non-capturing, or a lookaround assertion.
    Group(Box<Regex>, GroupKind),

    /// A quantifier applied to a sub-expression. The `bool` is `true` for greedy.
    Quantifier(Box<Regex>, QuantKind, bool),

    /// A sequence of sub-expressions matched one after the other.
    Concat(Vec<Regex>),

    /// An alternation of sub-expressions: `a|b|c`.
    Alternation(Vec<Regex>),
}

/// Zero-width boundary assertions.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// `^` — start of string (or line in multiline mode).
    Start,
    /// `$` — end of string (or line in multiline mode).
    End,
    /// `\b` — word boundary.
    WordBoundary,
    /// `\B` — non-word boundary.
    NonWordBoundary,
}

/// Predefined shorthand character classes.
#[derive(Debug, Clone, PartialEq)]
pub enum EscapeClass {
    /// `\d` — any ASCII digit `[0-9]`.
    Digit,
    /// `\D` — any non-digit.
    NonDigit,
    /// `\w` — word character `[a-zA-Z0-9_]`.
    Word,
    /// `\W` — non-word character.
    NonWord,
    /// `\s` — whitespace `[ \t\n\r\f\v]`.
    Space,
    /// `\S` — non-whitespace.
    NonSpace,
}

/// A character class `[...]`.
#[derive(Debug, Clone, PartialEq)]
pub struct CharClass {
    /// Items inside the brackets.
    pub items: Vec<CharClassItem>,
    /// `true` when the class is negated with `^`, e.g. `[^abc]`.
    pub negated: bool,
}

/// A single item inside `[...]`.
#[derive(Debug, Clone, PartialEq)]
pub enum CharClassItem {
    /// A single literal character.
    Literal(char),
    /// A character range, e.g. `a-z`. Invariant: `start <= end`.
    Range(char, char),
    /// An escape-class shorthand reused inside a bracket, e.g. `[\d]`.
    EscapeClass(EscapeClass),
}

/// How a group should be treated by a regex engine.
#[derive(Debug, Clone, PartialEq)]
pub enum GroupKind {
    /// Plain `(...)` — captures and numbers the match.
    Capturing,
    /// `(?P<name>...)` — captures with a name.
    Named(String),
    /// `(?:...)` — groups without capturing.
    NonCapturing,
    /// `(?=...)` — positive lookahead.
    LookaheadPos,
    /// `(?!...)` — negative lookahead.
    LookaheadNeg,
    /// `(?<=...)` — positive lookbehind.
    LookbehindPos,
    /// `(?<!...)` — negative lookbehind.
    LookbehindNeg,
}

/// The kind of repetition applied by a quantifier.
#[derive(Debug, Clone, PartialEq)]
pub enum QuantKind {
    /// `*` — zero or more.
    ZeroOrMore,
    /// `+` — one or more.
    OneOrMore,
    /// `?` — zero or one.
    ZeroOrOne,
    /// `{n}` — exactly *n* repetitions.
    Exactly(usize),
    /// `{n,}` — at least *n* repetitions.
    AtLeast(usize),
    /// `{n,m}` — between *n* and *m* repetitions (inclusive).
    Between(usize, usize),
}

// ── width analysis ────────────────────────────────────────────────────────────

impl Regex {
    /// Returns the minimum number of characters this node can match.
    ///
    /// Anchors (`^`, `$`, `\b`) and lookarounds contribute **zero** because
    /// they are zero-width assertions.
    ///
    /// ```rust
    /// use re_parser::parse;
    ///
    /// assert_eq!(parse(r"\d{2,4}").unwrap().min_width(), 2);
    /// assert_eq!(parse(r"^abc$").unwrap().min_width(), 3); // anchors are zero-width
    /// assert_eq!(parse(r"a*").unwrap().min_width(), 0);
    /// ```
    pub fn min_width(&self) -> usize {
        match self {
            Regex::Literal(_) | Regex::AnyChar | Regex::EscapeClass(_) | Regex::CharClass(_) => 1,

            Regex::Anchor(_) => 0,

            Regex::Group(inner, kind) => match kind {
                GroupKind::LookaheadPos
                | GroupKind::LookaheadNeg
                | GroupKind::LookbehindPos
                | GroupKind::LookbehindNeg => 0,
                _ => inner.min_width(),
            },

            Regex::Quantifier(inner, kind, _) => match kind {
                QuantKind::ZeroOrMore | QuantKind::ZeroOrOne => 0,
                QuantKind::OneOrMore => inner.min_width(),
                QuantKind::Exactly(n) => inner.min_width().saturating_mul(*n),
                QuantKind::AtLeast(n) => inner.min_width().saturating_mul(*n),
                QuantKind::Between(n, _) => inner.min_width().saturating_mul(*n),
            },

            Regex::Concat(nodes) => nodes.iter().map(Regex::min_width).sum(),

            Regex::Alternation(nodes) => {
                nodes.iter().map(Regex::min_width).min().unwrap_or(0)
            }
        }
    }

    /// Returns the maximum number of characters this node can match, or `None`
    /// if the match length is unbounded (the node contains `*`, `+`, or
    /// `{n,}`).
    ///
    /// ```rust
    /// use re_parser::parse;
    ///
    /// assert_eq!(parse(r"\d{2,4}").unwrap().max_width(), Some(4));
    /// assert_eq!(parse(r"a+").unwrap().max_width(), None); // unbounded
    /// assert_eq!(parse(r"foo(?=bar)").unwrap().max_width(), Some(3)); // lookahead is zero-width
    /// ```
    pub fn max_width(&self) -> Option<usize> {
        match self {
            Regex::Literal(_) | Regex::AnyChar | Regex::EscapeClass(_) | Regex::CharClass(_) => {
                Some(1)
            }

            Regex::Anchor(_) => Some(0),

            Regex::Group(inner, kind) => match kind {
                GroupKind::LookaheadPos
                | GroupKind::LookaheadNeg
                | GroupKind::LookbehindPos
                | GroupKind::LookbehindNeg => Some(0),
                _ => inner.max_width(),
            },

            Regex::Quantifier(inner, kind, _) => match kind {
                QuantKind::ZeroOrMore | QuantKind::OneOrMore | QuantKind::AtLeast(_) => None,
                QuantKind::ZeroOrOne => inner.max_width(),
                QuantKind::Exactly(n) => inner.max_width()?.checked_mul(*n),
                QuantKind::Between(_, m) => inner.max_width()?.checked_mul(*m),
            },

            // None (unbounded) propagates: if any child is unbounded, the
            // whole concat is unbounded.
            Regex::Concat(nodes) => nodes
                .iter()
                .try_fold(0usize, |acc, n| acc.checked_add(n.max_width()?)),

            // None propagates: if any branch is unbounded, the alternation is
            // unbounded.
            Regex::Alternation(nodes) => nodes
                .iter()
                .try_fold(0usize, |acc, n| Some(acc.max(n.max_width()?))),
        }
    }
}
