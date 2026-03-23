use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::lexer::{Symbol, Token};
use swiftlet::ast::AST;
use swiftlet::{ParserOption, Swiftlet};

// ----------------------------- ? Optional Rule ----------------------------- //
#[test]
fn option_rule_clr_test() {
    let grammar = r#"
    s: e
    ?e: [e] t
    t: "i"
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let ast = parser.parse("ii");
    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "i".to_string(),
                        0,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("__STR__I__1".to_string())),
                    )))],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "i".to_string(),
                        1,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("__STR__I__1".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(ast.unwrap(), right);
}

#[test]
fn option_rule_earley_test() {
    let grammar = r#"
    s: e
    ?e: [e] t
    t: "i"
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let ast = parser.parse("ii");
    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "i".to_string(),
                        0,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("__STR__I__1".to_string())),
                    )))],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "i".to_string(),
                        1,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("__STR__I__1".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(ast.unwrap(), right);
}

// ----------------------------- ? Operation Rule ----------------------------- //
#[test]
fn optional_expr_operation_clr_test() {
    let grammar = r#"
    s: e
    e: (e "+")? t
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1 + 2 + 3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "e".to_string(),
                    vec![
                        AST::Tree(
                            "e".to_string(),
                            vec![AST::Tree(
                                "t".to_string(),
                                vec![AST::Token(Arc::new(Token::new(
                                    "1".to_string(),
                                    0,
                                    1,
                                    0,
                                    Arc::new(Symbol::Terminal("INT".to_string())),
                                )))],
                            )],
                        ),
                        AST::Token(Arc::new(Token::new(
                            "+".to_string(),
                            2,
                            9,
                            0,
                            Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                        ))),
                        AST::Tree(
                            "t".to_string(),
                            vec![AST::Token(Arc::new(Token::new(
                                "2".to_string(),
                                4,
                                5,
                                0,
                                Arc::new(Symbol::Terminal("INT".to_string())),
                            )))],
                        ),
                    ],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    6,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        8,
                        9,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn optional_expr_operation_earley_test() {
    let grammar = r#"
    s: e
    e: (e "+")? t
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1+2+3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "e".to_string(),
                    vec![
                        AST::Tree(
                            "e".to_string(),
                            vec![AST::Tree(
                                "t".to_string(),
                                vec![AST::Token(Arc::new(Token::new(
                                    "1".to_string(),
                                    0,
                                    1,
                                    0,
                                    Arc::new(Symbol::Terminal("INT".to_string())),
                                )))],
                            )],
                        ),
                        AST::Token(Arc::new(Token::new(
                            "+".to_string(),
                            1,
                            5,
                            0,
                            Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                        ))),
                        AST::Tree(
                            "t".to_string(),
                            vec![AST::Token(Arc::new(Token::new(
                                "2".to_string(),
                                2,
                                3,
                                0,
                                Arc::new(Symbol::Terminal("INT".to_string())),
                            )))],
                        ),
                    ],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    3,
                    5,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        4,
                        5,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

// -------------------------------------- ? in rule and operaton ----------------------------------- //
#[test]
fn question_rule_and_operation_clr_test() {
    let grammar = r#"
    s: e
    ?e: (e "+")? t
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1 + 2 + 3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "e".to_string(),
                    vec![
                        AST::Tree(
                            "t".to_string(),
                            vec![AST::Token(Arc::new(Token::new(
                                "1".to_string(),
                                0,
                                1,
                                0,
                                Arc::new(Symbol::Terminal("INT".to_string())),
                            )))],
                        ),
                        AST::Token(Arc::new(Token::new(
                            "+".to_string(),
                            2,
                            9,
                            0,
                            Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                        ))),
                        AST::Tree(
                            "t".to_string(),
                            vec![AST::Token(Arc::new(Token::new(
                                "2".to_string(),
                                4,
                                5,
                                0,
                                Arc::new(Symbol::Terminal("INT".to_string())),
                            )))],
                        ),
                    ],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    6,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        8,
                        9,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn question_rule_and_operation_earley_test() {
    let grammar = r#"
    s: e
    ?e: (e "+")? t
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1+2+3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "1".to_string(),
                        0,
                        1,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    1,
                    5,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "2".to_string(),
                        2,
                        3,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    3,
                    5,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        4,
                        5,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

// ------------------------------------------- + operation ----------------------------------- //
#[test]
fn plus_operation_clr_test() {
    let grammar = r#"
    s: e
    ?e: t (("+" | "-") t)+
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1 + 2 - 3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "1".to_string(),
                        0,
                        1,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    2,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "2".to_string(),
                        4,
                        5,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "-".to_string(),
                    6,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__-__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        8,
                        9,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn plus_operation_earley_test() {
    let grammar = r#"
    s: e
    ?e: t (("+" | "-") t)+
    t: INT
    %import (INT, WS)
    %ignore WS
    "#;
    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let left = parser.parse("1 + 2 - 3");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "1".to_string(),
                        0,
                        1,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    2,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "2".to_string(),
                        4,
                        5,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "-".to_string(),
                    6,
                    9,
                    0,
                    Arc::new(Symbol::Terminal("__STR__-__1".to_string())),
                ))),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        8,
                        9,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn terms_grammar_test() {
    let text = r#"
        start: expr
        expr: expr "+" INT | INT
        DIGIT: "0" .. "9"
        INT: DIGIT+
        %import ( WS, CNAME )
        %ignore WS
        "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        ..Default::default()
    });
    let parser = Swiftlet::from_string(text, conf);
    let left = parser.parse("1234 + 3953");

    let right = AST::Tree(
        "start".to_string(),
        vec![AST::Tree(
            "expr".to_string(),
            vec![
                AST::Tree(
                    "expr".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "1234".to_string(),
                        0,
                        4,
                        0,
                        Arc::new(Symbol::Terminal("INT".to_string())),
                    )))],
                ),
                AST::Token(Arc::new(Token::new(
                    "+".to_string(),
                    5,
                    11,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Token(Arc::new(Token::new(
                    "3953".to_string(),
                    7,
                    11,
                    0,
                    Arc::new(Symbol::Terminal("INT".to_string())),
                ))),
            ],
        )],
    );

    assert_eq!(left.unwrap(), right);
}
