use crate::ast::{Anchor, CharClass, CharClassItem, EscapeClass, GroupKind, QuantKind, Regex};
use crate::error::{ParseError, Result};

/// Recursive-descent parser for regex patterns.
pub(crate) struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    pub(crate) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    // ------------------------------------------------------------------ utils

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn expect(&mut self, expected: char) -> Result<()> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(ParseError::UnexpectedChar(ch, self.pos - 1)),
            None => Err(ParseError::UnexpectedEnd),
        }
    }


    // ---------------------------------------------------------------- grammar
    // regex       ::= alternation
    // alternation ::= concat ('|' concat)*
    // concat      ::= quantified*
    // quantified  ::= atom quantifier?
    // atom        ::= '(' ... ')' | '[' ... ']' | escaped | '.' | anchor | literal

    pub(crate) fn parse(&mut self) -> Result<Regex> {
        let node = self.parse_alternation()?;
        if self.peek().is_some() {
            // Unmatched ')' left over
            return Err(ParseError::UnmatchedCloseParen(self.pos));
        }
        Ok(node)
    }

    fn parse_alternation(&mut self) -> Result<Regex> {
        let mut branches = vec![self.parse_concat()?];
        while self.peek() == Some('|') {
            self.advance(); // consume '|'
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            Ok(branches.remove(0))
        } else {
            Ok(Regex::Alternation(branches))
        }
    }

    fn parse_concat(&mut self) -> Result<Regex> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                None | Some('|') | Some(')') => break,
                _ => items.push(self.parse_quantified()?),
            }
        }
        if items.len() == 1 {
            Ok(items.remove(0))
        } else {
            Ok(Regex::Concat(items))
        }
    }

    fn parse_quantified(&mut self) -> Result<Regex> {
        let atom = self.parse_atom()?;
        match self.peek() {
            Some('*') | Some('+') | Some('?') | Some('{') => {
                let (kind, greedy) = self.parse_quantifier()?;
                Ok(Regex::Quantifier(Box::new(atom), kind, greedy))
            }
            _ => Ok(atom),
        }
    }

    fn parse_quantifier(&mut self) -> Result<(QuantKind, bool)> {
        let start = self.pos;
        let kind = match self.advance().unwrap() {
            '*' => QuantKind::ZeroOrMore,
            '+' => QuantKind::OneOrMore,
            '?' => QuantKind::ZeroOrOne,
            '{' => {
                let kind = self.parse_brace_quantifier(start)?;
                kind
            }
            _ => unreachable!(),
        };
        // Lazy modifier
        let greedy = if self.peek() == Some('?') {
            self.advance();
            false
        } else {
            true
        };
        Ok((kind, greedy))
    }

    /// Parse `{n}`, `{n,}`, or `{n,m}` after the opening `{` has been consumed.
    fn parse_brace_quantifier(&mut self, start: usize) -> Result<QuantKind> {
        let n = self.parse_digits(start)?;
        match self.peek() {
            Some('}') => {
                self.advance();
                Ok(QuantKind::Exactly(n))
            }
            Some(',') => {
                self.advance();
                match self.peek() {
                    Some('}') => {
                        self.advance();
                        Ok(QuantKind::AtLeast(n))
                    }
                    Some(c) if c.is_ascii_digit() => {
                        let m = self.parse_digits(start)?;
                        self.expect('}')?;
                        if n > m {
                            return Err(ParseError::InvalidQuantifier(
                                start,
                                format!("{{{n},{m}}}: min > max"),
                            ));
                        }
                        Ok(QuantKind::Between(n, m))
                    }
                    Some(c) => Err(ParseError::UnexpectedChar(c, self.pos)),
                    None => Err(ParseError::UnexpectedEnd),
                }
            }
            Some(c) => Err(ParseError::InvalidQuantifier(
                start,
                format!("expected ',' or '}}', got '{c}'"),
            )),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_digits(&mut self, err_pos: usize) -> Result<usize> {
        let mut buf = String::new();
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            buf.push(self.advance().unwrap());
        }
        if buf.is_empty() {
            return Err(ParseError::InvalidQuantifier(
                err_pos,
                "expected digit".to_owned(),
            ));
        }
        buf.parse::<usize>().map_err(|_| {
            ParseError::InvalidQuantifier(err_pos, format!("'{buf}' overflows usize"))
        })
    }

    fn parse_atom(&mut self) -> Result<Regex> {
        match self.peek() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('\\') => self.parse_escape(),
            Some('.') => {
                self.advance();
                Ok(Regex::AnyChar)
            }
            Some('^') => {
                self.advance();
                Ok(Regex::Anchor(Anchor::Start))
            }
            Some('$') => {
                self.advance();
                Ok(Regex::Anchor(Anchor::End))
            }
            Some(c) => {
                self.advance();
                Ok(Regex::Literal(c))
            }
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    // ----------------------------------------------------------------- groups

    fn parse_group(&mut self) -> Result<Regex> {
        let start = self.pos;
        self.expect('(')?;

        let kind = if self.peek() == Some('?') {
            self.advance(); // consume '?'
            self.parse_group_kind(start)?
        } else {
            GroupKind::Capturing
        };

        let inner = self.parse_alternation()?;

        match self.advance() {
            Some(')') => {}
            Some(c) => return Err(ParseError::UnexpectedChar(c, self.pos - 1)),
            None => return Err(ParseError::UnmatchedOpenParen(start)),
        }

        Ok(Regex::Group(Box::new(inner), kind))
    }

    fn parse_group_kind(&mut self, start: usize) -> Result<GroupKind> {
        match self.peek() {
            Some(':') => {
                self.advance();
                Ok(GroupKind::NonCapturing)
            }
            Some('=') => {
                self.advance();
                Ok(GroupKind::LookaheadPos)
            }
            Some('!') => {
                self.advance();
                Ok(GroupKind::LookaheadNeg)
            }
            Some('<') => {
                self.advance();
                match self.peek() {
                    Some('=') => {
                        self.advance();
                        Ok(GroupKind::LookbehindPos)
                    }
                    Some('!') => {
                        self.advance();
                        Ok(GroupKind::LookbehindNeg)
                    }
                    _ => Err(ParseError::InvalidGroup(
                        start,
                        "expected '=' or '!' after '(?<'".to_owned(),
                    )),
                }
            }
            Some('P') => {
                self.advance();
                self.expect('<')?;
                let name = self.parse_group_name(start)?;
                Ok(GroupKind::Named(name))
            }
            Some(c) => Err(ParseError::InvalidGroup(
                start,
                format!("unknown group modifier '(?{c}'"),
            )),
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_group_name(&mut self, start: usize) -> Result<String> {
        let mut name = String::new();
        loop {
            match self.peek() {
                Some('>') => {
                    self.advance();
                    break;
                }
                Some(c) if c.is_alphanumeric() || c == '_' => {
                    name.push(c);
                    self.advance();
                }
                Some(c) => return Err(ParseError::InvalidGroupName(format!("{name}{c}"))),
                None => return Err(ParseError::UnmatchedOpenParen(start)),
            }
        }
        if name.is_empty() {
            return Err(ParseError::InvalidGroup(start, "empty group name".to_owned()));
        }
        Ok(name)
    }

    // ---------------------------------------------------------- character class

    fn parse_char_class(&mut self) -> Result<Regex> {
        let start = self.pos;
        self.expect('[')?;

        let negated = if self.peek() == Some('^') {
            self.advance();
            true
        } else {
            false
        };

        let mut items: Vec<CharClassItem> = Vec::new();

        loop {
            match self.peek() {
                None => return Err(ParseError::UnmatchedOpenBracket(start)),
                Some(']') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    let item = self.parse_class_escape()?;
                    items.push(item);
                }
                Some(_) => {
                    let ch = self.advance().unwrap();
                    // Check for range `a-z`
                    if self.peek() == Some('-') && self.peek2() != Some(']') {
                        self.advance(); // consume '-'
                        match self.peek() {
                            None => return Err(ParseError::UnmatchedOpenBracket(start)),
                            Some('\\') => {
                                // range end is an escape — treat '-' as literal
                                items.push(CharClassItem::Literal(ch));
                                items.push(CharClassItem::Literal('-'));
                                let item = self.parse_class_escape()?;
                                items.push(item);
                            }
                            Some(end) => {
                                self.advance();
                                if ch > end {
                                    return Err(ParseError::InvalidRange(ch, end));
                                }
                                items.push(CharClassItem::Range(ch, end));
                            }
                        }
                    } else {
                        items.push(CharClassItem::Literal(ch));
                    }
                }
            }
        }

        Ok(Regex::CharClass(CharClass { items, negated }))
    }

    fn parse_class_escape(&mut self) -> Result<CharClassItem> {
        self.expect('\\')?;
        match self.peek() {
            Some(c) => {
                self.advance();
                match c {
                    'd' => Ok(CharClassItem::EscapeClass(EscapeClass::Digit)),
                    'D' => Ok(CharClassItem::EscapeClass(EscapeClass::NonDigit)),
                    'w' => Ok(CharClassItem::EscapeClass(EscapeClass::Word)),
                    'W' => Ok(CharClassItem::EscapeClass(EscapeClass::NonWord)),
                    's' => Ok(CharClassItem::EscapeClass(EscapeClass::Space)),
                    'S' => Ok(CharClassItem::EscapeClass(EscapeClass::NonSpace)),
                    'n' => Ok(CharClassItem::Literal('\n')),
                    't' => Ok(CharClassItem::Literal('\t')),
                    'r' => Ok(CharClassItem::Literal('\r')),
                    'f' => Ok(CharClassItem::Literal('\x0C')),
                    'v' => Ok(CharClassItem::Literal('\x0B')),
                    '0' => Ok(CharClassItem::Literal('\0')),
                    c if is_meta(c) => Ok(CharClassItem::Literal(c)),
                    c => Err(ParseError::InvalidEscape(c, self.pos - 1)),
                }
            }
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    // --------------------------------------------------------- escape sequences

    fn parse_escape(&mut self) -> Result<Regex> {
        self.expect('\\')?;
        match self.peek() {
            Some(c) => {
                self.advance();
                match c {
                    'd' => Ok(Regex::EscapeClass(EscapeClass::Digit)),
                    'D' => Ok(Regex::EscapeClass(EscapeClass::NonDigit)),
                    'w' => Ok(Regex::EscapeClass(EscapeClass::Word)),
                    'W' => Ok(Regex::EscapeClass(EscapeClass::NonWord)),
                    's' => Ok(Regex::EscapeClass(EscapeClass::Space)),
                    'S' => Ok(Regex::EscapeClass(EscapeClass::NonSpace)),
                    'b' => Ok(Regex::Anchor(Anchor::WordBoundary)),
                    'B' => Ok(Regex::Anchor(Anchor::NonWordBoundary)),
                    'n' => Ok(Regex::Literal('\n')),
                    't' => Ok(Regex::Literal('\t')),
                    'r' => Ok(Regex::Literal('\r')),
                    'f' => Ok(Regex::Literal('\x0C')),
                    'v' => Ok(Regex::Literal('\x0B')),
                    '0' => Ok(Regex::Literal('\0')),
                    c if is_meta(c) => Ok(Regex::Literal(c)),
                    c => Err(ParseError::InvalidEscape(c, self.pos - 1)),
                }
            }
            None => Err(ParseError::UnexpectedEnd),
        }
    }
}

/// Returns `true` for characters that have special meaning in regex syntax and
/// may be escaped to produce a literal match.
fn is_meta(c: char) -> bool {
    matches!(
        c,
        '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\'
    )
}
