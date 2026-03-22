use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::lexer::{AST, Symbol, Token};
use swiftlet::{ParserOption, Swiftlet};

#[test]
fn plus_int_grammar_clr_test() {
    let grammar = r#"
    s: e
    e: e "+" INT | INT
    %import INT
    "#;
    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let ast = parser.parse("1+2");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "e".to_string(),
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
                    3,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Token(Arc::new(Token::new(
                    "2".to_string(),
                    2,
                    3,
                    0,
                    Arc::new(Symbol::Terminal("INT".to_string())),
                ))),
            ],
        )],
    );
    assert_eq!(ast.unwrap(), right);
}

#[test]
fn plus_int_rule_grammar_clr_test() {
    let grammar = r#"
    s: e
    e: e "+" t | t
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
    let ast = parser.parse("1 + 2");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
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
                    5,
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
        )],
    );
    assert_eq!(ast.unwrap(), right);
}

#[test]
fn plus_int_grammar_earley_test() {
    let grammar = r#"
    s: e
    e: e "+" INT | INT
    %import INT
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf);
    let ast = parser.parse("1+2");

    let right = AST::Tree(
        "s".to_string(),
        vec![AST::Tree(
            "e".to_string(),
            vec![
                AST::Tree(
                    "e".to_string(),
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
                    3,
                    0,
                    Arc::new(Symbol::Terminal("__STR__+__1".to_string())),
                ))),
                AST::Token(Arc::new(Token::new(
                    "2".to_string(),
                    2,
                    3,
                    0,
                    Arc::new(Symbol::Terminal("INT".to_string())),
                ))),
            ],
        )],
    );
    assert_eq!(ast.unwrap(), right);
}
