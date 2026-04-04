use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserOption, Swiftlet};

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
    Algorithm::Earley
);

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
    Algorithm::Earley
);

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
    Algorithm::Earley
);

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
    Algorithm::Earley
);

multi_test!(
    rule_clr_opt,
    rule_earley_opt,
    r#"
    s: A b?
    A: "x"
    b: B
    B: "y"
    "#,
    "x",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_opt_with_char,
    rule_earley_opt_with_char,
    r#"
    s: A b?
    A: "x"
    b: B
    B: "y"
    "#,
    "xy",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_context_aware,
    rule_earley_context_aware,
    r#"
    s: "A" r
    r: /\w/
    "#,
    "AB",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);


multi_test_multi_input_texts!(
    rule_clr_next_line,
    rule_earley_next_line,
    r#"
    s: "A" _NL "B"
    _NL: /(\r|\n|\s)*/
    "#,
    [
        "AB",
        "A\nB",
        r#"A
        B"#
    ],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    rule_clr_alternatives,
    rule_earley_alternatives,
    r#"
    s: hello | namaste
    hello: "Hello"
    namaste: "Namaste"
    "#,
    ["Hello", "Namaste"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    rule_clr_grouped_alternatives,
    rule_earley_grouped_alternatives,
    r#"
    s: (hello | namaste) world
    hello: "Hello"
    namaste: "Namaste"
    world: "World"
    "#,
    ["HelloWorld", "NamasteWorld"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_group_repeat_plus,
    rule_earley_group_repeat_plus,
    r#"
    s: number ("+" number)+
    number: /\d+/
    "#,
    "1+2+3",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    rule_clr_group_repeat_star,
    rule_earley_group_repeat_star,
    r#"
    s: number ("+" number)*
    number: /\d+/
    "#,
    ["1", "1+2+3"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_alias_alternatives,
    rule_earley_alias_alternatives,
    r#"
    s: expr
    expr: expr "+" INT -> add
        | expr "-" INT -> sub
        | INT
    %import (WS, INT)
    %ignore WS
    "#,
    "3 + 10 - 5 + 20",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);
