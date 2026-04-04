use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::{ParserConfig, Swiftlet};

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

multi_test!(
    rule_clr_common_numeric_terminals,
    rule_earley_common_numeric_terminals,
    r#"
    s: integer ":" signed ":" negative ":" decimal
    integer: INT
    signed: SIGNED_INT
    negative: N_INT
    decimal: DECIMAL
    %import (INT, SIGNED_INT, N_INT, DECIMAL)
    "#,
    "123:+42:- 9:12.34",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_common_word_and_quote_terminals,
    rule_earley_common_word_and_quote_terminals,
    r#"
    s: name word lower upper text quoted
    name: CNAME
    word: WORD
    lower: LCASE_LETTER
    upper: UCASE_LETTER
    text: STRING
    quoted: QUOTE
    %import (CNAME, WORD, LCASE_LETTER, UCASE_LETTER, STRING, QUOTE, WS)
    %ignore WS
    "#,
    r#"swiftlet1 parser a Z "value" 'x'"#,
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_common_digit_and_hex_terminals,
    rule_earley_common_digit_and_hex_terminals,
    r#"
    s: DIGIT ":" HEXDIGIT
    %import (DIGIT, HEXDIGIT)
    "#,
    "7:BEEF42",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test!(
    rule_clr_common_comment_and_newline_terminals,
    rule_earley_common_comment_and_newline_terminals,
    r#"
    s: COMMENT _NL assignment _NL
    assignment: key "=" value
    key: CNAME
    value: STRING
    COMMENT: SH_COMMENT
    _NL: NEWLINE
    %import (CNAME, STRING, SH_COMMENT, NEWLINE, WS_INLINE)
    %ignore WS_INLINE
    "#,
    "# service settings\nHOST=\"localhost\"\n",
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);

multi_test_multi_input_texts!(
    rule_clr_common_cr_lf_terminals,
    rule_earley_common_cr_lf_terminals,
    r#"
    s: line_end WORD
    line_end: CR LF | LF
    %import (CR, LF, WORD)
    "#,
    ["\r\nReady", "\nReady"],
    "s",
    Algorithm::CLR,
    Algorithm::Earley
);
