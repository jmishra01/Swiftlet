use std::fs;
use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{Ambiguity, ParserConfig, Swiftlet};

#[macro_use]
mod common;

multi_test!(
    terminal_clr_inline_terminal_concatenate,
    terminal_earley_inline_terminal_concatenate,
    r#"
    s: A
    A: "a" "b"
    "#,
    "ab",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    terminal_clr_flag_i_text_ab,
    terminal_earley_flag_i_text_ab,
    r#"
    s: A
    A: "a" "b"i
    "#,
    ["ab", "aB"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_terminal_reference,
    terminal_earley_terminal_reference,
    r#"
    s: A
    A: "a" B
    B: "b"
    "#,
    "ab",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_terminal_multi_reference,
    terminal_earley_terminal_multi_reference,
    r#"
    s: A
    A: X "a" B
    B: "b"
    X: "x"
    "#,
    "xab",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_range,
    terminal_earley_range,
    r#"
    s: A
    A: X R+
    X: "x"
    R: "1".."9"
    "#,
    "x12345",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    terminal_clr_or_xb_ab,
    terminal_earley_or_xb_ab,
    r#"
    s: A
    A: (X | "a") B
    B: "b"
    X: "x"
    "#,
    ["xb", "ab"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    terminal_clr_or_op_xab,
    terminal_earley_or_op_xab,
    r#"
    s: A
    A: (X | "a")+ B
    B: "b"
    X: "x"
    "#,
    ["xab", "xaxxab"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    terminal_clr_regex,
    terminal_earley_regex,
    r#"
    s: A
    A: "a" B
    B: /b+c*/
    "#,
    ["abb", "abbccc"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    terminal_clr_regex_flags_i,
    terminal_earley_regex_flags_i,
    r#"
    s: A
    A: "a" B
    B: /b+c*/i
    "#,
    ["abBcCc", "abb", "abbccc"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_priority_keyword_over_identifier,
    terminal_earley_priority_keyword_over_identifier,
    r#"
    s: SELECT NAME
    SELECT.10: "select"
    NAME: /[a-z]+/
    %import WS
    %ignore WS
    "#,
    "select users",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_maybe_ab,
    terminal_earley_maybe_ab,
    r#"
    s: A
    A: "a" B
    B: ["b" "c"]
    "#,
    "ab",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_maybe_ac,
    terminal_earley_maybe_ac,
    r#"
    s: A
    A: "a" B
    B: ["b" "c"]
    "#,
    "ac",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    terminal_clr_imported_terminal_aliases,
    terminal_earley_imported_terminal_aliases,
    r#"
    s: COMMENT _NL
    COMMENT: SH_COMMENT
    _NL: NEWLINE
    %import (SH_COMMENT, NEWLINE)
    "#,
    "# service settings\n",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

#[test]
fn parser_option_default_values() {
    let opt = ParserConfig::default();
    assert_eq!(opt.start, "start".to_string());
    assert!(matches!(opt.algorithm, Algorithm::Earley));
    assert!(matches!(opt.ambiguity, Ambiguity::Resolve));
    assert!(!opt.debug);
}

#[test]
fn swiftlet_from_file_parses_input() {
    let grammar = r#"
        start: expr
        expr: INT
        %import (WS, INT)
        %ignore WS
        "#;
    let path = std::env::temp_dir().join("swiftlet_test_grammar.lark");
    fs::write(&path, grammar).unwrap();

    let parser_option = Arc::new(ParserConfig {
        algorithm: Algorithm::CLR,
        ..ParserConfig::default()
    });
    let parser = Swiftlet::from_file(path.to_string_lossy().as_ref())
        .expect("failed to load grammar")
        .parser(parser_option);
    assert!(parser.parse("10").is_ok());

    let _ = fs::remove_file(path);
}
