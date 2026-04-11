//! The [`Width`] type and the [`node_width`] convenience constructor.
//!
//! The actual computation lives on [`crate::ast::Regex`] via
//! [`Regex::min_width`] and [`Regex::max_width`].

use crate::ast::Regex;

/// The range of character widths a pattern can match.
///
/// - `min` — fewest characters consumed; always finite.
/// - `max` — most characters consumed; `None` means unbounded (`*`, `+`, `{n,}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Width {
    pub min: usize,
    pub max: Option<usize>,
}

impl Width {
    /// Fixed width: `min == max`.
    pub fn fixed(n: usize) -> Self {
        Self {
            min: n,
            max: Some(n),
        }
    }

    /// Unbounded: at least `min` characters, no upper limit.
    pub fn unbounded(min: usize) -> Self {
        Self { min, max: None }
    }

    /// `true` when the pattern always matches the same number of characters.
    pub fn is_fixed(&self) -> bool {
        self.max == Some(self.min)
    }

    /// `true` when the pattern can match the empty string.
    pub fn is_nullable(&self) -> bool {
        self.min == 0
    }

    /// `true` when there is no upper bound on the match length.
    pub fn is_unbounded(&self) -> bool {
        self.max.is_none()
    }
}

impl std::fmt::Display for Width {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.max {
            Some(max) if max == self.min => write!(f, "exactly {}", self.min),
            Some(max) => write!(f, "{}..={}", self.min, max),
            None => write!(f, "{}..", self.min),
        }
    }
}

/// Build a [`Width`] from a parsed [`Regex`] node.
///
/// This is a thin wrapper — the computation is delegated to
/// [`Regex::min_width`] and [`Regex::max_width`].
pub fn node_width(node: &Regex) -> Width {
    Width {
        min: node.min_width(),
        max: node.max_width(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn ast(pattern: &str) -> Regex {
        parse(pattern).unwrap()
    }

    // ── single atoms ──────────────────────────────────────────────────────────

    #[test]
    fn test_literal() {
        assert_eq!(ast("a").min_width(), 1);
        assert_eq!(ast("a").max_width(), Some(1));
    }

    #[test]
    fn test_any_char() {
        assert_eq!(ast(".").min_width(), 1);
        assert_eq!(ast(".").max_width(), Some(1));
    }

    #[test]
    fn test_anchor_zero_width() {
        for p in ["^", "$", r"\b"] {
            assert_eq!(ast(p).min_width(), 0, "min_width of {p}");
            assert_eq!(ast(p).max_width(), Some(0), "max_width of {p}");
        }
    }

    #[test]
    fn test_escape_class() {
        assert_eq!(ast(r"\d").min_width(), 1);
        assert_eq!(ast(r"\w").max_width(), Some(1));
    }

    #[test]
    fn test_char_class() {
        assert_eq!(ast("[abc]").min_width(), 1);
        assert_eq!(ast("[^0-9]").max_width(), Some(1));
    }

    // ── quantifiers ───────────────────────────────────────────────────────────

    #[test]
    fn test_star() {
        assert_eq!(ast("a*").min_width(), 0);
        assert_eq!(ast("a*").max_width(), None);
    }

    #[test]
    fn test_plus() {
        assert_eq!(ast("a+").min_width(), 1);
        assert_eq!(ast("a+").max_width(), None);
    }

    #[test]
    fn test_question() {
        assert_eq!(ast("a?").min_width(), 0);
        assert_eq!(ast("a?").max_width(), Some(1));
    }

    #[test]
    fn test_exact() {
        assert_eq!(ast("a{4}").min_width(), 4);
        assert_eq!(ast("a{4}").max_width(), Some(4));
    }

    #[test]
    fn test_at_least() {
        assert_eq!(ast("a{3,}").min_width(), 3);
        assert_eq!(ast("a{3,}").max_width(), None);
    }

    #[test]
    fn test_between() {
        assert_eq!(ast("a{2,5}").min_width(), 2);
        assert_eq!(ast("a{2,5}").max_width(), Some(5));
    }

    #[test]
    fn test_lazy_same_width_as_greedy() {
        assert_eq!(ast("a*?").min_width(), ast("a*").min_width());
        assert_eq!(ast("a*?").max_width(), ast("a*").max_width());
        assert_eq!(ast("a+?").min_width(), ast("a+").min_width());
        assert_eq!(ast("a{1,3}?").max_width(), ast("a{1,3}").max_width());
    }

    // ── concat ────────────────────────────────────────────────────────────────

    #[test]
    fn test_concat_fixed() {
        assert_eq!(ast("abc").min_width(), 3);
        assert_eq!(ast("abc").max_width(), Some(3));
    }

    #[test]
    fn test_concat_with_quantifier() {
        assert_eq!(ast("ab+").min_width(), 2);
        assert_eq!(ast("ab+").max_width(), None);
    }

    #[test]
    fn test_concat_optional_suffix() {
        assert_eq!(ast("ab?").min_width(), 1);
        assert_eq!(ast("ab?").max_width(), Some(2));
    }

    #[test]
    fn test_anchored_pattern() {
        assert_eq!(ast("^abc$").min_width(), 3);
        assert_eq!(ast("^abc$").max_width(), Some(3));
    }

    // ── alternation ───────────────────────────────────────────────────────────

    #[test]
    fn test_alternation_same_width() {
        assert_eq!(ast("cat|dog").min_width(), 3);
        assert_eq!(ast("cat|dog").max_width(), Some(3));
    }

    #[test]
    fn test_alternation_different_width() {
        assert_eq!(ast("a|bb|ccc").min_width(), 1);
        assert_eq!(ast("a|bb|ccc").max_width(), Some(3));
    }

    #[test]
    fn test_alternation_unbounded_branch() {
        assert_eq!(ast("a|b+").min_width(), 1);
        assert_eq!(ast("a|b+").max_width(), None);
    }

    // ── groups ────────────────────────────────────────────────────────────────

    #[test]
    fn test_capturing_group() {
        assert_eq!(ast("(abc)").min_width(), 3);
        assert_eq!(ast("(abc)").max_width(), Some(3));
    }

    #[test]
    fn test_non_capturing_group() {
        assert_eq!(ast("(?:ab)+").min_width(), 2);
        assert_eq!(ast("(?:ab)+").max_width(), None);
    }

    #[test]
    fn test_lookahead_zero_width() {
        assert_eq!(ast("foo(?=bar)").min_width(), 3);
        assert_eq!(ast("foo(?=bar)").max_width(), Some(3));
    }

    #[test]
    fn test_lookbehind_zero_width() {
        assert_eq!(ast(r"(?<=\d)px").min_width(), 2);
        assert_eq!(ast(r"(?<=\d)px").max_width(), Some(2));
    }

    // ── real-world ────────────────────────────────────────────────────────────

    #[test]
    fn test_ipv4_octet() {
        assert_eq!(ast(r"\d{1,3}").min_width(), 1);
        assert_eq!(ast(r"\d{1,3}").max_width(), Some(3));
    }

    #[test]
    fn test_iso_date() {
        assert_eq!(ast(r"\d{4}-\d{2}-\d{2}").min_width(), 10);
        assert_eq!(ast(r"\d{4}-\d{2}-\d{2}").max_width(), Some(10));
    }

    #[test]
    fn test_hex_colour() {
        assert_eq!(ast(r"#[0-9a-fA-F]{6}").min_width(), 7);
        assert_eq!(ast(r"#[0-9a-fA-F]{6}").max_width(), Some(7));
    }

    // ── node_width wrapper ────────────────────────────────────────────────────

    #[test]
    fn test_node_width_wrapper() {
        let w = node_width(&ast(r"\d{2,4}"));
        assert_eq!(w.min, 2);
        assert_eq!(w.max, Some(4));
    }

    // ── Width helpers ─────────────────────────────────────────────────────────

    #[test]
    fn test_is_fixed() {
        assert!(Width::fixed(5).is_fixed());
        assert!(!Width::unbounded(1).is_fixed());
        assert!(!Width { min: 1, max: Some(3) }.is_fixed());
    }

    #[test]
    fn test_is_nullable() {
        assert!(Width::fixed(0).is_nullable());
        assert!(Width { min: 0, max: Some(3) }.is_nullable());
        assert!(!Width::fixed(1).is_nullable());
    }

    #[test]
    fn test_display() {
        assert_eq!(Width::fixed(3).to_string(), "exactly 3");
        assert_eq!(Width { min: 1, max: Some(5) }.to_string(), "1..=5");
        assert_eq!(Width::unbounded(2).to_string(), "2..");
    }
}
