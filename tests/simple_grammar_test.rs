use barat::grammar::Algorithm;
use barat::lexer::{Symbol, Token, AST};
use barat::{Barat, ParserOption};
use std::sync::Arc;

#[test]
fn simple_grammar_clr_test() {
    let grammar = r#"
    s: e
    e: e t | t
    t: "i"
    "#.to_string();

    let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, start: "s".to_string(), ..Default::default() });
    let mut parser = Barat::from_string(grammar, conf);
    let left = parser.parse("ii");
    let right = AST::Tree("s".to_string(),
                          vec![
                              AST::Tree("e".to_string(),
                                        vec![
                                            AST::Tree("e".to_string(), vec![AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("i".to_string(), 0, 2, 0, Arc::new(Symbol::Terminal("__STR__I__1".to_string())))))])]),
                                            AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("i".to_string(), 1, 2, 0, Arc::new(Symbol::Terminal("__STR__I__1".to_string())))))])
                                        ]
                              )
                          ],
    );
    assert_eq!(left.unwrap(), right);
}

#[test]
fn simple_grammar_earley_test() {
    let grammar = r#"
    s: e
    e: e t | t
    t: "i"
    "#.to_string();

    let conf = Arc::new(ParserOption { start: "s".to_string(), ..Default::default() });
    let mut parser = Barat::from_string(grammar, conf);
    let left = parser.parse("ii");
    let right = AST::Tree("s".to_string(),
                          vec![
                              AST::Tree("e".to_string(),
                                        vec![
                                            AST::Tree("e".to_string(), vec![AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("i".to_string(), 0, 2, 0, Arc::new(Symbol::Terminal("__STR__I__1".to_string())))))])]),
                                            AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("i".to_string(), 1, 2, 0, Arc::new(Symbol::Terminal("__STR__I__1".to_string())))))])
                                        ]
                              )
                          ],
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
    "#.to_string();

    let conf = Arc::new(ParserOption { algorithm: Algorithm::CLR, start: "s".to_string(), ..Default::default() });
    let mut parser = Barat::from_string(grammar, conf);
    let left = format!("{:?}", parser.parse("123").unwrap());


    let right = format!("{:?}", AST::Tree("s".to_string(),
                                          vec![AST::Tree("e".to_string(),
                                                         vec![AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("1".to_string(), 0, 1, 0, Arc::new(Symbol::Terminal("DIGIT".to_string())))))]),
                                                              AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("2".to_string(), 1, 2, 0, Arc::new(Symbol::Terminal("DIGIT".to_string())))))]),
                                                              AST::Tree("t".to_string(), vec![AST::Token(Arc::new(Token::new("3".to_string(), 2, 3, 0, Arc::new(Symbol::Terminal("DIGIT".to_string())))))])])]));

    assert_eq!(left, right);
}
