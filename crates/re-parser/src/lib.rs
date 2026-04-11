//! # re-parser
//!
//! A library for parsing regular expression patterns into an abstract syntax tree (AST).
//!
//! ## Supported syntax
//!
//! | Syntax | Description |
//! |--------|-------------|
//! | `a` | Literal character |
//! | `.` | Any character (except newline) |
//! | `^` `$` | Start / end of string anchors |
//! | `\b` `\B` | Word / non-word boundary |
//! | `\d` `\D` `\w` `\W` `\s` `\S` | Predefined character classes |
//! | `[abc]` `[^abc]` `[a-z]` | Character classes |
//! | `(...)` | Capturing group |
//! | `(?P<name>...)` | Named capturing group |
//! | `(?:...)` | Non-capturing group |
//! | `(?=...)` `(?!...)` | Positive / negative lookahead |
//! | `(?<=...)` `(?<!...)` | Positive / negative lookbehind |
//! | `*` `+` `?` | Greedy quantifiers |
//! | `*?` `+?` `??` | Lazy quantifiers |
//! | `{n}` `{n,}` `{n,m}` | Counted quantifiers |
//! | `a\|b` | Alternation |
//! | `\n` `\t` `\r` | Common escape sequences |
//!
//! ## Example
//!
//! ```rust
//! use re_parser::parse;
//! use re_parser::ast::{Regex, QuantKind};
//!
//! let ast = parse(r"\d+").unwrap();
//! // Regex::Quantifier(Box::new(Regex::EscapeClass(EscapeClass::Digit)), QuantKind::OneOrMore, true)
//! println!("{ast:#?}");
//! ```

pub mod ast;
pub mod error;
mod parser;
pub mod width;

use crate::error::Result;
use crate::parser::Parser;

pub use width::Width;

/// Parse a regex pattern string into an [`ast::Regex`] AST.
///
/// Returns [`error::ParseError`] on invalid syntax.
pub fn parse(pattern: &str) -> Result<ast::Regex> {
    Parser::new(pattern).parse()
}

/// Parse `pattern` and return the minimum and maximum number of characters it
/// can match.
///
/// This is a convenience wrapper around [`parse`] + [`width::node_width`].
///
/// ```rust
/// use re_parser::pattern_width;
///
/// let w = pattern_width(r"\d{4}-\d{2}-\d{2}").unwrap();
/// assert_eq!(w.min, 10);
/// assert_eq!(w.max, Some(10));
/// assert!(w.is_fixed());
/// ```
pub fn pattern_width(pattern: &str) -> Result<Width> {
    let ast = parse(pattern)?;
    Ok(width::node_width(&ast))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::error::ParseError;

    // ---------------------------------------------------------------- literals

    #[test]
    fn test_single_literal() {
        assert_eq!(parse("a"), Ok(Regex::Literal('a')));
    }

    #[test]
    fn test_concat_literals() {
        assert_eq!(
            parse("ab"),
            Ok(Regex::Concat(vec![Regex::Literal('a'), Regex::Literal('b')]))
        );
    }

    #[test]
    fn test_empty_pattern() {
        assert_eq!(parse(""), Ok(Regex::Concat(vec![])));
    }

    // ---------------------------------------------------------------- any char

    #[test]
    fn test_any_char() {
        assert_eq!(parse("."), Ok(Regex::AnyChar));
    }

    // --------------------------------------------------------------- anchors

    #[test]
    fn test_start_anchor() {
        assert_eq!(parse("^"), Ok(Regex::Anchor(Anchor::Start)));
    }

    #[test]
    fn test_end_anchor() {
        assert_eq!(parse("$"), Ok(Regex::Anchor(Anchor::End)));
    }

    #[test]
    fn test_word_boundary() {
        assert_eq!(parse(r"\b"), Ok(Regex::Anchor(Anchor::WordBoundary)));
    }

    #[test]
    fn test_non_word_boundary() {
        assert_eq!(parse(r"\B"), Ok(Regex::Anchor(Anchor::NonWordBoundary)));
    }

    // --------------------------------------------------------- escape classes

    #[test]
    fn test_digit_class() {
        assert_eq!(parse(r"\d"), Ok(Regex::EscapeClass(EscapeClass::Digit)));
    }

    #[test]
    fn test_non_digit_class() {
        assert_eq!(parse(r"\D"), Ok(Regex::EscapeClass(EscapeClass::NonDigit)));
    }

    #[test]
    fn test_word_class() {
        assert_eq!(parse(r"\w"), Ok(Regex::EscapeClass(EscapeClass::Word)));
    }

    #[test]
    fn test_space_class() {
        assert_eq!(parse(r"\s"), Ok(Regex::EscapeClass(EscapeClass::Space)));
    }

    #[test]
    fn test_escaped_literal_dot() {
        assert_eq!(parse(r"\."), Ok(Regex::Literal('.')));
    }

    #[test]
    fn test_escaped_newline() {
        assert_eq!(parse(r"\n"), Ok(Regex::Literal('\n')));
    }

    #[test]
    fn test_invalid_escape() {
        assert!(matches!(parse(r"\z"), Err(ParseError::InvalidEscape('z', _))));
    }

    // --------------------------------------------------------------- quantifiers

    #[test]
    fn test_star_greedy() {
        assert_eq!(
            parse("a*"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::ZeroOrMore,
                true
            ))
        );
    }

    #[test]
    fn test_plus_lazy() {
        assert_eq!(
            parse("a+?"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::OneOrMore,
                false
            ))
        );
    }

    #[test]
    fn test_question_mark() {
        assert_eq!(
            parse("a?"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::ZeroOrOne,
                true
            ))
        );
    }

    #[test]
    fn test_exact_quantifier() {
        assert_eq!(
            parse("a{3}"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::Exactly(3),
                true
            ))
        );
    }

    #[test]
    fn test_at_least_quantifier() {
        assert_eq!(
            parse("a{2,}"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::AtLeast(2),
                true
            ))
        );
    }

    #[test]
    fn test_between_quantifier() {
        assert_eq!(
            parse("a{1,5}"),
            Ok(Regex::Quantifier(
                Box::new(Regex::Literal('a')),
                QuantKind::Between(1, 5),
                true
            ))
        );
    }

    #[test]
    fn test_between_min_gt_max_error() {
        assert!(matches!(
            parse("a{5,1}"),
            Err(ParseError::InvalidQuantifier(_, _))
        ));
    }

    // ---------------------------------------------------------- alternation

    #[test]
    fn test_alternation() {
        assert_eq!(
            parse("a|b"),
            Ok(Regex::Alternation(vec![
                Regex::Literal('a'),
                Regex::Literal('b')
            ]))
        );
    }

    #[test]
    fn test_multi_alternation() {
        assert_eq!(
            parse("a|b|c"),
            Ok(Regex::Alternation(vec![
                Regex::Literal('a'),
                Regex::Literal('b'),
                Regex::Literal('c'),
            ]))
        );
    }

    // --------------------------------------------------------------- groups

    #[test]
    fn test_capturing_group() {
        assert_eq!(
            parse("(a)"),
            Ok(Regex::Group(
                Box::new(Regex::Literal('a')),
                GroupKind::Capturing
            ))
        );
    }

    #[test]
    fn test_non_capturing_group() {
        assert_eq!(
            parse("(?:ab)"),
            Ok(Regex::Group(
                Box::new(Regex::Concat(vec![
                    Regex::Literal('a'),
                    Regex::Literal('b')
                ])),
                GroupKind::NonCapturing
            ))
        );
    }

    #[test]
    fn test_named_group() {
        assert_eq!(
            parse("(?P<year>\\d+)"),
            Ok(Regex::Group(
                Box::new(Regex::Quantifier(
                    Box::new(Regex::EscapeClass(EscapeClass::Digit)),
                    QuantKind::OneOrMore,
                    true
                )),
                GroupKind::Named("year".to_owned())
            ))
        );
    }

    #[test]
    fn test_lookahead_pos() {
        assert_eq!(
            parse("a(?=b)"),
            Ok(Regex::Concat(vec![
                Regex::Literal('a'),
                Regex::Group(Box::new(Regex::Literal('b')), GroupKind::LookaheadPos)
            ]))
        );
    }

    #[test]
    fn test_lookahead_neg() {
        assert_eq!(
            parse("a(?!b)"),
            Ok(Regex::Concat(vec![
                Regex::Literal('a'),
                Regex::Group(Box::new(Regex::Literal('b')), GroupKind::LookaheadNeg)
            ]))
        );
    }

    #[test]
    fn test_lookbehind_pos() {
        assert_eq!(
            parse("(?<=a)b"),
            Ok(Regex::Concat(vec![
                Regex::Group(Box::new(Regex::Literal('a')), GroupKind::LookbehindPos),
                Regex::Literal('b'),
            ]))
        );
    }

    #[test]
    fn test_lookbehind_neg() {
        assert_eq!(
            parse("(?<!a)b"),
            Ok(Regex::Concat(vec![
                Regex::Group(Box::new(Regex::Literal('a')), GroupKind::LookbehindNeg),
                Regex::Literal('b'),
            ]))
        );
    }

    #[test]
    fn test_unmatched_open_paren() {
        assert!(matches!(parse("(a"), Err(ParseError::UnmatchedOpenParen(_))));
    }

    #[test]
    fn test_unmatched_close_paren() {
        assert!(matches!(parse("a)"), Err(ParseError::UnmatchedCloseParen(_))));
    }

    // -------------------------------------------------------- character classes

    #[test]
    fn test_char_class_literals() {
        assert_eq!(
            parse("[abc]"),
            Ok(Regex::CharClass(CharClass {
                items: vec![
                    CharClassItem::Literal('a'),
                    CharClassItem::Literal('b'),
                    CharClassItem::Literal('c'),
                ],
                negated: false,
            }))
        );
    }

    #[test]
    fn test_char_class_negated() {
        assert_eq!(
            parse("[^abc]"),
            Ok(Regex::CharClass(CharClass {
                items: vec![
                    CharClassItem::Literal('a'),
                    CharClassItem::Literal('b'),
                    CharClassItem::Literal('c'),
                ],
                negated: true,
            }))
        );
    }

    #[test]
    fn test_char_class_range() {
        assert_eq!(
            parse("[a-z]"),
            Ok(Regex::CharClass(CharClass {
                items: vec![CharClassItem::Range('a', 'z')],
                negated: false,
            }))
        );
    }

    #[test]
    fn test_char_class_with_escape() {
        assert_eq!(
            parse(r"[\d]"),
            Ok(Regex::CharClass(CharClass {
                items: vec![CharClassItem::EscapeClass(EscapeClass::Digit)],
                negated: false,
            }))
        );
    }

    #[test]
    fn test_invalid_range() {
        assert!(matches!(parse("[z-a]"), Err(ParseError::InvalidRange('z', 'a'))));
    }

    #[test]
    fn test_unmatched_open_bracket() {
        assert!(matches!(
            parse("[abc"),
            Err(ParseError::UnmatchedOpenBracket(_))
        ));
    }

    // ----------------------------------------------------------- complex patterns

    #[test]
    fn test_email_like_pattern() {
        // \w+@\w+\.\w+
        let result = parse(r"\w+@\w+\.\w+");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_groups() {
        let result = parse("(a(b)c)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_digit_plus() {
        assert_eq!(
            parse(r"\d+"),
            Ok(Regex::Quantifier(
                Box::new(Regex::EscapeClass(EscapeClass::Digit)),
                QuantKind::OneOrMore,
                true
            ))
        );
    }

    #[test]
    fn test_complex_alternation_in_group() {
        // (foo|bar)
        let result = parse("(foo|bar)");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Regex::Group(
                Box::new(Regex::Alternation(vec![
                    Regex::Concat(vec![
                        Regex::Literal('f'),
                        Regex::Literal('o'),
                        Regex::Literal('o')
                    ]),
                    Regex::Concat(vec![
                        Regex::Literal('b'),
                        Regex::Literal('a'),
                        Regex::Literal('r')
                    ]),
                ])),
                GroupKind::Capturing,
            )
        );
    }
}
