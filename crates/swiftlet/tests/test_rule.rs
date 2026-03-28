use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};
use std::sync::Arc;


#[macro_use]
mod common;

multi_test!(
    rule_clr_text,
    rule_earley_text,
    r#"
    s: "a"
    "#,
    "a",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);

multi_test!(
    rule_clr_char_repeat,
    rule_earley_char_repeat,
    r#"
    s: a+
    a: "a"
    "#,
    "aaa",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);

multi_test!(
    rule_clr_alias,
    rule_earley_alias,
    r#"
    s: a+ -> x
    a: "a"
    "#,
    "aaa",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);

multi_test!(
    rule_clr_repeat_expr,
    rule_earley_repeat_expr,
    r#"
    s: e
    e: e "-" N | N
    N: /\d+/
    "#,
    "1-23-456-78-9",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);

multi_test!(
    rule_clr_opt,
    rule_earley_opt,
    r#"
    s: a
    a: A b?
    A: "x"
    b: B
    B: "y"
    "#,
    "x",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);

multi_test!(
    rule_clr_opt_with_char,
    rule_earley_opt_with_char,
    r#"
    s: a
    a: A b?
    A: "x"
    b: B
    B: "y"
    "#,
    "xy",
    "s",
    Algorithm::CLR,
    Algorithm::Earley);
