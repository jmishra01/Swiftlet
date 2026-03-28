use std::sync::Arc;
use swiftlet::grammar::Algorithm;
use swiftlet::lexer::{Symbol, Token};
use swiftlet::ast::AST;
use swiftlet::{ParserOption, Swiftlet};

#[test]
fn simple_grammar_clr_test() {
    let grammar = r#"
    s: e
    e: e t | t
    t: "i"
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    let left = parser.parse("ii");
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
                            "ii".to_string(),
                            0,
                            1,
                            0,
                            Arc::new(Symbol::Terminal("I".to_string())),
                        )))],
                    )],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "ii".to_string(),
                        1,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("I".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn simple_grammar_earley_test() {
    let grammar = r#"
    s: e
    e: e t | t
    t: "i"
    "#;

    let conf = Arc::new(ParserOption {
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    let left = parser.parse("ii");
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
                            "ii".to_string(),
                            0,
                            1,
                            0,
                            Arc::new(Symbol::Terminal("I".to_string())),
                        )))],
                    )],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "ii".to_string(),
                        1,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("I".to_string())),
                    )))],
                ),
            ],
        )],
    );
    assert_eq!(left.unwrap(), right);
}

// ------------------------------------------- //

#[test]
fn terms_clr_test() {
    let grammar = r#"
    s: e
    ?e: t+ t
    t: DIGIT
    DIGIT: "0" .. "9"
    "#;

    let conf = Arc::new(ParserOption {
        algorithm: Algorithm::CLR,
        start: "s".to_string(),
        ..Default::default()
    });
    let parser = Swiftlet::from_string(grammar, conf).expect("failed to build parser");
    let left = parser.parse("123").unwrap();

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
                        Arc::new(Symbol::Terminal("DIGIT".to_string())),
                    )))],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "2".to_string(),
                        1,
                        2,
                        0,
                        Arc::new(Symbol::Terminal("DIGIT".to_string())),
                    )))],
                ),
                AST::Tree(
                    "t".to_string(),
                    vec![AST::Token(Arc::new(Token::new(
                        "3".to_string(),
                        2,
                        3,
                        0,
                        Arc::new(Symbol::Terminal("DIGIT".to_string())),
                    )))],
                ),
            ],
        )],
    );

    assert_eq!(left, right);
}
