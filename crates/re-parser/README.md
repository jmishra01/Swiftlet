# re-parser

A Rust library that parses regular expression patterns into a typed abstract syntax tree (AST).

`re-parser` does **not** execute regex patterns against input strings. Instead it gives you a structured representation of the pattern that you can inspect, transform, analyse, or use to drive your own matching engine.

---

## Contents

- [Features](#features)
- [Installation](#installation)
- [Quick start](#quick-start)
- [Tutorial](#tutorial)
  - [Parsing a pattern](#1-parsing-a-pattern)
  - [The AST node types](#2-the-ast-node-types)
  - [Literals and any-char](#3-literals-and-any-char)
  - [Anchors and escape classes](#4-anchors-and-escape-classes)
  - [Character classes](#5-character-classes)
  - [Quantifiers](#6-quantifiers)
  - [Groups and lookarounds](#7-groups-and-lookarounds)
  - [Alternation and concatenation](#8-alternation-and-concatenation)
  - [Width analysis](#9-width-analysis)
  - [Walking the AST](#10-walking-the-ast)
  - [Error handling](#11-error-handling)
- [Supported syntax](#supported-syntax)
- [API reference](#api-reference)
- [Running the examples](#running-the-examples)

---

## Features

- Parses the full common regex syntax into a clean, exhaustive enum tree.
- Every node carries only the information it needs — no `Option`-heavy structs.
- Built-in **width analysis**: `min_width()` and `max_width()` on every node.
- Precise, structured error types with byte positions.
- Zero runtime dependencies (only `thiserror` for error derive).

---

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
re-parser = { path = "../re-parser" }   # or version = "..." once published
```

---

## Quick start

```rust
use re_parser::parse;

fn main() {
    let ast = parse(r"\d{4}-\d{2}-\d{2}").unwrap();

    println!("min chars: {}", ast.min_width()); // 10
    println!("max chars: {:?}", ast.max_width()); // Some(10)
    println!("{ast:#?}");
}
```

Output:

```
min chars: 10
max chars: Some(10)
Concat(
    [
        Quantifier(EscapeClass(Digit), Exactly(4), true),
        Literal('-'),
        Quantifier(EscapeClass(Digit), Exactly(2), true),
        Literal('-'),
        Quantifier(EscapeClass(Digit), Exactly(2), true),
    ],
)
```

---

## Tutorial

### 1. Parsing a pattern

The entry point is `re_parser::parse`. It takes a `&str` and returns
`Result<ast::Regex, error::ParseError>`.

```rust
use re_parser::parse;

// Success
let ast = parse(r"hello\s+world").unwrap();

// Failure — structured error with position
match parse(r"(unclosed") {
    Ok(_)  => unreachable!(),
    Err(e) => eprintln!("parse error: {e}"),
    // "unmatched '(' at position 0"
}
```

---

### 2. The AST node types

All types live in the `re_parser::ast` module.

```
Regex                   — the root enum
├── Literal(char)       — a single character: a  \n  \.
├── AnyChar             — dot: .
├── Anchor(Anchor)      — ^  $  \b  \B
├── EscapeClass(…)      — \d  \D  \w  \W  \s  \S
├── CharClass(…)        — [abc]  [^a-z]  [\d_]
├── Group(Box<Regex>, GroupKind)
│   ├── Capturing       — (...)
│   ├── Named(String)   — (?P<name>...)
│   ├── NonCapturing    — (?:...)
│   ├── LookaheadPos    — (?=...)
│   ├── LookaheadNeg    — (?!...)
│   ├── LookbehindPos   — (?<=...)
│   └── LookbehindNeg   — (?<!...)
├── Quantifier(Box<Regex>, QuantKind, bool)
│   ├── ZeroOrMore      — *   (bool = greedy)
│   ├── OneOrMore       — +
│   ├── ZeroOrOne       — ?
│   ├── Exactly(n)      — {n}
│   ├── AtLeast(n)      — {n,}
│   └── Between(n, m)   — {n,m}
├── Concat(Vec<Regex>)  — sequence: ab\d
└── Alternation(…)      — a|b|c
```

---

### 3. Literals and any-char

```rust
use re_parser::{ast::Regex, parse};

assert_eq!(parse("a").unwrap(), Regex::Literal('a'));
assert_eq!(parse(".").unwrap(), Regex::AnyChar);

// Escaped literal — the backslash is consumed by the parser
assert_eq!(parse(r"\.").unwrap(), Regex::Literal('.'));
assert_eq!(parse(r"\n").unwrap(), Regex::Literal('\n'));

// Multiple characters become a Concat
if let Regex::Concat(nodes) = parse("hi").unwrap() {
    assert_eq!(nodes.len(), 2);
}
```

---

### 4. Anchors and escape classes

```rust
use re_parser::ast::{Anchor, EscapeClass, Regex};
use re_parser::parse;

// Anchors — zero-width assertions
assert_eq!(parse("^").unwrap(), Regex::Anchor(Anchor::Start));
assert_eq!(parse("$").unwrap(), Regex::Anchor(Anchor::End));
assert_eq!(parse(r"\b").unwrap(), Regex::Anchor(Anchor::WordBoundary));

// Predefined character-class shorthands
assert_eq!(parse(r"\d").unwrap(), Regex::EscapeClass(EscapeClass::Digit));
assert_eq!(parse(r"\W").unwrap(), Regex::EscapeClass(EscapeClass::NonWord));
assert_eq!(parse(r"\s").unwrap(), Regex::EscapeClass(EscapeClass::Space));
```

Available shorthands: `\d` `\D` `\w` `\W` `\s` `\S`.

---

### 5. Character classes

A character class `[...]` is represented by `Regex::CharClass`, which holds a
`Vec<CharClassItem>` and a `negated` flag.

```rust
use re_parser::ast::{CharClass, CharClassItem, EscapeClass, Regex};
use re_parser::parse;

// [a-z0-9_]
let Regex::CharClass(cls) = parse(r"[a-z0-9_]").unwrap() else { panic!() };
assert!(!cls.negated);
// items: [Range('a','z'), Range('0','9'), Literal('_')]

// [^\d] — negated class containing an escape shorthand
let Regex::CharClass(cls) = parse(r"[^\d]").unwrap() else { panic!() };
assert!(cls.negated);
assert_eq!(cls.items[0], CharClassItem::EscapeClass(EscapeClass::Digit));
```

`CharClassItem` variants:

| Variant | Example |
|---------|---------|
| `Literal(char)` | `[abc]` |
| `Range(char, char)` | `[a-z]` |
| `EscapeClass(EscapeClass)` | `[\d\w]` |

---

### 6. Quantifiers

A quantifier wraps an inner node, a `QuantKind`, and a `bool` that is `true`
for greedy and `false` for lazy.

```rust
use re_parser::ast::{QuantKind, Regex};
use re_parser::parse;

// Greedy: a+
let Regex::Quantifier(inner, kind, greedy) = parse("a+").unwrap() else { panic!() };
assert_eq!(*inner, Regex::Literal('a'));
assert_eq!(kind, QuantKind::OneOrMore);
assert!(greedy);

// Lazy: a+?
let Regex::Quantifier(_, _, greedy) = parse("a+?").unwrap() else { panic!() };
assert!(!greedy);

// Counted: \d{2,4}
let Regex::Quantifier(_, kind, _) = parse(r"\d{2,4}").unwrap() else { panic!() };
assert_eq!(kind, QuantKind::Between(2, 4));
```

| Syntax | `QuantKind` |
|--------|-------------|
| `*` | `ZeroOrMore` |
| `+` | `OneOrMore` |
| `?` | `ZeroOrOne` |
| `{n}` | `Exactly(n)` |
| `{n,}` | `AtLeast(n)` |
| `{n,m}` | `Between(n, m)` |

Append `?` to any of the above to make it lazy: `*?`, `+?`, `??`, `{n,m}?`.

---

### 7. Groups and lookarounds

```rust
use re_parser::ast::{GroupKind, Regex};
use re_parser::parse;

// Capturing
let Regex::Group(_, kind) = parse("(abc)").unwrap() else { panic!() };
assert_eq!(kind, GroupKind::Capturing);

// Named capturing
let Regex::Group(_, kind) = parse(r"(?P<year>\d+)").unwrap() else { panic!() };
assert_eq!(kind, GroupKind::Named("year".into()));

// Non-capturing
let Regex::Group(_, kind) = parse("(?:abc)").unwrap() else { panic!() };
assert_eq!(kind, GroupKind::NonCapturing);

// Lookarounds — these are zero-width: they do not consume characters
// "foo(?=bar)" matches "foo" only when followed by "bar"
// "foo(?!bar)" matches "foo" only when NOT followed by "bar"
// "(?<=\d)px"  matches "px"  only when preceded by a digit
// "(?<!\d)px"  matches "px"  only when NOT preceded by a digit
```

---

### 8. Alternation and concatenation

```rust
use re_parser::ast::Regex;
use re_parser::parse;

// Alternation
let Regex::Alternation(branches) = parse("cat|dog|bird").unwrap() else { panic!() };
assert_eq!(branches.len(), 3);

// Concatenation
let Regex::Concat(nodes) = parse("abc").unwrap() else { panic!() };
assert_eq!(nodes.len(), 3);

// Mix: (foo|bar)\d+
// → Concat([Group(Alternation([…]), Capturing), Quantifier(EscapeClass(Digit), OneOrMore, true)])
```

---

### 9. Width analysis

Every `Regex` node has two methods:

| Method | Return type | Meaning |
|--------|-------------|---------|
| `.min_width()` | `usize` | Fewest characters the pattern can consume |
| `.max_width()` | `Option<usize>` | Most characters consumed; `None` = unbounded |

Anchors (`^`, `$`, `\b`) and lookaround groups (`(?=…)`, `(?<=…)`, …) are
**zero-width** — they do not consume any characters.

```rust
use re_parser::parse;

let ast = parse(r"\d{4}-\d{2}-\d{2}").unwrap();  // ISO date
assert_eq!(ast.min_width(), 10);
assert_eq!(ast.max_width(), Some(10));

let ast = parse(r"https?://\S+").unwrap();
assert_eq!(ast.min_width(), 8);   // "http://" + at least one non-space
assert_eq!(ast.max_width(), None); // unbounded

let ast = parse(r"^hello$").unwrap(); // anchors are zero-width
assert_eq!(ast.min_width(), 5);
assert_eq!(ast.max_width(), Some(5));

let ast = parse(r"foo(?=bar)").unwrap(); // lookahead is zero-width
assert_eq!(ast.min_width(), 3);
assert_eq!(ast.max_width(), Some(3));
```

The `pattern_width` convenience function parses and measures in one call and
returns a `Width` struct:

```rust
use re_parser::pattern_width;

let w = pattern_width(r"[a-zA-Z]{2,8}").unwrap();
println!("{w}");          // "2..=8"
println!("{}", w.min);    // 2
println!("{:?}", w.max);  // Some(8)

assert!(!w.is_fixed());
assert!(!w.is_nullable());
assert!(!w.is_unbounded());
```

`Width` helper predicates:

| Method | Returns `true` when |
|--------|---------------------|
| `is_fixed()` | `min == max` |
| `is_nullable()` | `min == 0` |
| `is_unbounded()` | `max.is_none()` |

Width rules at a glance:

| Node | `min_width` | `max_width` |
|------|------------|-------------|
| `Literal` / `AnyChar` / `\d` / `[…]` | 1 | `Some(1)` |
| `^` `$` `\b` / lookaround | 0 | `Some(0)` |
| `expr*` | 0 | `None` |
| `expr+` | inner.min | `None` |
| `expr?` | 0 | inner.max |
| `expr{n,m}` | n × inner.min | m × inner.max |
| `Concat` | Σ mins | Σ maxes (`None` if any child is unbounded) |
| `Alternation` | min of mins | max of maxes (`None` if any branch is unbounded) |

---

### 10. Walking the AST

Because `Regex` is a plain Rust enum you can walk it with a normal recursive
function — no visitor trait required.

```rust
use re_parser::ast::{CharClassItem, Regex};
use re_parser::parse;

/// Recursively collect every literal character in the pattern.
fn literals(node: &Regex, out: &mut Vec<char>) {
    match node {
        Regex::Literal(c) => out.push(*c),
        Regex::CharClass(cls) => {
            for item in &cls.items {
                if let CharClassItem::Literal(c) = item {
                    out.push(*c);
                }
            }
        }
        Regex::Group(inner, _) => literals(inner, out),
        Regex::Quantifier(inner, _, _) => literals(inner, out),
        Regex::Concat(nodes) | Regex::Alternation(nodes) => {
            for n in nodes { literals(n, out); }
        }
        _ => {}
    }
}

let ast = parse(r"(?P<proto>https?)://\S+").unwrap();
let mut chars = Vec::new();
literals(&ast, &mut chars);
let s: String = chars.into_iter().collect();
assert_eq!(s, "https://"); // fixed characters in the pattern
```

You can also call `min_width` / `max_width` on any sub-node, not just the root:

```rust
use re_parser::ast::Regex;
use re_parser::parse;

let ast = parse(r"(\d{4})-(\d{2})-(\d{2})").unwrap();

if let Regex::Concat(nodes) = &ast {
    for (i, node) in nodes.iter().enumerate() {
        println!("child[{i}]  min={}  max={:?}", node.min_width(), node.max_width());
    }
}
// child[0]  min=4  max=Some(4)   — (\d{4})
// child[1]  min=1  max=Some(1)   — literal '-'
// child[2]  min=2  max=Some(2)   — (\d{2})
// child[3]  min=1  max=Some(1)   — literal '-'
// child[4]  min=2  max=Some(2)   — (\d{2})
```

---

### 11. Error handling

`parse` returns `Result<Regex, ParseError>`. All variants carry byte positions
so you can report them to users.

```rust
use re_parser::{error::ParseError, parse};

match parse(r"[z-a]") {
    Err(ParseError::InvalidRange(lo, hi)) => {
        eprintln!("range '{lo}-{hi}' is invalid: start > end");
    }
    _ => {}
}
```

Full `ParseError` variant list:

| Variant | Cause |
|---------|-------|
| `UnexpectedEnd` | Pattern ends where more input was expected |
| `UnexpectedChar(char, pos)` | Character that cannot start a valid construct |
| `UnmatchedOpenParen(pos)` | `(` with no matching `)` |
| `UnmatchedCloseParen(pos)` | `)` with no preceding `(` |
| `UnmatchedOpenBracket(pos)` | `[` with no matching `]` |
| `InvalidQuantifier(pos, msg)` | Bad `{n,m}` syntax or `min > max` |
| `InvalidEscape(char, pos)` | Unrecognised `\x` sequence |
| `InvalidRange(lo, hi)` | `[z-a]` — start > end |
| `InvalidGroup(pos, msg)` | Unknown `(?…)` modifier |
| `InvalidGroupName(name)` | `(?P<…>)` name contains illegal characters |

---

## Supported syntax

| Syntax | Description |
|--------|-------------|
| `a` | Literal character |
| `.` | Any character (except newline) |
| `^` | Start-of-string anchor |
| `$` | End-of-string anchor |
| `\b` `\B` | Word / non-word boundary |
| `\d` `\D` | Digit / non-digit |
| `\w` `\W` | Word char / non-word char |
| `\s` `\S` | Whitespace / non-whitespace |
| `\n` `\t` `\r` `\f` `\v` `\0` | Common escape sequences |
| `\.` `\*` `\+` … | Escaped metacharacter → literal |
| `[abc]` | Character class — any of `a`, `b`, `c` |
| `[^abc]` | Negated class |
| `[a-z]` | Character range |
| `[\d_]` | Escape shorthands inside `[…]` |
| `(…)` | Capturing group |
| `(?P<name>…)` | Named capturing group |
| `(?:…)` | Non-capturing group |
| `(?=…)` `(?!…)` | Positive / negative lookahead |
| `(?<=…)` `(?<!…)` | Positive / negative lookbehind |
| `*` `+` `?` | Greedy quantifiers |
| `*?` `+?` `??` | Lazy (non-greedy) quantifiers |
| `{n}` | Exactly *n* repetitions |
| `{n,}` | At least *n* repetitions |
| `{n,m}` | Between *n* and *m* repetitions (inclusive) |
| `a\|b` | Alternation |

---

## API reference

### Functions

```rust
// Parse a pattern into an AST
pub fn parse(pattern: &str) -> Result<ast::Regex, error::ParseError>

// Parse and compute width in one call
pub fn pattern_width(pattern: &str) -> Result<Width, error::ParseError>
```

### `ast::Regex` methods

```rust
impl Regex {
    pub fn min_width(&self) -> usize
    pub fn max_width(&self) -> Option<usize>
}
```

### `Width`

```rust
pub struct Width {
    pub min: usize,
    pub max: Option<usize>, // None = unbounded
}

impl Width {
    pub fn fixed(n: usize) -> Self
    pub fn unbounded(min: usize) -> Self
    pub fn is_fixed(&self) -> bool
    pub fn is_nullable(&self) -> bool
    pub fn is_unbounded(&self) -> bool
}

impl Display for Width { /* "exactly N", "N..=M", "N.." */ }
```

### `width::node_width`

```rust
pub fn node_width(node: &ast::Regex) -> Width
```

Thin wrapper: delegates to `.min_width()` / `.max_width()`.

---

## Running the examples

```bash
# Parsing fundamentals
cargo run -p re-parser --example basic_parsing

# Every quantifier form, greedy and lazy
cargo run -p re-parser --example quantifiers

# All group kinds including lookarounds
cargo run -p re-parser --example groups

# Character class expressions
cargo run -p re-parser --example char_classes

# Recursive AST visitors (node count, literal extraction, pretty-printer)
cargo run -p re-parser --example ast_visitor

# Real-world patterns: IPv4, ISO date, email, semver, URL
cargo run -p re-parser --example real_world_patterns

# min_width / max_width analysis
cargo run -p re-parser --example width
```

Run the test suite:

```bash
cargo test -p re-parser
```
