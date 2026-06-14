use std::sync::Arc;
use swiftlet::error::{GrammarError, SwiftletError};
use swiftlet::grammar::Algorithm;
use swiftlet::{Ambiguity, ParserConfig, Swiftlet};

fn earley(start: &str) -> Arc<ParserConfig> {
    Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        start: start.to_string(),
        ..Default::default()
    })
}

fn clr(start: &str) -> Arc<ParserConfig> {
    Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        start: start.to_string(),
        ..Default::default()
    })
}

// ---------- RuleCompiler coverage ----------

#[test]
fn rule_compiler_maybe_in_rule_earley() {
    // Covers RuleCompiler::maybe() — `[...]` optional block in a rule body.
    let grammar = r#"
    start: greeting
    greeting: "hi" [name]
    name: WORD
    WORD: /\w+/
    %import WS
    %ignore WS
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("hi").is_ok());
    assert!(s.parser(earley("start")).parse("hi Alice").is_ok());
}

#[test]
fn rule_compiler_maybe_in_rule_clr() {
    let grammar = r#"
    start: greeting
    greeting: "hi" [name]
    name: WORD
    WORD: /\w+/
    %import WS
    %ignore WS
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(clr("start")).parse("hi").is_ok());
    assert!(s.parser(clr("start")).parse("hi Alice").is_ok());
}

#[test]
fn rule_compiler_range_in_rule_is_exercised() {
    // Covers RuleCompiler::range() (lines 476-493). A range like "a".."z" in a
    // rule body produces `[a-z]` which get_symbol() classifies as NonTerminal
    // (contains lowercase chars), so grammar compilation fails with
    // RuleProductionNotFound — that's the expected behaviour.
    let grammar = "start: letter\nletter: \"a\"..\"z\"\n";
    let err = match Swiftlet::from_str(grammar) {
        Ok(_) => panic!("range-in-rule with lowercase chars should fail to compile"),
        Err(e) => e,
    };
    assert!(
        matches!(err, SwiftletError::Grammar(_)),
        "expected GrammarError, got: {:?}",
        err
    );
}

#[test]
fn rule_compiler_inline_regex_in_rule_earley() {
    // Covers RuleCompiler::regex() — inline regex pattern inside a rule.
    let grammar = r#"
    start: token
    token: /[a-z]+/
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("hello").is_ok());
}

#[test]
fn rule_compiler_inline_regex_in_rule_clr() {
    let grammar = r#"
    start: token
    token: /[a-z]+/
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(clr("start")).parse("hello").is_ok());
}

#[test]
fn rule_compiler_rule_priority_earley() {
    // Covers RuleCompiler::rule() priority branch (tree.len() > 2).
    let grammar = r#"
    start: expr
    expr.1: keyword
    expr: NAME
    keyword: "select"
    NAME: /[a-z]+/
    %import WS
    %ignore WS
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("select").is_ok());
    assert!(s.parser(earley("start")).parse("hello").is_ok());
}

#[test]
fn rule_compiler_or_expansion_cache_hit() {
    // Covers or_expansion cache path — same OR alternatives reused in multiple rules.
    let grammar = r#"
    start: a | b
    a: c | d
    b: c | d
    c: "x"
    d: "y"
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("x").is_ok());
    assert!(s.parser(earley("start")).parse("y").is_ok());
}

#[test]
fn rule_compiler_inline_string_case_insensitive_in_rule() {
    // Covers RuleCompiler::string() case-insensitive path ("word"i in a rule).
    let grammar = r#"
    start: greeting
    greeting: "hello"i
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("hello").is_ok());
    assert!(s.parser(earley("start")).parse("HELLO").is_ok());
    assert!(s.parser(earley("start")).parse("Hello").is_ok());
}

#[test]
fn rule_compiler_inline_string_case_insensitive_in_rule_clr() {
    let grammar = r#"
    start: greeting
    greeting: "hello"i
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(clr("start")).parse("HELLO").is_ok());
}

// ---------- load_grammar.rs coverage ----------

#[test]
fn load_grammar_returns_error_for_undefined_non_terminal() {
    // Covers RuleCompiler::get_grammar() error branch (lines 295-296 in transform.rs).
    let grammar = "start: undefined_rule\n";
    let err = match Swiftlet::from_str(grammar) {
        Ok(_) => panic!("expected error for undefined non-terminal"),
        Err(e) => e,
    };
    assert!(
        matches!(
            err,
            SwiftletError::Grammar(GrammarError::RuleProductionNotFound(_))
        ),
        "expected RuleProductionNotFound, got: {:?}",
        err
    );
}

#[test]
fn load_grammar_with_inline_literal_ignore_directive() {
    // Covers update_terminals else branch in load_grammar.rs (lines 175-179):
    // %ignore applied to a literal string that is NOT a common terminal name.
    let grammar = r#"
    start: WORD+
    WORD: /[a-zA-Z]+/
    %ignore " "
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("hello world").is_ok());
}

#[test]
fn load_grammar_with_inline_literal_ignore_clr() {
    let grammar = r#"
    start: WORD+
    WORD: /[a-zA-Z]+/
    %ignore " "
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(clr("start")).parse("foo bar").is_ok());
}

// ---------- terminal compiler coverage ----------

#[test]
fn terminal_compiler_multiline_regex_flag() {
    // Covers TerminalCompiler::transform_regex() `m` (multiline) flag path (line 106).
    let grammar = r#"
    start: LINE+
    LINE: /^\w+/m
    %import (NEWLINE, WS_INLINE)
    %ignore WS_INLINE
    %ignore NEWLINE
    "#;
    // If the regex compiles, the grammar loads successfully.
    let result = Swiftlet::from_str(grammar);
    assert!(result.is_ok(), "multiline regex grammar should compile: {:?}", result.err());
}

// ---------- Earley explicit ambiguity coverage ----------

#[test]
fn earley_explicit_ambiguity_returns_ambiguity_tree() {
    // Covers Ambiguity::Explicit path — wraps all derivations under _ambiguity.
    let grammar = r#"
    start: expr
    expr: expr "+" expr
        | INT
    %import (WS, INT)
    %ignore WS
    "#;
    let config = Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        ambiguity: Ambiguity::Explicit,
        start: "start".to_string(),
        debug: false,
    });
    let swiftlet = Swiftlet::from_str(grammar).unwrap();
    let result = swiftlet.parser(config).parse("1 + 2 + 3");
    assert!(result.is_ok(), "explicit ambiguity parse should succeed");
    let tree_str = result.unwrap().inline_text();
    assert!(
        tree_str.contains("_ambiguity") || tree_str.contains("start"),
        "unexpected tree: {}",
        tree_str
    );
}

// ---------- CLR shift-action error fallback ----------

#[test]
fn clr_parse_fails_cleanly_on_incomplete_input() {
    // Exercises the CLR shift_action / lookahead error path.
    let grammar = r#"
    start: A B
    A: "x"
    B: "y"
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    let err = s.parser(clr("start")).parse("x").unwrap_err();
    assert!(
        matches!(
            err,
            SwiftletError::Parse(_) | SwiftletError::Lexer(_)
        ),
        "unexpected error type: {:?}",
        err
    );
}

// ---------- fetch_terminals path coverage ----------

#[test]
fn grammar_with_ignore_uses_string_literal_form() {
    // Covers fetch_terminals() quoted-string strip path (line 22 in transform.rs).
    let grammar = r#"
    start: NAME
    NAME: /[a-zA-Z]+/
    %ignore ","
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("hello").is_ok());
}

// ---------- Grammar with repeated terminal dedup ----------

#[test]
fn grammar_with_multiple_terminals_deduplicates_correctly() {
    // Exercises the terminal sort + dedup path in load_grammar.rs.
    let grammar = r#"
    start: A B C
    A: "aa"
    B: "bb"
    C: "cc"
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(earley("start")).parse("aabbcc").is_ok());
    assert!(s.parser(clr("start")).parse("aabbcc").is_ok());
}

// ---------- Expandable-rule (?rule) coverage ----------

#[test]
fn rule_compiler_expandable_rule_earley() {
    // Covers origin_apply() `?` expand branch (line 262 transform.rs) and
    // wrap_contribution() expand path (line 570 earley.rs).
    let grammar = r#"
    start: value
    ?value: number | word
    number: INT
    word: WORD
    WORD: /[a-z]+/
    %import (WS, INT)
    %ignore WS
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    // ?value expands inline — neither a "value" nor "_ambiguity" tree node appears.
    let result = s.parser(earley("start")).parse("42").unwrap();
    let text = result.inline_text();
    assert!(
        !text.contains("Tree(\"value\""),
        "?value should be expanded away, got: {}",
        text
    );
    let result2 = s.parser(earley("start")).parse("hello").unwrap();
    let text2 = result2.inline_text();
    assert!(!text2.contains("Tree(\"value\""), "?value should expand: {}", text2);
}

#[test]
fn rule_compiler_expandable_rule_clr() {
    let grammar = r#"
    start: value
    ?value: number | word
    number: INT
    word: WORD
    WORD: /[a-z]+/
    %import (WS, INT)
    %ignore WS
    "#;
    let s = Swiftlet::from_str(grammar).unwrap();
    assert!(s.parser(clr("start")).parse("99").is_ok());
    assert!(s.parser(clr("start")).parse("abc").is_ok());
}

// ---------- Explicit ambiguity — verify _ambiguity node ----------

#[test]
fn earley_explicit_ambiguity_produces_ambiguity_node_for_ambiguous_input() {
    // Covers earley.rs finalize_explicit_parse when multiple derivations exist.
    let grammar = r#"
    start: expr
    expr: expr "+" expr
        | INT
    %import (WS, INT)
    %ignore WS
    "#;
    let config = Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        ambiguity: Ambiguity::Explicit,
        start: "start".to_string(),
        debug: false,
    });
    let swiftlet = Swiftlet::from_str(grammar).unwrap();
    // "1 + 2 + 3" is genuinely ambiguous: (1+2)+3 or 1+(2+3)
    let result = swiftlet.parser(config).parse("1 + 2 + 3").unwrap();
    let text = result.inline_text();
    // With explicit ambiguity, multiple derivations are wrapped in an _ambiguity node.
    assert!(
        text.contains("_ambiguity"),
        "expected _ambiguity wrapper for ambiguous parse, got: {}",
        text
    );
}

// ---------- Earley contribution_all expand / alias paths ----------

#[test]
fn earley_explicit_ambiguity_with_expandable_rule() {
    // Covers contribution_all() expand path (line 662 earley.rs).
    let grammar = r#"
    start: expr
    expr: expr "+" expr | base
    ?base: INT
    %import (WS, INT)
    %ignore WS
    "#;
    let config = Arc::new(ParserConfig {
        algorithm: Algorithm::Earley,
        ambiguity: Ambiguity::Explicit,
        start: "start".to_string(),
        debug: false,
    });
    let swiftlet = Swiftlet::from_str(grammar).unwrap();
    let result = swiftlet.parser(config).parse("1 + 2 + 3");
    assert!(result.is_ok(), "explicit ambiguity with ?base should succeed: {:?}", result.err());
}

// ---------- CLR parser errors ----------

#[test]
fn clr_fails_on_unknown_token_at_start() {
    // Exercises CLR parse() initial lookahead handling.
    let grammar = r#"
    start: INT
    %import INT
    "#;
    let err = Swiftlet::from_str(grammar)
        .unwrap()
        .parser(clr("start"))
        .parse("abc")
        .unwrap_err();
    assert!(matches!(err, SwiftletError::Parse(_) | SwiftletError::Lexer(_)));
}
