use std::sync::Arc;
use swiftlet::error::{ParseError, SwiftletError};
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

fn clr_option(start: &str) -> Arc<ParserConfig> {
    Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        start: start.to_string(),
        ..Default::default()
    })
}

#[test]
fn clr_returns_conflict_for_ambiguous_grammar() {
    let grammar = r#"
    start: expr
    expr: expr "+" expr
        | INT
    %import (WS, INT)
    %ignore WS
    "#;

    let parser = Swiftlet::from_str(grammar)
        .expect("ambiguous grammar should still build")
        .parser(clr_option("start"));

    let err = parser
        .parse("1 + 2 + 3")
        .expect_err("CLR parse should report a conflict");

    assert!(matches!(
        err,
        SwiftletError::Parse(ParseError::Conflict { .. })
    ));
}

#[test]
fn clr_returns_rule_not_found_for_unknown_start_token() {
    let grammar = r#"
    start: expr
    expr: INT
    %import (WS, INT)
    %ignore WS
    "#;

    let parser = Swiftlet::from_str(grammar)
        .expect("parser should build")
        .parser(clr_option("start"));

    let err = parser
        .parse("abc")
        .expect_err("non-numeric input should be rejected");

    assert!(matches!(
        err,
        SwiftletError::Parse(ParseError::RuleNotFound(word)) if word.is_empty()
    ));
}

#[test]
fn clr_returns_rule_not_found_for_incomplete_expression() {
    let grammar = r#"
    start: expr
    expr: INT "+" INT
    %import (WS, INT)
    %ignore WS
    "#;

    let parser = Swiftlet::from_str(grammar)
        .expect("parser should build")
        .parser(clr_option("start"));

    let err = parser
        .parse("42 +")
        .expect_err("missing right-hand token should be rejected");

    assert!(matches!(
        err,
        SwiftletError::Parse(ParseError::RuleNotFound(word)) if word.is_empty()
    ));
}

#[test]
fn new_returns_file_read_error_for_missing_grammar_file() {
    let missing_path = std::env::temp_dir().join("swiftlet_missing_grammar_file.lark");
    let path = missing_path.to_string_lossy().into_owned();

    let err = match Swiftlet::from_file(path.as_str()) {
        Ok(_) => panic!("missing grammar file should return an error"),
        Err(err) => err,
    };

    assert!(matches!(
        err,
        SwiftletError::GrammarFileReadError { path: err_path, .. } if err_path == path
    ));
}
